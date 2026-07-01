pub struct Varta {
    canbus_manager: zencan_client::BusManager<zencan_client::common::SocketCanSender>,
}

impl Varta {
    pub fn new(canbus_interface: &str) -> Self {
        let (tx, rx) = zencan_client::open_socketcan(canbus_interface)
            .unwrap_or_else(|e| panic!("Failed to open CAN interface {}: {}", canbus_interface, e));
        Self {
            canbus_manager: zencan_client::BusManager::new(tx, rx),
        }
    }

    pub async fn scan(&mut self) {
        match self.canbus_manager.scan_nodes().await {
            Ok(nodes) => {
                for n in &nodes {
                    println!("{n}");
                }
            }
            Err(e) => {
                println!("Error during scan: ");
                println!("{e}");
            }
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn it_works() {
//         let result = add(2, 2);
//         assert_eq!(result, 4);
//     }
// }
