mod bindings;

use bindings::*;
use std::ffi::CStr;
use std::os::raw::c_void;
use std::ptr::null_mut;
use std::slice::from_raw_parts;
use tokio::task;

struct SensorPtr(*mut Sensor);
struct SampleScanner {
    scanner: *mut SensorScanner,
    listener_handle: SensorsListenerHandle,
}

unsafe impl Send for SensorPtr {}

fn empty_status() -> OpStatus {
    OpStatus {
        Success: 0,
        Error: 0,
        ErrorMsg: [0; 512],
    }
}

fn status_message(status: &OpStatus) -> String {
    unsafe {
        CStr::from_ptr(status.ErrorMsg.as_ptr())
            .to_string_lossy()
            .into_owned()
    }
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
            println!("Sensor: {:?}", sensor);
        }
    }
}

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

impl SampleScanner {
    fn new(filters: &[SensorFamily]) -> Result<Self, String> {
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

    fn add_sensors_callback(&mut self, user_data: *mut c_void) -> Result<(), String> {
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

    fn remove_sensors_callback(&mut self) {
        if !self.listener_handle.is_null() {
            unsafe { removeSensorsCallbackScanner(self.listener_handle) };
            self.listener_handle = null_mut();
        }
    }

    fn start(&mut self, attempts: i32) -> Result<(), String> {
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

    fn stop(&mut self) -> Result<(), String> {
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

    fn fetch_devices(&mut self, mut capacity: i32) -> Result<Vec<SensorInfo>, String> {
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
            Err("Сканер не вернул ни одного устройства".into())
        } else {
            Ok(sensors_vec)
        }
    }

    async fn create_sensor(&self, sensor_info: SensorInfo) -> Result<*mut Sensor, String> {
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

    if !device_founded {
        println!("Поиск устройства");
        if let Err(err) = scanner.start(10) {
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
        Ok(sensor_ptr) => {
            println!("Sensor created: {:?}", sensor_ptr);
            unsafe {
                freeSensor(sensor_ptr);
            }
        }
        Err(err) => {
            eprintln!("{err}");
        }
    }

    if let Err(err) = scanner.stop() {
        eprintln!("{err}");
    }

    scanner.remove_sensors_callback();
}
