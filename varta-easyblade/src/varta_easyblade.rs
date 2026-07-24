use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Debug, Clone)]
pub struct DeviceConfigInfo {
    pub config_1: String,
    pub config_2: String,
    pub config_3: String,
}

#[derive(Debug, Clone)]
pub struct DeviceSerialNumberInfo {
    pub serial_part_1: u32,
    pub serial_part_2: u32,
    pub serial_part_3: u32,
}

#[derive(Debug, Clone)]
pub struct DeviceDateInfo {
    pub year: u16,
    pub month: u16,
    pub day: u16,
}

#[derive(Debug, Clone)]
pub struct DeviceVariantInfo {
    pub variant_1: u8,
    pub variant_2: u8,
    pub variant_3: u8,
    pub variant_4: u16,
    pub variant_5: u16,
    pub variant_6: u16,
    pub variant_7: u8,
}

#[derive(Debug, Clone)]
pub struct DeviceOperationTime {
    pub minutes_below_zero: u8,
    pub minutes_zero_to_40: u8,
    pub minutes_40_to_60: u8,
    pub minutes_60_to_80: u8,
    pub minutes_above_80: u8,
    pub hours_below_zero: u32,
    pub hours_zero_to_40: u32,
    pub hours_40_to_60: u32,
    pub hours_60_to_80: u32,
    pub hours_above_80: u32,
}

