mod bindings;
mod callibri;
mod scanner;
mod sensor_info;
mod signal;
mod utils;

use bindings::*;
use callibri::CallibriSensor;
use scanner::SampleScanner;
use sensor_info::{describe_sensor_commands, describe_sensor_features, describe_sensor_parameters};
use std::future::Future;
use std::os::raw::c_void;
use tokio::signal::ctrl_c;
use tokio::sync::watch;
use tokio::time::{Duration, sleep};

#[tokio::main]
async fn main() {
    let (shutdown_tx, shutdown_rx) = watch::channel(false);

    tokio::spawn(async move {
        if ctrl_c().await.is_ok() {
            let _ = shutdown_tx.send(true);
        }
    });

    let mut shutdown = false;
    let filter: [SensorFamily; 1] = [SensorFamily_SensorLECallibri];

    let mut scanner = match SampleScanner::new(&filter) {
        Ok(scanner) => scanner,
        Err(err) => {
            eprintln!("{err}");
            return;
        }
    };

    let mut device_founded = false;
    let user_data: *mut c_void = &mut device_founded as *mut _ as *mut c_void;

    if let Err(err) = scanner.add_sensors_callback(user_data) {
        eprintln!("{err}");
    }

    let secs_to_found = 10;

    if !device_founded {
        println!("Поиск устройства {} сек", secs_to_found);
        if let Err(err) = scanner.start(secs_to_found) {
            eprintln!("{err}");
            return;
        }
    }

    let sensors = match scanner.fetch_devices(32) {
        Ok(list) => list,
        Err(err) => {
            eprintln!("{err}");
            return;
        }
    };

    println!("Найдено {} сенсоров", sensors.len());
    let sensor_info = sensors[0];

    let mut callibri_sensor: Option<CallibriSensor> = None;

    if !shutdown {
        let mut rx = shutdown_rx.clone();
        match race_with_shutdown(scanner.create_sensor(sensor_info), &mut rx).await {
            Ok(res) => match res {
                Ok(ptr) => callibri_sensor = Some(CallibriSensor::new(ptr)),
                Err(err) => eprintln!("{err}"),
            },
            Err(_) => announce_shutdown(
                &mut shutdown,
                "Получен Ctrl+C во время создания сенсора. Завершаем работу...",
            ),
        }
    }

    if let Some(mut session) = callibri_sensor {
        if !shutdown {
            if let Err(err) = session.connect() {
                eprintln!("{err}");
            } else {
                if let Err(err) = describe_sensor_features(session.ptr()) {
                    eprintln!("{err}");
                }
                if let Err(err) = describe_sensor_commands(session.ptr()) {
                    eprintln!("{err}");
                }
                if let Err(err) = describe_sensor_parameters(session.ptr()) {
                    eprintln!("{err}");
                }

                if let Err(err) =
                    session.configure_sampling_frequency(SensorSamplingFrequency_FrequencyHz500)
                {
                    eprintln!("{err}");
                } else {
                    println!("Частота дискретизации установлена на 500 Гц.");
                }

                if !shutdown {
                    let mut rx = shutdown_rx.clone();
                    match race_with_shutdown(session.start_signal_stream(), &mut rx).await {
                        Ok(res) => {
                            if let Err(err) = res {
                                eprintln!("{err}");
                            }
                        }
                        Err(_) => announce_shutdown(
                            &mut shutdown,
                            "Получен Ctrl+C во время запуска потока сигнала.",
                        ),
                    }

                    if session.is_signal_running() && !shutdown {
                        let mut rx = shutdown_rx.clone();
                        println!("Получаем сигнал от устройства... Нажмите Ctrl+C для остановки.");
                        tokio::select! {
                            _ = sleep(Duration::from_secs(999)) => {
                                println!("Время приёма истекло, останавливаем поток.");
                            }
                            _ = wait_for_shutdown(&mut rx) => {
                                announce_shutdown(&mut shutdown, "Получен Ctrl+C, завершаем приём сигнала.");
                            }
                        }
                    }
                }
            }
        }

        if session.is_signal_running() {
            if let Err(err) = session.stop_signal_stream().await {
                eprintln!("{err}");
            }
        }
        session.unsubscribe_signal();

        if session.is_connected() {
            if let Err(err) = session.disconnect() {
                eprintln!("{err}");
            }
        }
    }

    if let Err(err) = scanner.stop() {
        eprintln!("{err}");
    }

    scanner.remove_sensors_callback();
}

fn announce_shutdown(shutdown: &mut bool, message: &str) {
    if !*shutdown {
        println!("\n{message}");
        *shutdown = true;
    }
}

async fn race_with_shutdown<F, T>(
    future: F,
    shutdown_rx: &mut watch::Receiver<bool>,
) -> Result<T, ()>
where
    F: Future<Output = T>,
{
    tokio::select! {
        res = future => Ok(res),
        _ = wait_for_shutdown(shutdown_rx) => Err(()),
    }
}

async fn wait_for_shutdown(shutdown_rx: &mut watch::Receiver<bool>) {
    if *shutdown_rx.borrow() {
        return;
    }
    let _ = shutdown_rx.changed().await;
}
