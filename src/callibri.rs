use crate::bindings::*;
use crate::signal::CallibriSignalListener;
use crate::utils::{empty_status, status_message};

pub struct CallibriSensor {
    sensor: *mut Sensor,
    signal_listener: CallibriSignalListener,
    signal_running: bool,
    connected: bool,
}

impl CallibriSensor {
    pub fn new(sensor: *mut Sensor) -> Self {
        Self {
            sensor,
            signal_listener: CallibriSignalListener::default(),
            signal_running: false,
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
}

impl Drop for CallibriSensor {
    fn drop(&mut self) {
        if self.signal_running {
            eprintln!(
                "Предупреждение: поток сигнала всё ещё активен при завершении. Попробуйте остановить его явно."
            );
        }
        self.signal_listener.unsubscribe();

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
