use crate::core::bindings::*;
use crate::core::utils::{empty_status, status_message};
use std::ffi::CStr;
use std::os::raw::c_char;

const DEFAULT_HW_FILTER_CAPACITY: usize = 16;
const SENSOR_NAME_CAP: usize = SENSOR_NAME_LEN as usize;
const SENSOR_ADDRESS_CAP: usize = SENSOR_ADR_LEN as usize;
const SENSOR_SERIAL_CAP: usize = SENSOR_SN_LEN as usize;

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

#[allow(dead_code)]
pub fn describe_sensor_features(sensor_ptr: *mut Sensor) -> Result<(), String> {
    let features = collect_sensor_features(sensor_ptr)?;

    if features.is_empty() {
        println!("Сенсор не предоставляет список функций");
    } else {
        println!("Доступные функции сенсора:");
        for feature in features {
            println!("  - {}", sensor_feature_name(feature));
        }
    }

    Ok(())
}

#[allow(dead_code)]
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

#[allow(dead_code)]
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
            if let Err(err) = print_parameter_value(sensor_ptr, param.Param) {
                eprintln!("    {}", err);
            }
        }
    }

    Ok(())
}

pub fn gather_sensor_details(sensor_ptr: *mut Sensor) -> Result<SensorDetails, String> {
    let features = collect_sensor_features(sensor_ptr)?
        .into_iter()
        .map(|feature| sensor_feature_name(feature).to_string())
        .collect();

    let commands = collect_sensor_commands(sensor_ptr)?
        .into_iter()
        .map(|command| CommandDetail {
            code: command as i32,
            name: sensor_command_name(command).to_string(),
        })
        .collect();

    let mut parameters = Vec::new();
    for param in collect_sensor_parameters(sensor_ptr)? {
        let name = sensor_parameter_name(param.Param).to_string();
        let access = sensor_param_access_name(param.ParamAccess).to_string();
        let description = match parameter_value_description(sensor_ptr, param.Param) {
            Ok(desc) => desc,
            Err(err) => ParameterValueDescription {
                lines: vec![format!("Не удалось прочитать значение: {err}")],
                bullets: Vec::new(),
            },
        };

        parameters.push(ParameterDetail {
            code: param.Param as i32,
            name,
            access,
            lines: description.lines,
            bullets: description.bullets,
        });
    }

    Ok(SensorDetails {
        features,
        commands,
        parameters,
    })
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

#[allow(non_upper_case_globals)]
fn parameter_value_description(
    sensor_ptr: *mut Sensor,
    parameter: SensorParameter,
) -> Result<ParameterValueDescription, String> {
    let mut desc = ParameterValueDescription::default();
    match parameter {
        SensorParameter_ParameterName => {
            let name = read_sensor_string(sensor_ptr, SENSOR_NAME_CAP, readNameSensor)?;
            desc.lines.push(format!("Имя устройства: {name}"));
        }
        SensorParameter_ParameterState => {
            let state = read_sensor_state(sensor_ptr)?;
            desc.lines
                .push(format!("Состояние: {}", sensor_state_name(state)));
        }
        SensorParameter_ParameterAddress => {
            let addr = read_sensor_string(sensor_ptr, SENSOR_ADDRESS_CAP, readAddressSensor)?;
            desc.lines.push(format!("Адрес: {addr}"));
        }
        SensorParameter_ParameterSerialNumber => {
            let serial = read_sensor_string(sensor_ptr, SENSOR_SERIAL_CAP, readSerialNumberSensor)?;
            desc.lines.push(format!("Серийный номер: {serial}"));
        }
        SensorParameter_ParameterFirmwareMode => {
            let mode = read_firmware_mode(sensor_ptr)?;
            desc.lines
                .push(format!("Режим прошивки: {}", firmware_mode_name(mode)));
        }
        SensorParameter_ParameterHardwareFilterState => {
            let filters = read_hardware_filters(sensor_ptr)?;
            if filters.is_empty() {
                desc.lines.push("Аппаратные фильтры: отключены".into());
            } else {
                desc.lines.push("Аппаратные фильтры:".into());
                desc.bullets
                    .extend(filters.into_iter().map(|f| sensor_filter_name(f).to_string()));
            }
        }
        SensorParameter_ParameterSamplingFrequency => {
            let freq = read_sampling_frequency(sensor_ptr)?;
            desc.lines.push(format!(
                "Частота дискретизации: {}",
                sampling_frequency_name(freq)
            ));
        }
        _ => {}
    }

    Ok(desc)
}

#[allow(dead_code)]
fn print_parameter_value(
    sensor_ptr: *mut Sensor,
    parameter: SensorParameter,
) -> Result<(), String> {
    let description = parameter_value_description(sensor_ptr, parameter)?;
    for line in description.lines {
        println!("    {line}");
    }
    for bullet in description.bullets {
        println!("      • {bullet}");
    }
    Ok(())
}

fn read_sensor_string(
    sensor_ptr: *mut Sensor,
    capacity: usize,
    reader: unsafe extern "C" fn(*mut Sensor, *mut c_char, i32, *mut OpStatus) -> u8,
) -> Result<String, String> {
    unsafe {
        let mut buffer = vec![0i8; capacity];
        let mut status = empty_status();
        let ok = reader(
            sensor_ptr,
            buffer.as_mut_ptr(),
            capacity as i32,
            &mut status,
        ) != 0;

        if !ok || status.Success == 0 {
            return Err(format!(
                "Не удалось прочитать строковый параметр: {}",
                status_message(&status)
            ));
        }

        Ok(CStr::from_ptr(buffer.as_ptr())
            .to_string_lossy()
            .into_owned())
    }
}

fn read_sensor_state(sensor_ptr: *mut Sensor) -> Result<SensorState, String> {
    unsafe {
        let mut state = SensorState_StateOutOfRange;
        let mut status = empty_status();
        let ok = readStateSensor(sensor_ptr, &mut state, &mut status) != 0;
        if ok && status.Success != 0 {
            Ok(state)
        } else {
            Err(format!(
                "Не удалось прочитать состояние сенсора: {}",
                status_message(&status)
            ))
        }
    }
}

fn read_firmware_mode(sensor_ptr: *mut Sensor) -> Result<SensorFirmwareMode, String> {
    unsafe {
        let mut mode = SensorFirmwareMode_ModeApplication;
        let mut status = empty_status();
        let ok = readFirmwareModeSensor(sensor_ptr, &mut mode, &mut status) != 0;
        if ok && status.Success != 0 {
            Ok(mode)
        } else {
            Err(format!(
                "Не удалось прочитать режим прошивки: {}",
                status_message(&status)
            ))
        }
    }
}

fn read_hardware_filters(sensor_ptr: *mut Sensor) -> Result<Vec<SensorFilter>, String> {
    unsafe {
        let mut count = DEFAULT_HW_FILTER_CAPACITY as i32;
        let mut filters: Vec<SensorFilter> =
            vec![SensorFilter_FilterUnknown; DEFAULT_HW_FILTER_CAPACITY];

        let mut status = empty_status();
        let ok =
            readHardwareFiltersSensor(sensor_ptr, filters.as_mut_ptr(), &mut count, &mut status)
                != 0;

        if !ok || status.Success == 0 {
            return Err(format!(
                "Не удалось прочитать аппаратные фильтры: {}",
                status_message(&status)
            ));
        }

        let len = count.clamp(0, filters.len() as i32) as usize;
        filters.truncate(len);
        filters.retain(|f| *f != SensorFilter_FilterUnknown);
        Ok(filters)
    }
}

fn read_sampling_frequency(sensor_ptr: *mut Sensor) -> Result<SensorSamplingFrequency, String> {
    unsafe {
        let mut freq = SensorSamplingFrequency_FrequencyHz125;
        let mut status = empty_status();
        let ok = readSamplingFrequencySensor(sensor_ptr, &mut freq, &mut status) != 0;
        if ok && status.Success != 0 {
            Ok(freq)
        } else {
            Err(format!(
                "Не удалось прочитать частоту дискретизации: {}",
                status_message(&status)
            ))
        }
    }
}

#[allow(non_upper_case_globals)]
fn sensor_state_name(state: SensorState) -> &'static str {
    match state {
        SensorState_StateInRange => "InRange (устройство подключено)",
        SensorState_StateOutOfRange => "OutOfRange (устройство выключено или вне зоны)",
        _ => "Неизвестное состояние",
    }
}

