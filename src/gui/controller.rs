use crate::core::bindings::{SensorFamily, SensorFamily_SensorLECallibri, SensorInfo};
use crate::core::callibri::CallibriSensor;
use crate::core::scanner::SampleScanner;
use crate::core::sensor_info::{
    describe_sensor_commands, describe_sensor_features, describe_sensor_parameters,
};
use std::ffi::CStr;
use std::os::raw::c_void;
use std::thread;
use tokio::runtime::{Builder, Runtime};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};

pub enum ControllerEvent {
    Log(String),
    Devices(Vec<DeviceEntry>),
    Connected(String),
    Finished,
    Failed(String),
}

#[derive(Debug, Clone, Copy)]
pub enum ControllerCommand {
    Rescan,
    Connect { sensor: SensorInfo },
    Shutdown,
}

#[derive(Clone, Debug)]
pub struct DeviceEntry {
    pub info: SensorInfo,
    pub name: String,
    pub address: String,
    pub rssi: i16,
}

pub struct Controller {
    runtime: Runtime,
    events_tx: UnboundedSender<ControllerEvent>,
    command_tx: UnboundedSender<ControllerCommand>,
    command_rx: Option<UnboundedReceiver<ControllerCommand>>,
    started: bool,
}

impl Controller {
    pub fn new() -> (Self, UnboundedReceiver<ControllerEvent>) {
        let runtime = Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Не удалось создать tokio runtime для GUI");

        let (events_tx, events_rx) = unbounded_channel();
        let (command_tx, command_rx) = unbounded_channel();

        (
            Self {
                runtime,
                events_tx,
                command_tx,
                command_rx: Some(command_rx),
                started: false,
            },
            events_rx,
        )
    }

    pub fn start(&mut self) {
        if self.started {
            return;
        }

        self.started = true;
        let events = self.events_tx.clone();
        let handle = self.runtime.handle().clone();
        let command_rx = self
            .command_rx
            .take()
            .expect("command receiver already taken");

        thread::spawn(move || {
            let fut = run_controller(events.clone(), command_rx);
            match handle.block_on(fut) {
                Ok(_) => {
                    let _ = events.send(ControllerEvent::Finished);
                }
                Err(err) => {
                    let _ = events.send(ControllerEvent::Failed(err));
                }
            }
        });
    }

    pub fn request_rescan(&self) {
        let _ = self.command_tx.send(ControllerCommand::Rescan);
    }

    pub fn request_connect(&self, sensor: SensorInfo) {
        let _ = self
            .command_tx
            .send(ControllerCommand::Connect { sensor });
    }

    pub fn shutdown(&self) {
        let _ = self.command_tx.send(ControllerCommand::Shutdown);
    }
}

async fn run_controller(
    events: UnboundedSender<ControllerEvent>,
    mut command_rx: UnboundedReceiver<ControllerCommand>,
) -> Result<(), String> {
    let filter: [SensorFamily; 1] = [SensorFamily_SensorLECallibri];
    let mut scanner = SampleScanner::new(&filter)?;
    let mut session: Option<CallibriSensor> = None;

    while let Some(command) = command_rx.recv().await {
        match command {
            ControllerCommand::Rescan => {
                if let Some(mut active) = session.take() {
                    log(&events, "Отключаем текущее соединение перед повторным поиском.");
                    if active.is_connected() {
                        let _ = active.disconnect();
                    }
                }
                perform_scan(&events, &mut scanner).await?;
            }
            ControllerCommand::Connect { sensor } => {
                if let Some(mut active) = session.take() {
                    log(&events, "Отключаем предыдущее соединение...");
                    if active.is_connected() {
                        let _ = active.disconnect();
                    }
                }
                session = Some(connect_sensor(&events, &mut scanner, sensor).await?);
            }
            ControllerCommand::Shutdown => {
                break;
            }
        }
    }

    if let Some(mut active) = session {
        if active.is_connected() {
            let _ = active.disconnect();
        }
    }

    if let Err(err) = scanner.stop() {
        log(&events, format!("Ошибка остановки сканера: {err}"));
    }
    scanner.remove_sensors_callback();
    log(&events, "Работа завершена");
    Ok(())
}

fn log(events: &UnboundedSender<ControllerEvent>, message: impl Into<String>) {
    let _ = events.send(ControllerEvent::Log(message.into()));
}

impl From<SensorInfo> for DeviceEntry {
    fn from(info: SensorInfo) -> Self {
        Self {
            name: extract_str(&info.Name),
            address: extract_str(&info.Address),
            rssi: info.RSSI,
            info,
        }
    }
}

fn extract_str(buf: &[std::os::raw::c_char]) -> String {
    let slice = unsafe { CStr::from_ptr(buf.as_ptr()) };
    slice.to_string_lossy().trim().to_string()
}

async fn perform_scan(
    events: &UnboundedSender<ControllerEvent>,
    scanner: &mut SampleScanner,
) -> Result<(), String> {
    log(events, "Запускаем сканирование...");
    let mut device_found = false;
    let user_data: *mut c_void = &mut device_found as *mut _ as *mut c_void;
    scanner.add_sensors_callback(user_data)?;

    if !device_found {
        log(events, "Начинаем поиск устройств (10 сек)...");
        scanner.start(10)?;
    }

    let sensors = scanner.fetch_devices(32)?;
    log(events, format!("Найдено {} сенсоров", sensors.len()));
    let entries = sensors.into_iter().map(DeviceEntry::from).collect();
    let _ = events.send(ControllerEvent::Devices(entries));

    if let Err(err) = scanner.stop() {
        log(events, format!("Ошибка остановки сканера: {err}"));
    }
    scanner.remove_sensors_callback();
    log(events, "Поиск завершён");
    Ok(())
}

async fn connect_sensor(
    events: &UnboundedSender<ControllerEvent>,
    scanner: &mut SampleScanner,
    sensor: SensorInfo,
) -> Result<CallibriSensor, String> {
    log(events, format!("Подключение к {}...", extract_str(&sensor.Name)));

    let sensor_ptr = scanner.create_sensor(sensor).await?;
    let mut session = CallibriSensor::new(sensor_ptr);

    session.connect()?;

    if let Err(err) = describe_sensor_features(session.ptr()) {
        log(events, format!("Не удалось получить функции: {err}"));
    }
    if let Err(err) = describe_sensor_commands(session.ptr()) {
        log(events, format!("Не удалось получить команды: {err}"));
    }
    if let Err(err) = describe_sensor_parameters(session.ptr()) {
        log(events, format!("Не удалось получить параметры: {err}"));
    }

    log(events, "Сенсор готов к дальнейшим командам.");
    let name = extract_str(&sensor.Name);
    let _ = events.send(ControllerEvent::Connected(name));
    Ok(session)
}
