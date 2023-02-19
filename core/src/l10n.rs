use crate::locales::Locales;
use crate::resource::L10nResource;
use crate::utils::{for_locales, grammar_number, locales_to_string, values_to_string};
use fluent_bundle::{bundle::FluentBundle, FluentArgs, FluentResource};
use fluent_bundle::{FluentError, FluentValue};
use fluent_syntax::ast::{Entry, Expression, InlineExpression, Pattern, PatternElement};
use intl_memoizer::concurrent::IntlLangMemoizer;
use self_cell::self_cell;
use std::ffi::OsStr;
use std::fmt;
use std::path::{Path, PathBuf};
use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
    fmt::Debug,
    fs, io,
};
use thiserror::Error;
use unic_langid::LanguageIdentifier;

type FluentResources = Vec<FluentResource>;
type Resources<'s> = HashMap<String, L10nResource<&'s FluentResource>>;
type ResourceIndex = usize;
type ResourceName = String;
type GlobalUnnamedResources = Vec<ResourceIndex>;
type UnnamedResources = HashMap<(String, LanguageIdentifier), Vec<ResourceIndex>>;
type NamedResources = HashMap<ResourceName, HashMap<LanguageIdentifier, ResourceIndex>>;
type Functions = HashMap<String, for<'a> fn(&[FluentValue<'a>], &FluentArgs) -> FluentValue<'a>>;

self_cell!(
    struct InnerL10n {
        owner: FluentResources,
        #[covariant]
        dependent: Resources,
    }
);

pub struct L10n {
    inner: InnerL10n,
    pub locales: Locales,
}

pub struct L10nBuilder {
    locales: Locales,
    fluent_resources: FluentResources,
    global_unnamed_resources: GlobalUnnamedResources,
    unnamed_resources: UnnamedResources,
    named_resources: NamedResources,
    transform: Option<fn(&str) -> Cow<str>>,
    formatter: Option<fn(&FluentValue, &IntlLangMemoizer) -> Option<String>>,
    use_isolating: bool,
    functions: Functions,
}

#[derive(Error, PartialEq, Eq, Debug)]
#[error("build l10n errors:\n  - {}", values_to_string(.0, "\n  - "))]
pub struct BuildErrors(Vec<BuildError>);

#[derive(Error, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum BuildError {
    #[error(r#"missing resource "{}" {}"#, .resource, for_locales(.locales))]
    MissingResource {
        resource: String,
        locales: Vec<LanguageIdentifier>,
    },
    #[error(r#"missing message "{message}" in resource "{resource}" for locales: {}"#, locales_to_string(.locales, ", "))]
    MissingMessage {
        resource: String,
        message: String,
        locales: Vec<LanguageIdentifier>,
    },
    #[error(r#"extra message "{message}" in resource "{resource}" for locales: {}"#, locales_to_string(.locales, ", "))]
    ExtraMessage {
        resource: String,
        message: String,
        locales: Vec<LanguageIdentifier>,
    },
    #[error(r#"missing attribute "{attribute}" for message "{message}" in resource "{resource}" for locales: {}"#, locales_to_string(.locales, ", "))]
    MissingAttribute {
        resource: String,
        message: String,
        attribute: String,
        locales: Vec<LanguageIdentifier>,
    },
    #[error(r#"extra attribute "{attribute}" for message "{message}" in resource "{resource}" for locales: {}"#, locales_to_string(.locales, ", "))]
    ExtraAttribute {
        resource: String,
        message: String,
        attribute: String,
        locales: Vec<LanguageIdentifier>,
    },
}

#[derive(Error, Debug)]
pub enum ParserError {
    #[error("impossible to read path `{}` (io error: {err})", path.display())]
    ReadPath { path: PathBuf, err: io::Error },
    #[error("impossible to parse directory `{dir_name}` as a language identifier, (error: {err})")]
    ParseLangDir {
        dir_name: String,
        err: unic_langid::LanguageIdentifierError,
    },
    #[error(
        "missing mandatory locale {}: {}",
        grammar_number(.0, "directory", "directories"),
        locales_to_string(.0, ", ")
    )]
    MissingLocales(Vec<LanguageIdentifier>),
    #[error(r#"named resource "{}" cannot be global, please prefix file name with `_`"#, path.display())]
    GlobalNamedResource { path: PathBuf },
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error("parsing errors: {errors:#?}")]
    FluentParser {
        errors: Vec<fluent_syntax::parser::ParserError>,
    },
}

