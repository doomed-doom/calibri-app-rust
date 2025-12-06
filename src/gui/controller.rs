use crate::core::bindings::{
    SensorFamily, SensorFamily_SensorLECallibri, SensorSamplingFrequency_FrequencyHz500,
};
use crate::core::callibri::CallibriSensor;
use crate::core::scanner::SampleScanner;
use crate::core::sensor_info::{
    describe_sensor_commands, describe_sensor_features, describe_sensor_parameters,
};
use std::os::raw::c_void;
use std::thread;
use tokio::runtime::{Builder, Runtime};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};
use tokio::time::{Duration, sleep};

pub enum ControllerEvent {
    Log(String),
    Finished,
    Failed(String),
}

pub struct Controller {
    runtime: Runtime,
    events_tx: UnboundedSender<ControllerEvent>,
    started: bool,
}

impl Controller {
    pub fn new() -> (Self, UnboundedReceiver<ControllerEvent>) {
        let runtime = Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Не удалось создать tokio runtime для GUI");

        let (events_tx, events_rx) = unbounded_channel();

        (
            Self {
                runtime,
                events_tx,
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

        thread::spawn(
            move || match handle.block_on(run_callibri(events.clone())) {
                Ok(_) => {
                    let _ = events.send(ControllerEvent::Finished);
                }
                Err(err) => {
                    let _ = events.send(ControllerEvent::Failed(err));
                }
            },
        );
    }
}

async fn run_callibri(events: UnboundedSender<ControllerEvent>) -> Result<(), String> {
    log(&events, "Запускаем сканер...");

    let filter: [SensorFamily; 1] = [SensorFamily_SensorLECallibri];
    let mut scanner = SampleScanner::new(&filter)?;

    let mut device_found = false;
    let user_data: *mut c_void = &mut device_found as *mut _ as *mut c_void;
    scanner.add_sensors_callback(user_data)?;

    if !device_found {
        log(&events, "Начинаем поиск устройств (10 сек)...");
        scanner.start(10)?;
    }

    let sensors = scanner.fetch_devices(32)?;
    log(&events, format!("Найдено {} сенсоров", sensors.len()));

    let Some(sensor_info) = sensors.first().copied() else {
        return Err("Сканер не вернул ни одного сенсора".into());
    };

    log(&events, "Создаём сенсор...");
    let sensor_ptr = scanner.create_sensor(sensor_info).await?;
    let mut session = CallibriSensor::new(sensor_ptr);

    log(&events, "Подключение к сенсору...");
    session.connect()?;

    log(&events, "Чтение функций/команд/параметров сенсора...");
    if let Err(err) = describe_sensor_features(session.ptr()) {
        log(&events, format!("Не удалось получить функции: {err}"));
    }
    if let Err(err) = describe_sensor_commands(session.ptr()) {
        log(&events, format!("Не удалось получить команды: {err}"));
    }
    if let Err(err) = describe_sensor_parameters(session.ptr()) {
        log(&events, format!("Не удалось получить параметры: {err}"));
    }

    log(&events, "Настраиваем частоту дискретизации 500 Гц...");
    session.configure_sampling_frequency(SensorSamplingFrequency_FrequencyHz500)?;

    log(&events, "Запускаем поток сигнала...");
    session.start_signal_stream().await?;

    log(&events, "Получаем данные 5 секунд...");
    sleep(Duration::from_secs(5)).await;

    if session.is_signal_running() {
        log(&events, "Останавливаем поток сигнала...");
        if let Err(err) = session.stop_signal_stream().await {
            log(&events, format!("Ошибка остановки сигнала: {err}"));
        }
    }

    session.unsubscribe_signal();

    if session.supports_mems() {
        log(
            &events,
            "Устройство поддерживает MEMS. Проверяем состояние калибровки...",
        );
        match session.read_mems_calibration_state() {
            Ok(true) => log(&events, "MEMS уже откалиброван."),
            Ok(false) => {
                log(&events, "MEMS не откалиброван. Запускаем калибровку...");
                if let Err(err) = session.calibrate_mems().await {
                    log(&events, format!("Ошибка калибровки MEMS: {err}"));
                } else {
                    log(&events, "Команда калибровки MEMS отправлена.");
                }
            }
            Err(err) => log(
                &events,
                format!("Не удалось получить состояние MEMS: {err}"),
            ),
        }

        if let Err(err) = session.subscribe_mems() {
            log(&events, format!("Не удалось подписаться на MEMS: {err}"));
        }
        if let Err(err) = session.subscribe_quaternion() {
            log(
                &events,
                format!("Не удалось подписаться на кватернионы: {err}"),
            );
        }

        log(&events, "Запускаем поток MEMS на 5 секунд...");
        match session.start_mems_stream().await {
            Ok(_) => {
                sleep(Duration::from_secs(5)).await;
                if session.is_mems_running() {
                    if let Err(err) = session.stop_mems_stream().await {
                        log(&events, format!("Ошибка остановки MEMS: {err}"));
                    }
                }
            }
            Err(err) => log(&events, format!("Не удалось запустить MEMS: {err}")),
        }

        session.unsubscribe_mems();
        session.unsubscribe_quaternion();
    } else {
        log(&events, "MEMS не поддерживается данным устройством.");
    }

    if session.is_connected() {
        log(&events, "Отключаем сенсор...");
        if let Err(err) = session.disconnect() {
            log(&events, format!("Ошибка отключения: {err}"));
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
