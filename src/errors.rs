//! Error handling utilities for ElevenLabs CLI
//!
//! This module provides comprehensive error handling including:
//! - Error type detection (unauthorized, forbidden, rate limited, etc.)
//! - Retry logic with exponential backoff for transient errors
//! - Helpful error messages with links to ElevenLabs documentation

#![allow(dead_code)]

use colored::*;
use std::fmt;
use std::time::Duration;

/// Maximum number of retry attempts for transient errors
const MAX_RETRIES: u32 = 3;

/// Initial delay between retries (in milliseconds)
const INITIAL_BACKOFF_MS: u64 = 1000;

/// Maximum backoff delay (in milliseconds)
const MAX_BACKOFF_MS: u64 = 30000;

/// API Error types for better error messages
#[derive(Debug, Clone)]
pub enum ApiError {
    /// Missing or empty API key
    MissingApiKey,
    /// Invalid API key (401)
    Unauthorized,
    /// Permission denied (403)
    Forbidden(String),
    /// Resource not found (404)
    NotFound(String),
    /// Rate limited (429)
    RateLimited,
    /// Server error (5xx)
    ServerError(u16, String),
    /// Network error
    NetworkError(String),
    /// Timeout
    Timeout,
    /// Other errors
    Other(String),
}

impl std::error::Error for ApiError {}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiError::MissingApiKey => write!(f, "API key is required"),
            ApiError::Unauthorized => write!(f, "Invalid API key"),
            ApiError::Forbidden(feature) => write!(f, "Permission denied: {}", feature),
            ApiError::NotFound(resource) => write!(f, "Not found: {}", resource),
            ApiError::RateLimited => write!(f, "Rate limited"),
            ApiError::ServerError(code, msg) => write!(f, "Server error {}: {}", code, msg),
            ApiError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            ApiError::Timeout => write!(f, "Request timed out"),
            ApiError::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

/// Whether an error type can be retried
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Retryable {
    /// Error is retryable (e.g., rate limited, server error, network issue)
    Yes,
    /// Error is not retryable (e.g., invalid auth, permission denied, not found)
    No,
    /// Error should be retried with a longer delay (rate limit)
    YesWithBackoff,
}

impl ApiError {
    /// Determine if this error type is retryable
    pub fn retryable(&self) -> Retryable {
        match self {
            ApiError::MissingApiKey => Retryable::No,
            ApiError::Unauthorized => Retryable::No,
            ApiError::Forbidden(_) => Retryable::No,
            ApiError::NotFound(_) => Retryable::No,
            ApiError::RateLimited => Retryable::YesWithBackoff,
            ApiError::ServerError(_, _) => Retryable::Yes,
            ApiError::NetworkError(_) => Retryable::Yes,
            ApiError::Timeout => Retryable::Yes,
            ApiError::Other(_) => Retryable::Yes,
        }
    }

    /// Get recommended wait time before retry (for rate limits)
    pub fn retry_after(&self) -> Option<Duration> {
        match self {
            ApiError::RateLimited => Some(Duration::from_secs(60)),
            ApiError::ServerError(_, _) => Some(Duration::from_secs(5)),
            _ => None,
        }
    }
}

/// Check if the error is due to missing API key
pub fn is_missing_api_key(err: &anyhow::Error) -> bool {
    let error_str = err.to_string().to_lowercase();
    error_str.contains("api key") && (error_str.contains("required") || error_str.contains("none"))
}

/// Check if the error is due to unauthorized/invalid API key (401)
pub fn is_unauthorized(err: &anyhow::Error) -> bool {
    let error_str = err.to_string().to_lowercase();
    error_str.contains("401")
        || error_str.contains("unauthorized")
        || error_str.contains("invalid api key")
}

/// Check if the error is due to permission denied (403)
pub fn is_forbidden(err: &anyhow::Error) -> bool {
    let error_str = err.to_string().to_lowercase();
    error_str.contains("403")
        || error_str.contains("forbidden")
        || error_str.contains("permission denied")
}

/// Check if the error is due to not found (404)
pub fn is_not_found(err: &anyhow::Error) -> bool {
    let error_str = err.to_string().to_lowercase();
    error_str.contains("404") || error_str.contains("not found")
}

/// Check if the error is due to rate limiting (429)
pub fn is_rate_limited(err: &anyhow::Error) -> bool {
    let error_str = err.to_string().to_lowercase();
    error_str.contains("429")
        || error_str.contains("rate limit")
        || error_str.contains("too many requests")
}

/// Check if the error is a server error (5xx)
pub fn is_server_error(err: &anyhow::Error) -> bool {
    let error_str = err.to_string().to_lowercase();
    error_str.contains("500")
        || error_str.contains("502")
        || error_str.contains("503")
        || error_str.contains("504")
        || error_str.contains("server error")
}