#[derive(Error, PartialEq, Debug)]
pub enum TranslateError {
    #[error(r#"resource "{0}" not exists"#)]
    ResourceNotExists(String),
    #[error(r#"locale "{locale}" not supported"#)]
    LocaleNotSupported { locale: LanguageIdentifier },
    #[error(r#"message id: "{id}", not exists for locale "{locale}""#)]
    MessageIdNotExists {
        id: String,
        locale: LanguageIdentifier,
    },
    #[error(
        r#"attribute: "{attribute}", not exists on message id: "{id}", for locale "{locale}""#
    )]
    MessageAttributeNotExists {
        attribute: String,
        id: String,
        locale: LanguageIdentifier,
    },
    #[error(r#"message value: "{id}", not defined for locale "{locale}""#)]
    MessageIdValueNotExists {
        id: String,
        locale: LanguageIdentifier,
    },
    #[error("format errors:\n  - {}", values_to_string(.0, "\n  - "))]
    FormatErrors(Vec<FluentError>),
}

impl Debug for L10n {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("L10n").finish()
    }
}

impl L10n {
    fn new(builder: L10nBuilder) -> Result<Self, BuildErrors> {
        Self::check_consistency(&builder)?;

        let L10nBuilder {
            locales,
            fluent_resources,
            global_unnamed_resources,
            unnamed_resources,
            named_resources,
            transform,
            formatter,
            use_isolating,
            functions,
        } = builder;

        let inner_translator = InnerL10n::new(fluent_resources, |fluent_resources| {
            named_resources.keys().map(|name| {
                let mut l10n_resource = L10nResource::new();
                for locale in locales.main_locales() {
                    let locales_resolution = locales
                        .locale_resolution_route(&locale)
                        .expect("Unexpected error, `locale_resolution_route` should not be None in this context!");
                    let mut inverted_locales_resolution = locales_resolution.clone();
                    inverted_locales_resolution.reverse();
                    let mut fl_bundle = FluentBundle::new_concurrent(
                        locales_resolution.into_iter().cloned().collect(),
                    );

                    for fl_res in Self::global_unnamed_fluent_resources(
                        &global_unnamed_resources,
                        fluent_resources,
                    ) {
                        fl_bundle.add_resource_overriding(fl_res);
                    }

                    let mut relative_paths = vec![];
                    let mut relative_path = Some(
                        name.parse::<PathBuf>()
                            .unwrap()
                            .parent()
                            .unwrap()
                            .to_path_buf(),
                    );
                    while let Some(path) = relative_path {
                        relative_paths.push(path.clone());
                        relative_path = path.parent().map(|p| p.to_path_buf());
                    }
                    relative_paths.reverse();

                    for relative_path in relative_paths {
                        for locale in &inverted_locales_resolution {
                            for fl_res in Self::unnamed_fluent_resources(
                                &relative_path,
                                locale,
                                &unnamed_resources,
                                fluent_resources,
                            ) {
                                fl_bundle.add_resource_overriding(fl_res);
                            }
                        }
                    }

                    for locale in &inverted_locales_resolution {
                        if let Some(fl_res) = Self::named_fluent_resource(
                            name,
                            locale,
                            &named_resources,
                            fluent_resources,
                        ) {
                            fl_bundle.add_resource_overriding(fl_res);
                        }
                    }

                    fl_bundle.set_transform(transform);
                    fl_bundle.set_formatter(formatter);
                    fl_bundle.set_use_isolating(use_isolating);

                    for (name, function) in functions.clone() {
                        // Future improvement: only add functions to bundle when is needed
                        fl_bundle
                            .add_function(&name, function)
                            .expect("Unexpected error, there should not be functions with same names");
                    }

                    l10n_resource.add_bundle(locale.to_owned(), fl_bundle);
                }

                (name.to_string(), l10n_resource)
            })
            .collect()
        });

        Ok(Self {
            inner: inner_translator,
            locales,
        })
    }

