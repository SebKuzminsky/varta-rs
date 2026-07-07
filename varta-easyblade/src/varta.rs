use crate::Error;
use crate::VartaEasyblade;

pub struct Varta {
    canbus_manager: zencan_client::BusManager<zencan_client::common::SocketCanSender>,
}

// Public API
impl Varta {
    pub fn new(canbus_interface: &str) -> Self {
        let (tx, rx) = zencan_client::open_socketcan(canbus_interface)
            .unwrap_or_else(|e| panic!("Failed to open CAN interface {}: {}", canbus_interface, e));
        Self {
            canbus_manager: zencan_client::BusManager::new(tx, rx),
        }
    }

    pub async fn scan(&mut self) -> Result<Vec<VartaEasyblade>, Error> {
        let nodes = self.canbus_manager.scan_nodes().await?;
        let mut varta_easyblades: Vec<VartaEasyblade> = vec![];
        for n in nodes {
            let serial_number = self.read_serial_number(n.node_id).await?;
            varta_easyblades.push(VartaEasyblade {
                node_id: n.node_id,
                serial_number,
                software_version: n.software_version,
                hardware_version: n.hardware_version,
                last_seen: n.last_seen,
            });
        }
        Ok(varta_easyblades)
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
}
