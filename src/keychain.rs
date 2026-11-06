//! API key lookup from the macOS Keychain.
//!
//! Used as the last fallback after `--api-key`, `ELEVENLABS_API_KEY`, and the
//! config file, so the key never has to live in the shell environment. Store it
//! once with:
//!
//!     security add-generic-password -U -a "$USER" -s elevenlabs-api-key -w

use std::process::Command;

/// Keychain service name holding the API key.
pub const SERVICE: &str = "elevenlabs-api-key";

/// Return `existing` when it is a non-empty key, otherwise read the Keychain.
pub fn resolve_api_key(existing: Option<String>) -> Option<String> {
    resolve_with(existing, read_api_key)
}

fn resolve_with(
    existing: Option<String>,
    lookup: impl FnOnce() -> Option<String>,
) -> Option<String> {
    match existing {
        Some(key) if !key.is_empty() => Some(key),
        _ => lookup(),
    }
}

/// Read the key with `security find-generic-password`. Returns None off macOS,
/// when `security` is missing, or when no matching item exists.
pub fn read_api_key() -> Option<String> {
    if !cfg!(target_os = "macos") {
        return None;
    }
    let account = std::env::var("USER").ok()?;
    let output = Command::new("security")
        .args(["find-generic-password", "-a", &account, "-s", SERVICE, "-w"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    parse_password(&output.stdout)
}

fn parse_password(stdout: &[u8]) -> Option<String> {
    let key = String::from_utf8_lossy(stdout).trim().to_string();
    if key.is_empty() {
        None
    } else {
        Some(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_an_existing_key_without_looking_up() {
        let got = resolve_with(Some("sk_cli".into()), || panic!("lookup should not run"));
        assert_eq!(got.as_deref(), Some("sk_cli"));
    }

    #[test]
    fn falls_back_when_missing_or_empty() {
        assert_eq!(
            resolve_with(None, || Some("sk_kc".into())).as_deref(),
            Some("sk_kc")
        );
        assert_eq!(
            resolve_with(Some(String::new()), || Some("sk_kc".into())).as_deref(),
            Some("sk_kc")
        );
        assert_eq!(resolve_with(None, || None), None);
    }

    #[test]
    fn parses_security_output() {
        assert_eq!(parse_password(b"sk_abc\n").as_deref(), Some("sk_abc"));
        assert_eq!(parse_password(b"\n"), None);
    }
}
