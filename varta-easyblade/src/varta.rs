use socketcan::{CanFilter, EmbeddedFrame, SocketOptions};

use crate::Error;
use crate::MAX_MODULES;
use crate::VartaEasyblade;
use crate::varta_easyblade;
use crate::varta_easyblade_can_messages;

#[derive(Debug)]
pub struct Varta {
    canbus_manager: zencan_client::BusManager<zencan_client::common::SocketCanSender>,
    pub socketcan_interface: socketcan::tokio::AsyncCanSocket<socketcan::CanSocket>,

    pub canbus_interface: String,
    pub easyblades: [Option<VartaEasyblade>; MAX_MODULES],
}

// Public API
impl Varta {
    pub async fn new(canbus_interface: &str) -> Result<Self, Error> {
        let (tx, rx) = zencan_client::open_socketcan(canbus_interface).map_err(|e| Error::Io {
            can_interface: String::from(canbus_interface),
            e,
        })?;
        let canbus_manager = zencan_client::BusManager::new(tx, rx);

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
                CanFilter::new(0x264, 0x7ff), // master charge control
                CanFilter::new(0x19b, 0x7ff), // master info1: voltage & current for pack
                CanFilter::new(0x29b, 0x7ff), // master info2: temps & design capacity
                CanFilter::new(0x39b, 0x7ff), // master info3: full & remaining capacity
            ])
            .map_err(|e| Error::Io {
                can_interface: String::from(canbus_interface),
                e,
            })?;

        let mut varta = Self {
            canbus_manager,
            socketcan_interface,
            canbus_interface: String::from(canbus_interface),
            easyblades: [const { None }; MAX_MODULES],
        };
        varta.scan().await?;
        Ok(varta)
    }

    /// Read and process one message from socketcan interface.
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

        match msg {
            varta_easyblade_can_messages::Messages::Pack01PackInfo1(pack01_packinfo1) => {
                self.update_easyblade_voltage_current(
                    1,
                    pack01_packinfo1.voltage(),
                    pack01_packinfo1.current(),
                )?;
            },

            varta_easyblade_can_messages::Messages::Pack02PackInfo1(pack02_packinfo1) => {
                self.update_easyblade_voltage_current(
                    2,
                    pack02_packinfo1.voltage(),
                    pack02_packinfo1.current(),
                )?;
            },

            varta_easyblade_can_messages::Messages::Pack03PackInfo1(pack03_packinfo1) => {
                self.update_easyblade_voltage_current(
                    3,
                    pack03_packinfo1.voltage(),
                    pack03_packinfo1.current(),
                )?;
            },

            varta_easyblade_can_messages::Messages::Pack04PackInfo1(pack04_packinfo1) => {
                self.update_easyblade_voltage_current(
                    4,
                    pack04_packinfo1.voltage(),
                    pack04_packinfo1.current(),
                )?;
            },

            varta_easyblade_can_messages::Messages::Pack05PackInfo1(pack05_packinfo1) => {
                self.update_easyblade_voltage_current(
                    5,
                    pack05_packinfo1.voltage(),
                    pack05_packinfo1.current(),
                )?;
            },

            varta_easyblade_can_messages::Messages::Pack06PackInfo1(pack06_packinfo1) => {
                self.update_easyblade_voltage_current(
                    6,
                    pack06_packinfo1.voltage(),
                    pack06_packinfo1.current(),
                )?;
            },

            varta_easyblade_can_messages::Messages::Pack07PackInfo1(pack07_packinfo1) => {
                self.update_easyblade_voltage_current(
                    7,
                    pack07_packinfo1.voltage(),
                    pack07_packinfo1.current(),
                )?;
            },

            varta_easyblade_can_messages::Messages::Pack08PackInfo1(pack08_packinfo1) => {
                self.update_easyblade_voltage_current(
                    8,
                    pack08_packinfo1.voltage(),
                    pack08_packinfo1.current(),
                )?;
            },

            varta_easyblade_can_messages::Messages::Pack09PackInfo1(pack09_packinfo1) => {
                self.update_easyblade_voltage_current(
                    9,
                    pack09_packinfo1.voltage(),
                    pack09_packinfo1.current(),
                )?;
            },

            varta_easyblade_can_messages::Messages::Pack10PackInfo1(pack10_packinfo1) => {
                self.update_easyblade_voltage_current(
                    10,
                    pack10_packinfo1.voltage(),
                    pack10_packinfo1.current(),
                )?;
            },

            varta_easyblade_can_messages::Messages::Pack11PackInfo1(pack11_packinfo1) => {
                self.update_easyblade_voltage_current(
                    11,
                    pack11_packinfo1.voltage(),
                    pack11_packinfo1.current(),
                )?;
            },

            varta_easyblade_can_messages::Messages::Pack12PackInfo1(pack12_packinfo1) => {
                self.update_easyblade_voltage_current(
                    12,
                    pack12_packinfo1.voltage(),
                    pack12_packinfo1.current(),
                )?;
            },

            varta_easyblade_can_messages::Messages::Pack13PackInfo1(pack13_packinfo1) => {
                self.update_easyblade_voltage_current(
                    13,
                    pack13_packinfo1.voltage(),
                    pack13_packinfo1.current(),
                )?;
            },

            varta_easyblade_can_messages::Messages::Pack14PackInfo1(pack14_packinfo1) => {
                self.update_easyblade_voltage_current(
                    14,
                    pack14_packinfo1.voltage(),
                    pack14_packinfo1.current(),
                )?;
            },

            varta_easyblade_can_messages::Messages::Pack15PackInfo1(pack15_packinfo1) => {
                self.update_easyblade_voltage_current(
                    15,
                    pack15_packinfo1.voltage(),
                    pack15_packinfo1.current(),
                )?;
            },

            _ => {},
        }

        Ok(())
    }

    pub async fn scan(&mut self) -> Result<(), Error> {
        // Drop old list of scanned modules.
        self.easyblades = [const { None }; MAX_MODULES];

        let scanned_canopen_nodes = self.canbus_manager.scan_nodes().await?;
        for n in scanned_canopen_nodes {
            let serial_number = self.read_serial_number(n.node_id).await?;
            let easyblade = VartaEasyblade {
                node_id: n.node_id,
                serial_number,
                software_version: n.software_version,
                hardware_version: n.hardware_version,
                last_seen: n.last_seen,
                voltage: None,
                current: None,
            };
            self.easyblades[n.node_id as usize] = Some(easyblade);
        }

        Ok(())
    }

    pub async fn read_device_error_history(&self, node: &VartaEasyblade) -> Result<(), Error> {
        let mut sdo_client = self.canbus_manager.sdo_client(node.node_id);
        let highest_subindex = sdo_client.read_u8(0x2018, 0x00).await?;
        assert_eq!(highest_subindex, 16);

        for sub_index in 1..=highest_subindex {
            let val = sdo_client.read_u8(0x2018, sub_index).await?;
            let e: varta_easyblade::DeviceError = match varta_easyblade::DeviceError::try_from(val)
            {
                Ok(e) => e,
                Err(_) => varta_easyblade::DeviceError::Unknown,
            };
            println!("2018.{sub_index:02x}: {val:02x} ({e:?})");
        }

        Ok(())
    }

    pub async fn read_cell_voltages(&self, node: &VartaEasyblade) -> Result<Vec<f32>, Error> {
        let mut sdo_client = self.canbus_manager.sdo_client(node.node_id);
        let highest_subindex = sdo_client.read_u8(0x2100, 0x00).await?;
        assert_eq!(highest_subindex, 16);

        let mut cell_voltages: Vec<f32> = vec![];
        for sub_index in 1..=14 {
            let val = sdo_client.read_u32(0x2100, sub_index).await?;
            cell_voltages.push((val as f32) / 1000.0);
        }

        Ok(cell_voltages)
    }
}

// Private API
impl Varta {
    async fn read_serial_number(&self, node_id: u8) -> Result<u16, Error> {
        let mut sdo_client = self.canbus_manager.sdo_client(node_id);
        let bytes = sdo_client.upload(0x2004, 0x01).await?;
        let serial_number = u16::from_le_bytes([bytes[0], bytes[1]]);
        Ok(serial_number)
    }

    fn update_easyblade_voltage_current(
        &mut self,
        node_id: u8,
        voltage: f32,
        current: f32,
    ) -> Result<(), Error> {
        let Some(easyblade) = &mut self.easyblades[node_id as usize] else {
            return Err(Error::UnexpectedModule { node_id });
        };
        easyblade.voltage = Some(voltage);
        easyblade.current = Some(current);
        Ok(())
    }
}
