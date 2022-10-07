use crate::l10n::TranslateError;
use crate::UNEXPECTED_MESSAGE;
use fluent_bundle::FluentArgs;
use std::borrow::Cow;
use unic_langid::LanguageIdentifier;

pub trait L10nMessage<'s, 'r> {
    fn try_translate_with_args(
        &'s self,
        locale: &LanguageIdentifier,
        args: Option<&'s FluentArgs<'s>>,
    ) -> Result<Cow<'r, str>, TranslateError>;

    fn translate_with_args(
        &'s self,
        locale: &LanguageIdentifier,
        args: Option<&'s FluentArgs<'s>>,
    ) -> Cow<'r, str> {
        self.try_translate_with_args(locale, args)
            .unwrap_or_else(|_| Cow::from(UNEXPECTED_MESSAGE))
    }

    fn try_translate(
        &'s self,
        locale: &LanguageIdentifier,
    ) -> Result<Cow<'r, str>, TranslateError> {
        self.try_translate_with_args(locale, None)
    }

    fn translate(&'s self, locale: &LanguageIdentifier) -> Cow<'r, str> {
        self.try_translate_with_args(locale, None)
            .unwrap_or_else(|_| Cow::from(UNEXPECTED_MESSAGE))
    }
}
