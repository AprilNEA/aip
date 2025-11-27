//! AIP-193: Error handling

#[cfg(feature = "derive")]
pub use aip_193_derive::IntoStatus;

#[doc(hidden)]
pub mod __private {
    pub use super::{Code, IntoStatus};
    pub use std::collections::HashMap;
}

mod code;
mod error_details;
mod status;
mod traits;

pub use code::Code;
pub use error_details::{ErrorInfo, ErrorInfoBuilder};
pub use status::{Status, StatusDetails};
pub use traits::IntoStatus;