#[derive(Debug, Clone)]
pub struct CellVoltageMinMax {
    pub min_voltage_v: f32,
    pub max_voltage_v: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct CellVoltageLimit {
    pub over_voltage_error_v: f32,
}

#[derive(Debug, Clone)]
pub struct BatteryVoltage {
    pub sum_of_cell_voltage_v: f32,
    pub internal_connector_voltage_v: f32,
    pub external_connector_voltage_v: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct BatteryVoltageLimit {
    pub internal_external_min_delta_v: f32,
}

#[derive(Debug, Clone)]
pub struct BatteryCurrent {
    pub fast_current_a: f32,
    pub weighted_avg_current_a: f32,
    pub integrated_current_a: f32,
    pub average_1s_current_a: f32,
    pub average_10s_current_a: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct BatteryCurrentLimit {
    pub discharge_sc_error_a: f32,
}

#[derive(Debug, Clone)]
pub struct FetTemperature {
    pub temperature_1_c: f32,
    pub temperature_2_c: f32,
}

#[derive(Debug, Clone)]
pub struct FetTemperatureMinMax {
    pub min_temperature_c: f32,
    pub max_temperature_c: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct FetTemperatureLimit {
    pub discharge_over_temp_c: f32,
}

#[derive(Debug, Clone)]
pub struct CellTemperature {
    pub temperature_1_c: f32,
    pub temperature_2_c: f32,
    pub temperature_3_c: f32,
    pub temperature_4_c: f32,
    pub temperature_5_c: f32,
    pub temperature_6_c: f32,
}

#[derive(Debug, Clone)]
pub struct CellTemperatureMinMax {
    pub min_temperature_c: f32,
    pub max_temperature_c: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct CellTemperatureLimit {
    pub discharge_over_temp_c: f32,
}

#[derive(Debug, Clone)]
pub struct CellBalanceStatus {
    pub balance_status_register: u16,
    pub balance_fet_active: u16,
    pub balance_fet_active_persistent: u16,
}

#[derive(Debug, Clone, Copy)]
pub struct CellBalanceLimit {
    pub balance_start_diff_voltage_v: f32,
}

#[derive(Debug, Clone)]
pub struct CellImpedance {
    pub cell_impedances_mohm: [u16; 16],
    pub low_temp_factor: u16,
    pub high_temp_factor: u16,
}

#[derive(Debug, Clone)]
pub struct BatteryCapacity {
    pub design_capacity_ah: f32,
    pub full_charge_capacity_ah: f32,
    pub remaining_capacity_ah: f32,
    pub soc_percent: f32,
    pub soh_percent: f32,
    pub total_discharged_capacity_ah: f32,
    pub total_charged_capacity_ah: f32,
}

#[derive(Debug, Clone)]
pub struct BatteryCycleCount {
    pub discharge_cycles: u32,
    pub discharge_learning_cycles: u32,
    pub discharge_cycles_after_learning: u32,
    pub charge_cycles_completed: u32,
    pub charge_cycles_started: u32,
    pub discharge_use_detect: u32,
    pub charge_use_low_temperature: u32,
    pub charge_use_normal_temperature: u32,
    pub charge_use_high_temperature: u32,
}

#[derive(Debug, Clone)]
pub struct BatteryChargeVoltage {
    pub charge_voltage_valid_v: f32,
    pub charge_max_voltage_v: f32,
    pub charge_keep_power_voltage_v: f32,
}

#[derive(Debug, Clone)]
pub struct BatteryChargeCurrent {
    pub charge_current_valid_a: f32,
    pub charge_max_current_n_a: f32,
    pub charge_max_current_low_a: f32,
    pub charge_max_current_high_a: f32,
    pub charge_keep_power_current_a: f32,
    pub charge_temp_min_low_c: f32,
    pub charge_temp_min_normal_c: f32,
    pub charge_temp_max_normal_c: f32,
    pub charge_temp_max_high_c: f32,
    pub charge_current_config: u32,
}

#[derive(Debug, Clone)]
pub struct BatteryChargeTemperature {
    pub temp_min_low_c: f32,
    pub temp_min_normal_c: f32,
    pub temp_max_normal_c: f32,
    pub temp_max_high_c: f32,
    pub temp_5_c: f32,
    pub temp_6_c: f32,
}

#[derive(Debug, Clone)]
pub struct MasterBatteryTemperature {
    pub max_fet_temperature_c: f32,
    pub max_cell_temperature_c: f32,
}

#[allow(clippy::type_complexity)]
pub enum SdoRequest {
    SerialNumber,
    SoftwareVersion,
    HardwareVersion,
    DeviceErrorHistory,
    CellVoltages,
    DeviceConfigInfo,
    DeviceSerialNumberInfo,
    DeviceDateInfo,
    DeviceVariantInfo,
    DeviceControlParam,
    DeviceOperationTime,
    DeviceErrorCounter,
    CellVoltageMinMax,
    CellVoltageLimit,
    BatteryVoltage,
    BatteryVoltageLimit,
    BatteryCurrent,
    BatteryCurrentLimit,
    FetTemperature,
    FetTemperatureMinMax,
    FetTemperatureLimit,
    CellTemperature,
    CellTemperatureMinMax,
    CellTemperatureLimit,
    CellBalanceStatus,
    CellBalanceLimit,
    CellImpedance,
    BatteryCapacity,
    BatteryCapacityParam,
    BatteryCycleCount,
    BatteryChargeVoltage,
    BatteryChargeCurrent,
    BatteryChargeTemperature,
    MasterBatteryTemperature,
}

pub enum SdoResponse {
    SerialNumber { node_id: u8, value: Result<u16, String> },
    SoftwareVersion { node_id: u8, value: Result<String, String> },
    HardwareVersion { node_id: u8, value: Result<String, String> },
    DeviceErrorHistory { node_id: u8, value: Result<Vec<DeviceError>, String> },
    CellVoltages { node_id: u8, value: Result<Vec<f32>, String> },
    DeviceConfigInfo { node_id: u8, value: Result<DeviceConfigInfo, String> },
    DeviceSerialNumberInfo { node_id: u8, value: Result<DeviceSerialNumberInfo, String> },
    DeviceDateInfo { node_id: u8, value: Result<DeviceDateInfo, String> },
    DeviceVariantInfo { node_id: u8, value: Result<DeviceVariantInfo, String> },
    DeviceControlParam { node_id: u8, value: Result<u16, String> },
    DeviceOperationTime { node_id: u8, value: Result<DeviceOperationTime, String> },
    DeviceErrorCounter { node_id: u8, value: Result<Vec<u16>, String> },
    CellVoltageMinMax { node_id: u8, value: Result<CellVoltageMinMax, String> },
    CellVoltageLimit { node_id: u8, value: Result<CellVoltageLimit, String> },
    BatteryVoltage { node_id: u8, value: Result<BatteryVoltage, String> },
    BatteryVoltageLimit { node_id: u8, value: Result<BatteryVoltageLimit, String> },
    BatteryCurrent { node_id: u8, value: Result<BatteryCurrent, String> },
    BatteryCurrentLimit { node_id: u8, value: Result<BatteryCurrentLimit, String> },
    FetTemperature { node_id: u8, value: Result<FetTemperature, String> },
    FetTemperatureMinMax { node_id: u8, value: Result<FetTemperatureMinMax, String> },
    FetTemperatureLimit { node_id: u8, value: Result<FetTemperatureLimit, String> },
    CellTemperature { node_id: u8, value: Result<CellTemperature, String> },
    CellTemperatureMinMax { node_id: u8, value: Result<CellTemperatureMinMax, String> },
    CellTemperatureLimit { node_id: u8, value: Result<CellTemperatureLimit, String> },
    CellBalanceStatus { node_id: u8, value: Result<CellBalanceStatus, String> },
    CellBalanceLimit { node_id: u8, value: Result<CellBalanceLimit, String> },
    CellImpedance { node_id: u8, value: Result<CellImpedance, String> },
    BatteryCapacity { node_id: u8, value: Result<BatteryCapacity, String> },
    BatteryCapacityParam { node_id: u8, value: Result<u8, String> },
    BatteryCycleCount { node_id: u8, value: Result<BatteryCycleCount, String> },
    BatteryChargeVoltage { node_id: u8, value: Result<BatteryChargeVoltage, String> },
    BatteryChargeCurrent { node_id: u8, value: Result<BatteryChargeCurrent, String> },
    BatteryChargeTemperature { node_id: u8, value: Result<BatteryChargeTemperature, String> },
    MasterBatteryTemperature { node_id: u8, value: Result<MasterBatteryTemperature, String> },
}

#[derive(Debug, Clone, Default)]
pub struct MasterInfo {
    pub voltage: Option<f32>,
    pub current: Option<f32>,
    pub soc: Option<f32>,
    pub charge_voltage_request: Option<f32>,
    pub charge_current_request: Option<f32>,
    pub battery_status: Option<u8>,
    pub max_battery_fet_temp: Option<f32>,
    pub max_battery_cell_temp: Option<f32>,
    pub master_design_capacity: Option<f32>,
    pub master_full_charge_capacity: Option<f32>,
    pub master_remaining_capacity: Option<f32>,
    pub last_seen: Option<std::time::SystemTime>,
}

#[derive(Debug)]
pub struct VartaEasyblade {
    pub node_id: u8,
    pub serial_number: Option<u16>,
    pub software_version: Option<String>,
    pub hardware_version: Option<String>,
    pub last_seen: std::time::SystemTime,
    pub voltage: Option<f32>,
    pub current: Option<f32>,
    pub soc: Option<f32>,
    pub soh: Option<f32>,
    pub cell_voltages: Option<Vec<f32>>,
    pub device_errors: Option<Vec<DeviceError>>,
    pub fet_status: Option<(bool, bool, bool)>,
    pub device_config_info: Option<DeviceConfigInfo>,
    pub device_serial_number_info: Option<DeviceSerialNumberInfo>,
    pub device_date_info: Option<DeviceDateInfo>,
    pub device_variant_info: Option<DeviceVariantInfo>,
    pub device_control_param: Option<u16>,
    pub device_operation_time: Option<DeviceOperationTime>,
    pub device_error_counter: Option<Vec<u16>>,
    pub cell_voltage_min_max: Option<CellVoltageMinMax>,
    pub cell_voltage_limit: Option<CellVoltageLimit>,
    pub battery_voltage: Option<BatteryVoltage>,
    pub battery_voltage_limit: Option<BatteryVoltageLimit>,
    pub battery_current: Option<BatteryCurrent>,
    pub battery_current_limit: Option<BatteryCurrentLimit>,
    pub fet_temperature: Option<FetTemperature>,
    pub fet_temperature_min_max: Option<FetTemperatureMinMax>,
    pub fet_temperature_limit: Option<FetTemperatureLimit>,
    pub cell_temperature: Option<CellTemperature>,
    pub cell_temperature_min_max: Option<CellTemperatureMinMax>,
    pub cell_temperature_limit: Option<CellTemperatureLimit>,
    pub cell_balance_status: Option<CellBalanceStatus>,
    pub cell_balance_limit: Option<CellBalanceLimit>,
    pub cell_impedance: Option<CellImpedance>,
    pub battery_capacity: Option<BatteryCapacity>,
    pub battery_capacity_param: Option<u8>,
    pub battery_cycle_count: Option<BatteryCycleCount>,
    pub battery_charge_voltage: Option<BatteryChargeVoltage>,
    pub battery_charge_current: Option<BatteryChargeCurrent>,
    pub battery_charge_temperature: Option<BatteryChargeTemperature>,
    pub master_battery_temperature: Option<MasterBatteryTemperature>,
}

/// Device Error values logged by the Varta Easyblade module in SDO 0x2018
/// (DeviceError History Values).
#[repr(u8)]
#[derive(Debug, PartialEq, IntoPrimitive, TryFromPrimitive)]
pub enum DeviceError {
    None = 0x00,

