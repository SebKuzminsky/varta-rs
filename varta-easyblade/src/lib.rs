pub mod varta;
pub use varta::Varta;

pub mod varta_easyblade;
pub use varta_easyblade::DeviceError;
pub use varta_easyblade::MasterInfo;
pub use varta_easyblade::VartaEasyblade;

#[allow(clippy::too_many_arguments)]
mod varta_easyblade_can_messages;

/// Maximum number of EasyBlade modules supported.
pub const MAX_MODULES: usize = 16;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    ZenCan(#[from] zencan_client::SdoClientError),

    #[error(transparent)]
    Dbc(#[from] varta_easyblade_can_messages::CanError),

    #[error("IO Error on {can_interface}: {e}")]
    Io {
        can_interface: String,
        #[source]
        e: std::io::Error,
    },

    #[error("CAN packet from unexpected module {node_id}")]
    UnexpectedModule { node_id: u8 },
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
