mod core;
mod gui;

// // use enigo::{Coordinate, Enigo, Mouse, Settings};
// // use std::env;
// use std::os::raw::c_void;
// use tokio::time::{Duration, Instant, sleep};

// use crate::core::{bindings::*, callibri::*, scanner::*, sensor_info::*, utils::*};

// // #[derive(Copy, Clone)]
// // enum CursorControlMode {
// //     Free,
// //     StrictAxis,
// // }

fn main() -> iced::Result {
    gui::run()
}

// async fn perform_scan(scanner: &mut SampleScanner, secs: i32) -> Result<_SensorInfo, String> {
//     println!("Запускаем сканирование...");

//     let mut device_found = false;
//     let user_data: *mut c_void = &mut device_found as *mut _ as *mut c_void;

//     scanner.add_sensors_callback(user_data)?;

//     if !device_found {
//         println!("Начинаем поиск устройств ({secs} сек)...");
//         scanner.start(secs)?;
//     }

//     let sensors = scanner.fetch_devices(32)?;

//     if let Err(err) = scanner.stop() {
//         println!("Ошибка остановки сканера: {err}");
//     }

//     scanner.remove_sensors_callback();
//     println!("Поиск завершён");
//     Ok(sensors[0])
// }

// async fn connect_sensor(
//     scanner: &mut SampleScanner,
//     sensor: SensorInfo,
// ) -> Result<CallibriSensor, String> {
//     println!("\nПодключение к {}...", extract_str(&sensor.Name));

//     let sensor_ptr: *mut _Sensor = scanner.create_sensor(sensor).await?;
//     let mut session = CallibriSensor::new(sensor_ptr);

//     session.connect()?;

//     // if let Err(err) = describe_sensor_features(session.ptr()) {
//     //     println!("Не удалось получить функции: {err}");
//     // }
//     // if let Err(err) = describe_sensor_commands(session.ptr()) {
//     //     println!("Не удалось получить команды: {err}");
//     // }
//     // if let Err(err) = describe_sensor_parameters(session.ptr()) {
//     //     println!("Не удалось получить параметры: {err}");
//     // }

//     let name = extract_str(&sensor.Name);
//     println!("\nСенсор {name} готов к дальнейшим командам.\n");

//     Ok(session)
// }

// async fn get_signal_settings(sensor: &CallibriSensor, signal_type: &mut SignalTypeCallibri) {
//     let mut status = empty_status();

//     unsafe {
//         getSignalSettingsCallibri(sensor.ptr(), signal_type as *mut SignalTypeCallibri, &mut status);
//     }
// }

// async fn set_signal_settings(sensor: &CallibriSensor, signal_type: SignalTypeCallibri) {
//     let mut status = empty_status();

//     unsafe {
//         setSignalSettingsCallibri(sensor.ptr(), signal_type, &mut status);
//     }
// }

// async fn process_ecg(
//     session: &CallibriSensor,
//     duration: Duration,
// ) -> () {
//     const DEADZONE: f64 = 7.0;
//     let started = Instant::now();
//     let limit = if duration.as_nanos() == 0 {
//         None
//     } else {
//         Some(duration)
//     };

//     loop {
//         if let Some(limit) = limit {
//             if started.elapsed() >= limit {
//                 break;
//             }
//         }

//         let packets = session.drain_signal_packets();
//         if packets.is_empty() {
//             sleep(Duration::from_millis(5)).await;
//             continue;
//         }

//         for packet in packets {
//             // Преобразуем указатель в срез безопасно
//             let samples = unsafe {
//                 if packet.Samples.is_null() {
//                     continue; // если указатель пустой, пропускаем пакет
//                 }
//                 std::slice::from_raw_parts(packet.Samples, packet.SzSamples as usize)
//             };
//             let med: f64 = samples.iter().sum::<f64>() / samples.len() as f64;

//             println!("Ср. знач. пакета {}: {med}", packet.PackNum);

//             // Проверка, если хоть одно значение больше 1
//             for sample in samples {
//                 if *sample > 0.006 {
//                     println!("\n\nWarning: Value exceeded threshold! PackNum: {}\n\n", packet.PackNum);
//                     break; // Если один элемент больше 1, можно выйти из цикла
//                 }
//             }
//         }
//     }
// }

// #[tokio::main]
// async fn main() {
//     // let sett = Settings {wayland_display: Some(":0".to_string()), ..Default::default()};

//     // let mut enigo = Enigo::new(&sett).unwrap();

//     // enigo.move_mouse(100, 100, Coordinate::Rel).unwrap();