    fn check_consistency(builder: &L10nBuilder) -> Result<(), BuildErrors> {
        Self::check_named_resources_consistency(
            &builder.locales,
            &builder.named_resources,
            &builder.fluent_resources,
        )?;
        Ok(())
    }

    fn check_named_resources_consistency(
        locales: &Locales,
        named_resources: &NamedResources,
        fluent_resources: &FluentResources,
    ) -> Result<(), BuildErrors> {
        let mut errors = vec![];
        for named_resource in named_resources.keys() {
            let missing_locales: Vec<_> = locales
                .mandatory_locales()
                .iter()
                .filter_map(|locale| {
                    match Self::named_fluent_resource(
                        named_resource,
                        locale,
                        named_resources,
                        fluent_resources,
                    ) {
                        Some(_) => None,
                        None => Some(locale.clone()),
                    }
                })
                .collect();

            if !missing_locales.is_empty() {
                errors.push(BuildError::MissingResource {
                    resource: named_resource.to_owned(),
                    locales: missing_locales,
                });
            }
        }
        match errors.is_empty() {
            true => Ok(()),
            false => Err(BuildErrors(errors)),
        }
    }

    pub fn try_translate_with_args<'a>(
        &'a self,
        lang: &LanguageIdentifier,
        resource: &str,
        key: &str,
        args: Option<&FluentArgs<'_>>,
    ) -> Result<Cow<'a, str>, TranslateError> {
        self.inner
            .borrow_dependent()
            .get(resource)
            .ok_or_else(|| TranslateError::ResourceNotExists(resource.to_string()))?
            .translate(lang, key, args)
    }

    pub fn required_variables(
        &self,
        resource: &str,
        key: &str,
    ) -> Result<HashSet<&str>, TranslateError> {
        self.inner
            .borrow_dependent()
            .get(resource)
            .ok_or_else(|| TranslateError::ResourceNotExists(resource.to_string()))?
            .required_variables(key)
    }

    pub fn required_functions(&self) -> HashSet<&str> {
        let mut functions = HashSet::new();
        let resources = self.inner.borrow_owner();

        for resource in resources {
            for entry in resource.entries() {
                match entry {
                    Entry::Message(message) => {
                        if let Some(pattern) = &message.value {
                            self.parse_pattern_functions(pattern, &mut functions);
                        }
                        for attribute in &message.attributes {
                            self.parse_pattern_functions(&attribute.value, &mut functions);
                        }
                    }
                    Entry::Term(term) => {
                        self.parse_pattern_functions(&term.value, &mut functions);
                        for attribute in &term.attributes {
                            self.parse_pattern_functions(&attribute.value, &mut functions);
                        }
                    }
                    _ => {}
                }
            }
        }

        functions
    }

    fn global_unnamed_fluent_resources<'r>(
        global_unnamed_resources: &[ResourceIndex],
        fluent_resources: &'r [FluentResource],
    ) -> Vec<&'r FluentResource> {
        global_unnamed_resources
            .iter()
            .map(|resource_index| fluent_resources.get(*resource_index).expect("TODO 8"))
            .collect()
    }

    fn unnamed_fluent_resources<'r, 'a>(
        relative_path: &Path,
        locale: &'a LanguageIdentifier,
        unnamed_resources: &'a UnnamedResources,
        fluent_resources: &'r [FluentResource],
    ) -> Vec<&'r FluentResource> {
        let path = normalized_path(relative_path);
        let key = (path, locale.to_owned());
        if let Some(resources_index) = unnamed_resources.get(&key) {
            resources_index
                .iter()
                .map(|resource_index| fluent_resources.get(*resource_index).unwrap())
                .collect()
        } else {
            vec![]
        }
    }

    fn named_fluent_resource<'r, 'a>(
        name: &'a str,
        locale: &'a LanguageIdentifier,
        named_resources: &'a NamedResources,
        fluent_resources: &'r [FluentResource],
    ) -> Option<&'r FluentResource> {
        named_resources
            .get(name)
            .and_then(|localized_resources| localized_resources.get(locale))
            .map(|resource_index| fluent_resources.get(*resource_index).expect("TODO 10"))
    }

    fn parse_pattern_functions<'a>(
        &'a self,
        pattern: &Pattern<&'a str>,
        functions: &mut HashSet<&'a str>,
    ) {
        for element in &pattern.elements {
            if let PatternElement::Placeable { expression } = element {
                self.parse_expression_functions(expression, functions);
            }
        }
    }

    fn parse_expression_functions<'a>(
        &'a self,
        expression: &Expression<&'a str>,
        functions: &mut HashSet<&'a str>,
    ) {
        match expression {
            Expression::Select { selector, variants } => {
                self.parse_inline_expression_functions(selector, functions);
                for variant in variants {
                    self.parse_pattern_functions(&variant.value, functions);
                }
            }
            Expression::Inline(inline_expression) => {
                self.parse_inline_expression_functions(inline_expression, functions);
            }
        }
    }

    fn parse_inline_expression_functions<'a>(
        &'a self,
        inline_expression: &InlineExpression<&'a str>,
        functions: &mut HashSet<&'a str>,
    ) {
        if let InlineExpression::FunctionReference { id, .. } = inline_expression {
            functions.insert(id.name);
        }
    }
}

