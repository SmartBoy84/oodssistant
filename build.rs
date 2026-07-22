// - gabe

use std::{collections::HashMap, env, fs, path::PathBuf};

const REQUIRED_KEYS: &[&str] = &[
    "BARK_KEY",
    "GOOGLE_CLIENT_ID",
    "GOOGLE_CLIENT_SECRET",
    "GOOGLE_MY_REFRESH_TOKEN",
];

fn main() {
    // Allow overriding secrets file location when building in CI or elsewhere.
    let secrets_path = env::var("OOD_SECRETS_FILE").unwrap_or_else(|_| "SECRETS.env".to_string());
    let secrets_path = PathBuf::from(secrets_path);

    println!("cargo:rerun-if-changed={}", secrets_path.display());

    let raw = fs::read_to_string(&secrets_path).unwrap_or_else(|err| {
        panic!(
            "Failed to read secrets file at '{}': {err}",
            secrets_path.display()
        )
    });

    let parsed = parse_env_file(&raw);

    for key in REQUIRED_KEYS {
        let value = parsed.get(*key).unwrap_or_else(|| {
            panic!(
                "Missing required key '{key}' in {}",
                secrets_path.display()
            )
        });
        println!("cargo:rustc-env={key}={value}");
    }
}

fn parse_env_file(raw: &str) -> HashMap<String, String> {
    let mut out = HashMap::new();

    for line in raw.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if let Some((name, value)) = line.split_once('=') {
            out.insert(name.trim().to_string(), value.trim().to_string());
        }
    }

    out
}