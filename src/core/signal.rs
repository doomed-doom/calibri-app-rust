use crate::core::bindings::*;
use crate::core::utils::{empty_status, status_message};
use std::os::raw::c_void;
use std::ptr::null_mut;
use std::slice;
use tokio::task;

#[derive(Default)]
pub struct CallibriSignalListener {
    handle: CallibriSignalDataListenerHandle,
}

impl CallibriSignalListener {
    pub fn subscribe(&mut self, sensor_ptr: *mut Sensor) -> Result<(), String> {
        if !self.handle.is_null() {
            return Ok(());
        }

        let mut status = empty_status();
        let ok = unsafe {
            addSignalCallbackCallibri(
                sensor_ptr,
                Some(signal_callback),
                &mut self.handle,
                null_mut(),
                &mut status,
            )
        } != 0;

        if ok && status.Success != 0 {
            Ok(())
        } else {
            self.handle = null_mut();
            Err(format!(
                "Не удалось подписаться на сигнал: {}",
                status_message(&status)
            ))
        }
    }

    pub fn unsubscribe(&mut self) {
        if !self.handle.is_null() {
            unsafe { removeSignalCallbackCallibri(self.handle) };
            self.handle = null_mut();
        }
    }

    pub async fn start(&self, sensor_ptr: *mut Sensor) -> Result<(), String> {
        exec_sensor_command(sensor_ptr, SensorCommand_CommandStartSignal).await
    }

    pub async fn stop(&self, sensor_ptr: *mut Sensor) -> Result<(), String> {
        exec_sensor_command(sensor_ptr, SensorCommand_CommandStopSignal).await
    }

    pub fn is_subscribed(&self) -> bool {
        !self.handle.is_null()
    }
}

impl Drop for CallibriSignalListener {
    fn drop(&mut self) {
        self.unsubscribe();
    }
}

async fn exec_sensor_command(
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

unsafe extern "C" fn signal_callback(
    _sensor: *mut Sensor,
    data: *mut CallibriSignalData,
    sz_data: i32,
    _user_data: *mut c_void,
) {
    if data.is_null() || sz_data <= 0 {
        println!("Callibri signal callback: пустой пакет");
        return;
    }

    unsafe {
        let packets = slice::from_raw_parts(data, sz_data as usize);
        for packet in packets {
            if packet.Samples.is_null() || packet.SzSamples == 0 {
                println!("Получен пакет без данных");
                continue;
            }

            let samples = slice::from_raw_parts(packet.Samples, packet.SzSamples as usize);
            println!("{} отсчётов (В): {:?}", samples.len(), samples);
        }
    }
}