    CellOverTempWhileCharging = 0x01,
    CellUnderTempWhileCharging = 0x02,
    ChargeFetOverTempWhileCharging = 0x03,
    CellOverTempWhileDischarging = 0x04,
    CellUnderTempWhileDischarging = 0x05,
    DischargeFetOverTempWhileDischarging = 0x06,
    NotUsed0x07 = 0x07,
    CellOverVoltageWhileCharging = 0x08,
    CellUnderVoltageWhileDischarging = 0x09,
    CellVoltageTooLow = 0x0a,
    SevereCellUnbalance = 0x0b,
    NotUsed0x0c = 0x0c,
    PackOverVoltage = 0x0d,
    NotUsed0x0e = 0x0e,
    VoltageSumDifference = 0x0f,
    ModuleUnderVoltage = 0x10,
    NotUsed0x11 = 0x11,
    NotUsed0x12 = 0x12,
    NotUsed0x13 = 0x13,
    NotUsed0x14 = 0x14,
    ShortCircuitWhileCharging = 0x15,
    OverCurrent65AWhileCharging = 0x16,
    OverCurrent85AWhileCharging = 0x17,
    NotUsed0x18 = 0x18,
    ShortCircuitWhileDischarging = 0x19,
    OverCurrent65AWhileDischarging = 0x1a,
    OverCurrent85AWhileDischarging = 0x1b,
    OverCurrentWhileDischarging = 0x1c,
    CurrentMeasurementError = 0x1d,
    NotUsed0x1e = 0x1e,
    NotUsed0x1f = 0x1f,
    NotUsed0x20 = 0x20,
    NotUsed0x21 = 0x21,
    NotUsed0x22 = 0x22,
    NotUsed0x23 = 0x23,
    NotUsed0x24 = 0x24,
    NotUsed0x25 = 0x25,
    NotUsed0x26 = 0x26,
    NotUsed0x27 = 0x27,
    NotUsed0x28 = 0x28,
    AdcMinScale = 0x29,
    AdcMaxScale = 0x2a,
    CellTempSensorHighFailure = 0x2b,
    CellTempSensorLowFailure = 0x2c,
    FetTempSensorHighFailure = 0x2d,
    FetTempSensorLowFailure = 0x2e,
    NotUsed0x2f = 0x2f,
    NotUsed0x30 = 0x30,
    NotUsed0x31 = 0x31,
    CurrentSensorHighFailure = 0x32,
    NotUsed0x33 = 0x33,
    CurrentSensorHighFailureWhileDischarging = 0x34,
    DischargeFetShorted = 0x35,
    ChargeFetShorted = 0x36,
    NotUsed0x37 = 0x37,
    TempDischargeErrorLock = 0x38,
    TempChargeErrorLock = 0x39,
    OverCurrentWhileRecuperating = 0x3a,
    CellOverVoltageWhileRecuperating = 0x3b,
    V24UnderVoltage = 0x3c,
    V24OverVoltage = 0x3d,
    CanNodeIdNotAssigned = 0x3e,
    CanNodeIdDuplicate = 0x3f,
    ParameterConfigError = 0x40,
    AnalogFrontEndCommunicationErorr = 0x41,
    AnalogFrontEndSelftestError = 0x42,
    AnalogFrontEndFullScaleError = 0x43,
    TempMuxSelftestError = 0x44,

    Unknown = 0xff,
}
