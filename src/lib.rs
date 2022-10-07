//! Documentation
//!

pub use once_cell;

pub use l10n_core::fluent_bundle;
pub use l10n_core::intl_memoizer;
pub use l10n_core::unic_langid;

pub use l10n_core::l10n::{L10n, L10nBuilder, TranslateError};
pub use l10n_core::l10n_message::L10nMessage;
pub use l10n_core::locales::Locales;
pub use l10n_core::message::Message;
pub use l10n_core::UNEXPECTED_MESSAGE;

#[doc(hidden)]
pub use l10n_impl::*;

#[macro_export]
macro_rules! message_args {
    ($($key:expr => $value:expr),* $(,)?) => {
        {
            let mut args: $crate::fluent_bundle::FluentArgs = $crate::fluent_bundle::FluentArgs::new();
            $(args.set($key, $value);)*
            args
        }
    };
}
