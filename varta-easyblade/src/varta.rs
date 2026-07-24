use socketcan::{CanFilter, EmbeddedFrame, SocketOptions};
use zencan_client::common::traits::{AsyncCanReceiver, AsyncCanSender};

use crate::Error;
use crate::MAX_MODULES;
use crate::MasterInfo;
use crate::VartaEasyblade;
use crate::varta_easyblade;
use crate::varta_easyblade_can_messages;
use varta_easyblade::BatteryCapacity;
use varta_easyblade::BatteryChargeCurrent;
use varta_easyblade::BatteryChargeTemperature;
use varta_easyblade::BatteryChargeVoltage;
use varta_easyblade::BatteryCurrent;
use varta_easyblade::BatteryCurrentLimit;
use varta_easyblade::BatteryCycleCount;
use varta_easyblade::BatteryVoltage;
use varta_easyblade::BatteryVoltageLimit;
use varta_easyblade::CellBalanceLimit;
use varta_easyblade::CellBalanceStatus;
use varta_easyblade::CellImpedance;
use varta_easyblade::CellTemperature;
use varta_easyblade::CellTemperatureLimit;
use varta_easyblade::CellTemperatureMinMax;
use varta_easyblade::CellVoltageLimit;
use varta_easyblade::CellVoltageMinMax;
use varta_easyblade::DeviceConfigInfo;
use varta_easyblade::DeviceDateInfo;
use varta_easyblade::DeviceOperationTime;
use varta_easyblade::DeviceSerialNumberInfo;
use varta_easyblade::DeviceVariantInfo;
use varta_easyblade::FetTemperature;
use varta_easyblade::FetTemperatureLimit;
use varta_easyblade::FetTemperatureMinMax;
use varta_easyblade::MasterBatteryTemperature;

fn get_node_id_from_can_message(msg: &varta_easyblade_can_messages::Messages) -> Option<u8> {
    match msg {
        varta_easyblade_can_messages::Messages::Pack01PackInfo1(_)
        | varta_easyblade_can_messages::Messages::Pack01Info2(_)
        | varta_easyblade_can_messages::Messages::Pack01Info3(_)
        | varta_easyblade_can_messages::Messages::Pack01Msgs(_) => Some(1),
        varta_easyblade_can_messages::Messages::Pack02PackInfo1(_)
        | varta_easyblade_can_messages::Messages::Pack02Info2(_)
        | varta_easyblade_can_messages::Messages::Pack02Info3(_)
        | varta_easyblade_can_messages::Messages::Pack02Msgs(_) => Some(2),
        varta_easyblade_can_messages::Messages::Pack03PackInfo1(_)
        | varta_easyblade_can_messages::Messages::Pack03Info2(_)
        | varta_easyblade_can_messages::Messages::Pack03Info3(_)
        | varta_easyblade_can_messages::Messages::Pack03Msgs(_) => Some(3),
        varta_easyblade_can_messages::Messages::Pack04PackInfo1(_)
        | varta_easyblade_can_messages::Messages::Pack04Info2(_)
        | varta_easyblade_can_messages::Messages::Pack04Info3(_)
        | varta_easyblade_can_messages::Messages::Pack04Msgs(_) => Some(4),
        varta_easyblade_can_messages::Messages::Pack05PackInfo1(_)
        | varta_easyblade_can_messages::Messages::Pack05Info2(_)
        | varta_easyblade_can_messages::Messages::Pack05Info3(_)
        | varta_easyblade_can_messages::Messages::Pack05Msgs(_) => Some(5),
        varta_easyblade_can_messages::Messages::Pack06PackInfo1(_)
        | varta_easyblade_can_messages::Messages::Pack06Info2(_)
        | varta_easyblade_can_messages::Messages::Pack06Info3(_)
        | varta_easyblade_can_messages::Messages::Pack06Msgs(_) => Some(6),
        varta_easyblade_can_messages::Messages::Pack07PackInfo1(_)
        | varta_easyblade_can_messages::Messages::Pack07Info2(_)
        | varta_easyblade_can_messages::Messages::Pack07Info3(_)
        | varta_easyblade_can_messages::Messages::Pack07Msgs(_) => Some(7),
        varta_easyblade_can_messages::Messages::Pack08PackInfo1(_)
        | varta_easyblade_can_messages::Messages::Pack08Info2(_)
        | varta_easyblade_can_messages::Messages::Pack08Info3(_)
        | varta_easyblade_can_messages::Messages::Pack08Msgs(_) => Some(8),
        varta_easyblade_can_messages::Messages::Pack09PackInfo1(_)
        | varta_easyblade_can_messages::Messages::Pack09Info2(_)
        | varta_easyblade_can_messages::Messages::Pack09Info3(_)
        | varta_easyblade_can_messages::Messages::Pack09Msgs(_) => Some(9),
        varta_easyblade_can_messages::Messages::Pack10PackInfo1(_)
        | varta_easyblade_can_messages::Messages::Pack10Info2(_)
        | varta_easyblade_can_messages::Messages::Pack10Info3(_)
        | varta_easyblade_can_messages::Messages::Pack10Msgs(_) => Some(10),
        varta_easyblade_can_messages::Messages::Pack11PackInfo1(_)
        | varta_easyblade_can_messages::Messages::Pack11Info2(_)
        | varta_easyblade_can_messages::Messages::Pack11Info3(_)
        | varta_easyblade_can_messages::Messages::Pack11Msgs(_) => Some(11),
        varta_easyblade_can_messages::Messages::Pack12PackInfo1(_)
        | varta_easyblade_can_messages::Messages::Pack12Info2(_)
        | varta_easyblade_can_messages::Messages::Pack12Info3(_)
        | varta_easyblade_can_messages::Messages::Pack12Msgs(_) => Some(12),
        varta_easyblade_can_messages::Messages::Pack13PackInfo1(_)
        | varta_easyblade_can_messages::Messages::Pack13Info2(_)
        | varta_easyblade_can_messages::Messages::Pack13Info3(_)
        | varta_easyblade_can_messages::Messages::Pack13Msgs(_) => Some(13),
        varta_easyblade_can_messages::Messages::Pack14PackInfo1(_)
        | varta_easyblade_can_messages::Messages::Pack14Info2(_)
        | varta_easyblade_can_messages::Messages::Pack14Info3(_)
        | varta_easyblade_can_messages::Messages::Pack14Msgs(_) => Some(14),
        varta_easyblade_can_messages::Messages::Pack15PackInfo1(_)
        | varta_easyblade_can_messages::Messages::Pack15Info2(_)
        | varta_easyblade_can_messages::Messages::Pack15Info3(_)
        | varta_easyblade_can_messages::Messages::Pack15Msgs(_) => Some(15),
        _ => None,
    }
    .filter(|&n| n < MAX_MODULES as u8)
}

#[derive(Debug)]
pub struct Varta {
    pub socketcan_interface: socketcan::tokio::AsyncCanSocket<socketcan::CanSocket>,
    pub canbus_interface: String,
    pub master: MasterInfo,
    pub easyblades: [Option<VartaEasyblade>; MAX_MODULES],
}

