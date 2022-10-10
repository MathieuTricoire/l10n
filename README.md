# l10n

[![CI badge]][CI] [![Crate badge]][Crate] [![Rustc badge]][Rustc]

[CI badge]: https://img.shields.io/github/workflow/status/MathieuTricoire/l10n/CI/main
[CI]: https://github.com/MathieuTricoire/l10n/actions?query=branch%3Amain
[Crate badge]: https://img.shields.io/crates/v/l10n.svg
[Crate]: https://crates.io/crates/l10n
[Rustc badge]: https://img.shields.io/badge/rustc-1.56+-lightgray.svg
[Rustc]: https://blog.rust-lang.org/2021/10/21/Rust-1.56.0.html

[`l10n`] is a high level and opinionated localization crate built upon the excellent [`fluent-bundle`](https://crates.io/crates/fluent-bundle) crate, the [Fluent project](https://projectfluent.org) and inspired by the [`thiserror`](https://crates.io/crates/thiserror) crate.

The goal of this crate is to ease project localization and provide compile time checks (message exists, mandatory arguments are set, functions are defined).

You can check some examples here: <https://github.com/MathieuTricoire/l10n-examples>

Code repository: <https://github.com/MathieuTricoire/l10n>

## Installation

_Note: [`l10n`] is not yet published on crates.io because some PRs need to be merged and released on [`fluent-bundle`](https://crates.io/crates/fluent-bundle) [#264](https://github.com/projectfluent/fluent-rs/pull/264) and [#271](https://github.com/projectfluent/fluent-rs/pull/271) which l10n depends on (and I don’t want to publish a fork of fluent-bundle on crates.io or add fluent-bundle fork code in my crate for now)_

```toml
[dependencies]
l10n = { git = "https://github.com/MathieuTricoire/l10n.git" }
```

MSRV: rustc 1.56+

## Quick start

There is no configuration needed to start using [`l10n`], just create a `l10n` directory next to `Cargo.toml`, create as many locale directories (must be valid locales) containing fluent resources.

### Example

Localization directory tree structure:

```text
l10n
├── _brand.ftl          (global unnamed resource)
├── en
│   ├── _common.ftl     (unnamed resource)
│   ├── app.ftl         (named resource)
│   └── settings.ftl    (named resource)
├── en-CA
├── en-GB
│   └── app.ftl         (named resource)
├── fr
│   ├── _common.ftl     (unnamed resource)
│   ├── app.ftl         (named resource)
│   └── settings.ftl    (named resource)
└── fr-CA
    ├── _terms.ftl      (unnamed resource)
    └── settings.ftl    (named resource)
Cargo.toml
```

`l10n/fr/app.ftl` file:

```text
greeting = Bonjour { $first-name } !
```

`l10n/fr/settings.ftl` file:

```text
status =
    .online = En ligne
    .offline = Hors ligne
    .busy = { $gender ->
        [male] Occupé
        [female] Occupée
       *[other] Non disponible
    } ({ $reason })
```

Then in the root of your application or library initialize `l10n` (this create a `L10N` static ref used by other macros) and create l10n messages either with the `message!` macro or by deriving `L10nMessage`.

```rust
use l10n::unic_langid::langid;
use l10n::{message, message_args, L10nMessage};
use l10n::fluent_bundle::{FluentValue, FluentArgs}; // for functions, not necessary for this example

l10n::init!({
    // not necessary for this example
    functions: { "TIME": |_: &[FluentValue<'_>], _: &FluentArgs| FluentValue::None }
});

fn main() {
    let lang = langid!("fr");

    let username = "Alice";
    let greeting = message!("app", "greeting", "first-name" = username);
    assert_eq!(greeting.translate(&lang), "Bonjour \u{2068}Alice\u{2069} !");

    let status = Status::Busy {
        reason: "Meeting".to_string(),
    };
    assert_eq!(status.translate(&lang), "\u{2068}Non disponible\u{2069} (\u{2068}Meeting\u{2069})");
    assert_eq!(
        status.translate_with_args(&lang, Some(&message_args!("gender" => "female"))),
        "\u{2068}Occupée\u{2069} (\u{2068}Meeting\u{2069})"
    );
}

#[derive(L10nMessage)]
#[l10n_message("settings", "status")]
enum Status {
    #[l10n_message(".online")]
    Online,
    #[l10n_message(".offline")]
    Offline,
    #[l10n_message(".busy", reason, "gender" = "other")]
    Busy { reason: String },
}
```

## Advanced usage

### Configuration file

Create a `l10n.toml` or `config.toml` file next to `Cargo.toml`, to define the locales nor set a different path to the "localization" directory containing the locale directories and fluent files.

`l10n.toml` file example:

```toml
[l10n]
locales = [
    "en",
    { main = "en-GB", fallback = "en" },
    { main = "en-CA", fallback = "en-GB" },
    "fr",
    { main = "fr-CA", fallback = "fr" },
]
path = "localization_files"
```

To use another configuration file at compile time, set the environment variable `L10N_CONFIG_FILE` like this `L10N_CONFIG_FILE=/path/to/specific-config.toml`.

### Path to localization directory

To have different paths to the "localization" directory according to your need, use a map value for `path` (or `paths`) where the key is the name of the environment and the value the path to the "localization" directory. A `default` environment is required.

Then to compile your artificat with this environment use the environment variable `L10N_PATH_ENV`.

`l10n.toml` file example:

```toml
[l10n]
paths = { default = "l10n", prod = "/path/to/l10n" }
```

Build command:

```sh
L10N_PATH_ENV=prod cargo build --release
```

You can also prefix your path with a special variable `$ROOT` and the library will replace this variable with the path to the configuration file.

`/path/to/l10n.toml` file example:

```toml
[l10n]
path = "$ROOT/localization_files"
```

Produced path: `/path/to/localization_files`.

### Locales

#### Discovered locales

If no locales configuration is provided, `l10n` will discover the locales in the "localization" directory. `l10n` implements a __very basic__ fallback mechanism between discovered locales, if a locale contains a "region" code it will fallback to the same locale without the "region" code if exists.

Localization directory tree structure (only locale directories are shown):

```text
l10n
├── en
├── en-CA
├── en-GB
├── en-GB-variant
├── en-Latn
├── en-Latn-variant
├── en-Latn-GB
└── en-Latn-GB-variant
```

In the example above the fallbacks will be:

- `en`: _no fallback_
- `en-CA`: __fallback to__ `en`
- `en-GB`: __fallback to__ `en`
- `en-GB-variant`: _no fallback_
- `en-Latn`: _no fallback_
- `en-Latn-variant`: _no fallback_
- `en-Latn-GB`: __fallback to__ `en-Latn`
- `en-Latn-GB-variant`: __fallback to__ `en-Latn-variant`

#### Locales set in configuration

A locale can be used if set as a "main" locale, this means if a locale is only set as a fallback it will not be possible to translate messages in this locale.

`l10n.toml` file example

```toml
[l10n]
locales = [
  { main = "en-US", fallback = "en" },
  { main = "en-GB", fallback = "en" },
  { main = "en-CA", fallback = "en-GB" },
  { main = "fr" }, # same as writing `"fr",`
  { main = "fr-CA", fallback = "fr" },
]
```

In this example the messages can only be translated with the locales: `en-US`, `en-GB`, `en-CA`, `fr`, `fr-CA` and not `en` which is only set as a fallback "locale".

## Details

### Resources

There is 3 kind of resources:

- Global unnamed resources: Under the `l10n` directory starting with `_`, these resources are shared across all named resources in all locales.
- Unnamed resources: Under locale directories starting with `_`, these resources are shared across all named resources in the current locale.
- Named resources: Under locale directories, these are the resources containing the messages you can use in your code.

"Global unnamed resources" and "Unnamed resources" can be freely created and will be load according to their attached locale.

"Named resources" must exists for all "mandatory locales". "Mandatory locales" are all the locales at the end of a resolution route, in the next example the "mandatory locales" are: "en" and "fr".

```toml
[l10n]
locales = [
  { main = "en-GB", fallback = "en" },
  { main = "en-CA", fallback = "en-GB" },
  "fr",
  { main = "fr-CA", fallback = "fr" },
]
```

---

## License

Licensed under either of

- Apache License, Version 2.0, (LICENSE-APACHE or <https://www.apache.org/licenses/LICENSE-2.0>)
- MIT license (LICENSE-MIT or <https://opensource.org/licenses/MIT>)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.

[`l10n`]: https://crates.io/crates/l10n