/// Check if the error is a network error
pub fn is_network_error(err: &anyhow::Error) -> bool {
    let error_str = err.to_string().to_lowercase();
    error_str.contains("connection")
        || error_str.contains("network")
        || error_str.contains("timeout")
        || error_str.contains("connect")
}

/// Parse error into ApiError type
pub fn parse_api_error(err: &anyhow::Error) -> ApiError {
    if is_missing_api_key(err) {
        ApiError::MissingApiKey
    } else if is_unauthorized(err) {
        ApiError::Unauthorized
    } else if is_forbidden(err) {
        ApiError::Forbidden("Unknown feature".to_string())
    } else if is_not_found(err) {
        ApiError::NotFound("Unknown resource".to_string())
    } else if is_rate_limited(err) {
        ApiError::RateLimited
    } else if is_server_error(err) {
        ApiError::ServerError(500, err.to_string())
    } else if is_network_error(err) {
        ApiError::NetworkError(err.to_string())
    } else {
        ApiError::Other(err.to_string())
    }
}

/// Determine if an error is retryable
pub fn is_retryable(err: &anyhow::Error) -> Retryable {
    parse_api_error(err).retryable()
}

/// The ElevenLabs authentication failure behind an API error, if any.
#[derive(Debug, PartialEq)]
pub enum AuthProblem {
    /// The key ID (shown in the dashboard key list) was used instead of the `sk_` secret
    KeyIdUsedAsKey,
    /// The key is valid but restricted; carries the missing permission when the API names it
    MissingPermission(Option<String>),
    /// The key is unknown, revoked, or malformed
    InvalidKey,
}

/// Classify an ElevenLabs authentication error from the API's `status`/`code` fields.
pub fn auth_problem(err: &anyhow::Error) -> Option<AuthProblem> {
    let msg = format!("{:#}", err);
    if msg.contains("api_key_id_used_as_api_key") {
        return Some(AuthProblem::KeyIdUsedAsKey);
    }
    if msg.contains("missing_permissions") {
        let perm = msg
            .split("missing the permission ")
            .nth(1)
            .and_then(|rest| rest.split_whitespace().next())
            .map(|p| p.to_string());
        return Some(AuthProblem::MissingPermission(perm));
    }
    if msg.contains("invalid_api_key") || msg.contains("authentication_error") {
        return Some(AuthProblem::InvalidKey);
    }
    None
}

const KEYS_URL: &str = "https://elevenlabs.io/app/settings/api-keys";
const KEYCHAIN_UPDATE: &str =
    "security add-generic-password -U -a \"$USER\" -s elevenlabs-api-key -w";

/// Print fix steps for an authentication error. Returns false if `err` is not one.
pub fn print_auth_hint(err: &anyhow::Error) -> bool {
    let Some(problem) = auth_problem(err) else {
        return false;
    };
    eprintln!();
    match &problem {
        AuthProblem::KeyIdUsedAsKey => {
            eprintln!(
                "{}",
                "You used the API key's ID, not the key itself.".yellow()
            );
            eprintln!("  The key list at {} shows IDs. The secret key", KEYS_URL);
            eprintln!("  starts with `sk_` and is shown only once, when the key is created.");
            eprintln!("  If you no longer have it, rotate the key or create a new one.");
        }
        AuthProblem::MissingPermission(perm) => {
            let perm = perm.as_deref().unwrap_or("required");
            eprintln!(
                "{}",
                format!(
                    "Your API key is restricted and lacks the `{}` permission.",
                    perm
                )
                .yellow()
            );
            eprintln!("  Edit the key at {} and enable it,", KEYS_URL);
            eprintln!("  or use a key with broader access. Other commands may still work.");
            return true;
        }
        AuthProblem::InvalidKey => {
            eprintln!(
                "{}",
                "Your API key was rejected (invalid, revoked, or mistyped).".yellow()
            );
            eprintln!("  Check it at {}; keys start with `sk_`.", KEYS_URL);
        }
    }
    eprintln!();
    eprintln!("  Update the key in macOS Keychain (prompts for it, never echoes it):");
    eprintln!("    {}", KEYCHAIN_UPDATE);
    eprintln!("  Then open a new shell (or `source ~/.custom`) so ELEVENLABS_API_KEY reloads.");
    true
}

