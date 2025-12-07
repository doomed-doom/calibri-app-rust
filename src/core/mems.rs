use crate::core::bindings::*;
use crate::core::commands::exec_sensor_command;
use crate::core::utils::{empty_status, status_message};
use std::os::raw::c_void;
use std::ptr::null_mut;
use std::slice;
use std::sync::{Arc, Mutex};

pub struct CallibriMEMSListener {
    handle: MEMSDataListenerHandle,
    buffer: Arc<Mutex<Vec<_MEMSData>>>,
}

impl Default for CallibriMEMSListener {
    fn default() -> Self {
        Self {
            handle: null_mut(),
            buffer: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl CallibriMEMSListener {
    pub fn subscribe(&mut self, sensor_ptr: *mut Sensor) -> Result<(), String> {
        if !self.handle.is_null() {
            return Ok(());
        }

        let mut status = empty_status();
        let storage_ptr = Arc::as_ptr(&self.buffer) as *mut c_void;
        let ok = unsafe {
            addMEMSDataCallback(
                sensor_ptr,
                Some(mems_callback),
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
                "Не удалось подписаться на MEMS: {}",
                status_message(&status)
            ))
        }
    }

    pub fn unsubscribe(&mut self) {
        if !self.handle.is_null() {
            unsafe { removeMEMSDataCallback(self.handle) };
            self.handle = null_mut();
        }
    }

    pub async fn start(&self, sensor_ptr: *mut Sensor) -> Result<(), String> {
        exec_sensor_command(sensor_ptr, SensorCommand_CommandStartMEMS).await
    }

    pub async fn stop(&self, sensor_ptr: *mut Sensor) -> Result<(), String> {
        exec_sensor_command(sensor_ptr, SensorCommand_CommandStopMEMS).await
    }

    pub fn take_packets(&self) -> Vec<_MEMSData> {
        let mut guard = self
            .buffer
            .lock()
            .expect("Хранилище MEMS-пакетов запаниковало");
        guard.drain(..).collect()
    }
}

impl Drop for CallibriMEMSListener {
    fn drop(&mut self) {
        self.unsubscribe();
    }
}

pub struct CallibriQuaternionListener {
    handle: QuaternionDataListenerHandle,
    buffer: Arc<Mutex<Vec<_QuaternionData>>>,
}

impl Default for CallibriQuaternionListener {
    fn default() -> Self {
        Self {
            handle: null_mut(),
            buffer: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl CallibriQuaternionListener {
    pub fn subscribe(&mut self, sensor_ptr: *mut Sensor) -> Result<(), String> {
        if !self.handle.is_null() {
            return Ok(());
        }

        let mut status = empty_status();
        let storage_ptr = Arc::as_ptr(&self.buffer) as *mut c_void;
        let ok = unsafe {
            addQuaternionDataCallback(
                sensor_ptr,
                Some(quaternion_callback),
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
                "Не удалось подписаться на quaternion: {}",
                status_message(&status)
            ))
        }
    }

    pub fn unsubscribe(&mut self) {
        if !self.handle.is_null() {
            unsafe { removeQuaternionDataCallback(self.handle) };
            self.handle = null_mut();
        }
    }

    pub fn take_packets(&self) -> Vec<_QuaternionData> {
        let mut guard = self
            .buffer
            .lock()
            .expect("Хранилище quaternion-пакетов запаниковало");
        guard.drain(..).collect()
    }
}

impl Drop for CallibriQuaternionListener {
    fn drop(&mut self) {
        self.unsubscribe();
    }
}

unsafe extern "C" fn mems_callback(
    _sensor: *mut Sensor,
    data: *mut MEMSData,
    sz_data: i32,
    user_data: *mut c_void,
) {
    if data.is_null() || sz_data <= 0 {
        println!("MEMS callback: пустой пакет");
        return;
    }

    unsafe {
        let packets = slice::from_raw_parts(data, sz_data as usize);
        let storage_ptr = user_data as *const Mutex<Vec<_MEMSData>>;
        if let Some(storage) = storage_ptr.as_ref() {
            if let Ok(mut buffer) = storage.lock() {
                buffer.extend(packets.iter().copied());
            }
        }
        for packet in packets {
            println!(
                "MEMS пакет #{}: ACC[x:{:.3}, y:{:.3}, z:{:.3}] GYRO[x:{:.3}, y:{:.3}, z:{:.3}]",
                packet.PackNum,
                packet.Accelerometer.X,
                packet.Accelerometer.Y,
                packet.Accelerometer.Z,
                packet.Gyroscope.X,
                packet.Gyroscope.Y,
                packet.Gyroscope.Z,
            );
        }
    }
}

unsafe extern "C" fn quaternion_callback(
    _sensor: *mut Sensor,
    data: *mut QuaternionData,
    sz_data: i32,
    user_data: *mut c_void,
) {
    if data.is_null() || sz_data <= 0 {
        println!("Quaternion callback: пустой пакет");
        return;
    }

    unsafe {
        let packets = slice::from_raw_parts(data, sz_data as usize);
        let storage_ptr = user_data as *const Mutex<Vec<_QuaternionData>>;
        if let Some(storage) = storage_ptr.as_ref() {
            if let Ok(mut buffer) = storage.lock() {
                buffer.extend(packets.iter().copied());
            }
        }
        for packet in packets {
            println!(
                "Quaternion пакет #{}: [w:{:.3}, x:{:.3}, y:{:.3}, z:{:.3}]",
                packet.PackNum, packet.W, packet.X, packet.Y, packet.Z
            );
        }
    }
}