impl Default for L10nBuilder {
    fn default() -> Self {
        Self {
            locales: Default::default(),
            fluent_resources: Default::default(),
            global_unnamed_resources: Default::default(),
            unnamed_resources: Default::default(),
            named_resources: Default::default(),
            transform: Default::default(),
            formatter: Default::default(),
            use_isolating: true,
            functions: Default::default(),
        }
    }
}

impl Debug for L10nBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("L10nBuilder")
            .field("locales", &self.locales)
            .finish()
    }
}

impl L10nBuilder {
    pub fn new(locales: Locales) -> Self {
        Self {
            locales,
            ..Default::default()
        }
    }

    pub fn add_global_unnamed_resource(&mut self, resource: FluentResource) {
        self.global_unnamed_resources
            .push(self.fluent_resources.len());
        self.fluent_resources.push(resource);
    }

    pub fn add_unnamed_resource(
        &mut self,
        relative_path: &Path,
        locale: &LanguageIdentifier,
        fluent_resource: FluentResource,
    ) {
        let path = normalized_path(relative_path);
        let key = (path, locale.to_owned());
        let resources = match self.unnamed_resources.get_mut(&key) {
            Some(v) => v,
            None => {
                self.unnamed_resources
                    .insert(key.clone(), Vec::with_capacity(1));
                self.unnamed_resources.get_mut(&key).unwrap()
            }
        };
        resources.push(self.fluent_resources.len());
        self.fluent_resources.push(fluent_resource);
    }

    pub fn add_named_resource(
        &mut self,
        name: &str,
        relative_path: &Path,
        locale: &LanguageIdentifier,
        fluent_resource: FluentResource,
    ) {
        let resource_name = normalized_path(&relative_path.join(name));
        let resources = match self.named_resources.get_mut(&resource_name) {
            Some(v) => v,
            None => {
                self.named_resources
                    .insert(resource_name.clone(), HashMap::new());
                self.named_resources.get_mut(&resource_name).unwrap()
            }
        };
        if resources.contains_key(locale) {
            // Maybe a first improvement could be to override the resource
            // since it rely on fs I think it's ok for now.
            unreachable!(
                r#"named resource: "{}" already exists for locale: "{}""#,
                resource_name, locale
            );
        }
        resources.insert(locale.to_owned(), self.fluent_resources.len());
        self.fluent_resources.push(fluent_resource);
    }

    pub fn build(self) -> Result<L10n, BuildErrors> {
        L10n::new(self)
    }

