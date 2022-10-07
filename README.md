# l10n

[![CI badge]][CI] [![Crate badge]][Crate] [![Rustc badge]][Rustc]

[CI badge]: https://img.shields.io/github/workflow/status/MathieuTricoire/l10n/CI/main
[CI]: https://github.com/MathieuTricoire/l10n/actions?query=branch%3Amain
[Crate badge]: https://img.shields.io/crates/v/l10n.svg
[Crate]: https://crates.io/crates/l10n
[Rustc badge]: https://img.shields.io/badge/rustc-1.56+-lightgray.svg
[Rustc]: https://blog.rust-lang.org/2021/10/21/Rust-1.56.0.html

[`l10n`] is a high level and opinionated localization crate built upon the excellent [`fluent-bundle`](https://crates.io/crates/fluent-bundle) crate and the [Fluent project](https://projectfluent.org) and inspired by the [`thiserror`](https://crates.io/crates/thiserror) crate.

The goal of this crate is to ease project localization and provide compile time checks (message exists, mandatory arguments are set, functions are defined).

You can check some examples here: <https://github.com/MathieuTricoire/l10n-examples>

## Installation

```toml
[dependencies]
l10n = "0.1"
```

MSRV: rustc 1.56+

## Quick start

There is no configuration needed to start using [`l10n`], just create a `l10n` directory next to `Cargo.toml`, create as many locale directories (must be a valid locale) containing fluent resources.

### Example

```text
l10n
├── _brand.ftl          (global unnamed resource)
├── en
│   ├── _common.ftl     (unnamed resource)
│   ├── app.ftl         (named resource)
│   └── settings.ftl    (named resource)
├── en-CA
│   └── app.ftl         (named resource)
├── fr
│   ├── _common.ftl     (unnamed resource)
│   ├── app.ftl         (named resource)
│   └── settings.ftl    (named resource)
└── fr-CA
    ├── _terms.ftl      (unnamed resource)
    └── settings.ftl    (named resource)
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

l10n::init!();

fn main() {
    let lang = langid!("fr");

    let username = "Alice";
    let greeting = message!("app", "greeting", "first-name" = username);
    println!("{}", greeting.translate(&lang)); // "Bonjour Alice !"

    let status = Status::Busy {
        reason: "Meeting".to_string(),
    };
    println!("{}", status.translate(&lang)); // "Non disponible (Meeting)"
    println!(
        "{}",
        status.translate_with_args(&lang, Some(&message_args!("gender" => "female")))
    ); // "Occupée (Meeting)"
}

#[derive(L10nMessage)]
#[l10n_message("settings", "status")]
enum Status {
    #[l10n_message(".online")]
    Online,
    #[l10n_message(".offline")]
    Offline,
    #[l10n_message(".busy", .reason, "gender" = "other")]
    Busy(String),
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

To have different paths to the "localization" directory use a map value for `path` (or `paths`) where the key is the name of the environment and the value the path to the "localization" directory. A `default` environment is required.

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

## Details

Coming...

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
