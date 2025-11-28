use crate::utils::locales_to_string;
use serde::{de, Deserialize, Deserializer};
use std::{collections::HashSet, fmt, marker::PhantomData};
use thiserror::Error;
use unic_langid::{LanguageIdentifier, LanguageIdentifierError};

#[derive(Default, PartialEq, Eq, Debug)]
pub struct Locales {
    locales: Vec<LocaleEntry>,
}

#[derive(PartialEq, Eq, Hash, Debug)]
pub struct LocaleEntry {
    main: LanguageIdentifier,
    fallback: Option<LanguageIdentifier>,
}

#[derive(Error, Debug)]
pub enum InvariantError {
    #[error("infinite fallback loop detected: ({})", locales_to_string(.0, " -> "))]
    InfiniteFallbackLoop(Vec<LanguageIdentifier>),
    #[error("main locale duplicate: {0}")]
    MainLocaleDuplicate(LanguageIdentifier),
    #[error("empty")]
    Empty,
}

#[derive(Error, Debug)]
pub enum TryFromLocalesError {
    #[error(transparent)]
    ParseLocale(#[from] LanguageIdentifierError),
    #[error(transparent)]
    Invariant(#[from] InvariantError),
}

impl Locales {
    pub fn try_new(locales: Vec<LocaleEntry>) -> Result<Self, InvariantError> {
        let this = Self { locales };
        this.check_invariants()?;
        Ok(this)
    }

    fn check_invariants(&self) -> Result<(), InvariantError> {
        let mut main_locales = HashSet::new();
        for tr_locale in &self.locales {
            // Check main locale duplicate
            if main_locales.contains(&tr_locale.main) {
                return Err(InvariantError::MainLocaleDuplicate(tr_locale.main.clone()));
            }
            main_locales.insert(tr_locale.main.clone());

            // Check infinite fallback loop
            let mut visited_locales = vec![];
            let mut current_tr_locale = Some(tr_locale);

            while let Some(tr_locale) = current_tr_locale {
                visited_locales.push(tr_locale.main.clone());

                current_tr_locale = match &tr_locale.fallback {
                    Some(fallback) if visited_locales.contains(fallback) => {
                        visited_locales.push(fallback.clone());
                        return Err(InvariantError::InfiniteFallbackLoop(visited_locales));
                    }
                    Some(fallback) => self.find_with_main_locale(fallback),
                    None => None,
                };
            }
        }

        // Check empty
        if main_locales.is_empty() {
            return Err(InvariantError::Empty);
        }

        Ok(())
    }

    pub fn is_empty(&self) -> bool {
        self.locales.is_empty()
    }

    fn find_with_main_locale<'a>(&'a self, locale: &LanguageIdentifier) -> Option<&'a LocaleEntry> {
        self.locales
            .iter()
            .find(|LocaleEntry { main: i_locale, .. }| locale == i_locale)
    }

    fn mandatory_locale_for<'a>(&'a self, locale_entry: &'a LocaleEntry) -> &'a LanguageIdentifier {
        locale_entry
            .fallback
            .as_ref()
            .map(|fallback| {
                self.find_with_main_locale(fallback)
                    .map(|locale_entry| self.mandatory_locale_for(locale_entry))
                    .unwrap_or(fallback)
            })
            .unwrap_or(&locale_entry.main)
    }

    pub fn mandatory_locales(&self) -> HashSet<LanguageIdentifier> {
        self.locales
            .iter()
            .fold(HashSet::new(), |mut mandatory_locales, tr_locale| {
                let mandatory_locale = self.mandatory_locale_for(tr_locale);
                if !mandatory_locales.contains(mandatory_locale) {
                    mandatory_locales.insert(mandatory_locale.clone());
                }
                mandatory_locales
            })
    }

    pub fn all_locales(&self) -> HashSet<LanguageIdentifier> {
        self.locales
            .iter()
            .flat_map(
                |LocaleEntry {
                     main: locale,
                     fallback,
                 }| match fallback {
                    Some(fallback_locale) => {
                        HashSet::from([fallback_locale.clone(), locale.clone()])
                    }
                    None => HashSet::from([locale.clone()]),
                },
            )
            .fold(HashSet::new(), |mut locales, locale| {
                if !locales.contains(&locale) {
                    locales.insert(locale);
                }
                locales
            })
    }

