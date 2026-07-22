use socketcan::{CanFilter, EmbeddedFrame, SocketOptions};

use crate::Error;
use crate::MAX_MODULES;
use crate::MasterInfo;
use crate::SdoRequest;
use crate::SdoResponse;
use crate::VartaEasyblade;
use crate::varta_easyblade;
use crate::varta_easyblade_can_messages;

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
    pub sdo_response_tx: tokio::sync::mpsc::UnboundedSender<SdoResponse>,
}

// Public API
impl Varta {
    pub async fn new(
        canbus_interface: &str,
    ) -> Result<(Self, tokio::sync::mpsc::UnboundedReceiver<SdoResponse>), Error> {
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

        let (sdo_response_tx, sdo_response_rx) = tokio::sync::mpsc::unbounded_channel();

        let varta = Self {
            socketcan_interface,
            canbus_interface: String::from(canbus_interface),
            master: MasterInfo::default(),
            easyblades: [const { None }; MAX_MODULES],
            sdo_response_tx,
        };
        Ok((varta, sdo_response_rx))
    }

    pub fn send_sdo_request(&self, node_id: u8, request: SdoRequest) {
        if let Some(Some(eb)) = self.easyblades.get(node_id as usize) {
            let _ = eb.sdo_request_tx.send(request);
        }
    }

    pub async fn process_socketcan_msg(&mut self) -> Result<(), Error> {
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

        if let Some(node_id) = node_id {
            let was_new = self.easyblades[node_id as usize].is_none();
            let easyblade = self.get_or_init_easyblade(node_id);
            if was_new {
                easyblade.sdo_request_tx.send(SdoRequest::SerialNumber).ok();
                easyblade
                    .sdo_request_tx
                    .send(SdoRequest::SoftwareVersion)
                    .ok();
                easyblade
                    .sdo_request_tx
                    .send(SdoRequest::HardwareVersion)
                    .ok();
                easyblade
                    .sdo_request_tx
                    .send(SdoRequest::DeviceErrorHistory)
                    .ok();
                easyblade.sdo_request_tx.send(SdoRequest::CellVoltages).ok();
                easyblade
                    .sdo_request_tx
                    .send(SdoRequest::DeviceConfigInfo)
                    .ok();
                easyblade
                    .sdo_request_tx
                    .send(SdoRequest::DeviceSerialNumberInfo)
                    .ok();
                easyblade
                    .sdo_request_tx
                    .send(SdoRequest::DeviceDateInfo)
                    .ok();
                easyblade
                    .sdo_request_tx
                    .send(SdoRequest::DeviceVariantInfo)
                    .ok();
                easyblade
                    .sdo_request_tx
                    .send(SdoRequest::DeviceControlParam)
                    .ok();
                easyblade
                    .sdo_request_tx
                    .send(SdoRequest::DeviceOperationTime)
                    .ok();
                easyblade
                    .sdo_request_tx
                    .send(SdoRequest::DeviceErrorCounter)
                    .ok();
                easyblade
                    .sdo_request_tx
                    .send(SdoRequest::CellVoltageMinMax)
                    .ok();
                easyblade
                    .sdo_request_tx
                    .send(SdoRequest::CellVoltageLimit)
                    .ok();
                easyblade
                    .sdo_request_tx
                    .send(SdoRequest::BatteryVoltage)
                    .ok();
                easyblade
                    .sdo_request_tx
                    .send(SdoRequest::BatteryVoltageLimit)
                    .ok();
                easyblade
                    .sdo_request_tx
                    .send(SdoRequest::BatteryCurrent)
                    .ok();
                easyblade
                    .sdo_request_tx
                    .send(SdoRequest::BatteryCurrentLimit)
                    .ok();
                easyblade
                    .sdo_request_tx
                    .send(SdoRequest::FetTemperature)
                    .ok();
                easyblade
                    .sdo_request_tx
                    .send(SdoRequest::FetTemperatureMinMax)
                    .ok();
                easyblade
                    .sdo_request_tx
                    .send(SdoRequest::FetTemperatureLimit)
                    .ok();
                easyblade
                    .sdo_request_tx
                    .send(SdoRequest::CellTemperature)
                    .ok();
                easyblade
                    .sdo_request_tx
                    .send(SdoRequest::CellTemperatureMinMax)
                    .ok();
                easyblade
                    .sdo_request_tx
                    .send(SdoRequest::CellTemperatureLimit)
                    .ok();
                easyblade
                    .sdo_request_tx
                    .send(SdoRequest::CellBalanceStatus)
                    .ok();
                easyblade
                    .sdo_request_tx
                    .send(SdoRequest::CellBalanceLimit)
                    .ok();
                easyblade
                    .sdo_request_tx
                    .send(SdoRequest::CellImpedance)
                    .ok();
                easyblade
                    .sdo_request_tx
                    .send(SdoRequest::BatteryCapacity)
                    .ok();
                easyblade
                    .sdo_request_tx
                    .send(SdoRequest::BatteryCapacityParam)
                    .ok();
                easyblade
                    .sdo_request_tx
                    .send(SdoRequest::BatteryCycleCount)
                    .ok();
                easyblade
                    .sdo_request_tx
                    .send(SdoRequest::BatteryChargeVoltage)
                    .ok();
                easyblade
                    .sdo_request_tx
                    .send(SdoRequest::BatteryChargeCurrent)
                    .ok();
                easyblade
                    .sdo_request_tx
                    .send(SdoRequest::BatteryChargeTemperature)
                    .ok();
                easyblade
                    .sdo_request_tx
                    .send(SdoRequest::MasterBatteryTemperature)
                    .ok();
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

        Ok(())
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
                && let Some(eb) = entry.take()
            {
                eb.cancellation_token.cancel();
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
            let (sdo_request_tx, sdo_request_rx) = tokio::sync::mpsc::unbounded_channel();
            let (socketcan_tx, socketcan_rx) =
                zencan_client::open_socketcan(&self.canbus_interface).unwrap();
            let sdo_response_tx = self.sdo_response_tx.clone();
            let cancellation_token = tokio_util::sync::CancellationToken::new();
            let cancellation_token_clone = cancellation_token.clone();

            let task_handle = tokio::spawn(async move {
                Self::easyblade_task(
                    node_id,
                    socketcan_tx,
                    socketcan_rx,
                    sdo_request_rx,
                    sdo_response_tx,
                    cancellation_token_clone,
                )
                .await;
            });

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
                sdo_request_tx,
                task_handle,
                cancellation_token,
            });
        }
        self.easyblades[node_id as usize].as_mut().unwrap()
    }

