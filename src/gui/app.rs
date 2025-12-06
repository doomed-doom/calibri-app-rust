use std::time::Duration;

use iced::alignment::Horizontal;
use iced::widget::{button, column, container, scrollable, text};
use iced::{exit, window, Background, Color, Element, Length, Size, Subscription, Task, Theme};
use tokio::sync::mpsc::UnboundedReceiver;

use super::controller::{Controller, ControllerEvent, DeviceEntry};

const LOG_LIMIT: usize = 200;

#[derive(Default)]
pub struct CallibriApp {
    controller: Option<Controller>,
    events: Option<UnboundedReceiver<ControllerEvent>>,
    logs: Vec<String>,
    status: RunStatus,
    devices: Vec<DeviceEntry>,
    selected_device: Option<usize>,
    pending_close: Option<window::Id>,
    scan_started: bool,
    main_window: Option<window::Id>,
    modal_window: Option<window::Id>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum RunStatus {
    Idle,
    Scanning,
    Connecting(String),
    Connected(String),
    Finished,
    Failed(String),
}

impl Default for RunStatus {
    fn default() -> Self {
        RunStatus::Idle
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    ScanRequested,
    ConnectRequested(usize),
    Tick,
    CloseRequested(window::Id),
    WindowCreated(window::Id),
}

impl CallibriApp {
    pub fn init() -> (Self, Task<Message>) {
        let mut app = Self::default();
        let (main_id, main_task) = open_main_window();
        let (modal_id, modal_task) = open_modal_window();
        app.main_window = Some(main_id);
        app.modal_window = Some(modal_id);

        let task = Task::batch(vec![
            main_task.map(Message::WindowCreated),
            modal_task.map(Message::WindowCreated),
        ]);

        (app, task)
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ScanRequested => {
                self.logs.clear();
                self.status = RunStatus::Scanning;
                self.devices.clear();
                self.selected_device = None;
                self.scan_started = true;

                if let Some(controller) = self.controller.as_ref() {
                    controller.request_rescan();
                } else {
                    let (mut controller, events) = Controller::new();
                    controller.start();
                    controller.request_rescan();
                    self.events = Some(events);
                    self.controller = Some(controller);
                }
            }
            Message::ConnectRequested(index) => {
                if self.is_busy() {
                    return Task::none();
                }
                if let Some(device) = self.devices.get(index).cloned() {
                    self.selected_device = Some(index);
                    self.status = RunStatus::Connecting(device.name.clone());
                    if let Some(controller) = self.controller.as_ref() {
                        controller.request_connect(device.info);
                    } else {
                        self.logs
                            .push("Нет активного контроллера. Сначала выполните сканирование.".into());
                    }
                }
            }
            Message::Tick => {
                let mut should_reset = false;
                if let Some(events) = &mut self.events {
                    while let Ok(event) = events.try_recv() {
                        match event {
                            ControllerEvent::Log(line) => {
                                self.logs.push(line);
                                if self.logs.len() > LOG_LIMIT {
                                    let excess = self.logs.len() - LOG_LIMIT;
                                    self.logs.drain(0..excess);
                                }
                            }
                            ControllerEvent::Devices(list) => {
                                self.devices = list;
                                self.status = RunStatus::Finished;
                            }
                            ControllerEvent::Connected(name) => {
                                self.status = RunStatus::Connected(name);
                            }
                            ControllerEvent::Finished => {
                                self.status = RunStatus::Finished;
                                should_reset = true;
                            }
                            ControllerEvent::Failed(err) => {
                                self.logs.push(format!("Ошибка: {err}"));
                                self.status = RunStatus::Failed(err.clone());
                                should_reset = true;
                            }
                        }
                    }
                }

                if should_reset {
                    self.controller = None;
                    self.events = None;
                    if let Some(id) = self.pending_close.take() {
                        return self.finalize_close(id);
                    }
                }
            }
            Message::CloseRequested(id) => {
                if Some(id) == self.modal_window {
                    return self.handle_close_modal(id);
                }

                if Some(id) == self.main_window {
                    return self.handle_close_main(id);
                }

                return window::close(id);
            }
            Message::WindowCreated(id) => {
                if Some(id) == self.modal_window {
                    return self.focus_modal();
                }
            }
        }

        Task::none()
    }

    pub fn view(&self, window_id: window::Id) -> Element<'_, Message> {
        if Some(window_id) == self.modal_window {
            return self.view_modal();
        }

        self.view_main_window()
    }

    fn view_modal(&self) -> Element<'_, Message> {
        let status_text: Element<'_, Message> = container(text(self.status_text()).size(24))
            .width(Length::Fill)
            .center_x(Length::Fill)
            .into();

        let button_label = match self.status {
            RunStatus::Scanning => "Идёт поиск...",
            RunStatus::Connecting(_) => "Подключение...",
            _ => "Сканировать",
        };

        let button_text = container(text(button_label).size(18))
            .width(Length::Fill)
            .center_x(Length::Fill);

        let mut start_button = button(button_text)
            .width(Length::Fixed(240.0))
            .padding([12, 36])
            .style(primary_button_style());
        if !self.is_busy() {
            start_button = start_button.on_press(Message::ScanRequested);
        }

        let devices_view = self.view_devices();

        let hero = column![status_text, start_button]
            .spacing(20)
            .width(Length::Fill)
            .align_x(Horizontal::Center);

        let mut content = column![hero]
            .spacing(24)
            .width(Length::Fill)
            .align_x(Horizontal::Center);