/// Print error with helpful guidance based on error type
pub fn print_api_error(err: &anyhow::Error) {
    if is_missing_api_key(err) {
        eprintln!("{}", "Error: API key is required".red());
        eprintln!();
        eprintln!("{}", "To get an API key:".yellow());
        eprintln!("  1. Go to https://elevenlabs.io/app/settings/api-keys");
        eprintln!("  2. Create a new API key");
        eprintln!("  3. Copy it and set it as ELEVENLABS_API_KEY environment variable");
        eprintln!();
        eprintln!("{}", "Or use the --api-key flag:".yellow());
        eprintln!("  elevenlabs --api-key YOUR_API_KEY ...");
    } else if is_unauthorized(err) {
        eprintln!("{}", "Error: Invalid API key".red());
        if !print_auth_hint(err) {
            eprintln!();
            eprintln!("{}", "Your API key may be invalid or expired.".yellow());
            eprintln!("Get a new key from: {}", KEYS_URL);
        }
    } else if is_forbidden(err) {
        eprintln!("{}", "Error: Permission denied".red());
        eprintln!();
        eprintln!(
            "{}",
            "Your API key doesn't have permission for this feature.".yellow()
        );
        eprintln!("Check your subscription tier at: https://elevenlabs.io/app/settings");
        eprintln!();
        eprintln!("Some features require:");
        eprintln!("  - Paid subscription (for professional voice cloning)");
        eprintln!("  - Business subscription (for agents, phone, etc.)");
        eprintln!("See: https://elevenlabs.io/pricing");
    } else if is_not_found(err) {
        eprintln!("{}", "Error: Resource not found".red());
        eprintln!("The requested resource doesn't exist or has been deleted.");
    } else if is_rate_limited(err) {
        eprintln!("{}", "Error: Rate limited".red());
        eprintln!();
        eprintln!("{}", "You have made too many requests.".yellow());
        eprintln!("Please wait at least 60 seconds before trying again.");
        eprintln!();
        eprintln!("See: https://elevenlabs.io/docs/api-reference/rate-limits");
    } else if is_server_error(err) {
        eprintln!("{}", "Error: Server error".red());
        eprintln!();
        eprintln!("{}", "ElevenLabs servers are experiencing issues.".yellow());
        eprintln!("This is usually temporary. Try again in a few moments.");
    } else if is_network_error(err) {
        eprintln!("{}", "Error: Network error".red());
        eprintln!("Could not connect to ElevenLabs API.");
        eprintln!("Check your internet connection and try again.");
    } else {
        eprintln!("{}", format!("Error: {}", err).red());
    }
}

/// Print a retryable error with backoff information
pub fn print_retry_error(err: &anyhow::Error, attempt: u32, max_retries: u32) {
    let retryable = is_retryable(err);

    match retryable {
        Retryable::YesWithBackoff => {
            eprintln!(
                "{}",
                format!("Rate limited (attempt {}/{})", attempt, max_retries).yellow()
            );
            eprintln!("Waiting before retry...");
        }
        Retryable::Yes => {
            eprintln!(
                "{}",
                format!("Transient error (attempt {}/{})", attempt, max_retries).yellow()
            );
            eprintln!("Retrying...");
        }
        Retryable::No => {
            print_api_error(err);
        }
    }
}

/// Helper to check API key and print helpful message if missing
pub fn check_api_key(api_key: &Option<String>) -> Option<&String> {
    match api_key {
        Some(key) if !key.is_empty() => Some(key),
        _ => {
            eprintln!("{}", "Error: API key is required".red());
            eprintln!();
            eprintln!("{}", "Set your API key using:".yellow());
            eprintln!("  export ELEVENLABS_API_KEY=your_api_key");
            eprintln!();
            eprintln!("Or use the --api-key flag:");
            eprintln!("  elevenlabs --api-key YOUR_API_KEY <command>");
            eprintln!();
            eprintln!("Get your API key from: https://elevenlabs.io/app/settings/api-keys");
            None
        }
    }
}

/// Calculate backoff delay with exponential backoff and jitter
pub fn calculate_backoff(attempt: u32, retryable: Retryable) -> Duration {
    let base_delay = match retryable {
        Retryable::YesWithBackoff => {
            // Longer delay for rate limits
            INITIAL_BACKOFF_MS * 4
        }
        Retryable::Yes => {
            // Exponential backoff: 1s, 2s, 4s, ...
            INITIAL_BACKOFF_MS * (2u64.pow(attempt - 1))
        }
        Retryable::No => {
            return Duration::ZERO;
        }
    };

    // Cap at max delay
    let delay = base_delay.min(MAX_BACKOFF_MS);

    // Add jitter (random 0-25% of delay)
    let jitter = (rand_simple() as u64) * delay / 4000;
    Duration::from_millis(delay + jitter)
}

/// Simple pseudo-random number generator (for jitter)
fn rand_simple() -> u32 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos();
    nanos.wrapping_mul(1103515245).wrapping_add(12345)
}

