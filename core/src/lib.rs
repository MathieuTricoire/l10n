pub use fluent_bundle;
pub use intl_memoizer;
pub use unic_langid;

pub mod config;
pub mod l10n;
pub mod l10n_message;
pub mod locales;
pub mod message;

mod resource;
mod utils;

pub const UNEXPECTED_MESSAGE: &str = "Unexpected message";
