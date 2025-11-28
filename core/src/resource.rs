use crate::l10n::TranslateError;
use fluent_bundle::FluentArgs;
use fluent_bundle::{bundle::FluentBundle, FluentResource};
use fluent_syntax::ast::{Expression, InlineExpression, Pattern, PatternElement};
use intl_memoizer::concurrent::IntlLangMemoizer;
use std::borrow::{Borrow, Cow};
use std::collections::{HashMap, HashSet};
use unic_langid::LanguageIdentifier;

pub struct L10nResource<R> {
    bundles: HashMap<LanguageIdentifier, FluentBundle<R, IntlLangMemoizer>>,
}

impl<R> L10nResource<R> {
    pub fn new() -> Self {
        Self {
            bundles: HashMap::new(),
        }
    }
}

impl<R> Default for L10nResource<R> {
    fn default() -> Self {
        Self::new()
    }
}

impl<R> L10nResource<R>
where
    R: Borrow<FluentResource>,
{
    pub fn add_bundle(
        &mut self,
        lang: LanguageIdentifier,
        bundle: FluentBundle<R, IntlLangMemoizer>,
    ) {
        self.bundles.insert(lang, bundle);
    }

    pub fn translate<'a>(
        &'a self,
        locale: &LanguageIdentifier,
        key: &str,
        args: Option<&FluentArgs>,
    ) -> Result<Cow<'a, str>, TranslateError> {
        let bundle =
            self.bundles
                .get(locale)
                .ok_or_else(|| TranslateError::LocaleNotSupported {
                    locale: locale.to_owned(),
                })?;

        let (message_id, message_attribute_option) = key
            .split_once('.')
            .map(|(message_id, message_attribute)| (message_id, Some(message_attribute)))
            .unwrap_or((key, None));

        let message = match bundle.get_message(message_id) {
            Some(m) => m,
            None => {
                return Err(TranslateError::MessageIdNotExists {
                    id: message_id.to_owned(),
                    locale: bundle.locale(),
                });
            }
        };

        let pattern = match message_attribute_option {
            Some(attr) => match message.get_attribute(attr) {
                Some(attr) => attr.value(),
                None => {
                    return Err(TranslateError::MessageAttributeNotExists {
                        attribute: attr.to_owned(),
                        id: message_id.to_owned(),
                        locale: bundle.locale(),
                    });
                }
            },
            None => match message.value() {
                Some(p) => p,
                None => {
                    return Err(TranslateError::MessageIdValueNotExists {
                        id: message_id.to_owned(),
                        locale: bundle.locale(),
                    });
                }
            },
        };

        let mut errors = vec![];
        let translation = bundle.format_pattern(pattern, args, &mut errors);
        if !errors.is_empty() {
            return Err(TranslateError::FormatErrors(errors));
        }
        Ok(translation)
    }

    pub fn required_variables(&self, key: &str) -> Result<HashSet<&str>, TranslateError> {
        let mut variables = HashSet::new();

        let (message_id, message_attribute_option) = key
            .split_once('.')
            .map(|(message_id, message_attribute)| (message_id, Some(message_attribute)))
            .unwrap_or((key, None));

        let mut bundles: Vec<_> = self.bundles.values().collect();
        bundles.sort_by_key(|b| b.locale());
        for bundle in bundles {
            let message = match bundle.get_message(message_id) {
                Some(m) => m,
                None => {
                    return Err(TranslateError::MessageIdNotExists {
                        id: message_id.to_owned(),
                        locale: bundle.locale(),
                    });
                }
            };

            let pattern = match message_attribute_option {
                Some(attr) => match message.get_attribute(attr) {
                    Some(attr) => attr.value(),
                    None => {
                        return Err(TranslateError::MessageAttributeNotExists {
                            attribute: attr.to_owned(),
                            id: message_id.to_owned(),
                            locale: bundle.locale(),
                        });
                    }
                },
                None => match message.value() {
                    Some(p) => p,
                    None => {
                        return Err(TranslateError::MessageIdValueNotExists {
                            id: message_id.to_owned(),
                            locale: bundle.locale(),
                        });
                    }
                },
            };

            bundle.parse_pattern_variables(pattern, &mut variables)?;
        }

        Ok(variables)
    }
}

