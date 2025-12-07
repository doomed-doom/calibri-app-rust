use crate::core::bindings::*;
use crate::core::commands::exec_sensor_command;
use crate::core::utils::{empty_status, status_message};
use std::os::raw::c_void;
use std::ptr::null_mut;
use std::slice;
use std::sync::{Arc, Mutex};

pub struct CallibriSignalListener {
    handle: CallibriSignalDataListenerHandle,
    buffer: Arc<Mutex<Vec<_CallibriSignalData>>>,
}

impl Default for CallibriSignalListener {
    fn default() -> Self {
        Self {
            handle: null_mut(),
            buffer: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl CallibriSignalListener {
    pub fn subscribe(&mut self, sensor_ptr: *mut Sensor) -> Result<(), String> {
        if !self.handle.is_null() {
            return Ok(());
        }

        let mut status = empty_status();
        let storage_ptr = Arc::as_ptr(&self.buffer) as *mut c_void;
        let ok = unsafe {
            addSignalCallbackCallibri(
                sensor_ptr,
                Some(signal_callback),
                &mut self.handle,
                storage_ptr,
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

    pub fn take_packets(&self) -> Vec<_CallibriSignalData> {
        let mut guard = self
            .buffer
            .lock()
            .expect("Хранилище Signal-пакетов запаниковало");
        guard.drain(..).collect()
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

unsafe extern "C" fn signal_callback(
    _sensor: *mut Sensor,
    data: *mut CallibriSignalData,
    sz_data: i32,
    user_data: *mut c_void,
) {
    if data.is_null() || sz_data <= 0 {
        println!("Callibri signal callback: пустой пакет");
        return;
    }

    unsafe {
        let packets = slice::from_raw_parts(data, sz_data as usize);
        let storage_ptr = user_data as *const Mutex<Vec<_CallibriSignalData>>;
        if let Some(storage) = storage_ptr.as_ref() {
            if let Ok(mut buffer) = storage.lock() {
                buffer.extend(packets.iter().copied());
            }
        }
        for packet in packets {
            if packet.Samples.is_null() || packet.SzSamples == 0 {
                println!("Получен пакет без данных");
                continue;
            }

            let samples = slice::from_raw_parts(packet.Samples, packet.SzSamples as usize);
            // println!(
            //     "Signal packet #{}, {} отсчётов (В): {:?}",
            //     packet.PackNum,
            //     samples.len(),
            //     samples
            // );
        }
    }
}
