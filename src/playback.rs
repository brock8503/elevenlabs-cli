//! Play audio where the user is.
//!
//! When the CLI runs inside tmux and the user's most recently active tmux
//! client came in over ssh or mosh, the file is copied to that client's
//! machine with `scp` and played there. Otherwise it plays locally.

use anyhow::{bail, Context, Result};
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

/// Players tried in order, locally and on the remote.
const PLAYERS: &[(&str, &[&str])] = &[
    ("afplay", &[]),
    ("mpv", &["--no-video", "--really-quiet"]),
    ("ffplay", &["-nodisp", "-autoexit", "-loglevel", "quiet"]),
    ("paplay", &[]),
];

const REMOTE_SCRIPT: &str = r#"f="$1"
if command -v afplay >/dev/null 2>&1; then afplay "$f"
elif command -v mpv >/dev/null 2>&1; then mpv --no-video --really-quiet "$f"
elif command -v ffplay >/dev/null 2>&1; then ffplay -nodisp -autoexit -loglevel quiet "$f"
elif command -v paplay >/dev/null 2>&1; then paplay "$f"
else echo "no audio player found (afplay, mpv, ffplay, paplay)" >&2; rm -f "$f"; exit 1; fi
rm -f "$f"
"#;

#[derive(Debug, Default, Clone)]
pub struct PlayOptions {
    /// Always play on this machine.
    pub local: bool,
    /// Play on this host, skipping detection.
    pub remote_host: Option<String>,
    /// ssh user on the remote host (defaults to ssh config, then $USER).
    pub remote_user: Option<String>,
}

pub fn play_file(path: &Path, opts: &PlayOptions) -> Result<()> {
    if !opts.local {
        if let Some(host) = opts.remote_host.clone().or_else(detect_remote_host) {
            let target = match &opts.remote_user {
                Some(user) => format!("{user}@{host}"),
                None => host,
            };
            eprintln!("Remote session detected, playing on {target}");
            match play_remote(path, &target) {
                Ok(()) => return Ok(()),
                Err(e) => eprintln!(
                    "Remote playback on {target} failed: {e:#}\n\
                     hint: this machine needs key-based ssh to {target} (enable Remote Login \
                     there and add this machine's public key to its ~/.ssh/authorized_keys). \
                     Set the username with `elevenlabs-cli config set remote_user <name>`, \
                     or pass --local.\n\
                     Falling back to local playback."
                ),
            }
        }
    }
    play_local(path)
}

fn play_local(path: &Path) -> Result<()> {
    for (player, args) in PLAYERS {
        match Command::new(player).args(*args).arg(path).status() {
            Ok(status) if status.success() => return Ok(()),
            Ok(status) => bail!("{player} exited with {status}"),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
            Err(e) => return Err(e).with_context(|| format!("failed to run {player}")),
        }
    }
    let bytes = std::fs::read(path)?;
    elevenlabs_rs::utils::play(bytes.into()).map_err(|e| anyhow::anyhow!(e))
}

fn play_remote(path: &Path, target: &str) -> Result<()> {
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .context("audio path has no file name")?;
    // scp may or may not pass the path through a remote shell, so keep it
    // free of characters that would need quoting.
    let safe: String = name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || "._-".contains(c) {
                c
            } else {
                '_'
            }
        })
        .collect();
    let remote_file = format!("/tmp/elevenlabs-cli-{safe}");
    let ssh_opts = ["-o", "BatchMode=yes", "-o", "ConnectTimeout=5"];

    let scp = Command::new("scp")
        .arg("-q")
        .args(ssh_opts)
        .arg(path)
        .arg(format!("{target}:{remote_file}"))
        .status()
        .context("failed to run scp")?;
    if !scp.success() {
        bail!("scp exited with {scp}");
    }

    let mut ssh = Command::new("ssh")
        .args(ssh_opts)
        .arg(target)
        .args(["sh", "-s", "--", &remote_file])
        .stdin(Stdio::piped())
        .spawn()
        .context("failed to run ssh")?;
    ssh.stdin
        .take()
        .context("ssh stdin unavailable")?
        .write_all(REMOTE_SCRIPT.as_bytes())?;
    let status = ssh.wait()?;
    if !status.success() {
        bail!("remote playback exited with {status}");
    }
    Ok(())
}