//     let filter: [SensorFamily; 1] = [SensorFamily_SensorLECallibri];
//     let mut scanner = SampleScanner::new(&filter).unwrap();

//     let sensor = perform_scan(&mut scanner, 10).await.unwrap();

//     let mut session = connect_sensor(&mut scanner, sensor).await.unwrap();

//     // session.start_mems_stream().await.unwrap();
//     // session.configure_sampling_frequency(125);

//     let mut set: u8 = 52; // 52 NGG

//     set_signal_settings(&session, SignalTypeCallibri_CallibriSignalTypeEMG).await;

//     get_signal_settings(&session, &mut set).await;

//     println!("Callibri signal type: {set}");

//     sleep(Duration::from_secs(2)).await;

//     session.start_signal_stream().await.unwrap();
//     process_ecg(&session, Duration::from_secs(60)).await;
//     session.stop_signal_stream().await.unwrap();

//     // session.calibrate_mems().await.unwrap();

//     // loop {
//     //     if session.read_mems_calibration_state().unwrap() {
//     //         println!("\nCalibrated\n");
//     //         break;
//     //     }
//     //     sleep(Duration::from_secs(2)).await;
//     // }

//     // session.start_signal_stream().await.unwrap();

//     // process_ecg(&session, Duration::from_secs(10)).await;

//     // session.stop_signal_stream().await.unwrap();
//     // session.start_quaternion_stream().await.unwrap();

//     // if let Err(err) = session.reset_quaternion_orientation().await {
//     //     eprintln!("Не удалось сбросить ориентацию кватерниона: {err}");
//     // }

//     // sleep(Duration::from_secs(10)).await;

//     // let cursor_mode = CursorControlMode::StrictAxis;

//     // if let Err(err) =
//     //     drive_cursor_with_gyroscope(&session, Duration::from_secs(100), cursor_mode).await
//     // {
//     //     eprintln!("Ошибка управления курсором: {err}");
//     // }

//     // if let Err(err) = session.stop_mems_stream().await {
//     //     eprintln!("Не удалось остановить MEMS: {err}");
//     // }
// }

// // async fn drive_cursor_with_gyroscope(
// //     session: &CallibriSensor,
// //     duration: Duration,
// //     mode: CursorControlMode,
// // ) -> Result<(), String> {
// //     let wayland_display = env::var("WAYLAND_DISPLAY").ok();
// //     let settings = Settings {
// //         wayland_display,
// //         ..Default::default()
// //     };
// //     let mut enigo = Enigo::new(&settings).map_err(|err| format!("Enigo недоступен: {err}"))?;
// //     const GYRO_GAIN: f64 = 1.0;
// //     const DEADZONE: f64 = 7.0;
// //     let started = Instant::now();
// //     let limit = if duration.as_nanos() == 0 {
// //         None
// //     } else {
// //         Some(duration)
// //     };

// //     loop {
// //         if let Some(limit) = limit {
// //             if started.elapsed() >= limit {
// //                 break;
// //             }
// //         }

// //         let packets = session.drain_mems_packets();
// //         if packets.is_empty() {
// //             sleep(Duration::from_millis(5)).await;
// //             continue;
// //         }

// //         for packet in packets {
// //             let mut dx = 0;
// //             let mut dy = 0;

// //             match mode {
// //                 CursorControlMode::Free => {
// //                     let yaw = packet.Gyroscope.Y;
// //                     if yaw.abs() >= DEADZONE {
// //                         dx = (yaw * GYRO_GAIN).round() as i32;
// //                     }

// //                     let pitch = packet.Gyroscope.X;
// //                     if pitch.abs() >= DEADZONE {
// //                         dy = (pitch * GYRO_GAIN).round() as i32;
// //                     }
// //                 }
// //                 CursorControlMode::StrictAxis => {
// //                     let yaw = packet.Gyroscope.Y;
// //                     let pitch = packet.Gyroscope.X;
// //                     let yaw_abs = yaw.abs();
// //                     let pitch_abs = pitch.abs();

// //                     if yaw_abs < DEADZONE && pitch_abs < DEADZONE {
// //                         continue;
// //                     }

// //                     if yaw_abs >= pitch_abs {
// //                         dx = (yaw * GYRO_GAIN).round() as i32;
// //                     } else {
// //                         dy = (pitch * GYRO_GAIN).round() as i32;
// //                     }
// //                 }
// //             }

// //             if dx == 0 && dy == 0 {
// //                 continue;
// //             }

// //             enigo
// //                 .move_mouse(dx, dy, Coordinate::Rel)
// //                 .map_err(|err| format!("Не удалось переместить курсор: {err}"))?;
// //         }
        
// //     }

// //     Ok(())
// // }
