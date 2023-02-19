use crate::l10n::{L10n, TranslateError};
use crate::l10n_message::L10nMessage;
use crate::merge_args;
use fluent_bundle::FluentArgs;
use std::{borrow::Cow, fmt::Debug};
use unic_langid::LanguageIdentifier;

#[derive(Debug)]
pub struct Message<'l10n, 'args> {
    l10n: &'l10n L10n,
    resource: &'args str,
    key: &'args str,
    args: Option<FluentArgs<'args>>,
}

impl<'l10n, 'args> Message<'l10n, 'args> {
    pub fn new(
        l10n: &'l10n L10n,
        resource: &'args str,
        key: &'args str,
        args: Option<FluentArgs<'args>>,
    ) -> Message<'l10n, 'args> {
        Self {
            l10n,
            resource,
            key,
            args,
        }
    }
}

impl<'l10n, 'args> L10nMessage<'args, 'l10n> for Message<'l10n, 'args> {
    fn try_translate_with_args(
        &'args self,
        locale: &LanguageIdentifier,
        args: Option<&'args FluentArgs<'args>>,
    ) -> Result<Cow<'l10n, str>, TranslateError> {
        match (self.args.as_ref(), args) {
            (Some(local_args), Some(overriding_args)) => {
                let args = merge_args(local_args, overriding_args);
                self.l10n
                    .try_translate_with_args(locale, self.resource, self.key, Some(&args))
            }
            _ => self.l10n.try_translate_with_args(
                locale,
                self.resource,
                self.key,
                self.args.as_ref().or(args),
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::l10n::L10nBuilder;
    use crate::locales::Locales;
    use fluent_bundle::FluentResource;
    use std::path::PathBuf;
    use unic_langid::langid;

    #[test]
    fn test() {
        let locales = Locales::try_from([("en", None), ("fr", None)]).unwrap();
        let mut builder = L10nBuilder::new(locales);

        let en_home =
            FluentResource::try_new("welcome = Welcome { $first-name }!".to_string()).unwrap();
        builder.add_named_resource("home", &PathBuf::default(), &langid!("en"), en_home);

        let fr_home = FluentResource::try_new(
            "welcome = Bienvenue { $first-name } { $last-name }.".to_string(),
        )
        .unwrap();
        builder.add_named_resource("home", &PathBuf::default(), &langid!("fr"), fr_home);

        let l10n = builder.build().unwrap();

        let mut args = FluentArgs::new();
        args.set("first-name", "Alan Mathison");
        args.set("last-name", "Turing");

        let message = Message::new(&l10n, "home", "welcome", Some(args));

        assert_eq!(
            message.translate(&langid!("fr")),
            "Bienvenue \u{2068}Alan Mathison\u{2069} \u{2068}Turing\u{2069}."
        );
        assert_eq!(
            message.translate(&langid!("en")),
            "Welcome \u{2068}Alan Mathison\u{2069}!"
        );

        let mut args_override = FluentArgs::new();
        args_override.set("first-name", "Alan");
        assert_eq!(
            message.translate_with_args(&langid!("fr"), Some(&args_override)),
            "Bienvenue \u{2068}Alan\u{2069} \u{2068}Turing\u{2069}."
        );
        assert_eq!(
            message.translate_with_args(&langid!("en"), Some(&args_override)),
            "Welcome \u{2068}Alan\u{2069}!"
        );
    }
}
