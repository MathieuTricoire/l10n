use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use syn::{Error, LitStr, Result};

pub struct MessageKey {
    key: LitStr,
    id_span: Span,
}

impl MessageKey {
    pub fn from_enum_and_variant(enum_key: &Option<LitStr>, variant_key: LitStr) -> Result<Self> {
        match variant_key.value().find('.') {
            Some(0) => {
                match enum_key {
                    Some(enum_key) => {
                        let mut message_key = enum_key.value();
                        if let Some(dot_position) = message_key.find('.') {
                            message_key.truncate(dot_position);
                        }
                        message_key.push_str(&variant_key.value());
                        let variant_span = variant_key.span();
                        Ok(Self {
                            key: LitStr::new(&message_key, variant_span),
                            id_span: enum_key.span(),
                        })
                    },
                    None => Err(Error::new_spanned(variant_key, "attribute notation like `.attribute` is only available if a key is set on the enum"))
                }
            }
            _ => Ok(Self::from(variant_key)),
        }
    }

    pub fn value(&self) -> String {
        self.key.value()
    }

    pub fn span(&self) -> Span {
        self.key.span()
    }

    pub fn id_span(&self) -> Span {
        self.id_span
    }
}

impl ToTokens for MessageKey {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.key.to_tokens(tokens)
    }
}

impl From<LitStr> for MessageKey {
    fn from(key: LitStr) -> Self {
        let span = key.span();
        Self { key, id_span: span }
    }
}
