pub mod varta;
pub use varta::Varta;

pub mod varta_easyblade;
pub use varta_easyblade::VartaEasyblade;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    ZenCan(#[from] zencan_client::SdoClientError),
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
