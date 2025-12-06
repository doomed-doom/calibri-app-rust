use crate::bindings::*;
use crate::utils::{empty_status, status_message};

const FEATURE_NAMES: &[(SensorFeature, &str)] = &[
    (SensorFeature_FeatureSignal, "Электроды (Signal)"),
    (SensorFeature_FeatureMEMS, "MEMS (акселерометр/гироскоп)"),
    (SensorFeature_FeatureCurrentStimulator, "Токовый стимулятор"),
    (SensorFeature_FeatureRespiration, "Дыхание"),
    (SensorFeature_FeatureResist, "Резистивные каналы"),
    (SensorFeature_FeatureFPG, "FPG"),
    (SensorFeature_FeatureEnvelope, "Envelope"),
    (SensorFeature_FeaturePhotoStimulator, "Фотостимулятор"),
    (
        SensorFeature_FeatureAcousticStimulator,
        "Акустический стимулятор",
    ),
    (SensorFeature_FeatureFlashCard, "Flash-память"),
    (SensorFeature_FeatureLedChannels, "LED каналы"),
    (SensorFeature_FeatureSignalWithResist, "Signal + Resist"),
];

const COMMAND_NAMES: &[(SensorCommand, &str)] = &[
    (SensorCommand_CommandStartSignal, "Запуск Signal"),
    (SensorCommand_CommandStopSignal, "Остановка Signal"),
    (SensorCommand_CommandStartResist, "Запуск Resist"),
    (SensorCommand_CommandStopResist, "Остановка Resist"),
    (SensorCommand_CommandStartMEMS, "Запуск MEMS"),
    (SensorCommand_CommandStopMEMS, "Остановка MEMS"),
    (SensorCommand_CommandStartRespiration, "Запуск Respiration"),
    (
        SensorCommand_CommandStopRespiration,
        "Остановка Respiration",
    ),
    (
        SensorCommand_CommandStartCurrentStimulation,
        "Запуск стимулятора",
    ),
    (
        SensorCommand_CommandStopCurrentStimulation,
        "Остановка стимулятора",
    ),
    (
        SensorCommand_CommandEnableMotionAssistant,
        "Включить Motion Assistant",
    ),
    (
        SensorCommand_CommandDisableMotionAssistant,
        "Выключить Motion Assistant",
    ),
    (SensorCommand_CommandFindMe, "Найти устройство"),
    (SensorCommand_CommandStartAngle, "Запуск углов"),
    (SensorCommand_CommandStopAngle, "Остановка углов"),
    (SensorCommand_CommandCalibrateMEMS, "Калибровка MEMS"),
    (SensorCommand_CommandResetQuaternion, "Сброс кватернионов"),
    (SensorCommand_CommandStartEnvelope, "Запуск Envelope"),
    (SensorCommand_CommandStopEnvelope, "Остановка Envelope"),
    (
        SensorCommand_CommandResetMotionCounter,
        "Сброс счётчика движения",
    ),
    (
        SensorCommand_CommandCalibrateStimulation,
        "Калибровка стимулятора",
    ),
    (SensorCommand_CommandIdle, "Idle"),
    (SensorCommand_CommandPowerDown, "Power Down"),
    (SensorCommand_CommandStartFPG, "Запуск FPG"),
    (SensorCommand_CommandStopFPG, "Остановка FPG"),
    (
        SensorCommand_CommandStartSignalAndResist,
        "Запуск Signal+Resist",
    ),
    (
        SensorCommand_CommandStopSignalAndResist,
        "Остановка Signal+Resist",
    ),
    (
        SensorCommand_CommandStartPhotoStimulation,
        "Запуск фотостимуляции",
    ),
    (
        SensorCommand_CommandStopPhotoStimulation,
        "Остановка фотостимуляции",
    ),
    (
        SensorCommand_CommandStartAcousticStimulation,
        "Запуск акустической стимуляции",
    ),
    (
        SensorCommand_CommandStopAcousticStimulation,
        "Остановка акустической стимуляции",
    ),
    (
        SensorCommand_CommandFileSystemEnable,
        "Включить файловую систему",
    ),
    (
        SensorCommand_CommandFileSystemDisable,
        "Отключить файловую систему",
    ),
    (
        SensorCommand_CommandFileSystemStreamClose,
        "Закрыть поток файловой системы",
    ),
    (
        SensorCommand_CommandStartCalibrateSignal,
        "Запуск калибровки Signal",
    ),
    (
        SensorCommand_CommandStopCalibrateSignal,
        "Остановка калибровки Signal",
    ),
    (SensorCommand_CommandPhotoStimEnable, "Включить PhotoStim"),
    (SensorCommand_CommandPhotoStimDisable, "Выключить PhotoStim"),
];

