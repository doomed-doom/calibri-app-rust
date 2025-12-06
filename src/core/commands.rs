use crate::core::bindings::*;
use crate::core::utils::{empty_status, status_message};
use tokio::task;

pub async fn exec_sensor_command(
    sensor_ptr: *mut Sensor,
    command: SensorCommand,
) -> Result<(), String> {
    let sensor_addr = sensor_ptr as usize;
    task::spawn_blocking(move || unsafe {
        let mut status = empty_status();
        let ptr = sensor_addr as *mut Sensor;
        let ok = execCommandSensor(ptr, command, &mut status) != 0;
        if ok && status.Success != 0 {
            Ok(())
        } else {
            Err(format!(
                "Не удалось выполнить команду {}: {}",
                command,
                status_message(&status)
            ))
        }
    })
    .await
    .expect("Не удалось выполнить команду в отдельном потоке")
}
