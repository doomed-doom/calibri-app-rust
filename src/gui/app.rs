use std::time::Duration;

use iced::alignment::Horizontal;
use iced::widget::text::LineHeight;
use iced::widget::{column, container, row, scrollable, text, Space, toggler, rule};
use iced::{exit, window, Element, Length, Padding, Size, Subscription, Task};
use tokio::sync::mpsc::UnboundedReceiver;

use super::components::{
    device_badge_button, device_list_container, device_list_entry, device_menu,
    hero_button,
};
use super::controller::{Controller, ControllerEvent, DeviceEntry};
use crate::core::cursor::StrictAxisMode;
use crate::core::sensor_info::{CommandDetail, ParameterDetail as SensorParameterDetail, SensorDetails};

const LOG_LIMIT: usize = 200;
const HEADING_SIZE: f32 = 22.0;
const BODY_SIZE: f32 = 16.0;
const BUTTON_LINE_HEIGHT: LineHeight = LineHeight::Relative(1.2);
const DEVICE_BADGE_TEXT_SIZE: f32 = 14.0;

#[derive(Default)]
pub struct CallibriApp {
    controller: Option<Controller>,
    events: Option<UnboundedReceiver<ControllerEvent>>,
    logs: Vec<String>,
    status: RunStatus,
    devices: Vec<DeviceEntry>,
    devices_ready: bool,
    pending_close: Option<window::Id>,
    main_window: Option<window::Id>,
    modal_window: Option<window::Id>,
    pending_device: Option<DeviceEntry>,
    connected_device: Option<DeviceEntry>,
    device_menu_open: bool,
    info_window: Option<window::Id>,
    sensor_details: Option<SensorDetails>,
    cursor_mode_enabled: bool,
    wasd_mode_enabled: bool,
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
    ShowConnector,
    Tick,
    CloseRequested(window::Id),
    WindowCreated(window::Id),
    DeviceMenuToggled,
    DeviceInfoRequested,
    DisconnectRequested,
    CursorModeChanged(bool),
    WasdModeChanged(bool),
}