// Public API
impl Varta {
    pub async fn new(canbus_interface: &str) -> Result<Self, Error> {
        let socketcan_interface =
            socketcan::tokio::CanSocket::open(canbus_interface).map_err(|e| Error::Io {
                can_interface: String::from(canbus_interface),
                e,
            })?;

        socketcan_interface
            .set_filters(&[
                CanFilter::new(0x180, 0x7f0), // module info1: voltage & current for each module
                CanFilter::new(0x280, 0x7f0), // module info2: temps & charge requests for each module
                CanFilter::new(0x380, 0x7f0), // module info3: capacities for each module
                CanFilter::new(0x480, 0x7f0), // module msgs: FET status for each module
                CanFilter::new(0x264, 0x7ff), // master charge control
                CanFilter::new(0x19b, 0x7ff), // master info1: voltage & current for pack
                CanFilter::new(0x29b, 0x7ff), // master info2: temps & design capacity
                CanFilter::new(0x39b, 0x7ff), // master info3: full & remaining capacity
            ])
            .map_err(|e| Error::Io {
                can_interface: String::from(canbus_interface),
                e,
            })?;

        let varta = Self {
            socketcan_interface,
            canbus_interface: String::from(canbus_interface),
            master: MasterInfo::default(),
            easyblades: [const { None }; MAX_MODULES],
        };
        Ok(varta)
    }

