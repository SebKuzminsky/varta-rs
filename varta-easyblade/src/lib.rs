use zencan_client::common::traits::AsyncCanSender;

pub struct Varta<S>
where
    S: AsyncCanSender + Send + Sync,
{
    canbus_manager: zencan_client::BusManager<S>,
}

impl<S> Varta<S>
where
    S: AsyncCanSender + Send + Sync,
{
    pub fn new(
        tx: S,
        rx: impl zencan_client::common::traits::AsyncCanReceiver + Sync + 'static,
    ) -> Self {
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