const PARAMETER_NAMES: &[(SensorParameter, &str)] = &[
    (SensorParameter_ParameterName, "Имя устройства"),
    (SensorParameter_ParameterState, "Состояние"),
    (SensorParameter_ParameterAddress, "BLE адрес"),
    (SensorParameter_ParameterSerialNumber, "Серийный номер"),
    (
        SensorParameter_ParameterHardwareFilterState,
        "Аппаратный фильтр",
    ),
    (SensorParameter_ParameterFirmwareMode, "Режим прошивки"),
    (
        SensorParameter_ParameterSamplingFrequency,
        "Частота дискретизации Signal",
    ),
    (SensorParameter_ParameterGain, "Усиление"),
    (SensorParameter_ParameterOffset, "Смещение"),
    (
        SensorParameter_ParameterExternalSwitchState,
        "Внешний переключатель",
    ),
    (SensorParameter_ParameterADCInputState, "Вход АЦП"),
    (
        SensorParameter_ParameterAccelerometerSens,
        "Чувствительность акселерометра",
    ),
    (
        SensorParameter_ParameterGyroscopeSens,
        "Чувствительность гироскопа",
    ),
    (
        SensorParameter_ParameterStimulatorAndMAState,
        "Состояние стимулятора / MA",
    ),
    (
        SensorParameter_ParameterStimulatorParamPack,
        "Пакет параметров стимулятора",
    ),
    (
        SensorParameter_ParameterMotionAssistantParamPack,
        "Параметры Motion Assistant",
    ),
    (SensorParameter_ParameterFirmwareVersion, "Версия прошивки"),
    (
        SensorParameter_ParameterMEMSCalibrationStatus,
        "Статус калибровки MEMS",
    ),
    (
        SensorParameter_ParameterMotionCounterParamPack,
        "Настройки счётчика движения",
    ),
    (SensorParameter_ParameterMotionCounter, "Счётчик движения"),
    (SensorParameter_ParameterBattPower, "Заряд батареи"),
    (SensorParameter_ParameterSensorFamily, "Семейство сенсора"),
    (SensorParameter_ParameterSensorMode, "Режим сенсора"),
    (SensorParameter_ParameterIrAmplitude, "IR амплитуда"),
    (SensorParameter_ParameterRedAmplitude, "RED амплитуда"),
    (
        SensorParameter_ParameterEnvelopeAvgWndSz,
        "Envelope окно усреднения",
    ),
    (
        SensorParameter_ParameterEnvelopeDecimation,
        "Envelope decimation",
    ),
    (
        SensorParameter_ParameterSamplingFrequencyResist,
        "Частота Resist",
    ),
    (
        SensorParameter_ParameterSamplingFrequencyMEMS,
        "Частота MEMS",
    ),
    (SensorParameter_ParameterSamplingFrequencyFPG, "Частота FPG"),
    (SensorParameter_ParameterAmplifier, "Усилитель"),
    (SensorParameter_ParameterSensorChannels, "Каналы сенсора"),
    (
        SensorParameter_ParameterSamplingFrequencyResp,
        "Частота Respiration",
    ),
    (SensorParameter_ParameterSurveyId, "Survey ID"),
    (
        SensorParameter_ParameterFileSystemStatus,
        "Статус файловой системы",
    ),
    (
        SensorParameter_ParameterFileSystemDiskInfo,
        "Диск файловой системы",
    ),
    (SensorParameter_ParameterReferentsShort, "Referents Short"),
    (SensorParameter_ParameterReferentsGround, "Referents Ground"),
    (
        SensorParameter_ParameterSamplingFrequencyEnvelope,
        "Частота Envelope",
    ),
    (
        SensorParameter_ParameterChannelConfiguration,
        "Конфигурация каналов",
    ),
    (
        SensorParameter_ParameterElectrodeState,
        "Состояние электродов",
    ),
    (
        SensorParameter_ParameterChannelResistConfiguration,
        "Конфиг. Resist каналов",
    ),
    (SensorParameter_ParameterBattVoltage, "Напряжение батареи"),
    (
        SensorParameter_ParameterPhotoStimTimeDefer,
        "PhotoStim задержка",
    ),
    (
        SensorParameter_ParameterPhotoStimSyncState,
        "PhotoStim sync",
    ),
    (
        SensorParameter_ParameterSensorPhotoStim,
        "PhotoStim параметры сенсора",
    ),
    (SensorParameter_ParameterStimMode, "Режим стимулятора"),
    (SensorParameter_ParameterLedChannels, "LED каналы"),
    (SensorParameter_ParameterLedState, "Состояние LED"),
];

const PARAM_ACCESS_NAMES: &[(SensorParamAccess, &str)] = &[
    (SensorParamAccess_ParamAccessRead, "только чтение"),
    (SensorParamAccess_ParamAccessReadWrite, "чтение/запись"),
    (
        SensorParamAccess_ParamAccessReadNotify,
        "чтение + уведомления",
    ),
    (SensorParamAccess_ParamAccessWrite, "только запись"),
];

pub fn describe_sensor_features(sensor_ptr: *mut Sensor) -> Result<(), String> {
    let features = collect_sensor_features(sensor_ptr)?;

    if features.is_empty() {
        println!("Сенсор не предоставляет список функций");
    } else {
        println!("Доступные функции сенсора:");
        for feature in features {
            println!("  - {} (код {})", sensor_feature_name(feature), feature);
        }
    }

    Ok(())
}

