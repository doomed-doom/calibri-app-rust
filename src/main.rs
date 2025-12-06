mod bindings;
mod scanner;
mod sensor_info;
mod utils;

use bindings::*;
use scanner::SampleScanner;
use sensor_info::{describe_sensor_commands, describe_sensor_features, describe_sensor_parameters};
use std::os::raw::c_void;
use utils::empty_status;

#[tokio::main]
async fn main() {
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

    match scanner.create_sensor(sensor_info).await {
        Ok(sensor_ptr) => unsafe {
            println!("Sensor created: {:?}", sensor_ptr);

            let mut connect_status = empty_status();
            let connected =
                connectSensor(sensor_ptr, &mut connect_status) != 0 && connect_status.Success != 0;

            if !connected {
                eprintln!(
                    "Не удалось подключиться к сенсору: {}",
                    crate::utils::status_message(&connect_status)
                );
            } else {
                if let Err(err) = describe_sensor_features(sensor_ptr) {
                    eprintln!("{err}");
                }
                if let Err(err) = describe_sensor_commands(sensor_ptr) {
                    eprintln!("{err}");
                }
                if let Err(err) = describe_sensor_parameters(sensor_ptr) {
                    eprintln!("{err}");
                }

                let mut disconnect_status = empty_status();
                if disconnectSensor(sensor_ptr, &mut disconnect_status) == 0
                    || disconnect_status.Success == 0
                {
                    eprintln!(
                        "Не удалось отключить сенсор: {}",
                        crate::utils::status_message(&disconnect_status)
                    );
                }
            }

            freeSensor(sensor_ptr);
        },
        Err(err) => {
            eprintln!("{err}");
        }
    }

    if let Err(err) = scanner.stop() {
        eprintln!("{err}");
    }

    scanner.remove_sensors_callback();
}
