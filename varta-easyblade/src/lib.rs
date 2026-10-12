pub mod varta;
pub use varta::SdoSession;
pub use varta::Varta;

pub mod sdo;
pub use sdo::*;

pub mod varta_easyblade;

pub use varta_easyblade::Pdo;
pub use varta_easyblade::VartaEasyblade;

#[rustfmt::skip]
#[allow(clippy::too_many_arguments)]
mod varta_easyblade_can_messages {
    include!(concat!(env!("OUT_DIR"), "/varta_easyblade_can_messages.rs"));
}

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
}
