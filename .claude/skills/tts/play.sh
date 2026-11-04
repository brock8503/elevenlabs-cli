#!/usr/bin/env bash
# Play an audio file where the user is. If the most recently active tmux
# client for this session came in over ssh or mosh, copy the file to that
# client's machine and play it there; otherwise play it locally.
#
# Usage: play.sh <file> [--local]
# Env:   TTS_REMOTE_USER  ssh user on the remote machine (default: ssh config, else $USER)
#        TTS_REMOTE_HOST  force a remote host, skipping detection
set -euo pipefail

file="${1:?usage: play.sh <file> [--local]}"
[[ -f "$file" ]] || { echo "play.sh: no such file: $file" >&2; exit 1; }

play_local() {
  echo "play.sh: playing locally" >&2
  afplay "$file"
}

[[ "${2:-}" == "--local" ]] && { play_local; exit; }

# Walk a pid's ancestry and print "ssh" or "mosh" if a remote server is found.
remote_kind() {
  local pid="$1" comm
  while [[ -n "$pid" && "$pid" -gt 1 ]]; do
    comm="$(ps -o comm= -p "$pid" 2>/dev/null || true)"
    case "$comm" in
      *mosh-server*) echo mosh; return ;;
      *sshd*) echo ssh; return ;;
    esac
    pid="$(ps -o ppid= -p "$pid" 2>/dev/null | tr -d ' ')"
  done
}

detect_host() {
  [[ -n "${TTS_REMOTE_HOST:-}" ]] && { echo "$TTS_REMOTE_HOST"; return; }
  if [[ -n "${TMUX:-}" ]]; then
    local session client_pid kind
    session="$(tmux display -p '#{session_name}')"
    client_pid="$(tmux list-clients -t "$session" -F '#{client_activity} #{client_pid}' \
      | sort -rn | head -1 | cut -d' ' -f2)"
    [[ -n "$client_pid" ]] || return 0
    kind="$(remote_kind "$client_pid")"
    [[ -n "$kind" ]] || return 0
    # The client's environment carries SSH_CONNECTION (mosh-server inherits it too).
    ps -E -p "$client_pid" -o command= | tr ' ' '\n' \
      | sed -n 's/^SSH_CONNECTION=//p' | head -1
  elif [[ -n "${SSH_CONNECTION:-}" ]]; then
    echo "${SSH_CONNECTION%% *}"
  fi
}

host="$(detect_host)"
[[ -n "$host" ]] || { play_local; exit; }

target="$host"
[[ -n "${TTS_REMOTE_USER:-}" ]] && target="$TTS_REMOTE_USER@$host"
ssh_opts=(-o BatchMode=yes -o ConnectTimeout=5)
remote_file="/tmp/tts-$(basename "$file")"

echo "play.sh: remote session detected, playing on $target" >&2
if ! scp -q "${ssh_opts[@]}" "$file" "$target:$remote_file"; then
  echo "play.sh: cannot reach $target over ssh (needs key auth; set TTS_REMOTE_USER if the username differs). Falling back to local." >&2
  play_local
  exit
fi
ssh "${ssh_opts[@]}" "$target" sh -s -- "$remote_file" <<'REMOTE'
f="$1"
if command -v afplay >/dev/null; then afplay "$f"
elif command -v mpv >/dev/null; then mpv --no-video --really-quiet "$f"
elif command -v ffplay >/dev/null; then ffplay -nodisp -autoexit -loglevel quiet "$f"
elif command -v paplay >/dev/null; then paplay "$f"
else echo "no audio player on remote" >&2; exit 1; fi
rm -f "$f"
REMOTE
