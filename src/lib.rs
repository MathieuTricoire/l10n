//! [`l10n`](https://crates.io/crates/l10n) is a high level and opinionated localization crate built upon the excellent [`fluent-bundle`](https://crates.io/crates/fluent-bundle) crate, the [Fluent project](https://projectfluent.org) and inspired by the [`thiserror`](https://crates.io/crates/thiserror) crate.
//!
//! For more informations please visit the code repository: <https://github.com/MathieuTricoire/l10n>
//!
//! # Simple example
//!
//! ```rust
//! use l10n::unic_langid::langid;
//! use l10n::{message, message_args, L10nMessage};
//!
//! # // hack to not display init with functions...
//! # use l10n::fluent_bundle::{FluentValue, FluentArgs};
//! # l10n::init!({
//! #     functions: {
//! #         "TIME": |_: &[FluentValue<'_>], _: &FluentArgs| FluentValue::None
//! #     }
//! # });
//! # #[cfg(all(windows, unix, macos, wasm))]
//! l10n::init!({});
//!
//! fn main() {
//!     let lang = langid!("fr");
//!
//!     let username = "Alice";
//!     let greeting = message!("app", "greeting", "first-name" = username);
//!     assert_eq!(greeting.translate(&lang), "Bonjour \u{2068}Alice\u{2069} !");
//!
//!     let status = Status::Busy {
//!         reason: "Meeting".to_string(),
//!     };
//!     assert_eq!(
//!         status.translate(&lang),
//!         "\u{2068}Non disponible\u{2069} (\u{2068}Meeting\u{2069})"
//!     );
//!     assert_eq!(
//!         status.translate_with_args(&lang, Some(&message_args!("gender" => "female"))),
//!         "\u{2068}Occupée\u{2069} (\u{2068}Meeting\u{2069})"
//!     );
//! }
//!
//! #[derive(L10nMessage)]
//! #[l10n_message("settings", "status")]
//! enum Status {
//!     #[l10n_message(".online")]
//!     Online,
//!     #[l10n_message(".offline")]
//!     Offline,
//!     #[l10n_message(".busy", "reason" = reason.as_str(), "gender" = "other")]
//!     Busy { reason: String },
//! }
//! ```
//!
//! # Advanced example
//!
//! _This example is not really relevant, it is just to show how to use l10n._
//!
//! ```rust
//! use l10n::fluent_bundle::{FluentArgs, FluentValue};
//! use l10n::unic_langid::langid;
//! use l10n::L10nMessage;
//! use std::borrow::Cow;
//!
//! fn l10n_transform(s: &str) -> Cow<str> {
//!     Cow::from(s.replace("Occupée", "OcCuPéE🚫"))
//! }
//!
//! fn time<'a>(positional: &[FluentValue<'a>], _named: &FluentArgs) -> FluentValue<'a> {
//!     match positional.get(0) {
//!         Some(FluentValue::String(s)) => {
//!             FluentValue::String(Cow::from(format!("{}🕒", s)))
//!         },
//!         Some(v) => v.to_owned(),
//!         _ => FluentValue::Error,
//!     }
//! }
//!
//! l10n::init!({
//!     use_isolating: false, // Not recommended
//!     transform: Some(l10n_transform),
//!     functions: {
//!         "TIME": time
//!     }
//! });
//!
//! fn main() {
//!     let lang = langid!("fr");
//!     let status = Status::BusyFor {
//!         reason: "Meeting",
//!         gender: Gender::Female,
//!         time: Time::minutes(30),
//!     };
//!     assert_eq!(status.translate(&lang), "OcCuPéE🚫 (Meeting) [30m🕒]");
//! }
//!
//! #[derive(L10nMessage)]
//! #[l10n_message('a, "settings", "status")]
//! enum Status<'a, T>
//! where
//!     &'a T: 'a + Into<FluentValue<'a>>,
//! {
//!     #[l10n_message(".online")]
//!     Online,
//!     #[l10n_message(".offline")]
//!     Offline,
//!     #[l10n_message(".busy", "reason" = *.0, "gender" = .1)]
//!     Busy(&'a str, Gender),
//!     #[l10n_message(".busy-for", *reason, gender, time)]
//!     BusyFor { reason: &'a str, gender: Gender, time: T },
//! }
//!
//! enum Gender {
//!     Female,
//!     Male,
//!     Other,
//! }
//!
//! impl<'a> Into<FluentValue<'a>> for &'a Gender {
//!     fn into(self) -> FluentValue<'a> {
//!         FluentValue::String(Cow::from(match self {
//!             Gender::Female => "female",
//!             Gender::Male => "male",
//!             Gender::Other => "other",
//!         }))
//!     }
//! }
//!
//! pub struct Time(usize);
//!
//! impl Time {
//!     pub fn minutes(minutes: usize) -> Time {
//!         Time(minutes)
//!     }
//! }
//!
//! impl<'a> Into<FluentValue<'a>> for &'a Time {
//!     fn into(self) -> FluentValue<'a> {
//!         FluentValue::String(Cow::from(format!("{}m", self.0)))
//!     }
//! }
//! ```

pub use once_cell;

pub use l10n_core::fluent_bundle;
pub use l10n_core::intl_memoizer;
pub use l10n_core::unic_langid;

pub use l10n_core::l10n::{L10n, L10nBuilder, TranslateError};
pub use l10n_core::l10n_message::L10nMessage;
pub use l10n_core::locales::Locales;
pub use l10n_core::merge_args;
pub use l10n_core::message::Message;
pub use l10n_core::UNEXPECTED_MESSAGE;

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

#[cfg(doctest)]
mod test_readme {
    macro_rules! external_doc_test {
        ($x:expr) => {
            #[doc = $x]
            extern "C" {}
        };
    }

    external_doc_test!(include_str!("../README.md"));
}