        if self.scan_started {
            content = content.push(devices_view);
        }

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .padding(24)
            .into()
    }

    fn view_main_window(&self) -> Element<'_, Message> {
        let info = if self.modal_window.is_some() {
            "Основное окно заблокировано, пока открыто окно подключения."
        } else {
            "Основное окно Callibri."
        };

        container(text(info).size(20).width(Length::Fill))
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            iced::time::every(Duration::from_millis(200)).map(|_| Message::Tick),
            window::close_requests().map(Message::CloseRequested),
        ])
    }

    fn status_text(&self) -> String {
        match &self.status {
            RunStatus::Idle | RunStatus::Finished => {
                "Нажмите \"Сканировать\", чтобы найти Callibri".into()
            }
            RunStatus::Scanning => "Идёт поиск устройств...".into(),
            RunStatus::Connecting(name) => format!("Подключение к {name}..."),
            RunStatus::Connected(name) => format!("Подключено к {name}"),
            RunStatus::Failed(err) => format!("Ошибка: {err}"),
        }
    }

    fn is_busy(&self) -> bool {
        matches!(self.status, RunStatus::Scanning | RunStatus::Connecting(_))
    }

    fn view_devices(&self) -> Element<'_, Message> {
        let content: Element<'_, Message> = if self.devices.is_empty() {
            container(text("Список устройств пуст.").size(16))
                .padding(12)
                .width(Length::Fill)
                .into()
        } else {
            let mut list = column![];
            for (index, device) in self.devices.iter().enumerate() {
                let label = text(format!(
                    "{} — {} (RSSI: {})",
                    device.name, device.address, device.rssi
                ))
                .size(16)
                .width(Length::Fill);

                let mut entry = button(label).width(Length::Fill);
                if !self.is_busy() {
                    entry = entry.on_press(Message::ConnectRequested(index));
                }

                list = list.push(entry);
            }

            scrollable(list).height(200).width(Length::Fill).into()
        };

        container(content)
            .padding(12)
            .style(device_list_style())
            .width(Length::Fixed(420.0))
            .into()
    }
}

fn primary_button_style(
) -> impl Fn(&Theme, button::Status) -> button::Style + Copy {
    |_, status| {
        let (color, alpha) = match status {
            button::Status::Hovered => (Color::from_rgb(0.18, 0.4, 0.95), 1.0),
            button::Status::Pressed => (Color::from_rgb(0.12, 0.3, 0.75), 1.0),
            button::Status::Disabled => (Color::from_rgba(0.3, 0.3, 0.35, 0.6), 0.5),
            button::Status::Active => (Color::from_rgb(0.14, 0.34, 0.85), 1.0),
        };

        let mut style = button::Style::default();
        style.background = Some(Background::Color(color));
        style.text_color = Color::from_rgba(1.0, 1.0, 1.0, alpha);
        style.border.radius = 999.0.into();
        style
    }
}

fn device_list_style() -> impl Fn(&Theme) -> container::Style + Copy {
    |theme| {
        let mut style = container::Style::default();
        let palette = theme.extended_palette();
        let surface = palette.background.strong.color;
        style.background = Some(Background::Color(Color {
            a: 0.95,
            ..surface
        }));
        style.border.radius = 16.0.into();
        style.border.width = 1.0;
        style.border.color = palette.background.strong.color;
        style
    }
}

impl CallibriApp {
    fn handle_close_main(&mut self, id: window::Id) -> Task<Message> {
        if self.controller.is_none() {
            return self.finalize_close(id);
        }

        if let Some(controller) = self.controller.as_ref() {
            controller.shutdown();
            self.pending_close = Some(id);
            self.status = RunStatus::Finished;
            self.logs.push("Завершаем работу...".into());
        }

        Task::none()
    }

    fn handle_close_modal(&mut self, modal_id: window::Id) -> Task<Message> {
        if let Some(main) = self.main_window {
            self.modal_window = None;
            let mut tasks = vec![window::close(modal_id)];
            tasks.push(self.handle_close_main(main));
            return Task::batch(tasks);
        }

        self.finalize_close(modal_id)
    }

    fn finalize_close(&mut self, id: window::Id) -> Task<Message> {
        let mut tasks: Vec<Task<Message>> = Vec::new();

        if Some(id) == self.modal_window {
            self.modal_window = None;
            tasks.push(window::close(id));
            if let Some(main) = self.main_window.take() {
                tasks.push(window::close(main));
            }
            tasks.push(exit::<Message>());
        } else if Some(id) == self.main_window {
            self.main_window = None;
            tasks.push(window::close(id));
            if let Some(modal) = self.modal_window.take() {
                tasks.push(window::close(modal));
            }
            tasks.push(exit::<Message>());
        } else {
            tasks.push(window::close(id));
        }

        Task::batch(tasks)
    }

    fn focus_modal(&self) -> Task<Message> {
        if let Some(id) = self.modal_window {
            return window::gain_focus(id).map(|_: ()| Message::Tick);
        }
        Task::none()
    }
}

fn open_main_window() -> (window::Id, Task<window::Id>) {
    let mut settings = window::Settings::default();
    settings.platform_specific.application_id = String::from("callibri-gui");
    // if settings.size.width > 150.0 {
    //     settings.size.width -= 150.0;
    // }
    settings.exit_on_close_request = false;
    window::open(settings)
}

fn open_modal_window() -> (window::Id, Task<window::Id>) {
    let mut settings = window::Settings::default();
    settings.size = Size::new(520.0, 560.0);
    settings.platform_specific.application_id = String::from("callibri-gui-connector");
    settings.level = window::Level::AlwaysOnTop;
    settings.resizable = false;
    settings.exit_on_close_request = false;
    window::open(settings)
}
