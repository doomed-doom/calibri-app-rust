use std::env;

use enigo::{Coordinate, Direction, Enigo, Key, Mouse, Settings, Keyboard};
use tokio::time::{sleep, Duration, Instant};

use crate::core::bindings::_MEMSData;
use crate::core::callibri::CallibriSensor;

#[derive(Copy, Clone, Debug)]
pub enum CursorControlMode {
    #[allow(dead_code)]
    Free,
    StrictAxis(StrictAxisMode),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum StrictAxisMode {
    Default,
    Game,
}

// pub async fn drive_cursor_with_gyroscope(
//     session: &CallibriSensor,
//     duration: Duration,
//     mode: CursorControlMode,
// ) -> Result<(), String> {
//     let wayland_display = env::var("WAYLAND_DISPLAY").ok();
//     let settings = Settings {
//         wayland_display,
//         ..Default::default()
//     };
//     let mut enigo =
//         Enigo::new(&settings).map_err(|err| format!("Enigo недоступен: {err}"))?;

//     const GYRO_GAIN: f64 = 1.0;
//     const DEADZONE: f64 = 20.0;

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

//         let packets = session.drain_mems_packets();
//         if packets.is_empty() {
//             sleep(Duration::from_millis(5)).await;
//             continue;
//         }

//         for packet in packets {
//             match mode {
//                 CursorControlMode::Free => {
//                     let mut dx = 0;
//                     let mut dy = 0;
//                     let yaw = packet.Gyroscope.Y;
//                     if yaw.abs() >= DEADZONE {
//                         dx = (yaw * GYRO_GAIN).round() as i32;
//                     }

//                     let pitch = packet.Gyroscope.X;
//                     if pitch.abs() >= DEADZONE {
//                         dy = (pitch * GYRO_GAIN).round() as i32;
//                     }

//                     if dx == 0 && dy == 0 {
//                         continue;
//                     }

//                     enigo
//                         .move_mouse(dx, dy, Coordinate::Rel)
//                         .map_err(|err| format!("Не удалось переместить курсор: {err}"))?;
//                 }
//                 CursorControlMode::StrictAxis(strict_mode) => {
//                     if let Some((dx, dy)) =
//                         compute_strict_axis_delta(packet, GYRO_GAIN, DEADZONE)
//                     {
//                         apply_strict_axis_output(&mut enigo, strict_mode, dx, dy)?;
//                     }
//                 }
//             }
//         }
//     }

//     Ok(())
// }

pub async fn drive_cursor_with_gyroscope(
    session: &CallibriSensor,
    duration: Duration,
    mode: CursorControlMode,
) -> Result<(), String> {
    let wayland_display = env::var("WAYLAND_DISPLAY").ok();
    let settings = Settings {
        wayland_display,
        ..Default::default()
    };
    let mut enigo =
        Enigo::new(&settings).map_err(|err| format!("Enigo недоступен: {err}"))?;

    const GYRO_GAIN: f64 = 1.0;
    const DEADZONE: f64 = 20.0;

    let started = Instant::now();
    let limit = if duration.as_nanos() == 0 {
        None
    } else {
        Some(duration)
    };

    loop {
        if let Some(limit) = limit {
            if started.elapsed() >= limit {
                break;
            }
        }

        let packets = session.drain_mems_packets();
        if packets.is_empty() {
            sleep(Duration::from_millis(5)).await;
            continue;
        }

        for packet in packets {
            match mode {
                CursorControlMode::Free => {
                    let mut dx = 0;
                    let mut dy = 0;
                    let yaw = packet.Gyroscope.Y;
                    if yaw.abs() >= DEADZONE {
                        // ИНВЕРТИРОВАНО: yaw вправо → курсор вправо (положительный dx)
                        dx = (-yaw * GYRO_GAIN).round() as i32;  // Добавлен минус
                    }

                    let pitch = packet.Gyroscope.X;
                    if pitch.abs() >= DEADZONE {
                        // ИНВЕРТИРОВАНО: pitch вверх → курсор вверх (отрицательный dy)
                        dy = (-pitch * GYRO_GAIN).round() as i32; // Добавлен минус
                    }

                    if dx == 0 && dy == 0 {
                        continue;
                    }

                    enigo
                        .move_mouse(dx, dy, Coordinate::Rel)
                        .map_err(|err| format!("Не удалось переместить курсор: {err}"))?;
                }
                CursorControlMode::StrictAxis(strict_mode) => {
                    if let Some((dx, dy)) =
                        compute_strict_axis_delta(packet, GYRO_GAIN, DEADZONE)
                    {
                        apply_strict_axis_output(&mut enigo, strict_mode, dx, dy)?;
                    }
                }
            }
        }
    }

    Ok(())
}

// fn compute_strict_axis_delta(
//     packet: _MEMSData,
//     gain: f64,
//     deadzone: f64,
// ) -> Option<(i32, i32)> {
//     let yaw = packet.Gyroscope.Y;
//     let pitch = packet.Gyroscope.X;
//     let yaw_abs = yaw.abs();
//     let pitch_abs = pitch.abs();

//     if yaw_abs < deadzone && pitch_abs < deadzone {
//         return None;
//     }

//     if yaw_abs >= pitch_abs {
//         Some(((yaw * gain).round() as i32, 0))
//     } else {
//         Some((0, (pitch * gain).round() as i32))
//     }
// }

fn compute_strict_axis_delta(
    packet: _MEMSData,
    gain: f64,
    deadzone: f64,
) -> Option<(i32, i32)> {
    let yaw = packet.Gyroscope.Y;
    let pitch = packet.Gyroscope.X;
    let yaw_abs = yaw.abs();
    let pitch_abs = pitch.abs();

    if yaw_abs < deadzone && pitch_abs < deadzone {
        return None;
    }

    if yaw_abs >= pitch_abs {
        // ИНВЕРТИРОВАНО: yaw вправо → движение вправо
        Some(((-yaw * gain).round() as i32, 0))  // Добавлен минус
    } else {
        // ИНВЕРТИРОВАНО: pitch вверх → движение вверх
        Some((0, (-pitch * gain).round() as i32)) // Добавлен минус
    }
}

fn apply_strict_axis_output(
    enigo: &mut Enigo,
    mode: StrictAxisMode,
    dx: i32,
    dy: i32,
) -> Result<(), String> {
    match mode {
        StrictAxisMode::Default => {
            if dx == 0 && dy == 0 {
                return Ok(());
            }
            enigo
                .move_mouse(dx, dy, Coordinate::Rel)
                .map_err(|err| format!("Не удалось переместить курсор: {err}"))
        }
        StrictAxisMode::Game => {
            send_wasd_inputs(enigo, dx, dy);
            Ok(())
        }
    }
}

fn send_wasd_inputs(enigo: &mut Enigo, dx: i32, dy: i32) {
    const ACTIVATION: i32 = 12;

    if dy.abs() >= ACTIVATION {
        if dy < 0 {
            let _ = enigo.key(Key::Unicode('w'), Direction::Click);
        } else {
            let _ = enigo.key(Key::Unicode('s'), Direction::Click);
        }
    } else if dx.abs() >= ACTIVATION {
        if dx < 0 {
            let _ = enigo.key(Key::Unicode('a'), Direction::Click);
        } else {
            let _ = enigo.key(Key::Unicode('d'), Direction::Click);
        }
    }
}
