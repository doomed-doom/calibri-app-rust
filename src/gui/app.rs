use std::time::Duration;

use iced::widget::{button, column, container, scrollable, text};
use iced::{Element, Length, Subscription, Task};
use tokio::sync::mpsc::UnboundedReceiver;

use super::controller::{Controller, ControllerEvent};

const LOG_LIMIT: usize = 200;

#[derive(Default)]
pub struct CallibriApp {
    controller: Option<Controller>,
    events: Option<UnboundedReceiver<ControllerEvent>>,
    logs: Vec<String>,
    status: RunStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum RunStatus {
    Idle,
    Running,
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
    StartRequested,
    Tick,
}

impl CallibriApp {
    pub fn init() -> (Self, Task<Message>) {
        (Self::default(), Task::none())
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::StartRequested => {
                if !matches!(self.status, RunStatus::Running) {
                    self.logs.clear();
                    self.status = RunStatus::Running;
                    let (mut controller, events) = Controller::new();
                    controller.start();
                    self.controller = Some(controller);
                    self.events = Some(events);
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
                }
            }
        }

        Task::none()
    }

    pub fn view(&self) -> Element<'_, Message> {
        let status_text = text(self.status_text()).size(24);

        let button_label = match self.status {
            RunStatus::Running => "Выполняется...",
            RunStatus::Finished => "Перезапустить",
            RunStatus::Failed(_) | RunStatus::Idle => "Запустить",
        };

        let mut start_button = button(button_label).width(Length::Shrink);
        if !matches!(self.status, RunStatus::Running) {
            start_button = start_button.on_press(Message::StartRequested);
        }

        let mut log_column = column![];
        for line in &self.logs {
            log_column = log_column.push(text(line.clone()));
        }

        let logs_view = scrollable(log_column).height(Length::Fill);

        let layout = column![status_text, start_button, logs_view]
            .spacing(16)
            .width(Length::Fill)
            .height(Length::Fill);

        container(layout)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(24)
            .into()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        iced::time::every(Duration::from_millis(200)).map(|_| Message::Tick)
    }

    fn status_text(&self) -> String {
        match &self.status {
            RunStatus::Idle => "Нажмите \"Запустить\", чтобы начать поиск Callibri".into(),
            RunStatus::Running => "Поиск и подключение к Callibri...".into(),
            RunStatus::Finished => "Работа завершена".into(),
            RunStatus::Failed(err) => format!("Ошибка: {err}"),
        }
    }
}
