use super::ast::{Enum, Input, Struct, Variant};
use super::{field_to_ident, Field};
use crate::ast::{MessageArgs, MessageKey};
use crate::valid::validate_l10n;
use proc_macro2::{Span, TokenStream};
use syn::spanned::Spanned;
use syn::{Attribute, DeriveInput, Error, Ident, Lifetime, LitStr, Result};

pub enum Digest<'a> {
    Struct(StructDigest<'a>),
    Enum(EnumDigest<'a>),
}

pub struct StructDigest<'a> {
    pub derive_input: &'a DeriveInput,
    pub fields: Vec<Field<'a>>,
    pub self_lifetime: Option<Lifetime>,
    pub from_field: Option<Field<'a>>,
    pub message: Message,
}

pub struct EnumDigest<'a> {
    pub derive_input: &'a DeriveInput,
    pub l10n_self_lifetime: Option<Lifetime>,
    pub variants: Vec<VariantDigest<'a>>,
}

pub struct VariantDigest<'a> {
    pub variant_input: &'a syn::Variant,
    pub fields: Vec<Field<'a>>,
    pub from_field: Option<Field<'a>>,
    pub message: Message,
}

pub enum Message {
    Transparent {
        field: Ident,
    },
    Params {
        resource: LitStr,
        key: MessageKey,
        arguments: MessageArgs,
    },
}

impl<'a> Digest<'a> {
    pub fn from_input(input: Input<'a>) -> Result<Digest<'a>> {
        match input {
            Input::Struct(input) => Ok(Digest::Struct(StructDigest::from_input(input)?)),
            Input::Enum(input) => Ok(Digest::Enum(EnumDigest::from_input(input)?)),
        }
    }
}

impl<'a> StructDigest<'a> {
    fn from_input(input: Struct<'a>) -> Result<StructDigest<'a>> {
        let from = get_from(&input.fields)?;

        if let Some(span) = input.l10n_attribute.transparent {
            return if input.fields.len() == 1 {
                let field = field_to_ident(input.fields.first().unwrap());
                Ok(StructDigest {
                    derive_input: input.derive_input,
                    fields: input.fields,
                    self_lifetime: input.l10n_attribute.self_lifetime,
                    from_field: from,
                    message: Message::Transparent { field },
                })
            } else {
                Err(Error::new(
                    span,
                    "#[l10n_message(transparent)] requires exactly one field",
                ))
            };
        }

        let l10n_attribute = input.l10n_attribute.attribute.ok_or_else(|| {
            Error::new_spanned(
                input.derive_input,
                r#"missing #[l10n_message("...")] attribute"#,
            )
        })?;

        let argument_ts = input.l10n_attribute.arguments.first_to_token_stream();
        let resource = input.l10n_attribute.first_literal.ok_or_else(|| {
            missing_literal_message(l10n_attribute, &argument_ts, "resource", "main")
        })?;
        let key = input
            .l10n_attribute
            .second_literal
            .ok_or_else(|| {
                missing_literal_message(l10n_attribute, &argument_ts, "key", "id.attribute")
            })?
            .into();

        let arguments = input.l10n_attribute.arguments;
        arguments.validate()?;

        validate_l10n(
            &resource,
            &key,
            &arguments,
            input.l10n_attribute.closing_span.unwrap(), // TODO: Remove `unwrap()`
                                                        // attribute_closing_span(l10n_attribute),
        )?;

        Ok(StructDigest {
            derive_input: input.derive_input,
            fields: input.fields,
            self_lifetime: input.l10n_attribute.self_lifetime,
            from_field: from,
            message: Message::Params {
                resource,
                key,
                arguments,
            },
        })
    }
}

impl<'a> EnumDigest<'a> {
    fn from_input(mut input: Enum<'a>) -> Result<EnumDigest<'a>> {
        input.l10n_attribute.arguments.validate_for_enum()?;
        let input_variants = std::mem::take(&mut input.variants);
        let variants = input_variants
            .into_iter()
            .map(|variant_input| VariantDigest::from_input(variant_input, &input))
            .collect::<Result<_>>()?;

        Ok(EnumDigest {
            derive_input: input.derive_input,
            l10n_self_lifetime: input.l10n_attribute.self_lifetime,
            variants,
        })
    }
}

impl<'a> VariantDigest<'a> {
    fn from_input(variant_input: Variant<'a>, enum_input: &Enum<'a>) -> Result<VariantDigest<'a>> {
        let from = get_from(&variant_input.fields)?;