trait ParseVariables {
    fn locale(&self) -> LanguageIdentifier;

    fn get_pattern<'a>(
        &self,
        id: &'a str,
        attribute: Option<&'a str>,
    ) -> Result<&Pattern<&str>, TranslateError>;

    fn parse_pattern_variables<'a>(
        &'a self,
        pattern: &Pattern<&'a str>,
        variables: &mut HashSet<&'a str>,
    ) -> Result<(), TranslateError>;

    fn parse_expression_variables<'a>(
        &'a self,
        expression: &Expression<&'a str>,
        variables: &mut HashSet<&'a str>,
    ) -> Result<(), TranslateError>;

    fn parse_inline_expression_variables<'a>(
        &'a self,
        inline_expression: &InlineExpression<&'a str>,
        variables: &mut HashSet<&'a str>,
    ) -> Result<(), TranslateError>;
}

impl<R, M> ParseVariables for FluentBundle<R, M>
where
    R: Borrow<FluentResource>,
{
    fn locale(&self) -> LanguageIdentifier {
        self.locales.first().cloned().unwrap_or_default()
    }

    fn get_pattern<'a>(
        &self,
        id: &'a str,
        attribute: Option<&'a str>,
    ) -> Result<&Pattern<&str>, TranslateError> {
        let message = match self.get_message(id) {
            Some(m) => m,
            None => {
                return Err(TranslateError::MessageIdNotExists {
                    id: id.to_owned(),
                    locale: self.locale(),
                });
            }
        };

        let pattern = match attribute {
            Some(attr) => match message.get_attribute(attr) {
                Some(attr) => attr.value(),
                None => {
                    return Err(TranslateError::MessageAttributeNotExists {
                        attribute: attr.to_owned(),
                        id: id.to_owned(),
                        locale: self.locale(),
                    });
                }
            },
            None => match message.value() {
                Some(p) => p,
                None => {
                    return Err(TranslateError::MessageIdValueNotExists {
                        id: id.to_owned(),
                        locale: self.locale(),
                    });
                }
            },
        };

        Ok(pattern)
    }

    fn parse_pattern_variables<'a>(
        &'a self,
        pattern: &Pattern<&'a str>,
        variables: &mut HashSet<&'a str>,
    ) -> Result<(), TranslateError> {
        for element in &pattern.elements {
            if let PatternElement::Placeable { expression } = element {
                self.parse_expression_variables(expression, variables)?;
            }
        }
        Ok(())
    }

    fn parse_expression_variables<'a>(
        &'a self,
        expression: &Expression<&'a str>,
        variables: &mut HashSet<&'a str>,
    ) -> Result<(), TranslateError> {
        match expression {
            Expression::Select { selector, variants } => {
                self.parse_inline_expression_variables(selector, variables)?;
                for variant in variants {
                    self.parse_pattern_variables(&variant.value, variables)?;
                }
            }
            Expression::Inline(inline_expression) => {
                self.parse_inline_expression_variables(inline_expression, variables)?;
            }
        }

        Ok(())
    }

    fn parse_inline_expression_variables<'a>(
        &'a self,
        inline_expression: &InlineExpression<&'a str>,
        variables: &mut HashSet<&'a str>,
    ) -> Result<(), TranslateError> {
        match inline_expression {
            InlineExpression::VariableReference { id } => {
                variables.insert(id.name);
            }
            InlineExpression::FunctionReference { arguments, .. } => {
                for positional_argument in &arguments.positional {
                    self.parse_inline_expression_variables(positional_argument, variables)?;
                }
                for named_argument in &arguments.named {
                    self.parse_inline_expression_variables(&named_argument.value, variables)?;
                }
            }
            InlineExpression::MessageReference { id, attribute } => {
                let pattern =
                    self.get_pattern(id.name, attribute.as_ref().map(|attribute| attribute.name))?;
                self.parse_pattern_variables(pattern, variables)?;
            }
            InlineExpression::Placeable { expression } => {
                self.parse_expression_variables(expression, variables)?;
            }
            _ => {}
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;

    #[test]
    fn required_variables() {
        let source_en = indoc! {r#"
            hello = { $hello_var }
        "#};

        let resource = utils::build_resource(vec![("en", source_en.to_string())]);
        let actual = resource.required_variables("hello").unwrap();
        let expected = HashSet::from(["hello_var"]);
        assert_eq!(actual, expected);

        let source_en = indoc! {r#"
            hello = { $hello_var_en }
        "#};
        let source_fr = indoc! {r#"
            hello = { $hello_var_fr }
        "#};

        let resource = utils::build_resource(vec![
            ("en", source_en.to_string()),
            ("fr", source_fr.to_string()),
        ]);
        let actual = resource.required_variables("hello").unwrap();
        let expected = HashSet::from(["hello_var_en", "hello_var_fr"]);
        assert_eq!(actual, expected);

        // Refer another message
        let source_en = indoc! {r#"
            hello = { world }
            world = { $world_var_en }
        "#};
        let source_fr = indoc! {r#"
            hello = { $hello_var_fr }
            world = { $world_var_fr }
        "#};

        let resource = utils::build_resource(vec![
            ("en", source_en.to_string()),
            ("fr", source_fr.to_string()),
        ]);
        // message "hello"
        let actual = resource.required_variables("hello").unwrap();
        let expected = HashSet::from(["world_var_en", "hello_var_fr"]);
        assert_eq!(actual, expected);
        // message "world"
        let actual = resource.required_variables("world").unwrap();
        let expected = HashSet::from(["world_var_en", "world_var_fr"]);
        assert_eq!(actual, expected);

        // Select expression
        let source_en = indoc! {r#"
            result = { CHECK_WON($result, $passing_result) ->
                [passed] You have passed!
               *[failed] You have failed by { MISSING_POINTS($result, $passing_result) }. { retry }
            }
            retry = { $remaining_tries ->
                [0] You have no remaining tries :(
                [1] You have only one remaining try, you can do it!
               *[other] You have { $remaining_tries } remaining tries.
            }
            missing_points = { $points ->
                [1] only 1 point
               *[other] $points points
            }
        "#};

        let resource = utils::build_resource(vec![("en", source_en.to_string())]);
        let actual = resource.required_variables("result").unwrap();
        let expected = HashSet::from(["result", "passing_result", "remaining_tries"]);
        assert_eq!(actual, expected);

        // Referring terms
        let source_en = indoc! {r#"
            -brand = { $first ->
               *[uppercase] Brand
                [lowercase] brand
            } { $this_is_not_a_var_in_term }

            hello = Hello { $username } from the { -brand } team
            hello_lowercase = Hello { $username } from the { -brand(first: "lowercase") } team

            # Not valid syntax, therefore we do nothing when a term is parsed.
            # hello_case = Hello { $username } from the { -brand(first: $first) } team
        "#};
        let resource = utils::build_resource(vec![("en", source_en.to_string())]);
        // message "hello"
        let actual = resource.required_variables("hello").unwrap();
        let expected = HashSet::from(["username"]);
        assert_eq!(actual, expected);
        // message "hello_lowercase"
        let actual = resource.required_variables("hello_lowercase").unwrap();
        let expected = HashSet::from(["username"]);
        assert_eq!(actual, expected);

        // Nested placeable
        let source_en = indoc! {r#"
            hello = {{ $hello_var }}
        "#};

        let resource = utils::build_resource(vec![("en", source_en.to_string())]);
        let actual = resource.required_variables("hello").unwrap();
        let expected = HashSet::from(["hello_var"]);
        assert_eq!(actual, expected);
    }

    mod utils {
        use super::*;

        pub fn build_resource(sources: Vec<(&str, String)>) -> L10nResource<FluentResource> {
            let mut resource = L10nResource::new();

            for (lang, source) in sources {
                let lang_id = lang.parse().unwrap();
                let fluent_resource = FluentResource::try_new(source).unwrap();
                let mut bundle = FluentBundle::new_concurrent(vec![lang_id]);
                bundle.add_resource(fluent_resource).unwrap();
                resource.add_bundle(bundle.locales.first().unwrap().to_owned(), bundle);
            }
            resource
        }
    }
}
