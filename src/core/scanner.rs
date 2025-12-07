use crate::core::bindings::*;
use crate::core::utils::{empty_status, extract_str, status_message};
use std::os::raw::c_void;
use std::ptr::null_mut;
use std::slice::from_raw_parts;
use tokio::task;

struct SensorPtr(*mut Sensor);

unsafe impl Send for SensorPtr {}

pub async fn create_sensor_async(
    scanner: *mut SensorScanner,
    sensor_info: SensorInfo,
) -> Result<*mut Sensor, OpStatus> {
    let scanner_addr = scanner as usize;
    let result = task::spawn_blocking(move || unsafe {
        let scanner_ptr = scanner_addr as *mut SensorScanner;
        let mut status = empty_status();
        let sensor = createSensor(scanner_ptr, sensor_info, &mut status);

        if status.Success == 0 {
            Err(status)
        } else {
            Ok(SensorPtr(sensor))
        }
    })
    .await
    .expect("Failed to join sensor creation thread");

    result.map(|sensor_ptr| sensor_ptr.0)
}

unsafe extern "C" fn sensors_callback(
    _ptr: *mut SensorScanner,
    sensors: *mut SensorInfo,
    sz_sensors: i32,
    user_data: *mut c_void,
) {
    if sensors.is_null() || sz_sensors <= 0 {
        println!("Null");
        return;
    }

    unsafe {
        let device_founded: &mut bool = &mut *(user_data as *mut bool);
        *device_founded = true;

        let slice = from_raw_parts(sensors, sz_sensors as usize);

        for sensor in slice {
            println!(
                "\nНайдено устройство:\n\
                - Device Family: {}\n\
                - Model: {}\n\
                - Name: {}\n\
                - Address: {}\n\
                - Serial Number: {}\n\
                - Pairing Required: {}\n\
                - RSSI: {}\n",
                sensor.SensFamily,
                sensor.SensModel,
                extract_str(&sensor.Name).trim_end_matches('\0'),
                extract_str(&sensor.Address).trim_end_matches('\0'),
                extract_str(&sensor.SerialNumber).trim_end_matches('\0'),
                if sensor.PairingRequired != 0 {
                    "Yes"
                } else {
                    "No"
                },
                sensor.RSSI
            );
        }
    }
}

pub struct SampleScanner {
    scanner: *mut SensorScanner,
    listener_handle: SensorsListenerHandle,
}

impl SampleScanner {
    pub fn new(filters: &[SensorFamily]) -> Result<Self, String> {
        if filters.is_empty() {
            return Err("Не указан ни один фильтр сканера".into());
        }

        let mut status = empty_status();
        let scanner = unsafe {
            createScanner(
                filters.as_ptr() as *mut SensorFamily,
                filters.len() as i32,
                &mut status,
            )
        };

        if scanner.is_null() || status.Success == 0 {
            Err(format!(
                "Не удалось создать сканер: {}",
                status_message(&status)
            ))
        } else {
            Ok(Self {
                scanner,
                listener_handle: null_mut(),
            })
        }
    }

    pub fn add_sensors_callback(&mut self, user_data: *mut c_void) -> Result<(), String> {
        if self.scanner.is_null() {
            return Err("Сканер не инициализирован".into());
        }

        if !self.listener_handle.is_null() {
            return Ok(());
        }

        let mut status = empty_status();
        let result = unsafe {
            addSensorsCallbackScanner(
                self.scanner,
                Some(sensors_callback),
                &mut self.listener_handle,
                user_data,
                &mut status,
            )
        } != 0;

        if result && status.Success != 0 {
            Ok(())
        } else {
            self.listener_handle = null_mut();
            Err(format!(
                "Не удалось установить callback сканера: {}",
                status_message(&status)
            ))
        }
    }

    pub fn remove_sensors_callback(&mut self) {
        if !self.listener_handle.is_null() {
            unsafe { removeSensorsCallbackScanner(self.listener_handle) };
            self.listener_handle = null_mut();
        }
    }

    pub fn start(&mut self, attempts: i32) -> Result<(), String> {
        if self.scanner.is_null() {
            return Err("Сканер не инициализирован".into());
        }

        let mut status = empty_status();
        let result = unsafe { startScanner(self.scanner, &mut status, attempts) } != 0;

        if result && status.Success != 0 {
            Ok(())
        } else {
            Err(format!(
                "Ошибка при старте сканера: {}",
                status_message(&status)
            ))
        }
    }

    pub fn stop(&mut self) -> Result<(), String> {
        if self.scanner.is_null() {
            return Err("Сканер не инициализирован".into());
        }

        let mut status = empty_status();
        let result = unsafe { stopScanner(self.scanner, &mut status) } != 0;

        if result && status.Success != 0 {
            Ok(())
        } else {
            Err(format!(
                "Ошибка при остановке сканера: {}",
                status_message(&status)
            ))
        }
    }

    pub fn fetch_devices(&mut self, mut capacity: i32) -> Result<Vec<SensorInfo>, String> {
        if self.scanner.is_null() {
            return Err("Сканер не инициализирован".into());
        }

        if capacity <= 0 {
            capacity = 1;
        }

        let buffer_len = capacity as usize;
        let mut sensors_vec: Vec<SensorInfo> = Vec::with_capacity(buffer_len);
        let mut sz_sensors_in_out = capacity;
        let mut status = empty_status();

        unsafe {
            sensors_vec.set_len(buffer_len);
        }

        let result = unsafe {
            sensorsScanner(
                self.scanner,
                sensors_vec.as_mut_ptr(),
                &mut sz_sensors_in_out,
                &mut status,
            )
        } != 0;

        if !result || status.Success == 0 {
            unsafe {
                sensors_vec.set_len(0);
            }
            return Err(format!(
                "Не удалось получить список сенсоров: {}",
                status_message(&status)
            ));
        }

        let sensors_found = sz_sensors_in_out.clamp(0, buffer_len as i32) as usize;
        unsafe {
            sensors_vec.set_len(sensors_found);
        }

        if sensors_vec.is_empty() {
            Err("Устройства не найдены".into())
        } else {
            Ok(sensors_vec)
        }
    }

    pub async fn create_sensor(&self, sensor_info: SensorInfo) -> Result<*mut Sensor, String> {
        if self.scanner.is_null() {
            return Err("Сканер не инициализирован".into());
        }

        create_sensor_async(self.scanner, sensor_info)
            .await
            .map_err(|status| status_message(&status))
    }
}

impl Drop for SampleScanner {
    fn drop(&mut self) {
        self.remove_sensors_callback();

        unsafe {
            if !self.scanner.is_null() {
                freeScanner(self.scanner);
                self.scanner = null_mut();
            }
        }
    }
}
