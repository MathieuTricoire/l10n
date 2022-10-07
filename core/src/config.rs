use crate::locales::Locales;
use serde::{de::Error, Deserialize, Deserializer};
use std::{collections::HashMap, env, fs, path::PathBuf};
use thiserror::Error;

#[derive(Deserialize)]
struct ConfigFile {
    pub l10n: Config,
}

#[derive(Deserialize, PartialEq, Eq, Debug)]
pub struct Config {
    #[serde(alias = "path", default = "default_paths")]
    pub paths: Paths,
    pub locales: Option<Locales>,
}

#[derive(PartialEq, Eq, Debug)]
pub struct Paths {
    pub environments: HashMap<String, PathBuf>,
    pub default: PathBuf,
}

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("`CARGO_MANIFEST_DIR` env. variable not set, you can use `L10N_CONFIG_FILE` env. var if you are not using Cargo.")]
    CargoManifestDir,
    #[error("error reading file: {}", path.display())]
    ReadFile {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error(r#"error deserializing file "{}": {}"#, path.display(), source)]
    Deserialize {
        path: PathBuf,
        source: toml::de::Error,
    },
    #[error(r#"l10n path for environment "{0}" is not set in the configuration"#)]
    MissingPathError(String),
}

impl Config {
    pub fn path(&self) -> Result<PathBuf, ConfigError> {
        if let Ok(environment) = env::var("L10N_PATH_ENV") {
            self.paths
                .environments
                .get(&environment)
                .cloned()
                .ok_or_else(|| ConfigError::MissingPathError(environment))
        } else {
            Ok(self.paths.default.clone())
        }
    }
}

impl<'de> Deserialize<'de> for Paths {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Helper {
            Short(PathBuf),
            Full(HashMap<String, PathBuf>),
        }

        Ok(match Helper::deserialize(deserializer)? {
            Helper::Short(default) => Paths {
                environments: HashMap::new(),
                default,
            },
            Helper::Full(mut profiles) => {
                let default = profiles
                    .remove("default")
                    .ok_or(Error::missing_field("default"))?;
                Paths {
                    environments: profiles,
                    default,
                }
            }
        })
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            paths: default_paths(),
            locales: None,
        }
    }
}

fn default_paths() -> Paths {
    Paths {
        environments: HashMap::new(),
        default: PathBuf::from("l10n"),
    }
}

pub fn config_file_path() -> Result<Option<PathBuf>, ConfigError> {
    let l10n_config_file = env::var("L10N_CONFIG_FILE");
    if let Ok(l10n_config_file) = &l10n_config_file {
        let path = PathBuf::from(l10n_config_file);
        if path.is_absolute() {
            return path
                .canonicalize()
                .map(|path| Some(path))
                .map_err(|source| ConfigError::ReadFile { path, source });
        }
    }

    let root =
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").map_err(|_| ConfigError::CargoManifestDir)?);

    if let Ok(l10n_config_file) = l10n_config_file {
        let path = root.join(l10n_config_file);
        return path
            .canonicalize()
            .map(|path| Some(path))
            .map_err(|source| ConfigError::ReadFile { path, source });
    }

    let l10n_path = root.join("l10n.toml");
    if let Ok(path) = l10n_path.canonicalize() {
        return Ok(Some(path));
    }

    let config_path = root.join("config.toml");
    if let Ok(path) = config_path.canonicalize() {
        return Ok(Some(path));
    }

    Ok(None)
}

pub fn get_config() -> Result<Config, ConfigError> {
    let config = if let Some(config_path) = config_file_path()? {
        let toml_string =
            fs::read_to_string(&config_path).map_err(|source| ConfigError::ReadFile {
                path: config_path.clone(),
                source,
            })?;

        let mut config = deserialize_translator_config(&toml_string).map_err(|source| {
            ConfigError::Deserialize {
                path: config_path.clone(),
                source,
            }
        })?;

        replace_root_var_in_path(&mut config.paths.default, &config_path);
        config
            .paths
            .environments
            .iter_mut()
            .for_each(|(_, path)| replace_root_var_in_path(path, &config_path));

        config
    } else {
        Default::default()
    };

    Ok(config)
}

fn deserialize_translator_config(source: &str) -> Result<Config, toml::de::Error> {
    Ok(toml::from_str::<'_, ConfigFile>(source)?.l10n)
}

fn replace_root_var_in_path(path: &mut PathBuf, root_path: &PathBuf) {
    if !path.is_absolute() && path.starts_with("$ROOT") {
        let unprefixed_path = path.strip_prefix("$ROOT").unwrap();
        *path = match root_path.parent() {
            Some(parent) => parent.join(unprefixed_path),
            None => PathBuf::from("/").join(unprefixed_path),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;

    #[test]
    fn deserialize_config_ok_1() {
        let config = r#"
            [l10n]
            path = "l10n_directory"
            locales = [
                "en",
                { main = "en-GB", fallback = "en" },
                { main = "en-CA", fallback = "en-GB" },
                { main = "fr" },
                { main = "fr-CA", fallback = "fr" },
            ]

            [other-config]
            with = "different config"
        "#;
        let expected = Config {
            paths: Paths {
                environments: HashMap::new(),
                default: PathBuf::from("l10n_directory"),
            },
            locales: Some(
                Locales::try_from([
                    ("en", None),
                    ("en-GB", Some("en")),
                    ("en-CA", Some("en-GB")),
                    ("fr", None),
                    ("fr-CA", Some("fr")),
                ])
                .unwrap(),
            ),
        };
        let actual = deserialize_translator_config(&config).unwrap();
        assert_eq!(actual, expected);

        let config = r#"
            [l10n]
        "#;
        let expected = Config {
            paths: Paths {
                environments: HashMap::new(),
                default: PathBuf::from("l10n"),
            },
            locales: None,
        };
        let actual = deserialize_translator_config(&config).unwrap();
        assert_eq!(actual, expected);

        let config = r#"
            [l10n]
            paths = { default = "$ROOT/l10n", release = "/var/l10n" }
        "#;
        let expected = Config {
            paths: Paths {
                environments: HashMap::from([("release".to_string(), PathBuf::from("/var/l10n"))]),
                default: PathBuf::from("$ROOT/l10n"),
            },
            locales: None,
        };
        let actual = deserialize_translator_config(&config).unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn deserialize_config_paths_errors() {
        let config = indoc! {r#"
            [l10n]
            paths = { production = "/var/l10n" }
        "#};
        let error = deserialize_translator_config(&config).unwrap_err();
        assert_eq!(
            &error.to_string(),
            "missing field `default` for key `l10n.paths` at line 1 column 1"
        );
    }

    #[test]
    fn deserialize_config_locales_errors() {
        let config = indoc! {r#"
            [l10n]
            locales = [
                { another = "key" },
            ]
        "#};
        let error = deserialize_translator_config(&config).unwrap_err();
        assert_eq!(
            &error.to_string(),
            r#"missing field `main` for key `l10n.locales` at line 3 column 5"#
        );

        let config = indoc! {r#"
            [l10n]
            locales = [
                { main = "en-GB", fallback = "not-a-locale" },
                { main = "en-CA", fallback = "en-GB" },
                "fr",
                { main = "fr-CA", fallback = "fr" },
            ]
        "#};
        let error = deserialize_translator_config(&config).unwrap_err();
        assert_eq!(
            &error.to_string(),
            r#"invalid value: string "not-a-locale", expected a valid Unicode Language Identifier like "en-US" (Parser error: Invalid subtag) for key `l10n.locales` at line 3 column 34"#
        );
    }
}
