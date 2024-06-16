use crate::message::{expand as expand_message, MessageInput};
use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{Ident, Result, Token};

pub fn expand(input: TranslateInput) -> Result<TokenStream> {
    let locale = input.locale;
    let message_expanded = expand_message(input.message_input)?;

    Ok(quote! {
        {
            use ::l10n::L10nMessage;
            #message_expanded.translate(#locale)
        }
    })
}

pub struct TranslateInput {
    locale: Ident,
    pub message_input: MessageInput,
}

impl Parse for TranslateInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let locale = input.parse()?;
        input.parse::<Token![,]>()?;
        let message_input = input.parse()?;

        Ok(Self {
            locale,
            message_input,
        })
    }
}