    pub fn main_locales(&self) -> HashSet<LanguageIdentifier> {
        self.locales
            .iter()
            .map(|LocaleEntry { main: locale, .. }| locale.clone())
            .collect()
    }

    // Only for main locales
    pub fn locale_resolution_route<'a>(
        &'a self,
        locale: &LanguageIdentifier,
    ) -> Option<Vec<&'a LanguageIdentifier>> {
        let tr_locale = self.find_with_main_locale(locale)?;
        let mut resolution = vec![&tr_locale.main];
        let mut current_fallback = tr_locale.fallback.as_ref();

        while let Some(fallback) = &current_fallback {
            resolution.push(fallback);
            current_fallback = self
                .find_with_main_locale(fallback)
                .and_then(|tr_locale| tr_locale.fallback.as_ref());
        }

        Some(resolution)
    }
}

impl<'de> Deserialize<'de> for Locales {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(transparent)]
        struct This {
            locales: Vec<LocaleEntry>,
        }
        let this = This::deserialize(deserializer)?;
        Locales::try_new(this.locales).map_err(serde::de::Error::custom)
    }
}

impl<'a> IntoIterator for &'a Locales {
    type Item = &'a LocaleEntry;
    type IntoIter = std::slice::Iter<'a, LocaleEntry>;

    fn into_iter(self) -> Self::IntoIter {
        self.locales.iter()
    }
}

impl<T, const N: usize> TryFrom<[(T, Option<T>); N]> for Locales
where
    T: AsRef<str>,
{
    type Error = TryFromLocalesError;

    fn try_from(values: [(T, Option<T>); N]) -> Result<Self, Self::Error> {
        let locales = values
            .into_iter()
            .map(|(main_str, fallback_str)| {
                Ok(LocaleEntry {
                    main: main_str.as_ref().parse()?,
                    fallback: fallback_str.map(|str| str.as_ref().parse()).transpose()?,
                })
            })
            .collect::<Result<Vec<_>, LanguageIdentifierError>>()?;

        Ok(Self::try_new(locales)?)
    }
}

impl From<HashSet<LanguageIdentifier>> for Locales {
    fn from(locales: HashSet<LanguageIdentifier>) -> Self {
        let (primary_locales, secondary_locales): (Vec<_>, Vec<_>) = locales
            .into_iter()
            .partition(|locale| locale.region.is_none());

        let mut locales: Vec<_> = secondary_locales
            .into_iter()
            .map(|secondary_locale| {
                let mut stripped_locale = secondary_locale.clone();
                stripped_locale.region = None;
                match primary_locales.contains(&stripped_locale) {
                    true => LocaleEntry::new(secondary_locale, Some(stripped_locale)),
                    false => LocaleEntry::new(secondary_locale, None),
                }
            })
            .collect();

        locales.extend(
            primary_locales
                .into_iter()
                .map(|locale| LocaleEntry::new(locale, None))
                .collect::<Vec<_>>(),
        );

        Self { locales }
    }
}

impl LocaleEntry {
    fn new(main: LanguageIdentifier, fallback: Option<LanguageIdentifier>) -> Self {
        Self { main, fallback }
    }

    pub fn locale(&self) -> &LanguageIdentifier {
        &self.main
    }

    pub fn fallback(&self) -> &Option<LanguageIdentifier> {
        &self.fallback
    }
}

impl<'de> Deserialize<'de> for LocaleEntry {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct LocaleEntryVisitor(PhantomData<LocaleEntry>);

        fn parse_language_identifier<E>(s: &str) -> Result<LanguageIdentifier, E>
        where
            E: de::Error,
        {
            s.parse().map_err(|err| {
                let exp = format!(
                    r#"a valid Unicode Language Identifier like "en-US" ({})"#,
                    err
                );
                de::Error::invalid_value(de::Unexpected::Str(s), &exp.as_ref())
            })
        }

        // To set a different error message
        struct LangId(LanguageIdentifier);

        impl<'de> Deserialize<'de> for LangId {
            fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                struct LangIdVisitor(PhantomData<LangId>);

