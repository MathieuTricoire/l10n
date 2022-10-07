extern crate proc_macro;
use init::InitInput;
use message::MessageInput;
use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

mod ast;
mod derive;
mod init;
mod instance;
mod message;
mod valid;

#[proc_macro]
pub fn init(item: TokenStream) -> TokenStream {
    init::expand(parse_macro_input!(item as InitInput))
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

#[proc_macro]
pub fn message(item: TokenStream) -> TokenStream {
    message::expand(parse_macro_input!(item as MessageInput))
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

#[proc_macro_derive(L10nMessage, attributes(l10n_message, l10n_from))]
pub fn derive_l10n(token: TokenStream) -> TokenStream {
    derive::expand(parse_macro_input!(token as DeriveInput))
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}
