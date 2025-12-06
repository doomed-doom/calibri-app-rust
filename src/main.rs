mod bindings;

use bindings::*;
use tokio::task;
use std::os::raw::c_void;
use std::ptr::{self, NonNull, null_mut};
use std::slice::from_raw_parts;
use std::sync::{Arc, Mutex};

pub async fn create_sensor_async(scanner: *mut SensorScanner, sensor_info: SensorInfo) -> Result<*mut Sensor, OpStatus> {
    // Используем spawn_blocking для запуска блокирующего кода в отдельном потоке
    let sensor_result = tokio::task::spawn_blocking(move || {
        unsafe {
            let mut status = OpStatus {
                success: 0,
                error: 0,
                error_msg: String::new(),
            };
            // Вызов функции createSensor
            let sensor = createSensor(scanner, sensor_info, &mut status);
            
            if status.success == 0 {
                Err(status)  // Если ошибка, возвращаем её
            } else {
                Ok(sensor)  // Если успешное создание, возвращаем указатель на сенсор
            }
        }
    }).await.unwrap();  // Разворачиваем результат

    sensor_result
}


#[tokio::main]
async fn main() {
    let filter: [SensorFamily; 1] = [SensorFamily_SensorLECallibri];

    let mut st: OpStatus = OpStatus {
        Success: 0,
        Error: 0,
        ErrorMsg: [0; 512],
    };

    // Создаём сканер
    let scanner = unsafe {
        createScanner(
            filter.as_ptr() as *mut SensorFamily,
            size_of_val(&filter) as i32,
            &mut st,
        )
    };

    // Колбек ф. для сканера
    unsafe extern "C" fn sensors_callback(
        ptr: *mut SensorScanner,
        sensors: *mut SensorInfo,
        sz_sensors: i32,
        user_data: *mut c_void,
    ) {
        if sensors.is_null() {
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

    let mut l_handle: SensorsListenerHandle = null_mut();

    let mut device_founded: bool = false;
    let user_data: *mut c_void = &mut device_founded as *mut _ as *mut c_void;

    unsafe {
        addSensorsCallbackScanner(
            scanner,
            Some(sensors_callback),
            &mut l_handle,
            user_data,
            &mut st,
        );
    }

    // 5 сек
    if !device_founded {
        println!("Поиск устройства");
        unsafe {
            startScanner(scanner, &mut st as *mut OpStatus, 10);
        }
    }

    let mut sz_sensors_in_out: i32 = 32;
    let mut sensors_vec: Vec<SensorInfo> = Vec::with_capacity(sz_sensors_in_out as usize);

    unsafe {
        sensors_vec.set_len(sz_sensors_in_out as usize);

        sensorsScanner(
            scanner, 
            sensors_vec.as_mut_ptr(), 
            &mut sz_sensors_in_out as *mut i32, 
            &mut st as *mut OpStatus
        );

        println!("Found {} sensors", sz_sensors_in_out);
    }

    let sensor_info = sensors_vec[0];


    let sensor_ptr = *sensor_arc.lock().unwrap();
    println!("Sensor created: {:?}", sensor_ptr);

    unsafe {
        freeScanner(scanner);
    }

    //println!("hi")
}
