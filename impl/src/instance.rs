use l10n_core::config::{get_config, ConfigError};
use l10n_core::l10n::{BuildErrors, L10n, L10nBuilder, ParserError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum InitError {
    #[error(transparent)]
    Config(#[from] ConfigError),
    #[error(transparent)]
    Parse(#[from] ParserError),
    #[error(transparent)]
    Build(#[from] BuildErrors),
}

pub static L10N: once_cell::sync::Lazy<Result<L10n, InitError>> =
    once_cell::sync::Lazy::new(|| {
        let config = get_config()?;
        Ok(L10nBuilder::parse(config.path()?, config.locales)?.build()?)
    });
