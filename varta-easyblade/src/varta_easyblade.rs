use crate::sdo::*;

/// Records a change in the overall state of the Varta battery.
#[derive(Debug)]
pub struct StateChange {
    node_id: Option<u8>,
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

/// SDO data read from a Varta Easyblade module.
/// Each field is `None` until successfully read via SDO.
#[derive(Debug, Clone, serde::Serialize)]
pub struct SdoData {
    pub serial_number: Option<SerialNumber>,
    pub software_version: Option<SoftwareVersion>,
    pub hardware_version: Option<HardwareVersion>,
    pub cell_voltages: Option<CellVoltages>,
    pub device_errors: Option<DeviceErrorHistory>,
    pub device_config_info: Option<DeviceConfigInfo>,
    pub device_serial_number_info: Option<DeviceSerialNumberInfo>,
    pub device_date_info: Option<DeviceDateInfo>,
    pub device_variant_info: Option<DeviceVariantInfo>,
    pub device_control_param: Option<DeviceControlParam>,
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
    pub battery_capacity_param: Option<BatteryCapacityParam>,
    pub battery_cycle_count: Option<BatteryCycleCount>,
    pub battery_charge_voltage: Option<BatteryChargeVoltage>,
    pub battery_charge_current: Option<BatteryChargeCurrent>,
    pub battery_charge_current_valid: Option<BatteryChargeCurrentValid>,
    pub battery_charge_current_max_normal: Option<BatteryChargeCurrentMaxNormal>,
    pub battery_charge_current_max_low: Option<BatteryChargeCurrentMaxLow>,
    pub battery_charge_current_max_high: Option<BatteryChargeCurrentMaxHigh>,
    pub battery_charge_current_keep_power: Option<BatteryChargeCurrentKeepPower>,
    pub battery_charge_current_decrease_stepsize1: Option<BatteryChargeCurrentDecreaseStepsize1>,
    pub battery_charge_current_increase_stepsize1: Option<BatteryChargeCurrentIncreaseStepsize1>,
    pub battery_charge_current_decrease_stepsize2: Option<BatteryChargeCurrentDecreaseStepsize2>,
    pub battery_charge_current_increase_stepsize2: Option<BatteryChargeCurrentIncreaseStepsize2>,
    pub battery_charge_current_modify_interval: Option<BatteryChargeCurrentModifyInterval>,
    pub battery_charge_temperature: Option<BatteryChargeTemperature>,
    pub master_battery_temperature: Option<MasterBatteryTemperature>,
    pub keep_power_timer: Option<KeepPowerTimer>,
}

/// PDO data received from Varta Easyblade CAN messages.
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct Pdo {
    pub voltage: Option<f32>,
    pub current: Option<f32>,
    pub soc: Option<f32>,
    pub soh: Option<f32>,
    pub msg_bits: Option<MsgBits>,
}

/// Varta Easyblade module state.
#[derive(Debug)]
pub struct VartaEasyblade {
    pub node_id: u8,
    pub last_seen: std::time::SystemTime,
    pub sdo: SdoData,
    pub pdo: Pdo,
}