        if let Some(span) = variant_input.l10n_attribute.transparent.or(
            match variant_input.l10n_attribute.attribute {
                None => enum_input.l10n_attribute.transparent,
                _ => None,
            },
        ) {
            return if variant_input.fields.len() == 1 {
                let field = field_to_ident(variant_input.fields.first().unwrap());
                Ok(VariantDigest {
                    variant_input: variant_input.variant_input,
                    fields: variant_input.fields,
                    from_field: from,
                    message: Message::Transparent { field },
                })
            } else {
                Err(Error::new(
                    span,
                    "#[l10n_message(transparent)] requires exactly one field",
                ))
            };
        }

        if let Some(self_lifetime) = variant_input.l10n_attribute.self_lifetime {
            return Err(Error::new_spanned(
                self_lifetime,
                "lifetime is only supported on the enum, not on variants",
            ));
        }

        let enum_resource = &enum_input.l10n_attribute.first_literal;
        let enum_key = &enum_input.l10n_attribute.second_literal;
        let missing_span = variant_or_enum_attribute_span(
            variant_input.l10n_attribute.attribute,
            enum_input.l10n_attribute.attribute,
        );
        let (resource, key) = match (
            variant_input.l10n_attribute.first_literal,
            variant_input.l10n_attribute.second_literal,
        ) {
            (Some(resource), Some(key)) => {
                (resource, MessageKey::from_enum_and_variant(enum_key, key)?)
            }
            (Some(key), _) => (
                enum_resource.clone().ok_or_else(|| {
                    Error::new(
                        missing_span,
                        "missing l10n resource either on the enum or this variant",
                    )
                })?,
                MessageKey::from_enum_and_variant(enum_key, key)?,
            ),
            (_, _) => (
                enum_resource.clone().ok_or_else(|| {
                    Error::new(
                        missing_span,
                        "missing l10n resource either on the enum or this variant",
                    )
                })?,
                enum_key
                    .clone()
                    .ok_or_else(|| {
                        Error::new(
                            missing_span,
                            "missing l10n key either on the enum or this variant",
                        )
                    })?
                    .into(),
            ),
        };

        let mut arguments = variant_input.l10n_attribute.arguments;
        arguments.merge_enum_arguments(&enum_input.l10n_attribute.arguments);
        arguments.validate()?;

        validate_l10n(
            &resource,
            &key,
            &arguments,
            variant_input.l10n_attribute.closing_span.unwrap(), // TODO: Remove `unwrap()`
                                                                // variant_input
                                                                //     .l10n_attribute
                                                                //     .attribute
                                                                //     .map(attribute_closing_span)
                                                                //     .unwrap_or_else(|| variant_input.variant_input.ident.span()),
        )?;

        Ok(VariantDigest {
            variant_input: variant_input.variant_input,
            fields: variant_input.fields,
            from_field: from,
            message: Message::Params {
                resource,
                key,
                arguments,
            },
        })
    }
}

fn get_from<'a>(fields: &[Field<'a>]) -> Result<Option<Field<'a>>> {
    let mut from: Option<Field> = None;

    for field in fields {
        if let Some(attribute) = field.from {
            if from.is_some() {
                return Err(Error::new_spanned(
                    attribute,
                    "duplicate #[l10n_message_from] attribute",
                ));
            }
            from = Some(field.clone());
        }
    }

    Ok(from)
}

fn missing_literal_message(
    attribute: &Attribute,
    argument_ts: &Option<TokenStream>,
    expected: &str,
    example: &str,
) -> Error {
    match argument_ts {
        Some(ts) => Error::new_spanned(
            ts,
            format!(
                r#"expected a {} in place of the argument, example: `"{}", {}`"#,
                expected, example, ts
            ),
        ),
        _ => Error::new(
            attribute_closing_span(attribute),
            format!(r#"expected a {}, example: `"{}"`"#, expected, example),
        ),
    }
}

fn variant_or_enum_attribute_span(
    variant_attribute: Option<&Attribute>,
    enum_attribute: Option<&Attribute>,
) -> Span {
    variant_attribute
        .map(|attr| attr.path().span())
        .or_else(|| enum_attribute.map(|attr| attr.path().span()))
        .unwrap_or_else(Span::call_site)
}

fn attribute_closing_span(attr: &Attribute) -> Span {
    attr.bracket_token.span.close()
    // let token_stream = attr.bracket_token.span.close().to_token_stream();
    // let iter = token_stream.clone().into_iter();
    // let tokens: Vec<_> = iter.collect();

    // println!("tokens.len(): {}", tokens.len());
    // for token in tokens.clone() {
    //     println!("token: {}", token);
    // }

    // if tokens.len() == 1 {
    //     if let TokenTree::Group(group) = &tokens[0] {
    //         return group.span_close();
    //     }
    // }
    // token_stream.span()
}
