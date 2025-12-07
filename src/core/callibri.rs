#![allow(dead_code)]

use crate::core::bindings::*;
use crate::core::commands::exec_sensor_command;
use crate::core::mems::{CallibriMEMSListener, CallibriQuaternionListener};
use crate::core::signal::CallibriSignalListener;
use crate::core::utils::{empty_status, status_message};

pub struct CallibriSensor {
    sensor: *mut Sensor,
    signal_listener: CallibriSignalListener,
    mems_listener: CallibriMEMSListener,
    quaternion_listener: CallibriQuaternionListener,
    signal_running: bool,
    mems_running: bool,
    quaternion_running: bool,
    connected: bool,
}

impl CallibriSensor {
    pub fn new(sensor: *mut Sensor) -> Self {
        Self {
            sensor,
            signal_listener: CallibriSignalListener::default(),
            mems_listener: CallibriMEMSListener::default(),
            quaternion_listener: CallibriQuaternionListener::default(),
            signal_running: false,
            mems_running: false,
            quaternion_running: false,
            connected: false,
        }
    }

    pub fn ptr(&self) -> *mut Sensor {
        self.sensor
    }

    pub fn is_connected(&self) -> bool {
        self.connected
    }

    pub fn is_signal_running(&self) -> bool {
        self.signal_running
    }

    pub fn is_mems_running(&self) -> bool {
        self.mems_running
    }

    pub fn connect(&mut self) -> Result<(), String> {
        if self.sensor.is_null() {
            return Err("Null sensor pointer".into());
        }

        let mut status = empty_status();
        let ok = unsafe { connectSensor(self.sensor, &mut status) } != 0;
        if ok && status.Success != 0 {
            self.connected = true;
            Ok(())
        } else {
            Err(format!(
                "Не удалось подключиться к сенсору: {}",
                status_message(&status)
            ))
        }
    }

    pub fn disconnect(&mut self) -> Result<(), String> {
        if !self.connected {
            return Ok(());
        }

        let mut status = empty_status();
        let ok = unsafe { disconnectSensor(self.sensor, &mut status) } != 0;
        if ok && status.Success != 0 {
            self.connected = false;
            Ok(())
        } else {
            Err(format!(
                "Не удалось отключить сенсор: {}",
                status_message(&status)
            ))
        }
    }

    pub fn configure_sampling_frequency(
        &mut self,
        frequency: SensorSamplingFrequency,
    ) -> Result<(), String> {
        let mut status = empty_status();
        let ok = unsafe { writeSamplingFrequencySensor(self.sensor, frequency, &mut status) } != 0;
        if ok && status.Success != 0 {
            Ok(())
        } else {
            Err(format!(
                "Не удалось установить частоту дискретизации: {}",
                status_message(&status)
            ))
        }
    }

    fn ensure_signal_subscription(&mut self) -> Result<(), String> {
        if self.signal_listener.is_subscribed() {
            return Ok(());
        }

        self.signal_listener.subscribe(self.sensor)
    }

    pub async fn start_signal_stream(&mut self) -> Result<(), String> {
        if self.signal_running {
            return Ok(());
        }

        self.ensure_signal_subscription()?;
        self.signal_listener.start(self.sensor).await?;
        self.signal_running = true;
        Ok(())
    }

    pub async fn stop_signal_stream(&mut self) -> Result<(), String> {
        if !self.signal_running {
            return Ok(());
        }

        match self.signal_listener.stop(self.sensor).await {
            Ok(_) => {
                self.signal_running = false;
                Ok(())
            }
            Err(err) => Err(err),
        }
    }

    pub fn unsubscribe_signal(&mut self) {
        self.signal_listener.unsubscribe();
    }

    pub fn supports_mems(&self) -> bool {
        if self.sensor.is_null() {
            return false;
        }

        unsafe { isSupportedFeatureSensor(self.sensor, SensorFeature_FeatureMEMS) != 0 }
    }

