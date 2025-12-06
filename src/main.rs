mod bindings;

use bindings::*;
use std::ffi::CStr;
use std::os::raw::c_void;
use std::ptr::null_mut;
use std::slice::from_raw_parts;
use tokio::task;

struct SensorPtr(*mut Sensor);

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

#[tokio::main]
async fn main() {
    let filter: [SensorFamily; 1] = [SensorFamily_SensorLECallibri];

    let mut st = empty_status();
    let scanner = unsafe {
        createScanner(
            filter.as_ptr() as *mut SensorFamily,
            filter.len() as i32,
            &mut st,
        )
    };

    if scanner.is_null() || st.Success == 0 {
        eprintln!("Не удалось создать сканер: {}", status_message(&st));
        return;
    }

    let mut l_handle: SensorsListenerHandle = null_mut();
    let mut device_founded: bool = false;
    let user_data: *mut c_void = &mut device_founded as *mut _ as *mut c_void;

    st = empty_status();
    unsafe {
        addSensorsCallbackScanner(
            scanner,
            Some(sensors_callback),
            &mut l_handle,
            user_data,
            &mut st,
        );
    }

    if st.Success == 0 {
        eprintln!(
            "Не удалось установить callback сканера: {}",
            status_message(&st)
        );
    }

    if !device_founded {
        println!("Поиск устройства");
        st = empty_status();
        unsafe {
            startScanner(scanner, &mut st, 10);
        }

        if st.Success == 0 {
            eprintln!("Ошибка при старте сканера: {}", status_message(&st));
        }
    }

    let mut sz_sensors_in_out: i32 = 32;
    let buffer_len = sz_sensors_in_out as usize;
    let mut sensors_vec: Vec<SensorInfo> = Vec::with_capacity(buffer_len);

    st = empty_status();
    let sensors_found = unsafe {
        sensors_vec.set_len(buffer_len);

        if sensorsScanner(
            scanner,
            sensors_vec.as_mut_ptr(),
            &mut sz_sensors_in_out,
            &mut st,
        ) == 0
        {
            sensors_vec.set_len(0);
            0usize
        } else {
            let count = sz_sensors_in_out.clamp(0, buffer_len as i32) as usize;
            sensors_vec.set_len(count);
            count
        }
    };

    if st.Success == 0 || sensors_found == 0 {
        eprintln!(
            "Не удалось получить список сенсоров: {}",
            status_message(&st)
        );

        if !l_handle.is_null() {
            unsafe { removeSensorsCallbackScanner(l_handle) };
        }

        unsafe {
            stopScanner(scanner, &mut st);
            freeScanner(scanner);
        }
        return;
    }

    println!("Найдено {} сенсоров", sensors_found);
    let sensor_info = sensors_vec[0];

    match create_sensor_async(scanner, sensor_info).await {
        Ok(sensor_ptr) => {
            println!("Sensor created: {:?}", sensor_ptr);
            unsafe {
                freeSensor(sensor_ptr);
            }
        }
        Err(status) => {
            eprintln!("Не удалось создать сенсор: {}", status_message(&status));
        }
    }

    if !l_handle.is_null() {
        unsafe { removeSensorsCallbackScanner(l_handle) };
    }

    st = empty_status();
    unsafe {
        stopScanner(scanner, &mut st);
        freeScanner(scanner);
    }
}