                impl<'de> de::Visitor<'de> for LangIdVisitor {
                    type Value = LangId;

                    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                        formatter.write_str(r#"a valid Unicode Language Identifier like "en-US""#)
                    }

                    fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
                    where
                        E: de::Error,
                    {
                        Ok(LangId(parse_language_identifier(s)?))
                    }
                }

                deserializer.deserialize_any(LangIdVisitor(PhantomData::<LangId>))
            }
        }

        impl From<LangId> for LanguageIdentifier {
            fn from(langid: LangId) -> Self {
                langid.0
            }
        }

        impl<'de> de::Visitor<'de> for LocaleEntryVisitor {
            type Value = LocaleEntry;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(
                    r#"a locale like "en-US" or a detailed entry like { main = "fr-CA", fallback = "fr" }"#,
                )
            }

            fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(LocaleEntry::new(parse_language_identifier(s)?, None))
            }

            fn visit_map<V>(self, map: V) -> Result<Self::Value, V::Error>
            where
                V: de::MapAccess<'de>,
            {
                #[derive(Deserialize)]
                struct Values {
                    main: LangId,
                    fallback: Option<LangId>,
                }
                let Values { main, fallback } =
                    Values::deserialize(de::value::MapAccessDeserializer::new(map))?;
                Ok(LocaleEntry::new(main.into(), fallback.map(|f| f.into())))
            }
        }

        deserializer.deserialize_any(LocaleEntryVisitor(PhantomData::<LocaleEntry>))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use unic_langid::langid;

    // To test deserialization
    #[derive(Deserialize, Debug)]
    struct Container {
        locales: Locales,
    }

    #[test]
    fn deserialize_locale_entry_ok() {
        #[derive(Deserialize, Debug)]
        struct Locale {
            entry: LocaleEntry,
        }

        let source = toml::toml!(entry = "en-US");
        let locale: Locale = source.try_into().unwrap();
        assert_eq!(locale.entry, LocaleEntry::new(langid!("en-US"), None));

        let source = toml::toml!(entry = { main = "en-CA" });
        let locale: Locale = source.try_into().unwrap();
        assert_eq!(locale.entry, LocaleEntry::new(langid!("en-CA"), None));

        let source = toml::toml!(entry = { main = "fr-CA", fallback = "fr" });
        let locale: Locale = source.try_into().unwrap();
        assert_eq!(
            locale.entry,
            LocaleEntry::new(langid!("fr-CA"), Some(langid!("fr")))
        );
    }

    #[test]
    fn deserialize_locale_entry_errors() {
        #[allow(unused)]
        #[derive(Deserialize, Debug)]
        struct Locale {
            entry: LocaleEntry,
        }

        let source = toml::toml!(entry = false);
        let err = source.try_into::<Locale>().unwrap_err();
        assert_eq!(
            err.to_string(),
            r#"invalid type: boolean `false`, expected a locale like "en-US" or a detailed entry like { main = "fr-CA", fallback = "fr" } for key `entry`"#
        );

        let source = toml::toml!(entry = "not-a-locale");
        let err = source.try_into::<Locale>().unwrap_err();
        assert_eq!(
            err.to_string(),
            r#"invalid value: string "not-a-locale", expected a valid Unicode Language Identifier like "en-US" (Parser error: Invalid subtag) for key `entry`"#
        );

        let source = toml::toml!(entry = { main = false });
        let err = source.try_into::<Locale>().unwrap_err();
        assert_eq!(
            err.to_string(),
            r#"invalid type: boolean `false`, expected a valid Unicode Language Identifier like "en-US" for key `entry.main`"#
        );

        let source = toml::toml!(entry = { main = "not-a-locale" });
        let err = source.try_into::<Locale>().unwrap_err();
        assert_eq!(
            err.to_string(),
            r#"invalid value: string "not-a-locale", expected a valid Unicode Language Identifier like "en-US" (Parser error: Invalid subtag) for key `entry.main`"#
        );

        let source = toml::toml!(entry = { main = "en", fallback = false });
        let err = source.try_into::<Locale>().unwrap_err();
        assert_eq!(
            err.to_string(),
            r#"invalid type: boolean `false`, expected a valid Unicode Language Identifier like "en-US" for key `entry.fallback`"#
        );

        let source = toml::toml!(entry = { main = "en", fallback = "not-a-locale" });
        let err = source.try_into::<Locale>().unwrap_err();
        assert_eq!(
            err.to_string(),
            r#"invalid value: string "not-a-locale", expected a valid Unicode Language Identifier like "en-US" (Parser error: Invalid subtag) for key `entry.fallback`"#
        );
    }

    #[test]
    fn try_from_array_of_strings() {
        let actual = Locales::try_from([
            ("en", None),
            ("en-GB", Some("en")),
            ("en-CA", Some("en-GB")),
            ("fr", None),
            ("fr-CA", Some("fr")),
            ("de", None),
        ])
        .unwrap();
        let expected = Locales {
            locales: vec![
                LocaleEntry::new(langid!("en"), None),
                LocaleEntry::new(langid!("en-GB"), Some(langid!("en"))),
                LocaleEntry::new(langid!("en-CA"), Some(langid!("en-GB"))),
                LocaleEntry::new(langid!("fr"), None),
                LocaleEntry::new(langid!("fr-CA"), Some(langid!("fr"))),
                LocaleEntry::new(langid!("de"), None),
            ],
        };
        assert_eq!(actual, expected);
    }

    #[test]
    fn from_hashset_of_language_identifier() {
        let actual = Locales::from(HashSet::from([
            langid!("en"),
            langid!("en-CA"),
            langid!("en-GB"),
            langid!("en-GB-variant"),
            langid!("en-Latn"),
            langid!("en-Latn-variant"),
            langid!("en-Latn-GB"),
            langid!("en-Latn-GB-variant"),
            langid!("fr"),
            langid!("fr-CA"),
            langid!("fr-Latn-CA"),
            langid!("fr-Latn-CA-variant"),
            langid!("de"),
        ]));
        #[rustfmt::skip]
        let expected_locales = HashSet::from([
            LocaleEntry::new(langid!("en"), None),
            LocaleEntry::new(langid!("en-CA"), Some(langid!("en"))),
            LocaleEntry::new(langid!("en-GB"), Some(langid!("en"))),
            LocaleEntry::new(langid!("en-GB-variant"), None),
            LocaleEntry::new(langid!("en-Latn"), None),
            LocaleEntry::new(langid!("en-Latn-variant"), None),
            LocaleEntry::new(langid!("en-Latn-GB"), Some(langid!("en-Latn"))),
            LocaleEntry::new(langid!("en-Latn-GB-variant"), Some(langid!("en-Latn-variant"))),
            LocaleEntry::new(langid!("fr"), None),
            LocaleEntry::new(langid!("fr-CA"), Some(langid!("fr"))),
            LocaleEntry::new(langid!("fr-Latn-CA"), None),
            LocaleEntry::new(langid!("fr-Latn-CA-variant"), None),
            LocaleEntry::new(langid!("de"), None),
        ]);
        assert_eq!(
            actual.locales.into_iter().collect::<HashSet<_>>(),
            expected_locales
        );
    }

    #[test]
    fn locales_deserialize() {
        let source = toml::toml! {
            locales = [
                "en",
                { main = "en-GB", fallback = "en" },
                { main = "en-CA", fallback = "en-GB" },
                { main = "fr" },
                { main = "fr-CA", fallback = "fr" },
            ]
        };
        let actual = source.try_into::<Container>().unwrap().locales;
        let expected = Locales::try_from([
            ("en", None),
            ("en-GB", Some("en")),
            ("en-CA", Some("en-GB")),
            ("fr", None),
            ("fr-CA", Some("fr")),
        ])
        .unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn locales_deserialize_format_errors() {
        let source = toml::toml! {
            locales = [
                { something = "else" },
            ]
        };
        let error = source.try_into::<Container>().unwrap_err().to_string();
        assert_eq!(error, "missing field `main` for key `locales`");

        let source = toml::toml! {
            locales = [
                { main = "en", fallback = "not-a-locale" },
            ]
        };
        let error = source.try_into::<Container>().unwrap_err().to_string();
        assert_eq!(
            error,
            r#"invalid value: string "not-a-locale", expected a valid Unicode Language Identifier like "en-US" (Parser error: Invalid subtag) for key `locales.fallback`"#
        );
    }

    #[test]
    fn locales_deserialize_empty_error() {
        let source = toml::toml! {
            locales = []
        };
        let error = source.try_into::<Container>().unwrap_err().to_string();
        assert_eq!(error, "empty for key `locales`");
    }

    #[test]
    fn locales_deserialize_main_duplicate_error() {
        let source = toml::toml! {
            locales = [
                "en-CA",
                { main = "en-CA", fallback = "en" },
            ]
        };
        let error = source.try_into::<Container>().unwrap_err().to_string();
        assert_eq!(error, "main locale duplicate: en-CA for key `locales`");
    }

    #[test]
    fn locales_deserialize_infinite_loop_error() {
        let source = toml::toml! {
            locales = [
                { main = "en", fallback = "en" },
            ]
        };
        let error = source.try_into::<Container>().unwrap_err().to_string();
        assert_eq!(
            error,
            "infinite fallback loop detected: (en -> en) for key `locales`"
        );

        let source = toml::toml! {
            locales = [
                "en",
                { main = "en-GB", fallback = "en-CA" },
                { main = "en-IE", fallback = "en-GB" },
                { main = "en-CA", fallback = "en-IE" },
            ]
        };
        let error = source.try_into::<Container>().unwrap_err().to_string();
        assert_eq!(
            error,
            "infinite fallback loop detected: (en-GB -> en-CA -> en-IE -> en-GB) for key `locales`"
        );
    }

    #[test]
    fn all_locales() {
        let translator_locales = Locales::try_from([
            ("en-GB", Some("en")),
            ("en-CA", Some("en-GB")),
            ("fr-CA", Some("fr")),
            ("fr", None),
            ("de", None),
        ])
        .unwrap();
        let expected = HashSet::from([
            langid!("en"),
            langid!("en-GB"),
            langid!("en-CA"),
            langid!("fr"),
            langid!("fr-CA"),
            langid!("de"),
        ]);
        assert_eq!(translator_locales.all_locales(), expected);
    }

    #[test]
    fn mandatory_locales() {
        let translator_locales = Locales::try_from([
            ("en", None),
            ("en-GB", Some("en")),
            ("en-CA", Some("en-GB")),
            ("fr-CA", Some("fr")),
        ])
        .unwrap();
        let expected = HashSet::from([langid!("en"), langid!("fr")]);
        assert_eq!(translator_locales.mandatory_locales(), expected);
    }

    #[test]
    fn main_locales() {
        let translator_locales = Locales::try_from([
            ("en", None),
            ("en-GB", Some("en")),
            ("en-CA", Some("en-GB")),
            ("fr-CA", Some("fr")),
        ])
        .unwrap();
        let expected = HashSet::from([
            langid!("en"),
            langid!("en-GB"),
            langid!("en-CA"),
            langid!("fr-CA"),
        ]);
        assert_eq!(translator_locales.main_locales(), expected);
    }

    #[test]
    fn locale_resolution_route() {
        let en = langid!("en");
        let en_gb = langid!("en-GB");
        let en_ca = langid!("en-CA");
        let en_ie = langid!("en-IE");
        let fr = langid!("fr");
        let fr_ca = langid!("fr-CA");

        let translator_locales = Locales::try_from([
            ("en", None),
            ("en-GB", Some("en")),
            ("en-CA", Some("en-GB")),
            ("en-IE", Some("en-GB")),
            ("fr-CA", Some("fr")),
        ])
        .unwrap();

        let tests = [
            (&en, Some(vec![&en])),
            (&en_gb, Some(vec![&en_gb, &en])),
            (&en_ca, Some(vec![&en_ca, &en_gb, &en])),
            (&en_ie, Some(vec![&en_ie, &en_gb, &en])),
            (&fr_ca, Some(vec![&fr_ca, &fr])),
            (&fr, None),            // Not a main locale
            (&langid!("de"), None), // Locale not set
        ];

        for (locale, expected) in tests {
            assert_eq!(translator_locales.locale_resolution_route(locale), expected);
        }
    }
}
