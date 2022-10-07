use crate::ast::{MessageArgs, MessageKey};
use crate::instance::L10N;
use l10n_core::l10n::TranslateError;
use proc_macro2::Span;
use quote::ToTokens;
use std::collections::HashSet;
use syn::{Error, LitStr, Result};

pub fn validate_l10n(
    resource: &LitStr,
    key: &MessageKey,
    arguments: &MessageArgs,
    span_missing: Span,
) -> Result<()> {
    let required_arguments = L10N
        .as_ref()
        .map_err(|err| Error::new(Span::call_site(), err))?
        .required_variables(&resource.value(), &key.value())
        .map_err(|err| match err {
            TranslateError::ResourceNotExists(_) => Error::new_spanned(&resource, err),
            TranslateError::MessageIdNotExists { .. } => Error::new(key.id_span(), err),
            _ => Error::new_spanned(&key, err),
        })?;

    if arguments.is_complete() {
        let actual_arguments: HashSet<_> = arguments.iter().map(|arg| arg.name().value()).collect();
        let mut missing_arguments: Vec<_> = required_arguments
            .into_iter()
            .filter(|name| !actual_arguments.contains(*name))
            .collect();

        if !missing_arguments.is_empty() {
            missing_arguments.sort();
            return Err(Error::new(
                span_missing,
                format!(
                    r#"missing arguments: "{}" for resource: {} and key: {}"#,
                    missing_arguments.join("\", \""),
                    resource.to_token_stream(),
                    key.to_token_stream()
                ),
            ));
        }
    }

    Ok(())
}
