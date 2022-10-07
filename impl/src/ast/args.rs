use proc_macro2::{Delimiter, Group, TokenStream, TokenTree};
use quote::{format_ident, ToTokens};
use std::collections::HashSet;
use syn::parse::{Parse, ParseStream, Peek};
use syn::token::Dot3;
use syn::{
    braced, bracketed, parenthesized, token, Error, Ident, Index, LitInt, LitStr, Result, Token,
};

#[derive(Clone)]
pub struct MessageArgs {
    args: Vec<Argument>,
    incomplete: Option<Dot3>,
}

#[derive(Clone)]
pub enum Argument {
    Short {
        name: LitStr,
        value: TokenStream,
    },
    Long {
        name: LitStr,
        equal: Token![=],
        value: TokenStream,
    },
}

impl MessageArgs {
    pub fn first_to_token_stream(&self) -> Option<TokenStream> {
        self.args.first().map(|arg| arg.to_token_stream())
    }

    pub fn is_empty(&self) -> bool {
        self.args.is_empty()
    }

    pub fn is_complete(&self) -> bool {
        self.incomplete.is_none()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Argument> {
        self.args.iter()
    }

    pub fn validate(&self) -> Result<()> {
        #[cfg(not(feature = "allow-incomplete"))]
        if let Some(incomplete) = self.incomplete {
            return Err(Error::new_spanned(
                incomplete,
                r#"incomplete arguments syntax is not authorized because the feature flag "allow-incomplete" is not set "#,
            ));
        }

        let mut duplicate_error: Option<Error> = None;
        let mut visited_args: Vec<_> = vec![];
        for arg in &self.args {
            let name = arg.name();
            if !visited_args.contains(name) {
                visited_args.push(name.clone());
            } else {
                let err = Error::new_spanned(
                    arg.to_token_stream(),
                    format!("argument {} already set", name.to_token_stream()),
                );
                match duplicate_error {
                    Some(ref mut duplicate_error) => duplicate_error.combine(err),
                    _ => duplicate_error = Some(err),
                }
            }
        }

        if let Some(err) = duplicate_error {
            return Err(err);
        }

        Ok(())
    }

    pub fn validate_for_enum(&self) -> Result<()> {
        if let Some(incomplete) = self.incomplete {
            return Err(Error::new_spanned(
                incomplete,
                "incomplete arguments syntax is not authorized on enum",
            ));
        }

        self.validate()
    }

    pub fn merge_enum_arguments(&mut self, enum_arguments: &MessageArgs) {
        let current_argument_names = self
            .iter()
            .map(|arg| arg.name().to_owned())
            .collect::<HashSet<_>>();
        for argument in enum_arguments.args.iter() {
            if !current_argument_names.contains(argument.name()) {
                self.args.push(argument.clone());
            }
        }
    }
}

impl Default for MessageArgs {
    fn default() -> Self {
        Self {
            args: vec![],
            incomplete: None,
        }
    }
}

impl Parse for MessageArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut arguments = Self::default();
        while !input.is_empty() {
            arguments.incomplete = input.parse::<Option<Dot3>>()?;
            if let Some(incomplete) = arguments.incomplete {
                if !input.is_empty() {
                    return Err(Error::new_spanned(
                        incomplete,
                        "unknown arguments at compile time (i.e. `...`) must be positioned last",
                    ));
                }
            } else {
                arguments.args.push(input.parse()?);
            }

            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(arguments)
    }
}

impl Argument {
    pub fn name(&self) -> &LitStr {
        match self {
            Self::Short { name, .. } => name,
            Self::Long { name, .. } => name,
        }
    }

    pub fn value(&self) -> &TokenStream {
        match self {
            Self::Short { value, .. } => value,
            Self::Long { value, .. } => value,
        }
    }