pub fn describe_sensor_commands(sensor_ptr: *mut Sensor) -> Result<(), String> {
    let commands = collect_sensor_commands(sensor_ptr)?;

    if commands.is_empty() {
        println!("Сенсор не предоставляет список команд");
    } else {
        println!("Доступные команды сенсора:");
        for command in commands {
            println!("  - {} (код {})", sensor_command_name(command), command);
        }
    }

    Ok(())
}

pub fn describe_sensor_parameters(sensor_ptr: *mut Sensor) -> Result<(), String> {
    let parameters = collect_sensor_parameters(sensor_ptr)?;

    if parameters.is_empty() {
        println!("Сенсор не предоставляет параметры");
    } else {
        println!("Доступные параметры сенсора:");
        for param in parameters {
            println!(
                "  - {} (код {}): {}",
                sensor_parameter_name(param.Param),
                param.Param,
                sensor_param_access_name(param.ParamAccess)
            );
        }
    }

    Ok(())
}

fn collect_sensor_features(sensor_ptr: *mut Sensor) -> Result<Vec<SensorFeature>, String> {
    unsafe {
        let count = getFeaturesCountSensor(sensor_ptr);
        if count <= 0 {
            return Ok(Vec::new());
        }

        let mut features: Vec<SensorFeature> = Vec::with_capacity(count as usize);
        features.set_len(count as usize);

        let mut sz_in_out = count;
        let mut status = empty_status();

        let ok = getFeaturesSensor(
            sensor_ptr,
            features.as_mut_ptr(),
            &mut sz_in_out,
            &mut status,
        ) != 0;

        if !ok || status.Success == 0 {
            features.set_len(0);
            return Err(format!(
                "Не удалось получить список функций сенсора: {}",
                status_message(&status)
            ));
        }

        let final_len = sz_in_out.clamp(0, count) as usize;
        features.truncate(final_len);
        Ok(features)
    }
}

fn collect_sensor_commands(sensor_ptr: *mut Sensor) -> Result<Vec<SensorCommand>, String> {
    unsafe {
        let count = getCommandsCountSensor(sensor_ptr);
        if count <= 0 {
            return Ok(Vec::new());
        }

        let mut commands: Vec<SensorCommand> = Vec::with_capacity(count as usize);
        commands.set_len(count as usize);

        let mut sz_in_out = count;
        let mut status = empty_status();

        let ok = getCommandsSensor(
            sensor_ptr,
            commands.as_mut_ptr(),
            &mut sz_in_out,
            &mut status,
        ) != 0;

        if !ok || status.Success == 0 {
            commands.set_len(0);
            return Err(format!(
                "Не удалось получить список команд сенсора: {}",
                status_message(&status)
            ));
        }

        let final_len = sz_in_out.clamp(0, count) as usize;
        commands.truncate(final_len);
        Ok(commands)
    }
}

fn collect_sensor_parameters(sensor_ptr: *mut Sensor) -> Result<Vec<ParameterInfo>, String> {
    unsafe {
        let count = getParametersCountSensor(sensor_ptr);
        if count <= 0 {
            return Ok(Vec::new());
        }

        let mut parameters: Vec<ParameterInfo> = Vec::with_capacity(count as usize);
        parameters.set_len(count as usize);

        let mut sz_in_out = count;
        let mut status = empty_status();

        let ok = getParametersSensor(
            sensor_ptr,
            parameters.as_mut_ptr(),
            &mut sz_in_out,
            &mut status,
        ) != 0;

        if !ok || status.Success == 0 {
            parameters.set_len(0);
            return Err(format!(
                "Не удалось получить список параметров сенсора: {}",
                status_message(&status)
            ));
        }

        let final_len = sz_in_out.clamp(0, count) as usize;
        parameters.truncate(final_len);
        Ok(parameters)
    }
}

fn sensor_feature_name(feature: SensorFeature) -> &'static str {
    FEATURE_NAMES
        .iter()
        .find(|(code, _)| *code == feature)
        .map(|(_, name)| *name)
        .unwrap_or("Неизвестная функция")
}

fn sensor_command_name(command: SensorCommand) -> &'static str {
    COMMAND_NAMES
        .iter()
        .find(|(code, _)| *code == command)
        .map(|(_, name)| *name)
        .unwrap_or("Неизвестная команда")
}

fn sensor_parameter_name(parameter: SensorParameter) -> &'static str {
    PARAMETER_NAMES
        .iter()
        .find(|(code, _)| *code == parameter)
        .map(|(_, name)| *name)
        .unwrap_or("Неизвестный параметр")
}

fn sensor_param_access_name(access: SensorParamAccess) -> &'static str {
    PARAM_ACCESS_NAMES
        .iter()
        .find(|(code, _)| *code == access)
        .map(|(_, name)| *name)
        .unwrap_or("неизвестный доступ")
}
