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

// TODO To remove once https://github.com/projectfluent/fluent-rs/pull/271 is merged and released
pub fn merge_args<'a>(
    local_args: &'a fluent_bundle::FluentArgs,
    overriding_args: &'a fluent_bundle::FluentArgs,
) -> fluent_bundle::FluentArgs<'a> {
    let mut merged_args = std::collections::HashMap::new();
    for (key, value) in local_args.iter() {
        merged_args.insert(std::borrow::Cow::from(key), value.to_owned());
    }
    for (key, value) in overriding_args.iter() {
        merged_args.insert(std::borrow::Cow::from(key), value.to_owned());
    }
    fluent_bundle::FluentArgs::from_iter(merged_args.into_iter())
}