    pub fn parse(
        path: impl AsRef<Path>,
        locales_option: Option<Locales>,
    ) -> Result<Self, ParserError> {
        let mut builder = Self::default();
        let path = path.as_ref();
        let locales_to_visit = locales_option.as_ref().map(|locales| locales.all_locales());
        let mut locales_visited = HashSet::new();

        let dir = fs::read_dir(path).map_err(|err| match err.kind() {
            io::ErrorKind::NotFound => ParserError::ReadPath {
                path: path.to_path_buf(),
                err,
            },
            _ => err.into(),
        })?;

        for entry in dir {
            let entry_path = entry?.path();
            let entry_name = get_entry_name(&entry_path);

            if entry_path.is_file() {
                let name = match entry_name {
                    Some(v) => v.to_string_lossy(),
                    None => continue,
                };
                if !name.starts_with('_') {
                    return Err(ParserError::GlobalNamedResource { path: entry_path });
                }

                let fluent_resource = Self::read_fluent_resource(&entry_path)?;
                builder.add_global_unnamed_resource(fluent_resource);
            } else if entry_path.is_dir() {
                let dir_name = match entry_name.and_then(|v| v.to_str()) {
                    Some(v) => v,
                    None => continue,
                };

                let parsed_locale = dir_name.parse::<LanguageIdentifier>();
                let locale = match &locales_to_visit {
                    Some(locales_to_visit) => match parsed_locale {
                        Ok(locale) if locales_to_visit.contains(&locale) => locale,
                        _ => continue,
                    },
                    None => parsed_locale.map_err(|err| ParserError::ParseLangDir {
                        dir_name: dir_name.to_string(),
                        err,
                    })?,
                };
                locales_visited.insert(locale.clone());

                builder.parse_locale_directory(&locale, &entry_path, &PathBuf::default())?;
            }
        }

        if let Some(mandatory_locales) = locales_option
            .as_ref()
            .map(|locales| locales.mandatory_locales())
        {
            let differences: Vec<_> = mandatory_locales
                .difference(&locales_visited)
                .cloned()
                .collect();
            if !differences.is_empty() {
                return Err(ParserError::MissingLocales(differences));
            }
        }

        builder.locales = locales_option.unwrap_or_else(|| Locales::from(locales_visited));

        Ok(builder)
    }

    fn parse_locale_directory(
        &mut self,
        locale: &LanguageIdentifier,
        locale_path: &Path,
        relative_path: &Path,
    ) -> Result<(), ParserError> {
        let path = locale_path.join(relative_path);

        for entry in fs::read_dir(&path).map_err(|err| match err.kind() {
            io::ErrorKind::NotFound => ParserError::ReadPath { path, err },
            _ => err.into(),
        })? {
            let entry_path = entry?.path();
            let name = match get_entry_name(&entry_path) {
                Some(v) => v,
                None => continue,
            };

            if entry_path.is_file() {
                let resource = Self::read_fluent_resource(&entry_path)?;
                let name = name.to_string_lossy();
                if name.starts_with('_') {
                    self.add_unnamed_resource(relative_path, locale, resource);
                } else {
                    self.add_named_resource(&name, relative_path, locale, resource);
                }
            } else if entry_path.is_dir() {
                self.parse_locale_directory(locale, locale_path, &relative_path.join(name))?;
            }
        }

        Ok(())
    }

    pub fn set_transform(mut self, transform: Option<fn(&str) -> Cow<str>>) -> Self {
        self.transform = transform;
        self
    }

    pub fn set_formatter(
        mut self,
        formatter: Option<fn(&FluentValue, &IntlLangMemoizer) -> Option<String>>,
    ) -> Self {
        self.formatter = formatter;
        self
    }

    pub fn set_use_isolating(mut self, use_isolating: bool) -> Self {
        self.use_isolating = use_isolating;
        self
    }

    pub fn add_function(
        mut self,
        name: &str,
        function: for<'a> fn(&[FluentValue<'a>], &FluentArgs) -> FluentValue<'a>,
    ) -> Self {
        self.functions.insert(name.to_owned(), function);
        self
    }

    fn read_fluent_resource(path: &Path) -> Result<FluentResource, ParserError> {
        let source = fs::read_to_string(path)?;
        FluentResource::try_new(source).map_err(|(_, errors)| ParserError::FluentParser { errors })
    }
}