    async fn easyblade_task(
        node_id: u8,
        socketcan_tx: zencan_client::common::SocketCanSender,
        socketcan_rx: zencan_client::common::SocketCanReceiver,
        mut sdo_request_rx: tokio::sync::mpsc::UnboundedReceiver<SdoRequest>,
        sdo_response_tx: tokio::sync::mpsc::UnboundedSender<SdoResponse>,
        cancellation_token: tokio_util::sync::CancellationToken,
    ) {
        let mut sdo_client = zencan_client::SdoClient::new_std(node_id, socketcan_tx, socketcan_rx);
        loop {
            tokio::select! {
                _ = cancellation_token.cancelled() => {
                    break;
                }
                request = sdo_request_rx.recv() => {
                    match request {
                        Some(SdoRequest::SerialNumber) => {
                            let value = async {
                                let bytes = sdo_client.upload(0x2004, 0x01)
                                    .await
                                    .map_err(|e| e.to_string())?;
                                if bytes.len() < 2 {
                                    return Err("Serial number data too short".to_string());
                                }
                                Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
                            }.await;
                            let _ = sdo_response_tx.send(SdoResponse::SerialNumber { node_id, value });
                        },
                        Some(SdoRequest::SoftwareVersion) => {
                            let value = async {
                                let sw = sdo_client.upload(0x2000, 0x02)
                                    .await
                                    .map_err(|e| e.to_string())?;
                                let fw = sdo_client.upload(0x2000, 0x03)
                                    .await
                                    .map_err(|e| e.to_string())?;
                                let sw_str = String::from_utf8_lossy(&sw).trim_matches('\0').to_string();
                                let fw_str = String::from_utf8_lossy(&fw).trim_matches('\0').to_string();
                                Ok(format!("{}{}", sw_str, fw_str))
                            }.await;
                            let _ = sdo_response_tx.send(SdoResponse::SoftwareVersion { node_id, value });
                        },
                        Some(SdoRequest::HardwareVersion) => {
                            let value = async {
                                let bytes = sdo_client.upload(0x2000, 0x01)
                                    .await
                                    .map_err(|e| e.to_string())?;
                                Ok(String::from_utf8_lossy(&bytes).trim_matches('\0').to_string())
                            }.await;
                            let _ = sdo_response_tx.send(SdoResponse::HardwareVersion { node_id, value });
                        },
                        Some(SdoRequest::DeviceErrorHistory) => {
                            let value = async {
                                let highest_subindex = sdo_client.read_u8(0x2018, 0x00)
                                    .await
                                    .map_err(|e| e.to_string())?;
                                if highest_subindex != 16 {
                                    return Err(format!("Expected 16 error entries, got {}", highest_subindex));
                                }
                                let mut errors = Vec::new();
                                for sub_index in 1..=16u8 {
                                    let val = sdo_client.read_u8(0x2018, sub_index)
                                        .await
                                        .map_err(|e| e.to_string())?;
                                    let e: varta_easyblade::DeviceError =
                                        match varta_easyblade::DeviceError::try_from(val) {
                                            Ok(e) => e,
                                            Err(_) => varta_easyblade::DeviceError::Unknown,
                                        };
                                    errors.push(e);
                                }
                                Ok(errors)
                            }.await;
                            let _ = sdo_response_tx.send(SdoResponse::DeviceErrorHistory { node_id, value });
                        },
                        Some(SdoRequest::CellVoltages) => {
                            let value = async {
                                let highest_subindex = sdo_client.read_u8(0x2100, 0x00)
                                    .await
                                    .map_err(|e| e.to_string())?;
                                if highest_subindex != 16 {
                                    return Err(format!("Expected 16 cell voltage entries, got {}", highest_subindex));
                                }
                                // 48V Easyblade is 14S, so only sub-indices 1..=14 are valid
                                let mut cell_voltages: Vec<f32> = Vec::new();
                                for sub_index in 1..=14u8 {
                                    let val = sdo_client.read_u32(0x2100, sub_index)
                                        .await
                                        .map_err(|e| e.to_string())?;
                                    cell_voltages.push((val as f32) / 1000.0);
                                }
                                Ok(cell_voltages)
                            }.await;
                            let _ = sdo_response_tx.send(SdoResponse::CellVoltages { node_id, value });
                        },
                        Some(SdoRequest::DeviceConfigInfo) => {
                            let value = async {
                                let c1 = sdo_client.upload(0x2002, 0x01)
                                    .await
                                    .map_err(|e| e.to_string())?;
                                let c2 = sdo_client.upload(0x2002, 0x02)
                                    .await
                                    .map_err(|e| e.to_string())?;
                                let c3 = sdo_client.upload(0x2002, 0x03)
                                    .await
                                    .map_err(|e| e.to_string())?;
                                Ok((
                                    String::from_utf8_lossy(&c1).trim_matches('\0').to_string(),
                                    String::from_utf8_lossy(&c2).trim_matches('\0').to_string(),
                                    String::from_utf8_lossy(&c3).trim_matches('\0').to_string(),
                                ))
                            }.await;
                            let _ = sdo_response_tx.send(SdoResponse::DeviceConfigInfo { node_id, value });
                        },
                        Some(SdoRequest::DeviceSerialNumberInfo) => {
                            let value = async {
                                let s1 = sdo_client.read_u32(0x2004, 0x01)
                                    .await
                                    .map_err(|e| e.to_string())?;
                                let s2 = sdo_client.read_u32(0x2004, 0x02)
                                    .await
                                    .map_err(|e| e.to_string())?;
                                let s3 = sdo_client.read_u32(0x2004, 0x03)
                                    .await
                                    .map_err(|e| e.to_string())?;
                                Ok((s1, s2, s3))
                            }.await;
                            let _ = sdo_response_tx.send(SdoResponse::DeviceSerialNumberInfo { node_id, value });
                        },
                        Some(SdoRequest::DeviceDateInfo) => {
                            let value = async {
                                let d1 = sdo_client.read_u16(0x2006, 0x01)
                                    .await
                                    .map_err(|e| e.to_string())?;
                                let d2 = sdo_client.read_u16(0x2006, 0x02)
                                    .await
                                    .map_err(|e| e.to_string())?;
                                let d3 = sdo_client.read_u16(0x2006, 0x03)
                                    .await
                                    .map_err(|e| e.to_string())?;
                                Ok((d1, d2, d3))
                            }.await;
                            let _ = sdo_response_tx.send(SdoResponse::DeviceDateInfo { node_id, value });
                        },
                        Some(SdoRequest::DeviceVariantInfo) => {
                            let value = async {
                                let s1 = sdo_client.read_u8(0x2008, 0x01)
                                    .await
                                    .map_err(|e| e.to_string())?;
                                let s2 = sdo_client.read_u8(0x2008, 0x02)
                                    .await
                                    .map_err(|e| e.to_string())?;
                                let s3 = sdo_client.read_u8(0x2008, 0x03)
                                    .await
                                    .map_err(|e| e.to_string())?;
                                let s4 = sdo_client.read_u16(0x2008, 0x04)
                                    .await
                                    .map_err(|e| e.to_string())?;
                                let s5 = sdo_client.read_u16(0x2008, 0x05)
                                    .await
                                    .map_err(|e| e.to_string())?;
                                let s6 = sdo_client.read_u16(0x2008, 0x06)
                                    .await
                                    .map_err(|e| e.to_string())?;
                                let s7 = sdo_client.read_u8(0x2008, 0x07)
                                    .await
                                    .map_err(|e| e.to_string())?;
                                Ok((s1, s2, s3, s4, s5, s6, s7))
                            }.await;
                            let _ = sdo_response_tx.send(SdoResponse::DeviceVariantInfo { node_id, value });
                        },
                        Some(SdoRequest::DeviceControlParam) => {
                            let value = async {
                                let v = sdo_client.read_u16(0x2010, 0x01)
                                    .await
                                    .map_err(|e| e.to_string())?;
                                Ok(v)
                            }.await;
                            let _ = sdo_response_tx.send(SdoResponse::DeviceControlParam { node_id, value });
                        },
                        Some(SdoRequest::DeviceOperationTime) => {
                            let value = async {
                                let m1 = sdo_client.read_u8(0x2016, 0x01)
                                    .await
                                    .map_err(|e| e.to_string())?;
                                let m2 = sdo_client.read_u8(0x2016, 0x02)
                                    .await
                                    .map_err(|e| e.to_string())?;
                                let m3 = sdo_client.read_u8(0x2016, 0x03)
                                    .await
                                    .map_err(|e| e.to_string())?;
                                let m4 = sdo_client.read_u8(0x2016, 0x04)
                                    .await
                                    .map_err(|e| e.to_string())?;
                                let m5 = sdo_client.read_u8(0x2016, 0x05)
                                    .await
                                    .map_err(|e| e.to_string())?;
                                let h1 = sdo_client.read_u32(0x2016, 0x06)
                                    .await
                                    .map_err(|e| e.to_string())?;
                                let h2 = sdo_client.read_u32(0x2016, 0x07)
                                    .await
                                    .map_err(|e| e.to_string())?;
                                let h3 = sdo_client.read_u32(0x2016, 0x08)
                                    .await
                                    .map_err(|e| e.to_string())?;
                                let h4 = sdo_client.read_u32(0x2016, 0x09)
                                    .await
                                    .map_err(|e| e.to_string())?;
                                let h5 = sdo_client.read_u32(0x2016, 0x0a)
                                    .await
                                    .map_err(|e| e.to_string())?;
                                Ok((m1, m2, m3, m4, m5, h1, h2, h3, h4, h5))
                            }.await;
                            let _ = sdo_response_tx.send(SdoResponse::DeviceOperationTime { node_id, value });
                        },
                        Some(SdoRequest::DeviceErrorCounter) => {
                            let value = async {
                                let highest_subindex = sdo_client.read_u8(0x201a, 0x00)
                                    .await
                                    .map_err(|e| e.to_string())?;
                                let mut counters = Vec::new();
                                for sub_index in 1..=highest_subindex {
                                    let val = sdo_client.read_u16(0x201a, sub_index)
                                        .await
                                        .map_err(|e| e.to_string())?;
                                    counters.push(val);
                                }
                                Ok(counters)
                            }.await;
                            let _ = sdo_response_tx.send(SdoResponse::DeviceErrorCounter { node_id, value });
                        },
                        Some(SdoRequest::CellVoltageMinMax) => {
                            let value = async {
                                let min = sdo_client.read_u32(0x2102, 0x01)
                                    .await
                                    .map_err(|e| e.to_string())?;
                                let max = sdo_client.read_u32(0x2102, 0x02)
                                    .await
                                    .map_err(|e| e.to_string())?;
                                Ok((min, max))
                            }.await;
                            let _ = sdo_response_tx.send(SdoResponse::CellVoltageMinMax { node_id, value });
                        },
                        Some(SdoRequest::CellVoltageLimit) => {
                            let value = async {
                                let v = sdo_client.read_u32(0x2104, 0x01)
                                    .await
                                    .map_err(|e| e.to_string())?;
                                Ok(v)
                            }.await;
                            let _ = sdo_response_tx.send(SdoResponse::CellVoltageLimit { node_id, value });
                        },
                        Some(SdoRequest::BatteryVoltage) => {
                            let value = async {
                                let v1 = sdo_client.read_u32(0x2200, 0x01)
                                    .await
                                    .map_err(|e| e.to_string())?;
                                let v2 = sdo_client.read_u32(0x2200, 0x02)
                                    .await
                                    .map_err(|e| e.to_string())?;
                                let v3 = sdo_client.read_u32(0x2200, 0x03)
                                    .await
                                    .map_err(|e| e.to_string())?;
                                Ok((v1, v2, v3))
                            }.await;
                            let _ = sdo_response_tx.send(SdoResponse::BatteryVoltage { node_id, value });
                        },
                        Some(SdoRequest::BatteryVoltageLimit) => {
                            let value = async {
                                let v = sdo_client.read_u32(0x2204, 0x01)
                                    .await
                                    .map_err(|e| e.to_string())?;
                                Ok(v)
                            }.await;
                            let _ = sdo_response_tx.send(SdoResponse::BatteryVoltageLimit { node_id, value });
                        },
                        Some(SdoRequest::BatteryCurrent) => {
                            let value = async {
                                let c1 = sdo_client.read_i32(0x2300, 0x01)
                                    .await
                                    .map_err(|e| e.to_string())?;
                                let c2 = sdo_client.read_i32(0x2300, 0x02)
                                    .await
                                    .map_err(|e| e.to_string())?;
                                let c3 = sdo_client.read_i32(0x2300, 0x03)
                                    .await
                                    .map_err(|e| e.to_string())?;
                                let c4 = sdo_client.read_i32(0x2300, 0x04)
                                    .await
                                    .map_err(|e| e.to_string())?;
                                let c5 = sdo_client.read_i32(0x2300, 0x05)
                                    .await
                                    .map_err(|e| e.to_string())?;
                                Ok((c1, c2, c3, c4, c5))
                            }.await;
                            let _ = sdo_response_tx.send(SdoResponse::BatteryCurrent { node_id, value });
                        },
                        Some(SdoRequest::BatteryCurrentLimit) => {
                            let value = async {
                                let v = sdo_client.read_i32(0x2304, 0x01)
                                    .await
                                    .map_err(|e| e.to_string())?;
                                Ok(v)
                            }.await;
                            let _ = sdo_response_tx.send(SdoResponse::BatteryCurrentLimit { node_id, value });
                        },
                        Some(SdoRequest::FetTemperature) => {
                            let value = async {
                                let t1 = sdo_client.read_i32(0x2400, 0x01)
                                    .await
                                    .map_err(|e| e.to_string())?;
                                let t2 = sdo_client.read_i32(0x2400, 0x02)
                                    .await
                                    .map_err(|e| e.to_string())?;
                                Ok((t1, t2))
                            }.await;
                            let _ = sdo_response_tx.send(SdoResponse::FetTemperature { node_id, value });
                        },
                        Some(SdoRequest::FetTemperatureMinMax) => {
                            let value = async {
                                let min = sdo_client.read_i32(0x2402, 0x01)
                                    .await
                                    .map_err(|e| e.to_string())?;
                                let max = sdo_client.read_i32(0x2402, 0x02)
                                    .await
                                    .map_err(|e| e.to_string())?;
                                Ok((min, max))
                            }.await;
                            let _ = sdo_response_tx.send(SdoResponse::FetTemperatureMinMax { node_id, value });
                        },
                        Some(SdoRequest::FetTemperatureLimit) => {
                            let value = async {
                                let v = sdo_client.read_i32(0x2404, 0x01)
                                    .await
                                    .map_err(|e| e.to_string())?;
                                Ok(v)
                            }.await;
                            let _ = sdo_response_tx.send(SdoResponse::FetTemperatureLimit { node_id, value });
                        },
                        Some(SdoRequest::CellTemperature) => {
                            let value = async {
                                let t1 = sdo_client.read_i32(0x2500, 0x01)
                                    .await
                                    .map_err(|e| e.to_string())?;
                                let t2 = sdo_client.read_i32(0x2500, 0x02)
                                    .await
                                    .map_err(|e| e.to_string())?;
                                let t3 = sdo_client.read_i32(0x2500, 0x03)
                                    .await
                                    .map_err(|e| e.to_string())?;
                                let t4 = sdo_client.read_i32(0x2500, 0x04)
                                    .await
                                    .map_err(|e| e.to_string())?;
                                let t5 = sdo_client.read_i32(0x2500, 0x05)
                                    .await
                                    .map_err(|e| e.to_string())?;
                                let t6 = sdo_client.read_i32(0x2500, 0x06)
                                    .await
                                    .map_err(|e| e.to_string())?;
                                Ok((t1, t2, t3, t4, t5, t6))
                            }.await;
                            let _ = sdo_response_tx.send(SdoResponse::CellTemperature { node_id, value });
                        },
                        Some(SdoRequest::CellTemperatureMinMax) => {
                            let value = async {
                                let min = sdo_client.read_i32(0x2502, 0x01)
                                    .await
                                    .map_err(|e| e.to_string())?;
                                let max = sdo_client.read_i32(0x2502, 0x02)
                                    .await
                                    .map_err(|e| e.to_string())?;
                                Ok((min, max))
                            }.await;
                            let _ = sdo_response_tx.send(SdoResponse::CellTemperatureMinMax { node_id, value });
                        },
                        Some(SdoRequest::CellTemperatureLimit) => {
                            let value = async {
                                let v = sdo_client.read_i32(0x2504, 0x01)
                                    .await
                                    .map_err(|e| e.to_string())?;
                                Ok(v)
                            }.await;
                            let _ = sdo_response_tx.send(SdoResponse::CellTemperatureLimit { node_id, value });
                        },
                        Some(SdoRequest::CellBalanceStatus) => {
                            let value = async {
                                let s1 = sdo_client.read_u16(0x2600, 0x01)
                                    .await
                                    .map_err(|e| e.to_string())?;
                                let s2 = sdo_client.read_u16(0x2600, 0x02)
                                    .await
                                    .map_err(|e| e.to_string())?;
                                let s3 = sdo_client.read_u16(0x2600, 0x03)
                                    .await
                                    .map_err(|e| e.to_string())?;
                                Ok((s1, s2, s3))
                            }.await;
                            let _ = sdo_response_tx.send(SdoResponse::CellBalanceStatus { node_id, value });
                        },
                        Some(SdoRequest::CellBalanceLimit) => {
                            let value = async {
                                let v = sdo_client.read_u32(0x2604, 0x01)
                                    .await
                                    .map_err(|e| e.to_string())?;
                                Ok(v)
                            }.await;
                            let _ = sdo_response_tx.send(SdoResponse::CellBalanceLimit { node_id, value });
                        },
                        Some(SdoRequest::CellImpedance) => {
                            let value = async {
                                let mut vals = [0u16; 18];
                                for i in 0..18u8 {
                                    vals[i as usize] = sdo_client.read_u16(0x2700, i + 1)
                                        .await
                                        .map_err(|e| e.to_string())?;
                                }
                                Ok(vals)
                            }.await;
                            let _ = sdo_response_tx.send(SdoResponse::CellImpedance { node_id, value });
                        },
                        Some(SdoRequest::BatteryCapacity) => {
                            let value = async {
                                let design = sdo_client.read_u32(0x2800, 0x01)
                                    .await
                                    .map_err(|e| e.to_string())?;
                                let fcc = sdo_client.read_u32(0x2800, 0x02)
                                    .await
                                    .map_err(|e| e.to_string())?;
                                let remain = sdo_client.read_u32(0x2800, 0x03)
                                    .await
                                    .map_err(|e| e.to_string())?;
                                let soc = sdo_client.read_u8(0x2800, 0x04)
                                    .await
                                    .map_err(|e| e.to_string())?;
                                let soh = sdo_client.read_u8(0x2800, 0x05)
                                    .await
                                    .map_err(|e| e.to_string())?;
                                let discharged = sdo_client.read_u32(0x2800, 0x06)
                                    .await
                                    .map_err(|e| e.to_string())?;
                                let charged = sdo_client.read_u32(0x2800, 0x07)
                                    .await
                                    .map_err(|e| e.to_string())?;
                                Ok((design, fcc, remain, soc, soh, discharged, charged))
                            }.await;
                            let _ = sdo_response_tx.send(SdoResponse::BatteryCapacity { node_id, value });
                        },
                        Some(SdoRequest::BatteryCapacityParam) => {
                            let value = async {
                                let v = sdo_client.read_u8(0x2804, 0x01)
                                    .await
                                    .map_err(|e| e.to_string())?;
                                Ok(v)
                            }.await;
                            let _ = sdo_response_tx.send(SdoResponse::BatteryCapacityParam { node_id, value });
                        },
                        Some(SdoRequest::BatteryCycleCount) => {
                            let value = async {
                                let c1 = sdo_client.read_u32(0x2900, 0x01)
                                    .await
                                    .map_err(|e| e.to_string())?;
                                let c2 = sdo_client.read_u32(0x2900, 0x02)
                                    .await
                                    .map_err(|e| e.to_string())?;
                                let c3 = sdo_client.read_u32(0x2900, 0x03)
                                    .await
                                    .map_err(|e| e.to_string())?;
                                let c4 = sdo_client.read_u32(0x2900, 0x04)
                                    .await
                                    .map_err(|e| e.to_string())?;
                                let c5 = sdo_client.read_u32(0x2900, 0x05)
                                    .await
                                    .map_err(|e| e.to_string())?;
                                let c6 = sdo_client.read_u32(0x2900, 0x06)
                                    .await
                                    .map_err(|e| e.to_string())?;
                                let c7 = sdo_client.read_u32(0x2900, 0x07)
                                    .await
                                    .map_err(|e| e.to_string())?;
                                let c8 = sdo_client.read_u32(0x2900, 0x08)
                                    .await
                                    .map_err(|e| e.to_string())?;
                                let c9 = sdo_client.read_u32(0x2900, 0x09)
                                    .await
                                    .map_err(|e| e.to_string())?;
                                Ok((c1, c2, c3, c4, c5, c6, c7, c8, c9))
                            }.await;
                            let _ = sdo_response_tx.send(SdoResponse::BatteryCycleCount { node_id, value });
                        },
                        Some(SdoRequest::BatteryChargeVoltage) => {
                            let value = async {
                                let v1 = sdo_client.read_u32(0x3000, 0x01)
                                    .await
                                    .map_err(|e| e.to_string())?;
                                let v2 = sdo_client.read_u32(0x3000, 0x02)
                                    .await
                                    .map_err(|e| e.to_string())?;
                                let v3 = sdo_client.read_u32(0x3000, 0x03)
                                    .await
                                    .map_err(|e| e.to_string())?;
                                Ok((v1, v2, v3))
                            }.await;
                            let _ = sdo_response_tx.send(SdoResponse::BatteryChargeVoltage { node_id, value });
                        },
                        Some(SdoRequest::BatteryChargeCurrent) => {
                            let value = async {
                                let c1 = sdo_client.read_u32(0x3100, 0x01)
                                    .await
                                    .map_err(|e| e.to_string())?;
                                let c2 = sdo_client.read_u32(0x3100, 0x02)
                                    .await
                                    .map_err(|e| e.to_string())?;
                                let c3 = sdo_client.read_u32(0x3100, 0x03)
                                    .await
                                    .map_err(|e| e.to_string())?;
                                let c4 = sdo_client.read_u32(0x3100, 0x04)
                                    .await
                                    .map_err(|e| e.to_string())?;
                                let c5 = sdo_client.read_u32(0x3100, 0x05)
                                    .await
                                    .map_err(|e| e.to_string())?;
                                let c6 = sdo_client.read_u16(0x3100, 0x06)
                                    .await
                                    .map_err(|e| e.to_string())?;
                                let c7 = sdo_client.read_u16(0x3100, 0x07)
                                    .await
                                    .map_err(|e| e.to_string())?;
                                let c8 = sdo_client.read_u16(0x3100, 0x08)
                                    .await
                                    .map_err(|e| e.to_string())?;
                                let c9 = sdo_client.read_u16(0x3100, 0x09)
                                    .await
                                    .map_err(|e| e.to_string())?;
                                let c10 = sdo_client.read_u32(0x3100, 0x0a)
                                    .await
                                    .map_err(|e| e.to_string())?;
                                Ok((c1, c2, c3, c4, c5, c6, c7, c8, c9, c10))
                            }.await;
                            let _ = sdo_response_tx.send(SdoResponse::BatteryChargeCurrent { node_id, value });
                        },
                        Some(SdoRequest::BatteryChargeTemperature) => {
                            let value = async {
                                let t1 = sdo_client.read_i32(0x3200, 0x01)
                                    .await
                                    .map_err(|e| e.to_string())?;
                                let t2 = sdo_client.read_i32(0x3200, 0x02)
                                    .await
                                    .map_err(|e| e.to_string())?;
                                let t3 = sdo_client.read_i32(0x3200, 0x03)
                                    .await
                                    .map_err(|e| e.to_string())?;
                                let t4 = sdo_client.read_i32(0x3200, 0x04)
                                    .await
                                    .map_err(|e| e.to_string())?;
                                let t5 = sdo_client.read_i32(0x3200, 0x05)
                                    .await
                                    .map_err(|e| e.to_string())?;
                                let t6 = sdo_client.read_i32(0x3200, 0x06)
                                    .await
                                    .map_err(|e| e.to_string())?;
                                Ok((t1, t2, t3, t4, t5, t6))
                            }.await;
                            let _ = sdo_response_tx.send(SdoResponse::BatteryChargeTemperature { node_id, value });
                        },
                        Some(SdoRequest::MasterBatteryTemperature) => {
                            let value = async {
                                let t1 = sdo_client.read_i32(0x3700, 0x01)
                                    .await
                                    .map_err(|e| e.to_string())?;
                                let t2 = sdo_client.read_i32(0x3700, 0x02)
                                    .await
                                    .map_err(|e| e.to_string())?;
                                Ok((t1, t2))
                            }.await;
                            let _ = sdo_response_tx.send(SdoResponse::MasterBatteryTemperature { node_id, value });
                        },
                        None => {
                            break;
                        }
                    }
                }
            }
        }
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