impl CallibriApp {
    pub fn init() -> (Self, Task<Message>) {
        let mut app = Self::default();
        let (main_id, main_task) = open_main_window();
        app.main_window = Some(main_id);

        (app, main_task.map(Message::WindowCreated))
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ScanRequested => {
                self.logs.clear();
                self.status = RunStatus::Scanning;
                self.devices.clear();
                self.devices_ready = false;
                self.pending_device = None;
                self.connected_device = None;
                self.device_menu_open = false;
                self.sensor_details = None;
                self.cursor_mode_enabled = false;
                self.wasd_mode_enabled = false;
                let mut follow_up: Option<Task<Message>> = None;
                if let Some(info) = self.info_window.take() {
                    follow_up = Some(window::close(info));
                }

                if let Some(controller) = self.controller.as_ref() {
                    controller.request_rescan();
                } else {
                    let (mut controller, events) = Controller::new();
                    controller.start();
                    controller.request_rescan();
                    self.events = Some(events);
                    self.controller = Some(controller);
                }

                if let Some(task) = follow_up {
                    return task;
                }
            }
            Message::ConnectRequested(index) => {
                if self.is_busy() {
                    return Task::none();
                }
                if let Some(device) = self.devices.get(index).cloned() {
                    self.status = RunStatus::Connecting(device.name.clone());
                    self.pending_device = Some(device.clone());
                    if let Some(controller) = self.controller.as_ref() {
                        controller.request_connect(device.info);
                    } else {
                        self.logs.push(
                            "Нет активного контроллера. Сначала выполните сканирование.".into(),
                        );
                    }
                }
            }
            Message::ShowConnector => {
                if self.modal_window.is_some() {
                    return self.focus_modal();
                }
                let (modal_id, task) = open_modal_window();
                self.modal_window = Some(modal_id);
                return task.map(Message::WindowCreated);
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
                                self.devices_ready = true;
                                self.status = RunStatus::Finished;
                            }
                        ControllerEvent::Connected(name) => {
                            self.status = RunStatus::Connected(name);
                            if let Some(device) = self.pending_device.take() {
                                self.connected_device = Some(device);
                            }
                        }
                        ControllerEvent::SensorDetails(details) => {
                            self.sensor_details = Some(details);
                        }
                        ControllerEvent::CursorModeState(mode) => {
                            match mode {
                                Some(StrictAxisMode::Default) => {
                                    self.cursor_mode_enabled = true;
                                    self.wasd_mode_enabled = false;
                                }
                                Some(StrictAxisMode::Game) => {
                                    self.cursor_mode_enabled = false;
                                    self.wasd_mode_enabled = true;
                                }
                                None => {
                                    self.cursor_mode_enabled = false;
                                    self.wasd_mode_enabled = false;
                                }
                            }
                        }
                        ControllerEvent::Finished => {
                            self.status = RunStatus::Finished;
                            should_reset = true;
                        }
                            ControllerEvent::Failed(err) => {
                                self.logs.push(format!("Ошибка: {err}"));
                                self.status = RunStatus::Failed(err.clone());
                                self.pending_device = None;
                                self.connected_device = None;
                                self.device_menu_open = false;
                                should_reset = true;
                            }
                        }
                    }
                }

                if should_reset {
                    self.controller = None;
                    self.events = None;
                    self.pending_device = None;
                    self.connected_device = None;
                    self.device_menu_open = false;
                    self.sensor_details = None;
                    self.cursor_mode_enabled = false;
                    self.wasd_mode_enabled = false;
                    if !matches!(self.status, RunStatus::Failed(_)) {
                        self.status = RunStatus::Idle;
                    }
                    let mut tasks = Vec::new();
                    if let Some(info) = self.info_window.take() {
                        tasks.push(window::close(info));
                    }
                    if let Some(id) = self.pending_close.take() {
                        tasks.push(self.finalize_close(id));
                    }
                    if !tasks.is_empty() {
                        return Task::batch(tasks);
                    }
                }
            }
            Message::CloseRequested(id) => {
                if Some(id) == self.modal_window {
                    return self.handle_close_modal(id);
                }

                if Some(id) == self.info_window {
                    return self.handle_close_info(id);
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
                if Some(id) == self.info_window {
                    return self.focus_info_window();
                }
            }
            Message::DeviceMenuToggled => {
                self.device_menu_open = !self.device_menu_open;
            }
            Message::DeviceInfoRequested => {
                self.device_menu_open = false;
                if self.info_window.is_some() {
                    return self.focus_info_window();
                }
                if self.connected_device.is_some() && self.sensor_details.is_some() {
                    let (info_id, task) = open_info_window();
                    self.info_window = Some(info_id);
                    return task.map(Message::WindowCreated);
                } else {
                    self.logs.push(
                        "Нет данных для отображения информации об устройстве. Дождитесь подключения."
                            .into(),
                    );
                }
            }
            Message::DisconnectRequested => {
                self.device_menu_open = false;
                self.connected_device = None;
                self.pending_device = None;
                self.status = RunStatus::Idle;
                self.sensor_details = None;
                self.cursor_mode_enabled = false;
                self.wasd_mode_enabled = false;
                let mut tasks = Vec::new();
                if let Some(info) = self.info_window.take() {
                    tasks.push(window::close(info));
                }
                if let Some(controller) = self.controller.as_ref() {
                    controller.shutdown();
                    self.logs.push("Отключаем устройство...".into());
                }
                if !tasks.is_empty() {
                    return Task::batch(tasks);
                }
            }
            Message::CursorModeChanged(state) => {
                if state == self.cursor_mode_enabled {
                    return Task::none();
                }
                if state && self.connected_device.is_none() {
                    self.logs.push(
                        "Сначала подключите устройство, чтобы использовать режим курсора.".into(),
                    );
                    return Task::none();
                }
                if let Some(controller) = self.controller.as_ref() {
                    controller.request_cursor_mode(state, StrictAxisMode::Default);
                } else {
                    self.logs.push(
                        "Контроллер недоступен, режим курсора не изменён.".into(),
                    );
                    return Task::none();
                }
                self.cursor_mode_enabled = state;
                if state {
                    self.wasd_mode_enabled = false;
                }
            }
            Message::WasdModeChanged(state) => {
                if state == self.wasd_mode_enabled {
                    return Task::none();
                }
                if state && self.connected_device.is_none() {
                    self.logs.push(
                        "Сначала подключите устройство, чтобы использовать игровой режим.".into(),
                    );
                    return Task::none();
                }
                if let Some(controller) = self.controller.as_ref() {
                    controller.request_cursor_mode(state, StrictAxisMode::Game);
                } else {
                    self.logs.push("Контроллер недоступен, режим WASD не изменён.".into());
                    return Task::none();
                }
                self.wasd_mode_enabled = state;
                if state {
                    self.cursor_mode_enabled = false;
                }
            }
        }

        Task::none()
    }

    pub fn view(&self, window_id: window::Id) -> Element<'_, Message> {
        if Some(window_id) == self.modal_window {
            return self.view_modal();
        }

        if Some(window_id) == self.info_window {
            return self.view_info_window();
        }

        self.view_main_window()
    }

    fn view_modal(&self) -> Element<'_, Message> {
        let status_message = if matches!(self.status, RunStatus::Scanning) {
            "Пожалуйста, подождите...".to_string()
        } else {
            self.status_text()
        };

        let status_text: Element<'_, Message> = container(
            text(status_message)
                .size(HEADING_SIZE)
                .line_height(BUTTON_LINE_HEIGHT)
                .width(Length::Fill)
                .align_x(Horizontal::Center),
        )
        .width(Length::Fill)
        .center_x(Length::Fill)
        .into();

        let button_label = match self.status {
            RunStatus::Scanning => "Идёт поиск...",
            RunStatus::Connecting(_) => "Подключение...",
            _ => "Сканировать",
        };

        let mut start_button =
            hero_button::<Message>(button_label, BODY_SIZE, BUTTON_LINE_HEIGHT);
        if !self.is_busy() {
            start_button = start_button.on_press(Message::ScanRequested);
        }

        let hero = column![status_text, start_button]
            .spacing(20)
            .width(Length::Fill)
            .align_x(Horizontal::Center);

        let mut content = column![hero]
            .spacing(24)
            .width(Length::Fill)
            .align_x(Horizontal::Center);

        let devices_section: Element<'_, Message> = if self.devices_ready {
            self.view_devices()
        } else if matches!(self.status, RunStatus::Scanning) {
            Space::with_height(Length::Shrink).into()
        } else {
            container(
                text("Нажмите \"Сканировать\", чтобы увидеть доступные устройства.")
                    .size(BODY_SIZE)
                    .line_height(BUTTON_LINE_HEIGHT)
                    .width(Length::Fill)
                    .align_x(Horizontal::Center),
            )
            .padding(12)
            .width(Length::Fill)
            .into()
        };

        content = content.push(devices_section);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .padding(24)
            .into()
    }

    fn view_info_window(&self) -> Element<'_, Message> {
        let header_text = self
            .connected_device
            .as_ref()
            .map(|device| {
                text(format!("Информация о {}", device.name))
                    .size(HEADING_SIZE)
                    .line_height(BUTTON_LINE_HEIGHT)
                    .width(Length::Fill)
                    .align_x(Horizontal::Center)
            })
            .unwrap_or_else(|| {
                text("Нет подключённого устройства")
                    .size(HEADING_SIZE)
                    .line_height(BUTTON_LINE_HEIGHT)
                    .width(Length::Fill)
                    .align_x(Horizontal::Center)
            });

        let content: Element<'_, Message> = if let (Some(_device), Some(details)) =
            (&self.connected_device, &self.sensor_details)
        {
            let mut sections = column![
                self.render_features_section(&details.features),
                self.render_commands_section(&details.commands),
            ]
            .spacing(16);

            if !details.parameters.is_empty() {
                let params = details
                    .parameters
                    .iter()
                    .map(|detail| self.render_parameter_detail(detail))
                    .fold(column![].spacing(12), |col, el| col.push(el));
                sections = sections.push(params);
            }

            scrollable(sections).height(Length::Fill).into()
        } else if self.connected_device.is_some() {
            container(
                text("Получаем информацию об устройстве...")
                    .size(BODY_SIZE)
                    .line_height(BUTTON_LINE_HEIGHT)
                    .width(Length::Fill)
                    .align_x(Horizontal::Center),
            )
            .width(Length::Fill)
            .center_x(Length::Fill)
            .padding(12)
            .into()
        } else {
            container(
                text("Подключите устройство, чтобы увидеть доступные параметры.")
                    .size(BODY_SIZE)
                    .line_height(BUTTON_LINE_HEIGHT)
                    .width(Length::Fill)
                    .align_x(Horizontal::Center),
            )
            .width(Length::Fill)
            .center_x(Length::Fill)
            .padding(12)
            .into()
        };

        let body: Element<'_, Message> =
            container(content).width(Length::Fill).height(Length::Fill).into();

        container(column![header_text, body].spacing(16))
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(24)
            .into()
    }

    fn view_main_window(&self) -> Element<'_, Message> {
        match &self.status {
            RunStatus::Connected(name) => self.view_connected_main(name),
            _ => self.view_idle_main(),
        }
    }

    fn view_idle_main(&self) -> Element<'_, Message> {
        let status_text = container(
            text(self.status_text())
                .size(HEADING_SIZE)
                .line_height(BUTTON_LINE_HEIGHT)
                .width(Length::Fill)
                .align_x(Horizontal::Center),
        )
        .width(Length::Fill)
        .center_x(Length::Fill);

        let mut connect_button =
            hero_button::<Message>("Подключить", BODY_SIZE, BUTTON_LINE_HEIGHT);
        connect_button = connect_button.on_press(Message::ShowConnector);

        let content = column![status_text, connect_button]
            .spacing(20)
            .align_x(Horizontal::Center);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .padding(24)
            .into()
    }

    fn view_connected_main(&self, name: &str) -> Element<'_, Message> {
        let label_text = if let Some(device) = self.connected_device.as_ref() {
            format!("{} · {}", device.name, device.address)
        } else {
            format!("{name}")
        };

        let badge = device_badge_button::<Message>(
            label_text.as_str(),
            DEVICE_BADGE_TEXT_SIZE,
            BUTTON_LINE_HEIGHT,
        )
        .on_press(Message::DeviceMenuToggled);

        let controls = if self.device_menu_open {
            column![
                device_menu(
                    Message::DeviceInfoRequested,
                    Message::DisconnectRequested,
                    BODY_SIZE,
                    BUTTON_LINE_HEIGHT,
                ),
                badge
            ]
            .spacing(8)
            .align_x(Horizontal::Right)
        } else {
            column![badge].align_x(Horizontal::Right)
        };

        let modes_section = row![
            column![
                text("Обычный режим")
                    .size(HEADING_SIZE)
                    .line_height(BUTTON_LINE_HEIGHT),
                toggler(self.cursor_mode_enabled)
                    .label("Режим курсора")
                    .on_toggle(Message::CursorModeChanged)
                    .spacing(16.0)
                    .width(Length::Fill)
                    .text_size(16)
                    .size(20.0)
            ]
            .spacing(32)
            .padding(Padding {
                top: 0.0,
                right: 0.0,
                bottom: 0.0,
                left: 32.0,
            })
            .width(Length::Fill),
            rule::Rule::vertical(2),
            column![
                text("Игровой режим")
                    .size(HEADING_SIZE)
                    .line_height(BUTTON_LINE_HEIGHT),
                toggler(self.wasd_mode_enabled)
                    .label("Режим WASD")
                    .on_toggle(Message::WasdModeChanged)
                    .spacing(16.0)
                    .width(Length::Fill)
                    .text_size(16)
                    .size(22.0)
            ]
            .spacing(32)
            .padding(Padding {
                top: 0.0,
                right: 24.0,
                bottom: 0.0,
                left: 24.0,
            })
            .width(Length::Fill),
        ]
        .spacing(24)
        .width(Length::Fill);

        container(column![
            Space::with_height(Length::Fixed(48.0)),
            modes_section,
            Space::with_height(Length::Fill),
            row![
                Space::with_width(Length::Fill),
                container(controls).padding(Padding {
                    top: 0.0,
                    right: 4.0,
                    bottom: 4.0,
                    left: 0.0,
                })
            ]
        ])
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(0)
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
                "Для начала работы подключите устройство".into()
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
            device_list_container(
                container(
                    text("Список устройств пуст.")
                        .size(BODY_SIZE)
                        .line_height(BUTTON_LINE_HEIGHT)
                        .width(Length::Fill)
                        .align_x(Horizontal::Center),
                )
                .width(Length::Fill),
            )
            .width(Length::Fixed(420.0))
            .into()
        } else {
            let mut list = column![];
            for (index, device) in self.devices.iter().enumerate() {
                let is_selected = self
                    .connected_device
                    .as_ref()
                    .map(|d| d.address == device.address)
                    .unwrap_or(false);

                let mut entry = device_list_entry::<Message>(
                    format!(
                        "{} — {} (RSSI: {})",
                        device.name, device.address, device.rssi
                    ),
                    is_selected,
                    BODY_SIZE,
                    BUTTON_LINE_HEIGHT,
                );

                if !self.is_busy() {
                    entry = entry.on_press(Message::ConnectRequested(index));
                }

                list = list.push(entry);
            }

            device_list_container(scrollable(list).height(200).width(Length::Fill))
                .width(Length::Fixed(420.0))
                .into()
        };

        content
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
        if Some(modal_id) == self.modal_window {
            self.modal_window = None;
            return window::close(modal_id);
        }
        Task::none()
    }

    fn handle_close_info(&mut self, info_id: window::Id) -> Task<Message> {
        if Some(info_id) == self.info_window {
            self.info_window = None;
            return window::close(info_id);
        }
        Task::none()
    }

    fn finalize_close(&mut self, id: window::Id) -> Task<Message> {
        let mut tasks: Vec<Task<Message>> = Vec::new();

        if Some(id) == self.modal_window {
            self.modal_window = None;
            tasks.push(window::close(id));
            if let Some(main) = self.main_window.take() {
                tasks.push(window::close(main));
            }
            if let Some(info) = self.info_window.take() {
                tasks.push(window::close(info));
            }
            tasks.push(exit::<Message>());
        } else if Some(id) == self.main_window {
            self.main_window = None;
            tasks.push(window::close(id));
            if let Some(modal) = self.modal_window.take() {
                tasks.push(window::close(modal));
            }
            if let Some(info) = self.info_window.take() {
                tasks.push(window::close(info));
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

    fn focus_info_window(&self) -> Task<Message> {
        if let Some(id) = self.info_window {
            return window::gain_focus(id).map(|_: ()| Message::Tick);
        }
        Task::none()
    }

    fn render_features_section(&self, features: &[String]) -> Element<'_, Message> {
        if features.is_empty() {
            return Space::with_height(Length::Shrink).into();
        }

        let mut list = column![text("Доступные функции")
            .size(HEADING_SIZE)
            .align_x(Horizontal::Left)];
        for feature in features {
            list = list.push(
                row![
                    Space::with_width(Length::Fixed(16.0)),
                    text(format!("• {feature}")).size(BODY_SIZE)
                ]
                .spacing(8),
            );
        }

        list.spacing(6).into()
    }

    fn render_commands_section(&self, commands: &[CommandDetail]) -> Element<'_, Message> {
        if commands.is_empty() {
            return Space::with_height(Length::Shrink).into();
        }

        let mut list = column![text("Команды устройства")
            .size(HEADING_SIZE)
            .align_x(Horizontal::Left)];
        for command in commands {
            list = list.push(
                row![
                    Space::with_width(Length::Fixed(16.0)),
                    text(format!("• {} (код {})", command.name, command.code)).size(BODY_SIZE)
                ]
                .spacing(8),
            );
        }

        list.spacing(6).into()
    }

    fn render_parameter_detail<'a>(
        &self,
        detail: &'a SensorParameterDetail,
    ) -> Element<'a, Message> {
        let mut lines = column![text(format!(
            "{} (код {}): {}",
            detail.name, detail.code, detail.access
        ))
        .size(HEADING_SIZE)];

        for desc in &detail.lines {
            lines = lines.push(
                row![
                    Space::with_width(Length::Fixed(16.0)),
                    text(desc).size(BODY_SIZE)
                ]
                .spacing(8),
            );
        }

        if !detail.bullets.is_empty() {
            for bullet in &detail.bullets {
                lines = lines.push(
                    row![
                        Space::with_width(Length::Fixed(28.0)),
                        text(format!("• {bullet}")).size(BODY_SIZE)
                    ]
                    .spacing(6),
                );
            }
        }

        lines.spacing(6).into()
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

fn open_info_window() -> (window::Id, Task<window::Id>) {
    let mut settings = window::Settings::default();
    settings.size = Size::new(520.0, 640.0);
    settings.platform_specific.application_id = String::from("callibri-gui-info");
    settings.resizable = true;
    settings.exit_on_close_request = false;
    window::open(settings)
}
