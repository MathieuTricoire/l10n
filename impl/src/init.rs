use crate::instance::L10N;
use l10n_core::config::get_config;
use proc_macro2::TokenStream;
use proc_macro2::{Ident, Span};
use quote::quote;
use std::collections::HashSet;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{braced, Error, Expr, LitStr, Result, Token};

pub fn expand(input: InitInput) -> Result<TokenStream> {
    let config = get_config().map_err(|err| Error::new(Span::call_site(), err))?;

    let l10n_instance = L10N
        .as_ref()
        .map_err(|err| Error::new(Span::call_site(), err))?;

    let locales = l10n_instance.locales.into_iter().map(|entry| {
        let locale = entry.locale().to_string();
        let fallback = match entry.fallback() {
            Some(fallback) => {
                let value = fallback.to_string();
                quote!(std::option::Option::Some(#value))
            }
            None => quote!(std::option::Option::None),
        };
        quote!((#locale, #fallback))
    });

    let builder_locales = quote! {
        std::option::Option::Some(::l10n::Locales::try_from([
            #(#locales),*
        ]).expect("unexpected error parsing a locale"))
    };

    let config_path = config
        .path()
        .map_err(|err| Error::new(Span::call_site(), err))?;
    let builder_path = config_path.to_string_lossy();

    let transform = input
        .transform
        .map(|transform| quote!(.set_transform(#transform)));

    let formatter = input
        .formatter
        .map(|formatter| quote!(.set_formatter(#formatter)));

    let use_isolating = input
        .use_isolating
        .map(|use_isolating| quote!(.set_use_isolating(#use_isolating)));

    let add_functions = input.functions.map(|functions| {
        let add_functions = functions.iter().map(|function_input| {
            let name = &function_input.name;
            let function = &function_input.function;
            quote!(.add_function(#name, #function))
        });
        quote!(#(#add_functions)*)
    });

    let translator = quote! {
        {
            ::l10n::L10nBuilder::parse(#builder_path, #builder_locales)
                .expect("error parsing translation files")
                #transform
                #formatter
                #use_isolating
                #add_functions
                .build()
                .expect("error building translator")
        }
    };

    Ok(quote! {
        pub static L10N: ::l10n::once_cell::sync::Lazy<::l10n::L10n> = ::l10n::once_cell::sync::Lazy::new(|| #translator);
    })
}

#[derive(Default)]
pub struct InitInput {
    pub transform: Option<Expr>,
    pub formatter: Option<Expr>,
    pub use_isolating: Option<Expr>,
    pub functions_key: Option<Ident>,
    pub functions: Option<Punctuated<Function, Token![,]>>,
}

pub enum Field {
    Formatter(Ident, Expr),
    Transform(Ident, Expr),
    UseIsolating(Ident, Expr),
    Functions(Ident, Punctuated<Function, Token![,]>),
}

pub struct Function {
    pub name: LitStr,
    pub function: Expr,
}

impl InitInput {
    pub fn validate(&self) -> Result<()> {
        if let Some(functions) = &self.functions {
            let mut duplicate_error: Option<Error> = None;
            let mut visited_functions: HashSet<&LitStr> = HashSet::new();

            for function in functions {
                if !visited_functions.contains(&&function.name) {
                    visited_functions.insert(&function.name);
                } else {
                    let err = Error::new_spanned(&function.name, "function duplicate");
                    match duplicate_error {
                        Some(ref mut duplicate_error) => duplicate_error.combine(err),
                        _ => duplicate_error = Some(err),
                    }
                }
            }

            if let Some(err) = duplicate_error {
                return Err(err);
            }
        }

        let mut missing_functions = L10N
            .as_ref()
            .map_err(|err| Error::new(Span::call_site(), err))?
            .required_functions();

        if let Some(functions) = &self.functions {
            let actual_functions: HashSet<_> = functions.iter().map(|f| f.name.value()).collect();
            missing_functions.retain(|name| !actual_functions.contains(*name));
        };

        if !missing_functions.is_empty() {
            let mut missing_functions: Vec<_> = missing_functions.into_iter().collect();
            missing_functions.sort();
            let span = self
                .functions_key
                .as_ref()
                .map(|v| v.span())
                .unwrap_or_else(Span::call_site);
            return Err(Error::new(
                span,
                format!("missing functions: {}", missing_functions.join(", ")),
            ));
        }

        Ok(())
    }
}

impl Parse for InitInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut init_input = Self::default();

        if !input.is_empty() {
            let content;
            braced!(content in input);
            let fields: Punctuated<Field, Comma> =
                content.parse_terminated(Field::parse, Token![,])?;
            for field in fields {
                match field {
                    Field::Formatter(ident, formatter) => {
                        if init_input.formatter.is_none() {
                            init_input.formatter = Some(formatter);
                        } else {
                            return Err(Error::new_spanned(ident, "duplicate `formatter` field"));
                        }
                    }
                    Field::Transform(ident, transform) => {
                        if init_input.transform.is_none() {
                            init_input.transform = Some(transform);
                        } else {
                            return Err(Error::new_spanned(ident, "duplicate `transform` field"));
                        }
                    }
                    Field::UseIsolating(ident, use_isolating) => {
                        if init_input.use_isolating.is_none() {
                            init_input.use_isolating = Some(use_isolating);
                        } else {
                            return Err(Error::new_spanned(
                                ident,
                                "duplicate `use_isolating` field",
                            ));
                        }
                    }
                    Field::Functions(ident, functions) => {
                        if init_input.functions.is_none() {
                            init_input.functions_key = Some(ident);
                            init_input.functions = Some(functions);
                        } else {
                            return Err(Error::new_spanned(ident, "duplicate `functions` field"));
                        }
                    }
                }
            }
        }

        init_input.validate()?;

        Ok(init_input)
    }
}

impl Parse for Field {
    fn parse(input: ParseStream) -> Result<Self> {
        let ident: Ident = input.parse()?;
        input.parse::<Token![:]>()?;

        match ident.to_string().as_str() {
            "formatter" => Ok(Self::Formatter(ident, input.parse()?)),
            "transform" => Ok(Self::Transform(ident, input.parse()?)),
            "use_isolating" => Ok(Self::UseIsolating(ident, input.parse()?)),
            "functions" => {
                let content;
                braced!(content in input);
                Ok(Self::Functions(
                    ident,
                    content.parse_terminated(Function::parse, Token![,])?,
                ))
            }
            _ => Err(Error::new_spanned(
                ident,
                r#"invalid field (expected: "formatter", "transform", "use_isolating" or "functions")"#,
            )),
        }
    }
}

impl Parse for Function {
    fn parse(input: ParseStream) -> Result<Self> {
        let name = input.parse()?;
        input.parse::<Token![:]>()?;
        let function = input.parse()?;
        Ok(Self { name, function })
    }
}