/// Find the address of the machine the user is connected from, if any.
fn detect_remote_host() -> Option<String> {
    if std::env::var_os("TMUX").is_some() {
        return detect_from_tmux();
    }
    std::env::var("SSH_CONNECTION")
        .ok()
        .and_then(|v| v.split_whitespace().next().map(str::to_string))
}

/// tmux panes keep a stale environment, so look at the client the user is
/// actually typing in: the most recently active one attached to this session.
fn detect_from_tmux() -> Option<String> {
    let session = run("tmux", &["display", "-p", "#{session_name}"])?;
    let clients = run(
        "tmux",
        &[
            "list-clients",
            "-t",
            session.trim(),
            "-F",
            "#{client_activity} #{client_pid}",
        ],
    )?;
    let pid = most_active_client(&clients)?;
    if !has_remote_ancestor(pid) {
        return None;
    }
    client_ssh_connection(pid)
}

fn has_remote_ancestor(mut pid: u32) -> bool {
    while pid > 1 {
        let Some(out) = run("ps", &["-o", "ppid=,comm=", "-p", &pid.to_string()]) else {
            return false;
        };
        let line = out.trim();
        let (ppid, comm) = line.split_once(char::is_whitespace).unwrap_or((line, ""));
        if is_remote_server(comm) {
            return true;
        }
        match ppid.trim().parse() {
            Ok(p) => pid = p,
            Err(_) => return false,
        }
    }
    false
}

/// mosh-server inherits SSH_CONNECTION from the ssh session that started it,
/// so this works for both ssh and mosh clients.
fn client_ssh_connection(pid: u32) -> Option<String> {
    if let Ok(environ) = std::fs::read(format!("/proc/{pid}/environ")) {
        return environ
            .split(|b| *b == 0)
            .filter_map(|kv| std::str::from_utf8(kv).ok())
            .find_map(ssh_connection_host);
    }
    // macOS: `ps -E` appends the environment to the command line.
    let out = run("ps", &["-E", "-p", &pid.to_string(), "-o", "command="])?;
    out.split_whitespace().find_map(ssh_connection_host)
}

fn ssh_connection_host(entry: &str) -> Option<String> {
    let value = entry.strip_prefix("SSH_CONNECTION=")?;
    value
        .split_whitespace()
        .next()
        .filter(|h| !h.is_empty())
        .map(str::to_string)
}

fn most_active_client(list: &str) -> Option<u32> {
    list.lines()
        .filter_map(|l| {
            let (activity, pid) = l.trim().split_once(' ')?;
            Some((activity.parse::<u64>().ok()?, pid.parse::<u32>().ok()?))
        })
        .max_by_key(|(activity, _)| *activity)
        .map(|(_, pid)| pid)
}

fn is_remote_server(comm: &str) -> bool {
    let base = comm.trim().rsplit('/').next().unwrap_or("");
    base.starts_with("mosh-server") || base.starts_with("sshd")
}

fn run(cmd: &str, args: &[&str]) -> Option<String> {
    let out = Command::new(cmd)
        .args(args)
        .stderr(Stdio::null())
        .output()
        .ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn picks_most_recently_active_client() {
        let list = "1793743520 67521\n1793750817 6820\n1793668720 12284\n";
        assert_eq!(most_active_client(list), Some(6820));
        assert_eq!(most_active_client(""), None);
    }

    #[test]
    fn parses_ssh_connection_host() {
        assert_eq!(
            ssh_connection_host("SSH_CONNECTION=100.71.0.75"),
            Some("100.71.0.75".into())
        );
        assert_eq!(
            ssh_connection_host("SSH_CONNECTION=10.0.0.2 52311 10.0.0.1 22"),
            Some("10.0.0.2".into())
        );
        assert_eq!(ssh_connection_host("SSH_CLIENT=10.0.0.2"), None);
        assert_eq!(ssh_connection_host("SSH_CONNECTION="), None);
    }

    #[test]
    fn recognizes_remote_servers() {
        assert!(is_remote_server("/opt/homebrew/bin/mosh-server"));
        assert!(is_remote_server("sshd-session"));
        assert!(is_remote_server("sshd"));
        assert!(!is_remote_server("/usr/bin/login"));
        assert!(!is_remote_server("-bash"));
    }
}