    pub async fn process_socketcan_msg(&mut self) -> Result<Option<u8>, Error> {
        let can_frame = self
            .socketcan_interface
            .read_frame()
            .await
            .map_err(|e| Error::Io {
                can_interface: self.canbus_interface.clone(),
                e,
            })?;

        let msg = crate::varta_easyblade_can_messages::Messages::from_can_message(
            can_frame.id(),
            can_frame.data(),
        )?;

        let node_id = get_node_id_from_can_message(&msg);

        let mut new_module: Option<u8> = None;

        if let Some(node_id) = node_id {
            let was_new = self.easyblades[node_id as usize].is_none();
            let easyblade = self.get_or_init_easyblade(node_id);
            if was_new {
                new_module = Some(node_id);
            }
            match msg {
                varta_easyblade_can_messages::Messages::Pack01PackInfo1(m) => {
                    Self::update_easyblade_voltage_current(easyblade, m.voltage(), m.current());
                },
                varta_easyblade_can_messages::Messages::Pack02PackInfo1(m) => {
                    Self::update_easyblade_voltage_current(easyblade, m.voltage(), m.current());
                },
                varta_easyblade_can_messages::Messages::Pack03PackInfo1(m) => {
                    Self::update_easyblade_voltage_current(easyblade, m.voltage(), m.current());
                },
                varta_easyblade_can_messages::Messages::Pack04PackInfo1(m) => {
                    Self::update_easyblade_voltage_current(easyblade, m.voltage(), m.current());
                },
                varta_easyblade_can_messages::Messages::Pack05PackInfo1(m) => {
                    Self::update_easyblade_voltage_current(easyblade, m.voltage(), m.current());
                },
                varta_easyblade_can_messages::Messages::Pack06PackInfo1(m) => {
                    Self::update_easyblade_voltage_current(easyblade, m.voltage(), m.current());
                },
                varta_easyblade_can_messages::Messages::Pack07PackInfo1(m) => {
                    Self::update_easyblade_voltage_current(easyblade, m.voltage(), m.current());
                },
                varta_easyblade_can_messages::Messages::Pack08PackInfo1(m) => {
                    Self::update_easyblade_voltage_current(easyblade, m.voltage(), m.current());
                },
                varta_easyblade_can_messages::Messages::Pack09PackInfo1(m) => {
                    Self::update_easyblade_voltage_current(easyblade, m.voltage(), m.current());
                },
                varta_easyblade_can_messages::Messages::Pack10PackInfo1(m) => {
                    Self::update_easyblade_voltage_current(easyblade, m.voltage(), m.current());
                },
                varta_easyblade_can_messages::Messages::Pack11PackInfo1(m) => {
                    Self::update_easyblade_voltage_current(easyblade, m.voltage(), m.current());
                },
                varta_easyblade_can_messages::Messages::Pack12PackInfo1(m) => {
                    Self::update_easyblade_voltage_current(easyblade, m.voltage(), m.current());
                },
                varta_easyblade_can_messages::Messages::Pack13PackInfo1(m) => {
                    Self::update_easyblade_voltage_current(easyblade, m.voltage(), m.current());
                },
                varta_easyblade_can_messages::Messages::Pack14PackInfo1(m) => {
                    Self::update_easyblade_voltage_current(easyblade, m.voltage(), m.current());
                },
                varta_easyblade_can_messages::Messages::Pack15PackInfo1(m) => {
                    Self::update_easyblade_voltage_current(easyblade, m.voltage(), m.current());
                },
                varta_easyblade_can_messages::Messages::Pack01Info3(m) => {
                    Self::update_easyblade_soc_soh(
                        easyblade,
                        m.battery_full_cap(),
                        m.battery_rem_cap(),
                        m.battery_design_cap(),
                    );
                },
                varta_easyblade_can_messages::Messages::Pack02Info3(m) => {
                    Self::update_easyblade_soc_soh(
                        easyblade,
                        m.battery_full_cap(),
                        m.battery_rem_cap(),
                        m.battery_design_cap(),
                    );
                },
                varta_easyblade_can_messages::Messages::Pack03Info3(m) => {
                    Self::update_easyblade_soc_soh(
                        easyblade,
                        m.battery_full_cap(),
                        m.battery_rem_cap(),
                        m.battery_design_cap(),
                    );
                },
                varta_easyblade_can_messages::Messages::Pack04Info3(m) => {
                    Self::update_easyblade_soc_soh(
                        easyblade,
                        m.battery_full_cap(),
                        m.battery_rem_cap(),
                        m.battery_design_cap(),
                    );
                },
                varta_easyblade_can_messages::Messages::Pack05Info3(m) => {
                    Self::update_easyblade_soc_soh(
                        easyblade,
                        m.battery_full_cap(),
                        m.battery_rem_cap(),
                        m.battery_design_cap(),
                    );
                },
                varta_easyblade_can_messages::Messages::Pack06Info3(m) => {
                    Self::update_easyblade_soc_soh(
                        easyblade,
                        m.battery_full_cap(),
                        m.battery_rem_cap(),
                        m.battery_design_cap(),
                    );
                },
                varta_easyblade_can_messages::Messages::Pack07Info3(m) => {
                    Self::update_easyblade_soc_soh(
                        easyblade,
                        m.battery_full_cap(),
                        m.battery_rem_cap(),
                        m.battery_design_cap(),
                    );
                },
                varta_easyblade_can_messages::Messages::Pack08Info3(m) => {
                    Self::update_easyblade_soc_soh(
                        easyblade,
                        m.battery_full_cap(),
                        m.battery_rem_cap(),
                        m.battery_design_cap(),
                    );
                },
                varta_easyblade_can_messages::Messages::Pack09Info3(m) => {
                    Self::update_easyblade_soc_soh(
                        easyblade,
                        m.battery_full_cap(),
                        m.battery_rem_cap(),
                        m.battery_design_cap(),
                    );
                },
                varta_easyblade_can_messages::Messages::Pack10Info3(m) => {
                    Self::update_easyblade_soc_soh(
                        easyblade,
                        m.battery_full_cap(),
                        m.battery_rem_cap(),
                        m.battery_design_cap(),
                    );
                },
                varta_easyblade_can_messages::Messages::Pack11Info3(m) => {
                    Self::update_easyblade_soc_soh(
                        easyblade,
                        m.battery_full_cap(),
                        m.battery_rem_cap(),
                        m.battery_design_cap(),
                    );
                },
                varta_easyblade_can_messages::Messages::Pack12Info3(m) => {
                    Self::update_easyblade_soc_soh(
                        easyblade,
                        m.battery_full_cap(),
                        m.battery_rem_cap(),
                        m.battery_design_cap(),
                    );
                },
                varta_easyblade_can_messages::Messages::Pack13Info3(m) => {
                    Self::update_easyblade_soc_soh(
                        easyblade,
                        m.battery_full_cap(),
                        m.battery_rem_cap(),
                        m.battery_design_cap(),
                    );
                },
                varta_easyblade_can_messages::Messages::Pack14Info3(m) => {
                    Self::update_easyblade_soc_soh(
                        easyblade,
                        m.battery_full_cap(),
                        m.battery_rem_cap(),
                        m.battery_design_cap(),
                    );
                },
                varta_easyblade_can_messages::Messages::Pack15Info3(m) => {
                    Self::update_easyblade_soc_soh(
                        easyblade,
                        m.battery_full_cap(),
                        m.battery_rem_cap(),
                        m.battery_design_cap(),
                    );
                },
                varta_easyblade_can_messages::Messages::Pack01Msgs(m) => {
                    Self::update_easyblade_fet_status(
                        easyblade,
                        m.info_bit_2_chgfet_closed(),
                        m.info_bit_3_dsgfet_closed(),
                        m.info_bit_4_bypass_fet_on(),
                    );
                },
                varta_easyblade_can_messages::Messages::Pack02Msgs(m) => {
                    Self::update_easyblade_fet_status(
                        easyblade,
                        m.info_bit_2_chgfet_closed(),
                        m.info_bit_3_dsgfet_closed(),
                        m.info_bit_4_bypass_fet_on(),
                    );
                },
                varta_easyblade_can_messages::Messages::Pack03Msgs(m) => {
                    Self::update_easyblade_fet_status(
                        easyblade,
                        m.info_bit_2_chgfet_closed(),
                        m.info_bit_3_dsgfet_closed(),
                        m.info_bit_4_bypass_fet_on(),
                    );
                },
                varta_easyblade_can_messages::Messages::Pack04Msgs(m) => {
                    Self::update_easyblade_fet_status(
                        easyblade,
                        m.info_bit_2_chgfet_closed(),
                        m.info_bit_3_dsgfet_closed(),
                        m.info_bit_4_bypass_fet_on(),
                    );
                },
                varta_easyblade_can_messages::Messages::Pack05Msgs(m) => {
                    Self::update_easyblade_fet_status(
                        easyblade,
                        m.info_bit_2_chgfet_closed(),
                        m.info_bit_3_dsgfet_closed(),
                        m.info_bit_4_bypass_fet_on(),
                    );
                },
                varta_easyblade_can_messages::Messages::Pack06Msgs(m) => {
                    Self::update_easyblade_fet_status(
                        easyblade,
                        m.info_bit_2_chgfet_closed(),
                        m.info_bit_3_dsgfet_closed(),
                        m.info_bit_4_bypass_fet_on(),
                    );
                },
                varta_easyblade_can_messages::Messages::Pack07Msgs(m) => {
                    Self::update_easyblade_fet_status(
                        easyblade,
                        m.info_bit_2_chgfet_closed(),
                        m.info_bit_3_dsgfet_closed(),
                        m.info_bit_4_bypass_fet_on(),
                    );
                },
                varta_easyblade_can_messages::Messages::Pack08Msgs(m) => {
                    Self::update_easyblade_fet_status(
                        easyblade,
                        m.info_bit_2_chgfet_closed(),
                        m.info_bit_3_dsgfet_closed(),
                        m.info_bit_4_bypass_fet_on(),
                    );
                },
                varta_easyblade_can_messages::Messages::Pack09Msgs(m) => {
                    Self::update_easyblade_fet_status(
                        easyblade,
                        m.info_bit_2_chgfet_closed(),
                        m.info_bit_3_dsgfet_closed(),
                        m.info_bit_4_bypass_fet_on(),
                    );
                },
                varta_easyblade_can_messages::Messages::Pack10Msgs(m) => {
                    Self::update_easyblade_fet_status(
                        easyblade,
                        m.info_bit_2_chgfet_closed(),
                        m.info_bit_3_dsgfet_closed(),
                        m.info_bit_4_bypass_fet_on(),
                    );
                },
                varta_easyblade_can_messages::Messages::Pack11Msgs(m) => {
                    Self::update_easyblade_fet_status(
                        easyblade,
                        m.info_bit_2_chgfet_closed(),
                        m.info_bit_3_dsgfet_closed(),
                        m.info_bit_4_bypass_fet_on(),
                    );
                },
                varta_easyblade_can_messages::Messages::Pack12Msgs(m) => {
                    Self::update_easyblade_fet_status(
                        easyblade,
                        m.info_bit_2_chgfet_closed(),
                        m.info_bit_3_dsgfet_closed(),
                        m.info_bit_4_bypass_fet_on(),
                    );
                },
                varta_easyblade_can_messages::Messages::Pack13Msgs(m) => {
                    Self::update_easyblade_fet_status(
                        easyblade,
                        m.info_bit_2_chgfet_closed(),
                        m.info_bit_3_dsgfet_closed(),
                        m.info_bit_4_bypass_fet_on(),
                    );
                },
                varta_easyblade_can_messages::Messages::Pack14Msgs(m) => {
                    Self::update_easyblade_fet_status(
                        easyblade,
                        m.info_bit_2_chgfet_closed(),
                        m.info_bit_3_dsgfet_closed(),
                        m.info_bit_4_bypass_fet_on(),
                    );
                },
                varta_easyblade_can_messages::Messages::Pack15Msgs(m) => {
                    Self::update_easyblade_fet_status(
                        easyblade,
                        m.info_bit_2_chgfet_closed(),
                        m.info_bit_3_dsgfet_closed(),
                        m.info_bit_4_bypass_fet_on(),
                    );
                },
                _ => {},
            }
        }

        match msg {
            varta_easyblade_can_messages::Messages::MasterPackInfo1(m) => {
                self.master.voltage = Some(m.voltage());
                self.master.current = Some(m.current());
                self.master.last_seen = Some(std::time::SystemTime::now());
            },
            varta_easyblade_can_messages::Messages::MasterChargeControl(m) => {
                self.master.charge_voltage_request = Some(m.charge_voltage_request());
                self.master.charge_current_request = Some(m.charge_current_request());
                self.master.battery_status = Some(m.battery_status());
                self.master.last_seen = Some(std::time::SystemTime::now());
            },
            varta_easyblade_can_messages::Messages::MasterPackInfo2(m) => {
                self.master.max_battery_fet_temp = Some(m.max_battery_fet_temp());
                self.master.max_battery_cell_temp = Some(m.max_battery_cell_temp());
                self.master.master_design_capacity = Some(m.master_design_capacity());
                self.master.last_seen = Some(std::time::SystemTime::now());
            },
            varta_easyblade_can_messages::Messages::MasterPackInfo3(m) => {
                let full = m.master_full_charge_capacity();
                let remaining = m.master_remaining_capacity();
                self.master.master_full_charge_capacity = Some(full);
                self.master.master_remaining_capacity = Some(remaining);
                self.master.soc = if full > 0.0 {
                    Some((remaining / full) * 100.0)
                } else {
                    None
                };
                self.master.last_seen = Some(std::time::SystemTime::now());
            },
            _ => {},
        }

        self.expire_missing_modules();

        Ok(new_module)
    }