/// Retry a closure with exponential backoff for transient errors
///
/// # Arguments
/// * `max_retries` - Maximum number of retry attempts
/// * `operation` - The async operation to retry
///
/// # Returns
/// The result of the operation if successful, or the last error
pub async fn with_retry<T, F, Fut>(max_retries: u32, operation: F) -> anyhow::Result<T>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = anyhow::Result<T>>,
{
    let mut last_error: Option<anyhow::Error> = None;

    for attempt in 1..=max_retries {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                last_error = Some(e);

                let retryable = is_retryable(last_error.as_ref().unwrap());

                if retryable == Retryable::No || attempt == max_retries {
                    // Don't retry non-retryable errors or we've exhausted retries
                    break;
                }

                // Print retry info
                print_retry_error(last_error.as_ref().unwrap(), attempt, max_retries);

                // Calculate and wait
                let delay = calculate_backoff(attempt, retryable);
                tokio::time::sleep(delay).await;
            }
        }
    }

    Err(last_error.unwrap_or_else(|| anyhow::anyhow!("Unknown error")))
}

/// Print feature availability based on subscription tier
pub fn print_subscription_info(tier: &str) {
    println!("\n{}", "Your Subscription:".bold().underline());
    println!("  Tier: {}", tier.yellow());
    println!();

    println!("{}", "Feature Availability:".bold());

    let features = match tier.to_lowercase().as_str() {
        "free" => vec![
            ("TTS (Basic)", "✓", Some("Limited voices")),
            ("STT", "✓", Some("Limited usage")),
            ("Voice Library", "✓", Some("Browse only")),
            (
                "Professional Voice Cloning",
                "✗",
                Some("Upgrade to Starter"),
            ),
            ("Agents", "✗", Some("Upgrade to Starter")),
            ("Phone Numbers", "✗", Some("Upgrade to Starter")),
            ("Custom Pronunciations", "✗", Some("Upgrade to Starter")),
        ],
        "starter" => vec![
            ("TTS (All Voices)", "✓", None),
            ("STT", "✓", None),
            ("Voice Library", "✓", None),
            ("Professional Voice Cloning", "✓", None),
            ("Voice Fine-tuning", "✓", None),
            ("Agents", "✗", Some("Upgrade to Creator")),
            ("Phone Numbers", "✗", Some("Upgrade to Creator")),
            ("Custom Pronunciations", "✗", Some("Upgrade to Creator")),
        ],
        "creator" | "pro" | "business" => vec![
            ("All TTS Features", "✓", None),
            ("All STT Features", "✓", None),
            ("Voice Library", "✓", None),
            ("Professional Voice Cloning", "✓", None),
            ("Voice Fine-tuning", "✓", None),
            ("Agents", "✓", None),
            ("Phone Numbers", "✓", None),
            ("Custom Pronunciations", "✓", None),
            ("Priority Support", "✓", None),
        ],
        _ => vec![
            ("TTS", "?", Some("Check your dashboard")),
            ("STT", "?", Some("Check your dashboard")),
            ("Voice Library", "?", Some("Check your dashboard")),
            ("Agents", "?", Some("Check your dashboard")),
        ],
    };

    for (feature, status, note) in features {
        let status_colored = if status == "✓" {
            status.green()
        } else if status == "✗" {
            status.red()
        } else {
            status.yellow()
        };

        if let Some(n) = note {
            println!("  {} {} - {}", status_colored, feature, n.dimmed());
        } else {
            println!("  {} {}", status_colored, feature);
        }
    }

    println!();
    println!("{}", "View full feature comparison:".dimmed());
    println!("  https://elevenlabs.io/pricing");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auth_problem_detects_key_id() {
        let e = anyhow::Error::msg(
            r#"http error: {"detail":{"code":"invalid_api_key","status":"api_key_id_used_as_api_key"}}"#,
        );
        assert_eq!(auth_problem(&e), Some(AuthProblem::KeyIdUsedAsKey));
    }

    #[test]
    fn auth_problem_extracts_missing_permission() {
        let e = anyhow::Error::msg(
            r#"http error: {"detail":{"message":"The API key you used is missing the permission voices_read to execute this operation.","status":"missing_permissions","type":"authentication_error"}}"#,
        );
        assert_eq!(
            auth_problem(&e),
            Some(AuthProblem::MissingPermission(Some("voices_read".into())))
        );
    }

    #[test]
    fn auth_problem_ignores_other_errors() {
        let e = anyhow::Error::msg(
            r#"http error: {"detail":{"code":"voice_not_found","type":"not_found"}}"#,
        );
        assert_eq!(auth_problem(&e), None);
    }
}
