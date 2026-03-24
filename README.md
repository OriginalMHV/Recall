<div align="center">

<img src="https://capsule-render.vercel.app/api?type=waving&color=0:2E4057,50:048A81,100:54C6EB&height=200&text=RECALL&fontSize=80&fontColor=E8F1F2&fontAlignY=35&desc=find%20your%20way%20back.&descAlignY=55&descSize=22&descAlign=50&animation=fadeIn" width="100%" alt="Recall" />

[![Rust](https://img.shields.io/badge/Rust-048A81?style=for-the-badge&logo=rust&logoColor=E8F1F2&labelColor=1C1C1C)](https://rust-lang.org)
[![License](https://img.shields.io/badge/License-MIT-2E4057?style=for-the-badge&labelColor=1C1C1C)](LICENSE)
[![CI](https://img.shields.io/github/actions/workflow/status/OriginalMHV/recall/ci.yml?style=for-the-badge&label=CI&labelColor=1C1C1C&color=048A81)](https://github.com/OriginalMHV/Recall/actions)
[![Crates.io](https://img.shields.io/crates/v/recall-cli?style=for-the-badge&labelColor=1C1C1C&color=048A81)](https://crates.io/crates/recall-cli)
[![Downloads](https://img.shields.io/crates/d/recall-cli?style=for-the-badge&label=Downloads&labelColor=1C1C1C&color=54C6EB)](https://crates.io/crates/recall-cli)

[![Lines of Code](https://img.shields.io/badge/Lines_of_Code-2.2k-E8F1F2?style=for-the-badge&labelColor=1C1C1C)](https://github.com/OriginalMHV/Recall)
[![Tests](https://img.shields.io/badge/Tests-40-048A81?style=for-the-badge&labelColor=1C1C1C)](https://github.com/OriginalMHV/Recall/actions)
[![Providers](https://img.shields.io/badge/Providers-3-54C6EB?style=for-the-badge&labelColor=1C1C1C)](https://github.com/OriginalMHV/Recall)
[![GitHub Release](https://img.shields.io/github/v/release/OriginalMHV/Recall?style=for-the-badge&labelColor=1C1C1C&color=2E4057)](https://github.com/OriginalMHV/Recall/releases/latest)

[Install](#install) · [Usage](#usage) · [Providers](#supported-providers) · [Keybindings](#keybindings)

</div>

---

## What is Recall?

A TUI session browser for [GitHub Copilot CLI](https://github.com/github/copilot-cli), [Claude Code](https://docs.anthropic.com/en/docs/claude-code), and [OpenAI Codex CLI](https://github.com/openai/codex). Browse, search, preview, and resume your past sessions without memorizing UUIDs or scrolling through `/resume`.

The conversation sidebar your terminal never had.

### Features

| Feature | Description |
|---------|-------------|
| **Multi-provider** | Supports Copilot CLI, Claude Code, and Codex CLI sessions in one unified view |
| **Session list** | All sessions sorted by last activity, with summary and age |
| **Live preview** | Checkpoints, user messages, completed tasks at a glance |
| **Search** | Fuzzy filter across summaries, directories, and message content |
| **Resume** | Press Enter to launch `copilot --resume <id>` or `codex --resume <id>` directly |
| **Delete** | Clean up old sessions with confirmation |
| **CLI mode** | `--list` and `--count` flags for scripting |

### Supported providers

| Provider | Status | Session location |
|----------|--------|------------------|
| GitHub Copilot CLI | ✅ Supported | `~/.copilot/session-state/` |
| Claude Code | ✅ Supported | `~/.claude/projects/` |
| OpenAI Codex CLI | ✅ Supported | `~/.codex/sessions/` |

## Install

```bash
# Homebrew (macOS/Linux)
brew tap OriginalMHV/tap
brew install recall

# Cargo
cargo install recall-cli

# Shell (macOS/Linux)
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/OriginalMHV/recall/releases/latest/download/recall-cli-installer.sh | sh

# From source
git clone https://github.com/OriginalMHV/recall.git
cd recall && cargo install --path .
```

Requires Rust >= 1.85 for building from source.

## Usage

```bash
# launch the TUI
recall

# list sessions as plain text
recall --list

# get session count
recall --count
```

### Keybindings

| Key | Action |
|-----|--------|
| `↑` `↓` / `j` `k` | Navigate sessions |
| `Enter` | Resume selected session |
| `/` | Search sessions |
| `d` | Delete session (with confirmation) |
| `Shift+↑↓` / `←` `→` | Scroll preview panel |
| `Tab` | Cycle provider filter (All → Copilot → Claude → Codex → All) |
| `g` / `G` | Jump to first / last session |
| `q` / `Esc` | Quit |

### Search mode

| Key | Action |
|-----|--------|
| Type | Filter sessions |
| `Enter` / `Esc` | Exit search |
| `Ctrl+U` | Clear search |

## How it works

Recall uses a provider-based architecture to discover sessions from multiple CLI tools:

- **Copilot CLI**: reads from `~/.copilot/session-state/`, parsing `workspace.yaml` for metadata and `events.jsonl` for conversation content. Checkpoint history is pulled from `checkpoints/index.md`.
- **Claude Code**: reads from `~/.claude/projects/`, parsing JSONL conversation files for session metadata, messages, and tool usage.
- **Codex CLI**: reads from `~/.codex/sessions/` (and `~/.codex/archived_sessions/`), walking nested `YYYY/MM/DD` date directories for `rollout-*.jsonl` session files. Parses the session meta line for identity and subsequent response items for user messages.

Nothing is modified unless you explicitly delete a session. Recall is read-only by default.

When you hit Enter on a session, Recall exits cleanly and launches the appropriate CLI tool to resume the session.

## License

MIT. See [LICENSE](LICENSE).

<img src="https://capsule-render.vercel.app/api?type=waving&color=0:54C6EB,50:048A81,100:2E4057&height=120&section=footer&reversal=true" width="100%" alt="" />