#[allow(non_upper_case_globals)]
fn firmware_mode_name(mode: SensorFirmwareMode) -> &'static str {
    match mode {
        SensorFirmwareMode_ModeBootloader => "Bootloader",
        SensorFirmwareMode_ModeApplication => "Application",
        _ => "Неизвестный режим",
    }
}

#[allow(non_upper_case_globals)]
fn sensor_filter_name(filter: SensorFilter) -> &'static str {
    match filter {
        SensorFilter_FilterHPFBwhLvl1CutoffFreq1Hz => "HPF 1 Гц",
        SensorFilter_FilterHPFBwhLvl1CutoffFreq5Hz => "HPF 5 Гц",
        SensorFilter_FilterBSFBwhLvl2CutoffFreq45_55Hz => "Notch 50 Гц",
        SensorFilter_FilterBSFBwhLvl2CutoffFreq55_65Hz => "Notch 60 Гц",
        SensorFilter_FilterHPFBwhLvl2CutoffFreq10Hz => "HPF 10 Гц",
        SensorFilter_FilterLPFBwhLvl2CutoffFreq400Hz => "LPF 400 Гц",
        SensorFilter_FilterHPFBwhLvl2CutoffFreq80Hz => "HPF 80 Гц",
        _ => "Неизвестный фильтр",
    }
}