    /// Expire any modules that have not reported in the last 10 seconds.
    pub fn expire_missing_modules(&mut self) {
        let now = std::time::SystemTime::now();
        for entry in self.easyblades.iter_mut() {
            if entry
                .as_ref()
                .and_then(|eb| now.duration_since(eb.last_seen).ok())
                .map(|d| d.as_secs() > 10)
                .unwrap_or(false)
            {
                entry.take();
            }
        }
    }

    /// Returns the duration until the oldest module should be expired, or 10s if no modules exist.
    pub fn next_expiry_delay(&self) -> std::time::Duration {
        let now = std::time::SystemTime::now();
        let oldest_last_seen = self
            .easyblades
            .iter()
            .filter_map(|e| e.as_ref())
            .map(|e| e.last_seen)
            .min();

        match oldest_last_seen {
            Some(last_seen) => {
                let expiry = last_seen
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .checked_add(std::time::Duration::from_secs(10))
                    .unwrap();
                let now_since_epoch = now.duration_since(std::time::UNIX_EPOCH).unwrap();
                expiry.saturating_sub(now_since_epoch)
            },
            None => std::time::Duration::from_secs(10),
        }
    }

    pub fn get_or_init_easyblade(&mut self, node_id: u8) -> &mut VartaEasyblade {
        if self.easyblades[node_id as usize].is_none() {
            self.easyblades[node_id as usize] = Some(VartaEasyblade {
                node_id,
                serial_number: None,
                software_version: None,
                hardware_version: None,
                last_seen: std::time::SystemTime::now(),
                voltage: None,
                current: None,
                soc: None,
                soh: None,
                cell_voltages: None,
                device_errors: None,
                fet_status: None,
                device_config_info: None,
                device_serial_number_info: None,
                device_date_info: None,
                device_variant_info: None,
                device_control_param: None,
                device_operation_time: None,
                device_error_counter: None,
                cell_voltage_min_max: None,
                cell_voltage_limit: None,
                battery_voltage: None,
                battery_voltage_limit: None,
                battery_current: None,
                battery_current_limit: None,
                fet_temperature: None,
                fet_temperature_min_max: None,
                fet_temperature_limit: None,
                cell_temperature: None,
                cell_temperature_min_max: None,
                cell_temperature_limit: None,
                cell_balance_status: None,
                cell_balance_limit: None,
                cell_impedance: None,
                battery_capacity: None,
                battery_capacity_param: None,
                battery_cycle_count: None,
                battery_charge_voltage: None,
                battery_charge_current: None,
                battery_charge_temperature: None,
                master_battery_temperature: None,
            });
        }
        self.easyblades[node_id as usize].as_mut().unwrap()
    }

    // --- Reusable SDO read functions ---

    pub async fn sdo_read_serial_number<S: AsyncCanSender, R: AsyncCanReceiver>(
        sdo_client: &mut zencan_client::SdoClient<S, R>,
    ) -> Result<u16, String> {
        let bytes = sdo_client
            .upload(0x2004, 0x01)
            .await
            .map_err(|e| e.to_string())?;
        if bytes.len() < 2 {
            return Err("Serial number data too short".to_string());
        }
        Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
    }

    pub async fn sdo_read_software_version<S: AsyncCanSender, R: AsyncCanReceiver>(
        sdo_client: &mut zencan_client::SdoClient<S, R>,
    ) -> Result<String, String> {
        let sw = sdo_client
            .upload(0x2000, 0x02)
            .await
            .map_err(|e| e.to_string())?;
        let fw = sdo_client
            .upload(0x2000, 0x03)
            .await
            .map_err(|e| e.to_string())?;
        let sw_str = String::from_utf8_lossy(&sw).trim_matches('\0').to_string();
        let fw_str = String::from_utf8_lossy(&fw).trim_matches('\0').to_string();
        Ok(format!("{}{}", sw_str, fw_str))
    }

    pub async fn sdo_read_hardware_version<S: AsyncCanSender, R: AsyncCanReceiver>(
        sdo_client: &mut zencan_client::SdoClient<S, R>,
    ) -> Result<String, String> {
        let bytes = sdo_client
            .upload(0x2000, 0x01)
            .await
            .map_err(|e| e.to_string())?;
        Ok(String::from_utf8_lossy(&bytes)
            .trim_matches('\0')
            .to_string())
    }

    pub async fn sdo_read_device_error_history<S: AsyncCanSender, R: AsyncCanReceiver>(
        sdo_client: &mut zencan_client::SdoClient<S, R>,
    ) -> Result<Vec<varta_easyblade::DeviceError>, String> {
        let highest_subindex = sdo_client
            .read_u8(0x2018, 0x00)
            .await
            .map_err(|e| e.to_string())?;
        if highest_subindex != 16 {
            return Err(format!(
                "Expected 16 error entries, got {}",
                highest_subindex
            ));
        }
        let mut errors = Vec::new();
        for sub_index in 1..=16u8 {
            let val = sdo_client
                .read_u8(0x2018, sub_index)
                .await
                .map_err(|e| e.to_string())?;
            let e: varta_easyblade::DeviceError = match varta_easyblade::DeviceError::try_from(val)
            {
                Ok(e) => e,
                Err(_) => varta_easyblade::DeviceError::Unknown,
            };
            errors.push(e);
        }
        Ok(errors)
    }

    pub async fn sdo_read_cell_voltages<S: AsyncCanSender, R: AsyncCanReceiver>(
        sdo_client: &mut zencan_client::SdoClient<S, R>,
    ) -> Result<Vec<f32>, String> {
        let highest_subindex = sdo_client
            .read_u8(0x2100, 0x00)
            .await
            .map_err(|e| e.to_string())?;
        if highest_subindex != 16 {
            return Err(format!(
                "Expected 16 cell voltage entries, got {}",
                highest_subindex
            ));
        }
        let mut cell_voltages: Vec<f32> = Vec::new();
        for sub_index in 1..=14u8 {
            let val = sdo_client
                .read_u32(0x2100, sub_index)
                .await
                .map_err(|e| e.to_string())?;
            cell_voltages.push((val as f32) / 1000.0);
        }
        Ok(cell_voltages)
    }

