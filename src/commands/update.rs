use crate::cli::UpdateArgs;
use anyhow::{bail, Result};

/// Self-update is disabled in this fork.
///
/// The upstream implementation downloaded release archives from
/// hongkongkiwi/elevenlabs-cli and replaced the running binary without
/// checksum or signature verification. Rebuild from source instead.
pub async fn execute(_args: UpdateArgs) -> Result<()> {
    bail!(
        "self-update is disabled in this fork; update with:\n  \
         git -C ~/Development/elevenlabs-cli pull && \
         cargo install --path ~/Development/elevenlabs-cli --locked"
    )
}