#[allow(non_upper_case_globals)]
fn sampling_frequency_name(freq: SensorSamplingFrequency) -> &'static str {
    match freq {
        SensorSamplingFrequency_FrequencyHz125 => "125 Гц",
        SensorSamplingFrequency_FrequencyHz250 => "250 Гц",
        SensorSamplingFrequency_FrequencyHz500 => "500 Гц",
        SensorSamplingFrequency_FrequencyHz1000 => "1000 Гц",
        SensorSamplingFrequency_FrequencyHz2000 => "2000 Гц",
        SensorSamplingFrequency_FrequencyHz20 => "20 Гц",
        SensorSamplingFrequency_FrequencyHz100 => "100 Гц",
        SensorSamplingFrequency_FrequencyHz10 => "10 Гц",
        SensorSamplingFrequency_FrequencyHz4000 => "4000 Гц",
        SensorSamplingFrequency_FrequencyHz8000 => "8000 Гц",
        SensorSamplingFrequency_FrequencyHz10000 => "10000 Гц",
        SensorSamplingFrequency_FrequencyHz12000 => "12000 Гц",
        SensorSamplingFrequency_FrequencyHz16000 => "16000 Гц",
        SensorSamplingFrequency_FrequencyHz24000 => "24000 Гц",
        SensorSamplingFrequency_FrequencyHz32000 => "32000 Гц",
        SensorSamplingFrequency_FrequencyHz48000 => "48000 Гц",
        SensorSamplingFrequency_FrequencyHz64000 => "64000 Гц",
        _ => "Неизвестная частота",
    }
}

#[derive(Clone, Debug, Default)]
pub struct SensorDetails {
    pub features: Vec<String>,
    pub commands: Vec<CommandDetail>,
    pub parameters: Vec<ParameterDetail>,
}

#[derive(Clone, Debug)]
pub struct CommandDetail {
    pub code: i32,
    pub name: String,
}

#[derive(Clone, Debug)]
pub struct ParameterDetail {
    pub code: i32,
    pub name: String,
    pub access: String,
    pub lines: Vec<String>,
    pub bullets: Vec<String>,
}

#[derive(Default)]
struct ParameterValueDescription {
    lines: Vec<String>,
    bullets: Vec<String>,
}
