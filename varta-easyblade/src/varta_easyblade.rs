use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Debug, Clone, serde::Serialize)]
pub struct DeviceConfigInfo {
    pub config_1: String,
    pub config_2: String,
    pub config_3: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct DeviceSerialNumberInfo {
    pub serial_part_1: u32,
    pub serial_part_2: u32,
    pub serial_part_3: u32,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct DeviceDateInfo {
    pub year: u16,
    pub month: u16,
    pub day: u16,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct DeviceVariantInfo {
    pub variant_1: u8,
    pub variant_2: u8,
    pub variant_3: u8,
    pub variant_4: u16,
    pub variant_5: u16,
    pub variant_6: u16,
    pub variant_7: u8,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct DeviceErrorCounterInfo {
    pub over_temp_laden_zellen: u16,
    pub under_temp_laden_zellen: u16,
    pub over_temp_laden_fet: u16,
    pub over_temp_entladen_zellen: u16,
    pub under_temp_entladen_zellen: u16,
    pub over_temp_entladen_fet: u16,
    pub over_temp_clamp: u16,
    pub over_voltage: u16,
    pub under_voltage: u16,
    pub deep_low_voltage: u16,
    pub cell_disbalance: u16,
    pub akku_pack_spn_min_error: u16,
    pub akku_pack_spn_max_alarm: u16,
    pub akku_pack_fused_spn_diff_error: u16,
    pub akku_pwr_spn_diff_error: u16,
    pub akku_pwr_spn_min_error: u16,
    pub akku_pwr_spn_max_error: u16,
    pub akku_netz_spn_min_error: u16,
    pub akku_netz_spn_max_alarm: u16,
    pub akku_rekuperation_spn_max_alarm: u16,
    pub i_charge_sc: u16,
    pub i_charge_occ_1: u16,
    pub i_charge_occ_2: u16,
    pub i_charge_occ_3: u16,
    pub i_discharge_sc: u16,
    pub i_discharge_ocd_1: u16,
    pub i_discharge_ocd_2: u16,
    pub i_discharge_ocd_3: u16,
    pub i_akku_diff_error: u16,
    pub scnd_spn_min_error: u16,
    pub scnd_spn_max_error: u16,
    pub scnd_uc_fet_enable: u16,
    pub scnd_current_sense_ein: u16,
    pub scnd_voltage_sense_ein: u16,
    pub scnd_temp_cell_sense_ein: u16,
    pub scnd_temp_fet_sense_ein: u16,
    pub scnd_pyro_fuse_eject_sense_ein: u16,
    pub scnd_i_discharge_fet_error: u16,
    pub scnd_i_charge_fet_error: u16,
    pub scnd_voltage_error: u16,
    pub adc_spn_min_scale: u16,
    pub adc_spn_max_scale: u16,
    pub adc_temp_zellen_min_scale: u16,
    pub adc_temp_zellen_max_scale: u16,
    pub adc_temp_fet_min_scale: u16,
    pub adc_temp_fet_max_scale: u16,
    pub adc_temp_clamp_min_scale: u16,
    pub adc_temp_clamp_max_scale: u16,
    pub adc_i_charge_min_scale: u16,
    pub adc_i_charge_max_scale: u16,
    pub adc_i_discharge_min_scale: u16,
    pub adc_i_discharge_max_scale: u16,
    pub i_discharge_fet_error: u16,
    pub i_charge_fet_error: u16,
    pub i_discharge_charge_fet_error: u16,
    pub temp_discharge_error_lock: u16,
    pub temp_charge_error_lock: u16,
    pub over_charge_current_alarm_recuperation: u16,
    pub over_charge_cell_voltage_alarm_recuperation: u16,
    pub v24_spn_min_error: u16,
    pub v24_spn_max_error: u16,
    pub can_network_not_conf_node_id: u16,
    pub can_network_double_node_id: u16,
    pub parameter_configuration_error: u16,
    pub reserved_01: u16,
    pub reserved_02: u16,
    pub reserved_03: u16,
    pub reserved_04: u16,
    pub reserved_05: u16,
    pub reserved_06: u16,
    pub reserved_07: u16,
    pub reserved_08: u16,
    pub reserved_09: u16,
    pub reserved_0a: u16,
    pub reserved_0b: u16,
    pub reserved_0c: u16,
    pub reserved_0d: u16,
    pub reserved_0e: u16,
    pub reserved_0f: u16,
}

#[derive(Debug, Clone, serde::Serialize)]
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

#[derive(Debug, Clone, serde::Serialize)]
pub struct CellVoltageMinMax {
    pub min_voltage_v: f32,
    pub max_voltage_v: f32,
}

#[derive(Debug, Clone, Copy, serde::Serialize)]
pub struct CellVoltageLimit {
    /// Single Cell Over Voltage Error (0x2104 sub1)
    pub over_voltage_error_v: f32,
    /// Single Cell Max Charge Voltage (0x2104 sub2)
    pub max_charge_voltage_v: f32,
    /// Single Cell Fully Charged Voltage (0x2104 sub3)
    pub fully_charged_voltage_v: f32,
    /// Single Cell Near Fully Charged Voltage (0x2104 sub4)
    pub near_fully_charged_voltage_v: f32,
    /// Single Cell Fully Charged Reset Voltage (0x2104 sub5)
    pub fully_charged_reset_voltage_v: f32,
    /// Single Cell EDV Reset Voltage (0x2104 sub6)
    pub edv_reset_voltage_v: f32,
    /// Single Cell Near Empty Voltage Warning (0x2104 sub7)
    pub near_empty_voltage_warning_v: f32,
    /// Single Cell Near Empty Voltage EDV1 (0x2104 sub8)
    pub near_empty_voltage_edv1_v: f32,
    /// Single Cell Empty Voltage EDV0 (0x2104 sub9)
    pub empty_voltage_edv0_v: f32,
    /// Single Cell Min Error Reset Voltage (0x2104 subA)
    pub min_error_reset_voltage_v: f32,
    /// Single Cell EDV OFF Voltage (0x2104 subB)
    pub edv_off_voltage_v: f32,
    /// Single Cell Under Voltage Error (0x2104 subC)
    pub under_voltage_error_v: f32,
    /// Single Cell Deep Low Voltage Error (0x2104 subD)
    pub deep_low_voltage_error_v: f32,
    /// Single Cell Max Charge Voltage - no Password (0x2104 subE)
    pub max_charge_voltage_no_password_v: f32,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct BatteryVoltage {
    pub sum_of_cell_voltage_v: f32,
    pub internal_connector_voltage_v: f32,
    pub external_connector_voltage_v: f32,
}

#[derive(Debug, Clone, Copy, serde::Serialize)]
pub struct BatteryVoltageLimit {
    pub internal_external_min_delta_v: f32,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct BatteryCurrent {
    pub fast_current_a: f32,
    pub weighted_avg_current_a: f32,
    pub integrated_current_a: f32,
    pub average_1s_current_a: f32,
    pub average_10s_current_a: f32,
}

#[derive(Debug, Clone, Copy, serde::Serialize)]
pub struct BatteryCurrentLimit {
    pub discharge_sc_error_a: f32,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct FetTemperature {
    pub temperature_1_c: f32,
    pub temperature_2_c: f32,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct FetTemperatureMinMax {
    pub min_temperature_c: f32,
    pub max_temperature_c: f32,
}

#[derive(Debug, Clone, Copy, serde::Serialize)]
pub struct FetTemperatureLimit {
    pub discharge_over_temp_c: f32,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct CellTemperature {
    pub temperature_1_c: f32,
    pub temperature_2_c: f32,
    pub temperature_3_c: f32,
    pub temperature_4_c: f32,
    pub temperature_5_c: f32,
    pub temperature_6_c: f32,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct CellTemperatureMinMax {
    pub min_temperature_c: f32,
    pub max_temperature_c: f32,
}

#[derive(Debug, Clone, Copy, serde::Serialize)]
pub struct CellTemperatureLimit {
    pub discharge_over_temp_c: f32,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct CellBalanceStatus {
    pub balance_status_register: u16,
    pub balance_fet_active: u16,
    pub balance_fet_active_persistent: u16,
}

#[derive(Debug, Clone, Copy, serde::Serialize)]
pub struct CellBalanceLimit {
    pub balance_start_diff_voltage_v: f32,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct CellImpedance {
    pub cell_impedances_mohm: [u16; 16],
    pub low_temp_factor: u16,
    pub high_temp_factor: u16,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct BatteryCapacity {
    pub design_capacity_ah: f32,
    pub full_charge_capacity_ah: f32,
    pub remaining_capacity_ah: f32,
    pub soc_percent: f32,
    pub soh_percent: f32,
    pub total_discharged_capacity_ah: f32,
    pub total_charged_capacity_ah: f32,
}

#[derive(Debug, Clone, serde::Serialize)]
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

#[derive(Debug, Clone, serde::Serialize)]
pub struct BatteryChargeVoltage {
    pub charge_voltage_valid_v: f32,
    pub charge_max_voltage_v: f32,
    pub charge_keep_power_voltage_v: f32,
}

#[derive(Debug, Clone, serde::Serialize)]
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

#[derive(Debug, Clone, serde::Serialize)]
pub struct BatteryChargeTemperature {
    pub temp_min_low_c: f32,
    pub temp_min_normal_c: f32,
    pub temp_max_normal_c: f32,
    pub temp_max_high_c: f32,
    pub temp_5_c: f32,
    pub temp_6_c: f32,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct MasterBatteryTemperature {
    pub max_fet_temperature_c: f32,
    pub max_cell_temperature_c: f32,
}

#[derive(Debug, Clone, Copy, Default, serde::Serialize)]
pub struct MsgBits {
    pub info_bit_0_empty: bool,
    pub info_bit_1_almost_empty: bool,
    pub info_bit_2_chgfet_closed: bool,
    pub info_bit_3_dsgfet_closed: bool,
    pub info_bit_4_bypass_fet_on: bool,
    pub info_bit_6_fully_charged: bool,
    pub warn_bit_0_low_voltage: bool,
    pub warn_bit_1_low_soc: bool,
    pub warn_bit_2_reserve_soc: bool,
    pub warn_bit_3_over_or_under_temp_discharge: bool,
    pub warn_bit_4_over_or_under_temp_charge: bool,
    pub warn_bit_7_max_charge_condition_recuperation: bool,
    pub warn_bit_11_can_network_failure: bool,
    pub warn_bit_12_set_deactivation_enable: bool,
    pub warn_bit_14_set_node_id_process_enable: bool,
    pub warn_bit_15_unknown: bool,
    pub error_bit_0_error_lock_flag_discharge: bool,
    pub error_bit_1_error_lock_flag_charge: bool,
    pub error_bit_2_over_charge_condition_recuperation: bool,
    pub error_bit_3_shortcircuit_charge_alarm: bool,
    pub error_bit_4_shortcircuit_discharge_alarm: bool,
    pub error_bit_5_max_voltage_alarm: bool,
    pub error_bit_6_discharge_fet_error: bool,
    pub error_bit_7_charge_fet_error: bool,
    pub error_bit_8_max_charge_current_alarm: bool,
    pub error_bit_9_max_discharge_current_alarm: bool,
    pub error_bit_10_under_charge_alarm: bool,
    pub error_bit_11_over_charge_alarm: bool,
    pub error_bit_12_over_under_temp_charge: bool,
    pub error_bit_13_over_under_temp_discharge: bool,
    pub error_bit_14_module_defect: bool,
    pub error_bit_15_uknown: bool,
    pub charge_bit_0_charge_voltage_enabled: bool,
    pub charge_bit_1_charge_voltage_keep_power: bool,
    pub charge_bit_4_charge_current_enable: bool,
    pub charge_bit_5_charge_current_keep_power: bool,
    pub charge_bit_6_charge_current_low_temp_range: bool,
    pub charge_bit_7_charge_current_normal_temp_range: bool,
    pub charge_bit_8_charge_current_high_temp_range: bool,
    pub charge_bit_10_charge_max_charge_current_request: bool,
    pub charge_bit_11_charge_max_charge_cell_voltage_request: bool,
    pub charge_bit_12_charge_master_set_charger_output_off: bool,
    pub charge_bit_13_charge_fet_disable_temp_range_cells: bool,
    pub charge_bit_14_master_charger_control_charging_ready: bool,
    pub charge_bit_15_charger_supply_conditions_ready: bool,
}

#[allow(clippy::type_complexity)]
#[derive(strum::EnumCount, Clone)]
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
    SerialNumber { node_id: u8, value: u16 },
    SoftwareVersion { node_id: u8, value: String },
    HardwareVersion { node_id: u8, value: String },
    DeviceErrorHistory { node_id: u8, value: Vec<DeviceError> },
    CellVoltages { node_id: u8, value: Vec<f32> },
    DeviceConfigInfo { node_id: u8, value: DeviceConfigInfo },
    DeviceSerialNumberInfo { node_id: u8, value: DeviceSerialNumberInfo },
    DeviceDateInfo { node_id: u8, value: DeviceDateInfo },
    DeviceVariantInfo { node_id: u8, value: DeviceVariantInfo },
    DeviceControlParam { node_id: u8, value: u16 },
    DeviceOperationTime { node_id: u8, value: DeviceOperationTime },
    DeviceErrorCounter { node_id: u8, value: DeviceErrorCounterInfo },
    CellVoltageMinMax { node_id: u8, value: CellVoltageMinMax },
    CellVoltageLimit { node_id: u8, value: CellVoltageLimit },
    BatteryVoltage { node_id: u8, value: BatteryVoltage },
    BatteryVoltageLimit { node_id: u8, value: BatteryVoltageLimit },
    BatteryCurrent { node_id: u8, value: BatteryCurrent },
    BatteryCurrentLimit { node_id: u8, value: BatteryCurrentLimit },
    FetTemperature { node_id: u8, value: FetTemperature },
    FetTemperatureMinMax { node_id: u8, value: FetTemperatureMinMax },
    FetTemperatureLimit { node_id: u8, value: FetTemperatureLimit },
    CellTemperature { node_id: u8, value: CellTemperature },
    CellTemperatureMinMax { node_id: u8, value: CellTemperatureMinMax },
    CellTemperatureLimit { node_id: u8, value: CellTemperatureLimit },
    CellBalanceStatus { node_id: u8, value: CellBalanceStatus },
    CellBalanceLimit { node_id: u8, value: CellBalanceLimit },
    CellImpedance { node_id: u8, value: CellImpedance },
    BatteryCapacity { node_id: u8, value: BatteryCapacity },
    BatteryCapacityParam { node_id: u8, value: u8 },
    BatteryCycleCount { node_id: u8, value: BatteryCycleCount },
    BatteryChargeVoltage { node_id: u8, value: BatteryChargeVoltage },
    BatteryChargeCurrent { node_id: u8, value: BatteryChargeCurrent },
    BatteryChargeTemperature { node_id: u8, value: BatteryChargeTemperature },
    MasterBatteryTemperature { node_id: u8, value: MasterBatteryTemperature },
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
    pub master_msgs: Option<MsgBits>,
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
    pub device_config_info: Option<DeviceConfigInfo>,
    pub device_serial_number_info: Option<DeviceSerialNumberInfo>,
    pub device_date_info: Option<DeviceDateInfo>,
    pub device_variant_info: Option<DeviceVariantInfo>,
    pub device_control_param: Option<u16>,
    pub device_operation_time: Option<DeviceOperationTime>,
    pub device_error_counter: Option<DeviceErrorCounterInfo>,
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
    pub pack_msgs: Option<MsgBits>,
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