    pub async fn sdo_read_device_config_info<S: AsyncCanSender, R: AsyncCanReceiver>(
        sdo_client: &mut zencan_client::SdoClient<S, R>,
    ) -> Result<DeviceConfigInfo, String> {
        let c1 = sdo_client
            .upload(0x2002, 0x01)
            .await
            .map_err(|e| e.to_string())?;
        let c2 = sdo_client
            .upload(0x2002, 0x02)
            .await
            .map_err(|e| e.to_string())?;
        let c3 = sdo_client
            .upload(0x2002, 0x03)
            .await
            .map_err(|e| e.to_string())?;
        Ok(DeviceConfigInfo {
            config_1: String::from_utf8_lossy(&c1).trim_matches('\0').to_string(),
            config_2: String::from_utf8_lossy(&c2).trim_matches('\0').to_string(),
            config_3: String::from_utf8_lossy(&c3).trim_matches('\0').to_string(),
        })
    }

    pub async fn sdo_read_device_serial_number_info<S: AsyncCanSender, R: AsyncCanReceiver>(
        sdo_client: &mut zencan_client::SdoClient<S, R>,
    ) -> Result<DeviceSerialNumberInfo, String> {
        let s1 = sdo_client
            .read_u32(0x2004, 0x01)
            .await
            .map_err(|e| e.to_string())?;
        let s2 = sdo_client
            .read_u32(0x2004, 0x02)
            .await
            .map_err(|e| e.to_string())?;
        let s3 = sdo_client
            .read_u32(0x2004, 0x03)
            .await
            .map_err(|e| e.to_string())?;
        Ok(DeviceSerialNumberInfo {
            serial_part_1: s1,
            serial_part_2: s2,
            serial_part_3: s3,
        })
    }

    pub async fn sdo_read_device_date_info<S: AsyncCanSender, R: AsyncCanReceiver>(
        sdo_client: &mut zencan_client::SdoClient<S, R>,
    ) -> Result<DeviceDateInfo, String> {
        let d1 = sdo_client
            .read_u16(0x2006, 0x01)
            .await
            .map_err(|e| e.to_string())?;
        let d2 = sdo_client
            .read_u16(0x2006, 0x02)
            .await
            .map_err(|e| e.to_string())?;
        let d3 = sdo_client
            .read_u16(0x2006, 0x03)
            .await
            .map_err(|e| e.to_string())?;
        Ok(DeviceDateInfo { year: d1, month: d2, day: d3 })
    }

    pub async fn sdo_read_device_variant_info<S: AsyncCanSender, R: AsyncCanReceiver>(
        sdo_client: &mut zencan_client::SdoClient<S, R>,
    ) -> Result<DeviceVariantInfo, String> {
        let s1 = sdo_client
            .read_u8(0x2008, 0x01)
            .await
            .map_err(|e| e.to_string())?;
        let s2 = sdo_client
            .read_u8(0x2008, 0x02)
            .await
            .map_err(|e| e.to_string())?;
        let s3 = sdo_client
            .read_u8(0x2008, 0x03)
            .await
            .map_err(|e| e.to_string())?;
        let s4 = sdo_client
            .read_u16(0x2008, 0x04)
            .await
            .map_err(|e| e.to_string())?;
        let s5 = sdo_client
            .read_u16(0x2008, 0x05)
            .await
            .map_err(|e| e.to_string())?;
        let s6 = sdo_client
            .read_u16(0x2008, 0x06)
            .await
            .map_err(|e| e.to_string())?;
        let s7 = sdo_client
            .read_u8(0x2008, 0x07)
            .await
            .map_err(|e| e.to_string())?;
        Ok(DeviceVariantInfo {
            variant_1: s1,
            variant_2: s2,
            variant_3: s3,
            variant_4: s4,
            variant_5: s5,
            variant_6: s6,
            variant_7: s7,
        })
    }