fn normalized_path(path: &Path) -> String {
    path.iter()
        .map(|c| c.to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

fn get_entry_name(entry_path: &Path) -> Option<&OsStr> {
    if entry_path.is_dir() {
        entry_path.file_name()
    } else {
        match entry_path.extension() {
            Some(extension) if extension == "ftl" => entry_path.file_stem(),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;
    use unic_langid::langid;

    #[test]
    fn parse_ok() {
        let global_term = indoc! {r#"
            -global-brand = Global Brand
        "#};
        let lang_term = indoc! {r#"
            -brand = Root Brand
            -lang-term = Lang term
        "#};
        let nested_term = indoc! {r#"
            -brand = Nested Brand
        "#};
        let temp_dir = macro_files::create_temp!({
            "_term.ftl": &global_term,
            "en": {
                "nested": {
                    "_term.ftl": &nested_term,
                    "about.ftl": indoc! {r#"
                        about-us = (Nested) About { -brand } a { -global-brand } subdivision [{ -lang-term }].
                    "#}
                },
                "_term.ftl": &lang_term,
                "about.ftl": indoc! {r#"
                    about-us = About { -brand } [{ -lang-term }].
                "#}
            },
            "fr": {
                "nested": {
                    "_term.ftl": &nested_term,
                    "about.ftl": indoc! {r#"
                        about-us = (Nested) À propos de { -brand } une sous division de { -global-brand } [{ -lang-term }].
                    "#}
                },
                "_term.ftl": &lang_term,
                "about.ftl": indoc! {r#"
                    about-us = À propos de { -brand } [{ -lang-term }].
                "#}
            },
        })
        .unwrap();

        let locales = Locales::try_from([
            ("en", None),
            ("en-GB", Some("en")),
            ("en-CA", Some("en-GB")),
            ("fr", None),
            ("fr-CA", Some("fr")),
        ])
        .unwrap();

        let translator_builder = L10nBuilder::parse(temp_dir.path(), Some(locales)).unwrap();
        let translator = translator_builder.build().unwrap();

        assert_eq!(
            translator
                .try_translate_with_args(&langid!("en-CA"), "about", "about-us", None)
                .unwrap(),
            "About Root Brand [Lang term]."
        );
        assert_eq!(
            translator
                .try_translate_with_args(&langid!("en-CA"), "nested/about", "about-us", None)
                .unwrap(),
            "(Nested) About Nested Brand a Global Brand subdivision [Lang term]."
        );
        assert_eq!(
            translator
                .try_translate_with_args(&langid!("fr"), "about", "about-us", None)
                .unwrap(),
            "À propos de Root Brand [Lang term]."
        );
        assert_eq!(
            translator
                .try_translate_with_args(&langid!("fr"), "nested/about", "about-us", None)
                .unwrap(),
            "(Nested) À propos de Nested Brand une sous division de Global Brand [Lang term]."
        );
    }

    #[test]
    fn parse_missing_resource() {
        let temp_dir = macro_files::create_temp!({
            "en": {
                "resource-1.ftl": indoc! {r#"
                    first-key = First key [en]
                "#},
                "resource-2.ftl": indoc! {r#"
                    first-key = First key [en]
                "#}
            },
            "fr": {
                "resource-1.ftl": indoc! {r#"
                    first-key = First key [fr]
                "#}
            },
        })
        .unwrap();

        let locales =
            Locales::try_from([("en", None), ("fr", None), ("fr-CA", Some("fr"))]).unwrap();

        let translator_builder = L10nBuilder::parse(temp_dir.path(), Some(locales)).unwrap();
        let actual_err = translator_builder.build().unwrap_err();
        let expected_err = BuildErrors(vec![BuildError::MissingResource {
            resource: "resource-2".to_string(),
            locales: vec![langid!("fr")],
        }]);
        assert_eq!(actual_err, expected_err);
    }

    #[test]
    fn global_named_resource() {
        let temp_dir = macro_files::create_temp!({
            "global-resource.ftl": true
        })
        .unwrap();
        let locales = Locales::try_from([("en", None)]).unwrap();
        let actual_err = L10nBuilder::parse(temp_dir.path(), Some(locales)).unwrap_err();
        match actual_err {
            ParserError::GlobalNamedResource { .. } => (),
            _ => panic!("should return ParserError::GlobalNamedResource"),
        };
    }

    #[test]
    fn parse_fluent_resources_only() {
        let temp_dir = macro_files::create_temp!({
            ".DS_Store": true,
            "README.md": true,
            "_brand.ftl": true,
            "_ignored-resource.ftl": true,
            "en": {
                ".DS_Store": true,
                "README.md": true,
                "_brand.ftl": true,
                "_errors.ftl": true,
                "settings": {
                    "_ignored_file": true,
                    "ignored_file": true,
                    "_terms.ftl": true,
                    "account.ftl": true,
                    "preferences.ftl": true,
                    "advanced": {
                        ".DS_Store": true,
                        "README.md": true,
                        "admin.ftl": true,
                    }
                },
            },
            "fr": {
                ".DS_Store": true,
                "README.md": true,
                "_other_brand.ftl": true,
                "_errors.ftl": true,
                "settings": {
                    "_settings_terms.ftl": true,
                    "account.ftl": true,
                    "preferences.ftl": true,
                    "advanced": {
                        "_ignored_file": true,
                        "ignored_file": true,
                        ".DS_Store": true,
                        "README.md": true,
                        "admin.ftl": true,
                    }
                },
            },
        })
        .unwrap();

        let locales = Locales::try_from([("en", None)]).unwrap();

        let translator_builder = L10nBuilder::parse(temp_dir.path(), Some(locales)).unwrap();
        let actual_resources: HashSet<_> = translator_builder
            .named_resources
            .keys()
            .map(|resource| resource.to_string())
            .collect();

        let expected_resources = HashSet::from([
            "settings/account".to_string(),
            "settings/preferences".to_string(),
            "settings/advanced/admin".to_string(),
        ]);

        assert_eq!(actual_resources, expected_resources);
        let _ = translator_builder.build().unwrap();
    }

    #[test]
    fn required_functions() {
        let temp_dir = macro_files::create_temp!({
            "_term.ftl": "-brand-creation-date = 2000",
            "en": {
                "_term.ftl": indoc! {r#"
                    -brand = { LANG_TERM_EN_FUNCTION("Brand") }
                "#},
                "about.ftl": indoc! {r#"
                    about-us = About { -brand } since { SINCE_YEAR(-brand-creation, format: "special") }
                    contact-us = Contact us at { PHONE_NUMBER($country) } { PHONE_INFOS_EN() }.
                "#}
            },
            "en-CA": {
                "_term.ftl": indoc! {r#"
                    -canadian-brand = { LANG_TERM_EN_CA_FUNCTION("Canadian brand") }
                    -subdivision = { $first ->
                       *[uppercase] A
                        [lowercase] a
                    } subdivision of { -brand }
                "#},
                "about.ftl": indoc! {r#"
                    about-us = About { -canadian-brand } { -subdivision(first: "lowercase") } since { SINCE_YEAR(-brand-creation, format: "special") }
                "#}
            },
            "fr": {
                "_term.ftl": indoc! {r#"
                    -brand = { LANG_TERM_FR_FUNCTION("Brand") }
                "#},
                "about.ftl": indoc! {r#"
                    about-us = About { -brand } since { SINCE_YEAR(-brand-creation, format: "special") }
                    contact-us = Contact us at { PHONE_NUMBER($country) } { PHONE_INFOS_FR() }.
                "#}
            },
        })
        .unwrap();

        let locales = Locales::try_from([
            ("en", None),
            ("en-GB", Some("en")),
            ("en-CA", Some("en-GB")),
            ("fr", None),
            ("fr-CA", Some("fr")),
        ])
        .unwrap();

        let translator_builder = L10nBuilder::parse(temp_dir.path(), Some(locales)).unwrap();
        let translator = translator_builder.build().unwrap();

        let expected = HashSet::from([
            "LANG_TERM_EN_FUNCTION",
            "SINCE_YEAR",
            "PHONE_NUMBER",
            "PHONE_INFOS_EN",
            "LANG_TERM_EN_CA_FUNCTION",
            "LANG_TERM_FR_FUNCTION",
            "PHONE_INFOS_FR",
        ]);
        assert_eq!(translator.required_functions(), expected);
    }
}