    pub fn to_token_stream(&self) -> TokenStream {
        match self {
            Self::Short { value, .. } => value.to_token_stream(),
            Self::Long { name, equal, value } => TokenStream::from_iter([
                name.to_token_stream(),
                equal.to_token_stream(),
                value.to_token_stream(),
            ]),
        }
    }
}

impl Parse for Argument {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(Ident) || (input.peek(Token![*]) && input.peek2(Ident)) {
            let unary: Option<Token![*]> = input.parse()?;
            let ident: Ident = input.parse()?;
            let name = LitStr::new(&ident.to_string(), ident.span());
            let value = TokenStream::from_iter([unary.to_token_stream(), ident.to_token_stream()]);
            return if !input.is_empty() && !input.peek(Token![,]) {
                Err(input.error("expected `,` after a shorthand argument"))
            } else {
                Ok(Argument::Short { name, value })
            };
        }

        let name: LitStr = input.parse().map_err(|err| {
            Error::new(
                err.span(),
                r#"expected an argument, example: `"variable" = field` or `field`"#,
            )
        })?;
        let equal = input.parse().map_err(|err| {
            Error::new(
                err.span(),
                format!(
                    "expected a value assignment for this argument, example: `{} = field`",
                    name.to_token_stream()
                ),
            )
        })?;
        Ok(Argument::Long {
            name,
            equal,
            value: parse_argument_value(input, true, Token![,], false)?,
        })
    }
}

fn parse_argument_value<T: Peek>(
    input: ParseStream,
    mut begin_expr: bool,
    separator: T,
    in_group: bool,
) -> Result<TokenStream> {
    let mut tokens = Vec::new();
    while !input.is_empty() {
        if !in_group && input.peek(separator) {
            break;
        }

        if begin_expr && input.peek(Token![.]) {
            if input.peek2(Ident) {
                input.parse::<Token![.]>()?;
                begin_expr = false;
                continue;
            }
            if input.peek2(LitInt) {
                input.parse::<Token![.]>()?;
                let int: Index = input.parse()?;
                let ident = format_ident!("__self_{}", int.index, span = int.span);
                tokens.push(TokenTree::Ident(ident));
                begin_expr = false;
                continue;
            }
        }

        begin_expr = input.peek(Token![break])
            || input.peek(Token![continue])
            || input.peek(Token![if])
            || input.peek(Token![in])
            || input.peek(Token![match])
            || input.peek(Token![mut])
            || input.peek(Token![return])
            || input.peek(Token![while])
            || input.peek(Token![+])
            || input.peek(Token![&])
            || input.peek(Token![!])
            || input.peek(Token![^])
            || input.peek(Token![,])
            || input.peek(Token![/])
            || input.peek(Token![=])
            || input.peek(Token![>])
            || input.peek(Token![<])
            || input.peek(Token![|])
            || input.peek(Token![%])
            || input.peek(Token![;])
            || input.peek(Token![*])
            || input.peek(Token![-]);

        let token: TokenTree = if input.peek(token::Paren) {
            let content;
            let delimiter = parenthesized!(content in input);
            let nested = parse_argument_value(&content, true, separator, true)?;
            let mut group = Group::new(Delimiter::Parenthesis, nested);
            group.set_span(delimiter.span);
            TokenTree::Group(group)
        } else if input.peek(token::Brace) {
            let content;
            let delimiter = braced!(content in input);
            let nested = parse_argument_value(&content, true, separator, true)?;
            let mut group = Group::new(Delimiter::Brace, nested);
            group.set_span(delimiter.span);
            TokenTree::Group(group)
        } else if input.peek(token::Bracket) {
            let content;
            let delimiter = bracketed!(content in input);
            let nested = parse_argument_value(&content, true, separator, true)?;
            let mut group = Group::new(Delimiter::Bracket, nested);
            group.set_span(delimiter.span);
            TokenTree::Group(group)
        } else {
            input.parse()?
        };
        tokens.push(token);
    }
    Ok(TokenStream::from_iter(tokens))
}