    pub async fn sdo_read_device_control_param<S: AsyncCanSender, R: AsyncCanReceiver>(
        sdo_client: &mut zencan_client::SdoClient<S, R>,
    ) -> Result<u16, String> {
        sdo_client
            .read_u16(0x2010, 0x01)
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn sdo_read_device_operation_time<S: AsyncCanSender, R: AsyncCanReceiver>(
        sdo_client: &mut zencan_client::SdoClient<S, R>,
    ) -> Result<DeviceOperationTime, String> {
        let m1 = sdo_client
            .read_u8(0x2016, 0x01)
            .await
            .map_err(|e| e.to_string())?;
        let m2 = sdo_client
            .read_u8(0x2016, 0x02)
            .await
            .map_err(|e| e.to_string())?;
        let m3 = sdo_client
            .read_u8(0x2016, 0x03)
            .await
            .map_err(|e| e.to_string())?;
        let m4 = sdo_client
            .read_u8(0x2016, 0x04)
            .await
            .map_err(|e| e.to_string())?;
        let m5 = sdo_client
            .read_u8(0x2016, 0x05)
            .await
            .map_err(|e| e.to_string())?;
        let h1 = sdo_client
            .read_u32(0x2016, 0x06)
            .await
            .map_err(|e| e.to_string())?;
        let h2 = sdo_client
            .read_u32(0x2016, 0x07)
            .await
            .map_err(|e| e.to_string())?;
        let h3 = sdo_client
            .read_u32(0x2016, 0x08)
            .await
            .map_err(|e| e.to_string())?;
        let h4 = sdo_client
            .read_u32(0x2016, 0x09)
            .await
            .map_err(|e| e.to_string())?;
        let h5 = sdo_client
            .read_u32(0x2016, 0x0a)
            .await
            .map_err(|e| e.to_string())?;
        Ok(DeviceOperationTime {
            minutes_below_zero: m1,
            minutes_zero_to_40: m2,
            minutes_40_to_60: m3,
            minutes_60_to_80: m4,
            minutes_above_80: m5,
            hours_below_zero: h1,
            hours_zero_to_40: h2,
            hours_40_to_60: h3,
            hours_60_to_80: h4,
            hours_above_80: h5,
        })
    }

    pub async fn sdo_read_device_error_counter<S: AsyncCanSender, R: AsyncCanReceiver>(
        sdo_client: &mut zencan_client::SdoClient<S, R>,
    ) -> Result<Vec<u16>, String> {
        let highest_subindex = sdo_client
            .read_u8(0x201a, 0x00)
            .await
            .map_err(|e| e.to_string())?;
        let mut counters = Vec::new();
        for sub_index in 1..=highest_subindex {
            let val = sdo_client
                .read_u16(0x201a, sub_index)
                .await
                .map_err(|e| e.to_string())?;
            counters.push(val);
        }
        Ok(counters)
    }

    pub async fn sdo_read_cell_voltage_min_max<S: AsyncCanSender, R: AsyncCanReceiver>(
        sdo_client: &mut zencan_client::SdoClient<S, R>,
    ) -> Result<CellVoltageMinMax, String> {
        let min = sdo_client
            .read_u32(0x2102, 0x01)
            .await
            .map_err(|e| e.to_string())?;
        let max = sdo_client
            .read_u32(0x2102, 0x02)
            .await
            .map_err(|e| e.to_string())?;
        Ok(CellVoltageMinMax {
            min_voltage_v: min as f32 / 1000.0,
            max_voltage_v: max as f32 / 1000.0,
        })
    }

    pub async fn sdo_read_cell_voltage_limit<S: AsyncCanSender, R: AsyncCanReceiver>(
        sdo_client: &mut zencan_client::SdoClient<S, R>,
    ) -> Result<CellVoltageLimit, String> {
        let raw = sdo_client
            .read_u32(0x2104, 0x01)
            .await
            .map_err(|e| e.to_string())?;
        Ok(CellVoltageLimit {
            over_voltage_error_v: raw as f32 / 1000.0,
        })
    }

    pub async fn sdo_read_battery_voltage<S: AsyncCanSender, R: AsyncCanReceiver>(
        sdo_client: &mut zencan_client::SdoClient<S, R>,
    ) -> Result<BatteryVoltage, String> {
        let v1 = sdo_client
            .read_u32(0x2200, 0x01)
            .await
            .map_err(|e| e.to_string())?;
        let v2 = sdo_client
            .read_u32(0x2200, 0x02)
            .await
            .map_err(|e| e.to_string())?;
        let v3 = sdo_client
            .read_u32(0x2200, 0x03)
            .await
            .map_err(|e| e.to_string())?;
        Ok(BatteryVoltage {
            sum_of_cell_voltage_v: v1 as f32 / 1000.0,
            internal_connector_voltage_v: v2 as f32 / 1000.0,
            external_connector_voltage_v: v3 as f32 / 1000.0,
        })
    }

    pub async fn sdo_read_battery_voltage_limit<S: AsyncCanSender, R: AsyncCanReceiver>(
        sdo_client: &mut zencan_client::SdoClient<S, R>,
    ) -> Result<BatteryVoltageLimit, String> {
        let raw = sdo_client
            .read_u32(0x2204, 0x01)
            .await
            .map_err(|e| e.to_string())?;
        Ok(BatteryVoltageLimit {
            internal_external_min_delta_v: raw as f32 / 1000.0,
        })
    }

    pub async fn sdo_read_battery_current<S: AsyncCanSender, R: AsyncCanReceiver>(
        sdo_client: &mut zencan_client::SdoClient<S, R>,
    ) -> Result<BatteryCurrent, String> {
        let c1 = sdo_client
            .read_i32(0x2300, 0x01)
            .await
            .map_err(|e| e.to_string())?;
        let c2 = sdo_client
            .read_i32(0x2300, 0x02)
            .await
            .map_err(|e| e.to_string())?;
        let c3 = sdo_client
            .read_i32(0x2300, 0x03)
            .await
            .map_err(|e| e.to_string())?;
        let c4 = sdo_client
            .read_i32(0x2300, 0x04)
            .await
            .map_err(|e| e.to_string())?;
        let c5 = sdo_client
            .read_i32(0x2300, 0x05)
            .await
            .map_err(|e| e.to_string())?;
        Ok(BatteryCurrent {
            fast_current_a: c1 as f32 / 1000.0,
            weighted_avg_current_a: c2 as f32 / 1000.0,
            integrated_current_a: c3 as f32 / 1000.0,
            average_1s_current_a: c4 as f32 / 1000.0,
            average_10s_current_a: c5 as f32 / 1000.0,
        })
    }

    pub async fn sdo_read_battery_current_limit<S: AsyncCanSender, R: AsyncCanReceiver>(
        sdo_client: &mut zencan_client::SdoClient<S, R>,
    ) -> Result<BatteryCurrentLimit, String> {
        let raw = sdo_client
            .read_i32(0x2304, 0x01)
            .await
            .map_err(|e| e.to_string())?;
        Ok(BatteryCurrentLimit {
            discharge_sc_error_a: raw as f32 / 1000.0,
        })
    }

    pub async fn sdo_read_fet_temperature<S: AsyncCanSender, R: AsyncCanReceiver>(
        sdo_client: &mut zencan_client::SdoClient<S, R>,
    ) -> Result<FetTemperature, String> {
        let t1 = sdo_client
            .read_i32(0x2400, 0x01)
            .await
            .map_err(|e| e.to_string())?;
        let t2 = sdo_client
            .read_i32(0x2400, 0x02)
            .await
            .map_err(|e| e.to_string())?;
        Ok(FetTemperature {
            temperature_1_c: t1 as f32 / 10.0,
            temperature_2_c: t2 as f32 / 10.0,
        })
    }

    pub async fn sdo_read_fet_temperature_min_max<S: AsyncCanSender, R: AsyncCanReceiver>(
        sdo_client: &mut zencan_client::SdoClient<S, R>,
    ) -> Result<FetTemperatureMinMax, String> {
        let min = sdo_client
            .read_i32(0x2402, 0x01)
            .await
            .map_err(|e| e.to_string())?;
        let max = sdo_client
            .read_i32(0x2402, 0x02)
            .await
            .map_err(|e| e.to_string())?;
        Ok(FetTemperatureMinMax {
            min_temperature_c: min as f32 / 10.0,
            max_temperature_c: max as f32 / 10.0,
        })
    }

    pub async fn sdo_read_fet_temperature_limit<S: AsyncCanSender, R: AsyncCanReceiver>(
        sdo_client: &mut zencan_client::SdoClient<S, R>,
    ) -> Result<FetTemperatureLimit, String> {
        let raw = sdo_client
            .read_i32(0x2404, 0x01)
            .await
            .map_err(|e| e.to_string())?;
        Ok(FetTemperatureLimit { discharge_over_temp_c: raw as f32 / 10.0 })
    }

    pub async fn sdo_read_cell_temperature<S: AsyncCanSender, R: AsyncCanReceiver>(
        sdo_client: &mut zencan_client::SdoClient<S, R>,
    ) -> Result<CellTemperature, String> {
        let t1 = sdo_client
            .read_i32(0x2500, 0x01)
            .await
            .map_err(|e| e.to_string())?;
        let t2 = sdo_client
            .read_i32(0x2500, 0x02)
            .await
            .map_err(|e| e.to_string())?;
        let t3 = sdo_client
            .read_i32(0x2500, 0x03)
            .await
            .map_err(|e| e.to_string())?;
        let t4 = sdo_client
            .read_i32(0x2500, 0x04)
            .await
            .map_err(|e| e.to_string())?;
        let t5 = sdo_client
            .read_i32(0x2500, 0x05)
            .await
            .map_err(|e| e.to_string())?;
        let t6 = sdo_client
            .read_i32(0x2500, 0x06)
            .await
            .map_err(|e| e.to_string())?;
        Ok(CellTemperature {
            temperature_1_c: t1 as f32 / 10.0,
            temperature_2_c: t2 as f32 / 10.0,
            temperature_3_c: t3 as f32 / 10.0,
            temperature_4_c: t4 as f32 / 10.0,
            temperature_5_c: t5 as f32 / 10.0,
            temperature_6_c: t6 as f32 / 10.0,
        })
    }

    pub async fn sdo_read_cell_temperature_min_max<S: AsyncCanSender, R: AsyncCanReceiver>(
        sdo_client: &mut zencan_client::SdoClient<S, R>,
    ) -> Result<CellTemperatureMinMax, String> {
        let min = sdo_client
            .read_i32(0x2502, 0x01)
            .await
            .map_err(|e| e.to_string())?;
        let max = sdo_client
            .read_i32(0x2502, 0x02)
            .await
            .map_err(|e| e.to_string())?;
        Ok(CellTemperatureMinMax {
            min_temperature_c: min as f32 / 10.0,
            max_temperature_c: max as f32 / 10.0,
        })
    }

    pub async fn sdo_read_cell_temperature_limit<S: AsyncCanSender, R: AsyncCanReceiver>(
        sdo_client: &mut zencan_client::SdoClient<S, R>,
    ) -> Result<CellTemperatureLimit, String> {
        let raw = sdo_client
            .read_i32(0x2504, 0x01)
            .await
            .map_err(|e| e.to_string())?;
        Ok(CellTemperatureLimit { discharge_over_temp_c: raw as f32 / 10.0 })
    }

    pub async fn sdo_read_cell_balance_status<S: AsyncCanSender, R: AsyncCanReceiver>(
        sdo_client: &mut zencan_client::SdoClient<S, R>,
    ) -> Result<CellBalanceStatus, String> {
        let s1 = sdo_client
            .read_u16(0x2600, 0x01)
            .await
            .map_err(|e| e.to_string())?;
        let s2 = sdo_client
            .read_u16(0x2600, 0x02)
            .await
            .map_err(|e| e.to_string())?;
        let s3 = sdo_client
            .read_u16(0x2600, 0x03)
            .await
            .map_err(|e| e.to_string())?;
        Ok(CellBalanceStatus {
            balance_status_register: s1,
            balance_fet_active: s2,
            balance_fet_active_persistent: s3,
        })
    }

    pub async fn sdo_read_cell_balance_limit<S: AsyncCanSender, R: AsyncCanReceiver>(
        sdo_client: &mut zencan_client::SdoClient<S, R>,
    ) -> Result<CellBalanceLimit, String> {
        let raw = sdo_client
            .read_u32(0x2604, 0x01)
            .await
            .map_err(|e| e.to_string())?;
        Ok(CellBalanceLimit {
            balance_start_diff_voltage_v: raw as f32 / 1000.0,
        })
    }

    pub async fn sdo_read_cell_impedance<S: AsyncCanSender, R: AsyncCanReceiver>(
        sdo_client: &mut zencan_client::SdoClient<S, R>,
    ) -> Result<CellImpedance, String> {
        let mut raw = [0u16; 18];
        for i in 0..18u8 {
            raw[i as usize] = sdo_client
                .read_u16(0x2700, i + 1)
                .await
                .map_err(|e| e.to_string())?;
        }
        let mut cell_impedances_mohm = [0u16; 16];
        cell_impedances_mohm.copy_from_slice(&raw[..16]);
        Ok(CellImpedance {
            cell_impedances_mohm,
            low_temp_factor: raw[16],
            high_temp_factor: raw[17],
        })
    }

    pub async fn sdo_read_battery_capacity<S: AsyncCanSender, R: AsyncCanReceiver>(
        sdo_client: &mut zencan_client::SdoClient<S, R>,
    ) -> Result<BatteryCapacity, String> {
        let design = sdo_client
            .read_u32(0x2800, 0x01)
            .await
            .map_err(|e| e.to_string())?;
        let fcc = sdo_client
            .read_u32(0x2800, 0x02)
            .await
            .map_err(|e| e.to_string())?;
        let remain = sdo_client
            .read_u32(0x2800, 0x03)
            .await
            .map_err(|e| e.to_string())?;
        let soc = sdo_client
            .read_u8(0x2800, 0x04)
            .await
            .map_err(|e| e.to_string())?;
        let soh = sdo_client
            .read_u8(0x2800, 0x05)
            .await
            .map_err(|e| e.to_string())?;
        let discharged = sdo_client
            .read_u32(0x2800, 0x06)
            .await
            .map_err(|e| e.to_string())?;
        let charged = sdo_client
            .read_u32(0x2800, 0x07)
            .await
            .map_err(|e| e.to_string())?;
        Ok(BatteryCapacity {
            design_capacity_ah: design as f32 / 1000.0,
            full_charge_capacity_ah: fcc as f32 / 1000.0,
            remaining_capacity_ah: remain as f32 / 1000.0,
            soc_percent: soc as f32,
            soh_percent: soh as f32,
            total_discharged_capacity_ah: discharged as f32 / 1000.0,
            total_charged_capacity_ah: charged as f32 / 1000.0,
        })
    }

    pub async fn sdo_read_battery_capacity_param<S: AsyncCanSender, R: AsyncCanReceiver>(
        sdo_client: &mut zencan_client::SdoClient<S, R>,
    ) -> Result<u8, String> {
        sdo_client
            .read_u8(0x2804, 0x01)
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn sdo_read_battery_cycle_count<S: AsyncCanSender, R: AsyncCanReceiver>(
        sdo_client: &mut zencan_client::SdoClient<S, R>,
    ) -> Result<BatteryCycleCount, String> {
        let c1 = sdo_client
            .read_u32(0x2900, 0x01)
            .await
            .map_err(|e| e.to_string())?;
        let c2 = sdo_client
            .read_u32(0x2900, 0x02)
            .await
            .map_err(|e| e.to_string())?;
        let c3 = sdo_client
            .read_u32(0x2900, 0x03)
            .await
            .map_err(|e| e.to_string())?;
        let c4 = sdo_client
            .read_u32(0x2900, 0x04)
            .await
            .map_err(|e| e.to_string())?;
        let c5 = sdo_client
            .read_u32(0x2900, 0x05)
            .await
            .map_err(|e| e.to_string())?;
        let c6 = sdo_client
            .read_u32(0x2900, 0x06)
            .await
            .map_err(|e| e.to_string())?;
        let c7 = sdo_client
            .read_u32(0x2900, 0x07)
            .await
            .map_err(|e| e.to_string())?;
        let c8 = sdo_client
            .read_u32(0x2900, 0x08)
            .await
            .map_err(|e| e.to_string())?;
        let c9 = sdo_client
            .read_u32(0x2900, 0x09)
            .await
            .map_err(|e| e.to_string())?;
        Ok(BatteryCycleCount {
            discharge_cycles: c1,
            discharge_learning_cycles: c2,
            discharge_cycles_after_learning: c3,
            charge_cycles_completed: c4,
            charge_cycles_started: c5,
            discharge_use_detect: c6,
            charge_use_low_temperature: c7,
            charge_use_normal_temperature: c8,
            charge_use_high_temperature: c9,
        })
    }

    pub async fn sdo_read_battery_charge_voltage<S: AsyncCanSender, R: AsyncCanReceiver>(
        sdo_client: &mut zencan_client::SdoClient<S, R>,
    ) -> Result<BatteryChargeVoltage, String> {
        let v1 = sdo_client
            .read_u32(0x3000, 0x01)
            .await
            .map_err(|e| e.to_string())?;
        let v2 = sdo_client
            .read_u32(0x3000, 0x02)
            .await
            .map_err(|e| e.to_string())?;
        let v3 = sdo_client
            .read_u32(0x3000, 0x03)
            .await
            .map_err(|e| e.to_string())?;
        Ok(BatteryChargeVoltage {
            charge_voltage_valid_v: v1 as f32 / 1000.0,
            charge_max_voltage_v: v2 as f32 / 1000.0,
            charge_keep_power_voltage_v: v3 as f32 / 1000.0,
        })
    }

    pub async fn sdo_read_battery_charge_current<S: AsyncCanSender, R: AsyncCanReceiver>(
        sdo_client: &mut zencan_client::SdoClient<S, R>,
    ) -> Result<BatteryChargeCurrent, String> {
        let c1 = sdo_client
            .read_u32(0x3100, 0x01)
            .await
            .map_err(|e| e.to_string())?;
        let c2 = sdo_client
            .read_u32(0x3100, 0x02)
            .await
            .map_err(|e| e.to_string())?;
        let c3 = sdo_client
            .read_u32(0x3100, 0x03)
            .await
            .map_err(|e| e.to_string())?;
        let c4 = sdo_client
            .read_u32(0x3100, 0x04)
            .await
            .map_err(|e| e.to_string())?;
        let c5 = sdo_client
            .read_u32(0x3100, 0x05)
            .await
            .map_err(|e| e.to_string())?;
        let t1 = sdo_client
            .read_u16(0x3100, 0x06)
            .await
            .map_err(|e| e.to_string())?;
        let t2 = sdo_client
            .read_u16(0x3100, 0x07)
            .await
            .map_err(|e| e.to_string())?;
        let t3 = sdo_client
            .read_u16(0x3100, 0x08)
            .await
            .map_err(|e| e.to_string())?;
        let t4 = sdo_client
            .read_u16(0x3100, 0x09)
            .await
            .map_err(|e| e.to_string())?;
        let c10 = sdo_client
            .read_u32(0x3100, 0x0a)
            .await
            .map_err(|e| e.to_string())?;
        Ok(BatteryChargeCurrent {
            charge_current_valid_a: c1 as f32 / 1000.0,
            charge_max_current_n_a: c2 as f32 / 1000.0,
            charge_max_current_low_a: c3 as f32 / 1000.0,
            charge_max_current_high_a: c4 as f32 / 1000.0,
            charge_keep_power_current_a: c5 as f32 / 1000.0,
            charge_temp_min_low_c: t1 as f32 / 10.0,
            charge_temp_min_normal_c: t2 as f32 / 10.0,
            charge_temp_max_normal_c: t3 as f32 / 10.0,
            charge_temp_max_high_c: t4 as f32 / 10.0,
            charge_current_config: c10,
        })
    }

    pub async fn sdo_read_battery_charge_temperature<S: AsyncCanSender, R: AsyncCanReceiver>(
        sdo_client: &mut zencan_client::SdoClient<S, R>,
    ) -> Result<BatteryChargeTemperature, String> {
        let t1 = sdo_client
            .read_i32(0x3200, 0x01)
            .await
            .map_err(|e| e.to_string())?;
        let t2 = sdo_client
            .read_i32(0x3200, 0x02)
            .await
            .map_err(|e| e.to_string())?;
        let t3 = sdo_client
            .read_i32(0x3200, 0x03)
            .await
            .map_err(|e| e.to_string())?;
        let t4 = sdo_client
            .read_i32(0x3200, 0x04)
            .await
            .map_err(|e| e.to_string())?;
        let t5 = sdo_client
            .read_i32(0x3200, 0x05)
            .await
            .map_err(|e| e.to_string())?;
        let t6 = sdo_client
            .read_i32(0x3200, 0x06)
            .await
            .map_err(|e| e.to_string())?;
        Ok(BatteryChargeTemperature {
            temp_min_low_c: t1 as f32 / 10.0,
            temp_min_normal_c: t2 as f32 / 10.0,
            temp_max_normal_c: t3 as f32 / 10.0,
            temp_max_high_c: t4 as f32 / 10.0,
            temp_5_c: t5 as f32 / 10.0,
            temp_6_c: t6 as f32 / 10.0,
        })
    }

    pub async fn sdo_read_master_battery_temperature<S: AsyncCanSender, R: AsyncCanReceiver>(
        sdo_client: &mut zencan_client::SdoClient<S, R>,
    ) -> Result<MasterBatteryTemperature, String> {
        let t1 = sdo_client
            .read_i32(0x3700, 0x01)
            .await
            .map_err(|e| e.to_string())?;
        let t2 = sdo_client
            .read_i32(0x3700, 0x02)
            .await
            .map_err(|e| e.to_string())?;
        Ok(MasterBatteryTemperature {
            max_fet_temperature_c: t1 as f32 / 10.0,
            max_cell_temperature_c: t2 as f32 / 10.0,
        })
    }

    /// Returns the easyblade at the given index (0-based) among active modules.
    pub fn get_easyblade_by_index(&self, index: usize) -> Option<&VartaEasyblade> {
        self.easyblades.iter().filter_map(|e| e.as_ref()).nth(index)
    }

    /// Returns the total number of active easyblade modules.
    pub fn easyblade_count(&self) -> usize {
        self.easyblades.iter().filter_map(|e| e.as_ref()).count()
    }
}

// Private API
impl Varta {
    fn update_easyblade_voltage_current(
        easyblade: &mut VartaEasyblade,
        voltage: f32,
        current: f32,
    ) {
        easyblade.voltage = Some(voltage);
        easyblade.current = Some(current);
        easyblade.last_seen = std::time::SystemTime::now();
    }

    fn update_easyblade_soc_soh(
        easyblade: &mut VartaEasyblade,
        full_cap: f32,
        rem_cap: f32,
        design_cap: f32,
    ) {
        easyblade.soc = if full_cap > 0.0 {
            Some((rem_cap / full_cap) * 100.0)
        } else {
            None
        };
        easyblade.soh = if design_cap > 0.0 {
            Some((full_cap / design_cap) * 100.0)
        } else {
            None
        };
        easyblade.last_seen = std::time::SystemTime::now();
    }

    fn update_easyblade_fet_status(
        easyblade: &mut VartaEasyblade,
        charge_fet: bool,
        discharge_fet: bool,
        bypass_fet: bool,
    ) {
        easyblade.fet_status = Some((charge_fet, discharge_fet, bypass_fet));
        easyblade.last_seen = std::time::SystemTime::now();
    }
}
