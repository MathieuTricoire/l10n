use std::io::Read;
use std::path::PathBuf;
use std::{env, fs};

const L10N_CONFIG_FILE: &'static str = "L10N_CONFIG_FILE";

#[test]
fn ui() {
    let default_config_path = PathBuf::from("tests/ui/default_l10n.toml")
        .canonicalize()
        .unwrap();

    for entry in fs::read_dir("tests/ui").unwrap() {
        let entry_path = entry.unwrap().path();
        if !entry_path.is_dir() {
            continue;
        }

        let kind = entry_path.file_name().unwrap().to_str().unwrap();
        let is_pass = match kind {
            "pass" => true,
            "fail" => {
                // Prior to 1.58 errors are "spanned" differently for
                // `Span::call_site()` (i.e. include `;`)
                // So I don't check error messages for fail tests before 1.58.
                if rustversion::cfg!(since(1.58)) {
                    false
                } else {
                    continue;
                }
            }
            _ => continue,
        };

        for entry in entry_path.read_dir().unwrap() {
            env::remove_var("L10N_PATH_ENV");

            let entry_path = entry.unwrap().path();
            if let Some(name) = entry_path.file_name().map(|name| name.to_string_lossy()) {
                if let Some(feature) = name.strip_prefix("feature-") {
                    match feature {
                        "allow-incomplete" => {
                            if !(cfg!(feature = "allow-incomplete")) {
                                continue;
                            }
                        }
                        _ => {
                            unimplemented!("unrecognized feature flag {}", feature);
                        }
                    }
                }
            }

            env::set_var(L10N_CONFIG_FILE, default_config_path.as_os_str());

            let files: Vec<_> = if entry_path.is_dir() {
                if entry_path.join("l10n.toml").exists() {
                    let config_path = env::current_dir()
                        .unwrap()
                        .join(&entry_path)
                        .join("l10n.toml");
                    env::set_var(L10N_CONFIG_FILE, config_path.as_os_str());
                }

                let env_file = entry_path.join(".l10n_path_env");
                if env_file.exists() {
                    let mut f = fs::File::open(env_file).unwrap();
                    let mut s = String::new();
                    f.read_to_string(&mut s).unwrap();
                    env::set_var("L10N_PATH_ENV", s.trim());
                }

                entry_path
                    .read_dir()
                    .unwrap()
                    .filter_map(|entry| {
                        let path = entry.unwrap().path();
                        match path.extension() {
                            Some(ext) if ext == "rs" => Some(path),
                            _ => None,
                        }
                    })
                    .collect()
            } else {
                vec![entry_path]
            };

            let t = trybuild::TestCases::new();
            if is_pass {
                for file in files {
                    t.pass(&file);
                }
            } else {
                for file in files {
                    t.compile_fail(&file);
                }
            }
        }
    }
}
