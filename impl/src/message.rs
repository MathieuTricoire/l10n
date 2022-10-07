use crate::ast::{MessageArgs, MessageKey};
use crate::valid::validate_l10n;
use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{LitStr, Result, Token};

pub fn expand(input: MessageInput) -> Result<TokenStream> {
    let resource = input.resource;
    let key = input.key;

    let args = if input.arguments.is_empty() {
        quote!(std::option::Option::None)
    } else {
        let set_args = input.arguments.iter().map(|arg| {
            let name = arg.name();
            let value = arg.value();
            quote!(args.set(#name, #value);)
        });

        quote! {
            {
                let mut args = ::l10n::fluent_bundle::FluentArgs::new();
                #(#set_args)*
                std::option::Option::Some(args)
            }
        }
    };

    Ok(quote! {
        ::l10n::Message::new(
            &crate::L10N,
            #resource,
            #key,
            #args
        )
    })
}

pub struct MessageInput {
    pub resource: LitStr,
    pub key: MessageKey,
    pub arguments: MessageArgs,
}

impl Parse for MessageInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let resource = input.parse()?;
        input.parse::<Token![,]>()?;
        let key: MessageKey = input.parse::<LitStr>()?.into();

        if !input.is_empty() {
            input.parse::<Token![,]>()?;
        }

        let arguments: MessageArgs = input.parse()?;
        arguments.validate()?;

        validate_l10n(&resource, &key, &arguments, key.span())?;

        Ok(Self {
            resource,
            key,
            arguments,
        })
    }
}
