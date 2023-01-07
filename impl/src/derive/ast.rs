use crate::ast::MessageArgs;
use proc_macro2::Span;
use syn::parse::ParseStream;
use syn::spanned::Spanned;
use syn::{
    Attribute, Data, DataEnum, DataStruct, DeriveInput, Error, Fields, Ident, Index, Lifetime,
    LitStr, Member, Result, Token, Type,
};

pub enum Input<'a> {
    Struct(Struct<'a>),
    Enum(Enum<'a>),
}

pub struct Struct<'a> {
    pub derive_input: &'a DeriveInput,
    pub fields: Vec<Field<'a>>,
    pub l10n_attribute: L10nAttribute<'a>,
}

pub struct Enum<'a> {
    pub derive_input: &'a DeriveInput,
    pub variants: Vec<Variant<'a>>,
    pub l10n_attribute: L10nAttribute<'a>,
}

pub struct Variant<'a> {
    pub variant_input: &'a syn::Variant,
    pub fields: Vec<Field<'a>>,
    pub l10n_attribute: L10nAttribute<'a>,
}

#[derive(Clone)]
pub struct Field<'a> {
    pub field_input: &'a syn::Field,
    pub member: Member,
    pub ty: &'a Type,
    pub from: Option<&'a Attribute>,
}

#[derive(Default)]
pub struct L10nAttribute<'a> {
    pub attribute: Option<&'a Attribute>,
    pub transparent: Option<Span>,
    pub self_lifetime: Option<Lifetime>,
    pub first_literal: Option<LitStr>,
    pub second_literal: Option<LitStr>,
    pub arguments: MessageArgs,
}

impl<'a> Input<'a> {
    pub fn from_syn(derive_input: &'a DeriveInput) -> Result<Self> {
        match &derive_input.data {
            Data::Struct(data) => Struct::from_syn(derive_input, data).map(Input::Struct),
            Data::Enum(data) => Enum::from_syn(derive_input, data).map(Input::Enum),
            Data::Union(_) => Err(Error::new_spanned(derive_input, "union is not supported")),
        }
    }
}

impl<'a> Struct<'a> {
    fn from_syn(derive_input: &'a DeriveInput, data: &'a DataStruct) -> Result<Self> {
        let l10n_attribute = parse_l10n_attribute(&derive_input.attrs)?.ok_or_else(|| {
            Error::new_spanned(derive_input, r#"missing #[l10n_message("...")] attribute"#)
        })?;
        let fields = Field::multiple_from_syn(&data.fields)?;
        Ok(Struct {
            derive_input,
            fields,
            l10n_attribute,
        })
    }
}

impl<'a> Enum<'a> {
    fn from_syn(derive_input: &'a DeriveInput, data: &'a DataEnum) -> Result<Self> {
        let l10n_attribute = parse_l10n_attribute(&derive_input.attrs)?.unwrap_or_default();

        let variants = data
            .variants
            .iter()
            .map(Variant::from_syn)
            .collect::<Result<_>>()?;

        Ok(Enum {
            derive_input,
            variants,
            l10n_attribute,
        })
    }
}

impl<'a> Variant<'a> {
    fn from_syn(variant_input: &'a syn::Variant) -> Result<Self> {
        let l10n_attribute = parse_l10n_attribute(&variant_input.attrs)?.unwrap_or_default();
        let fields = Field::multiple_from_syn(&variant_input.fields)?;
        Ok(Variant {
            variant_input,
            fields,
            l10n_attribute,
        })
    }
}

fn parse_l10n_attribute(attrs: &[Attribute]) -> Result<Option<L10nAttribute<'_>>> {
    let mut l10n_attribute: Option<L10nAttribute> = None;
    for attr in attrs {
        if attr.path.is_ident("l10n_message") {
            if l10n_attribute.is_some() {
                return Err(Error::new_spanned(
                    attr,
                    "only one #[l10n_message(...)] attribute is allowed",
                ));
            }
            l10n_attribute = Some(_parse_l10n_attribute(attr)?);
        }
    }
    Ok(l10n_attribute)
}

fn _parse_l10n_attribute(attr: &Attribute) -> Result<L10nAttribute<'_>> {
    syn::custom_keyword!(transparent);

    attr.parse_args_with(|input: ParseStream| {
        let mut l10n_attribute = L10nAttribute {
            attribute: Some(attr),
            transparent: None,
            self_lifetime: None,
            first_literal: None,
            second_literal: None,
            arguments: Default::default(),
        };

        l10n_attribute.transparent = input
            .parse::<Option<transparent>>()
            .map(|r| r.map(|kw| kw.span()))?;
        if l10n_attribute.transparent.is_some() {
            return Ok(l10n_attribute);
        }

        l10n_attribute.self_lifetime = input.parse()?;
        if input.is_empty() {
            return Ok(l10n_attribute);
        } else if l10n_attribute.self_lifetime.is_some() {
            input.parse::<Token![,]>()?;
        }

        if !peek_potential_argument(input) {
            l10n_attribute.first_literal = input.parse()?;
            if input.is_empty() {
                return Ok(l10n_attribute);
            } else if l10n_attribute.first_literal.is_some() {
                input.parse::<Token![,]>()?;
            }
        }

        if !peek_potential_argument(input) {
            l10n_attribute.second_literal = input.parse()?;
            if input.is_empty() {
                return Ok(l10n_attribute);
            } else if l10n_attribute.second_literal.is_some() {
                input.parse::<Token![,]>()?;
            }
        }

        if !input.is_empty() {
            l10n_attribute.arguments = input.parse()?;
        }

        Ok(l10n_attribute)
    })
}

fn peek_potential_argument(input: ParseStream) -> bool {
    (input.peek(LitStr) && input.peek2(Token![=]))
        || input.peek(Ident)
        || (input.peek(Token![*]) && input.peek2(Ident))
}

impl<'a> Field<'a> {
    fn multiple_from_syn(fields: &'a Fields) -> Result<Vec<Self>> {
        fields
            .iter()
            .enumerate()
            .map(|(i, field)| Field::from_syn(i, field))
            .collect()
    }

    fn from_syn(i: usize, field_input: &'a syn::Field) -> Result<Self> {
        Ok(Field {
            field_input,
            member: field_input
                .ident
                .clone()
                .map(Member::Named)
                .unwrap_or_else(|| {
                    Member::Unnamed(Index {
                        index: i as u32,
                        span: field_input.ty.span(),
                    })
                }),
            ty: &field_input.ty,
            from: field_input
                .attrs
                .iter()
                .find(|attr| attr.path.is_ident("l10n_from")),
        })
    }
}