    pub fn subscribe_mems(&mut self) -> Result<(), String> {
        self.mems_listener.subscribe(self.sensor)
    }

    pub fn subscribe_quaternion(&mut self) -> Result<(), String> {
        self.quaternion_listener.subscribe(self.sensor)
    }

    pub fn unsubscribe_mems(&mut self) {
        self.mems_listener.unsubscribe();
    }

    pub fn unsubscribe_quaternion(&mut self) {
        self.quaternion_listener.unsubscribe();
    }

    pub fn drain_signal_packets(&self) -> Vec<_CallibriSignalData> {
        self.signal_listener.take_packets()
    }

    pub fn drain_mems_packets(&self) -> Vec<_MEMSData> {
        self.mems_listener.take_packets()
    }

    pub fn drain_quaternion_packets(&self) -> Vec<_QuaternionData> {
        self.quaternion_listener.take_packets()
    }

    pub fn read_mems_calibration_state(&self) -> Result<bool, String> {
        unsafe {
            let mut state: u8 = 0;
            let mut status = empty_status();
            let ok = readMEMSCalibrateStateCallibri(self.sensor, &mut state, &mut status) != 0;
            if ok && status.Success != 0 {
                Ok(state != 0)
            } else {
                Err(format!(
                    "Не удалось получить состояние калибровки MEMS: {}",
                    status_message(&status)
                ))
            }
        }
    }

    pub async fn calibrate_mems(&self) -> Result<(), String> {
        exec_sensor_command(self.sensor, SensorCommand_CommandCalibrateMEMS).await
    }

    pub async fn start_mems_stream(&mut self) -> Result<(), String> {
        if self.mems_running {
            return Ok(());
        }

        self.mems_listener.subscribe(self.sensor)?;
        self.mems_listener.start(self.sensor).await?;
        self.mems_running = true;
        Ok(())
    }

    pub async fn stop_mems_stream(&mut self) -> Result<(), String> {
        if !self.mems_running {
            return Ok(());
        }

        self.mems_listener.stop(self.sensor).await?;
        self.mems_running = false;
        Ok(())
    }

    pub async fn start_quaternion_stream(&mut self) -> Result<(), String> {
        if self.quaternion_running {
            return Ok(());
        }

        self.quaternion_listener.subscribe(self.sensor)?;
        exec_sensor_command(self.sensor, SensorCommand_CommandStartAngle).await?;
        self.quaternion_running = true;
        Ok(())
    }

    pub async fn stop_quaternion_stream(&mut self) -> Result<(), String> {
        if !self.quaternion_running {
            return Ok(());
        }

        exec_sensor_command(self.sensor, SensorCommand_CommandStopAngle).await?;
        self.quaternion_listener.unsubscribe();
        self.quaternion_running = false;
        Ok(())
    }

    pub async fn reset_quaternion_orientation(&self) -> Result<(), String> {
        exec_sensor_command(self.sensor, SensorCommand_CommandResetQuaternion).await
    }
}

impl Drop for CallibriSensor {
    fn drop(&mut self) {
        if self.signal_running {
            eprintln!(
                "Предупреждение: поток сигнала всё ещё активен при завершении. Попробуйте остановить его явно."
            );
        }
        self.signal_listener.unsubscribe();

        if self.mems_running {
            eprintln!(
                "Предупреждение: поток MEMS всё ещё активен при завершении. Попробуйте остановить его явно."
            );
        }
        self.mems_listener.unsubscribe();

        if self.quaternion_running {
            eprintln!(
                "Предупреждение: поток кватернионов всё ещё активен при завершении. Попробуйте остановить его явно."
            );
        }
        self.quaternion_listener.unsubscribe();

        if self.connected {
            if let Err(err) = self.disconnect() {
                eprintln!("{err}");
            }
        }

        unsafe {
            if !self.sensor.is_null() {
                freeSensor(self.sensor);
                self.sensor = std::ptr::null_mut();
            }
        }
    }
}
