<div align="center">

# recall

**Find your way back.**

[![crates.io](https://img.shields.io/crates/v/recall-cli?style=flat-square&color=048A81)](https://crates.io/crates/recall-cli)
[![license](https://img.shields.io/crates/l/recall-cli?style=flat-square&color=2E4057)](LICENSE)
[![ci](https://img.shields.io/github/actions/workflow/status/OriginalMHV/recall/ci.yml?style=flat-square&label=ci)](https://github.com/OriginalMHV/recall/actions)

A terminal session browser for [Copilot CLI](https://github.com/github/copilot-cli), [Claude Code](https://docs.anthropic.com/en/docs/claude-code), and [Codex CLI](https://github.com/openai/codex).

</div>

---

Browse, search, preview, and resume your past coding sessions from one place.
No more memorizing UUIDs or scrolling through `/resume`.

## Providers

| Provider | Session location |
|----------|------------------|
| GitHub Copilot CLI | `~/.copilot/session-state/` |
| Claude Code | `~/.claude/projects/` |
| OpenAI Codex CLI | `~/.codex/sessions/` |

## Install

**Homebrew**
```bash
brew tap OriginalMHV/tap && brew install recall
```

**Cargo**
```bash
cargo install recall-cli
```

**Shell script**
```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/OriginalMHV/recall/releases/latest/download/recall-cli-installer.sh | sh
```

**From source**
```bash
git clone https://github.com/OriginalMHV/recall.git
cd recall && cargo install --path .
```

Building from source requires Rust 1.85+.

## Usage

```bash
recall              # launch the TUI
recall --list       # plain text session list
recall --count      # session count
recall --provider copilot   # filter by provider
```

## Keybindings

| Key | Action |
|-----|--------|
| `j` / `k` or arrows | Navigate sessions |
| `Enter` | Resume selected session |
| `/` | Search |
| `d` | Delete (with confirmation) |
| `Tab` | Cycle provider filter |
| `Shift+arrows` or `Left`/`Right` | Scroll preview |
| `g` / `G` | Jump to first / last |
| `q` / `Esc` | Quit |

In search mode, type to filter. `Enter` or `Esc` to exit, `Ctrl+U` to clear.

## How it works

Recall discovers sessions through a provider-based architecture. Each provider reads from its tool's local storage:

- **Copilot CLI** parses `workspace.yaml` for metadata and `events.jsonl` for conversation content. Checkpoint history comes from `checkpoints/index.md`.
- **Claude Code** parses JSONL conversation files under project directories for messages and metadata.
- **Codex CLI** walks `YYYY/MM/DD` date directories for `rollout-*.jsonl` session files, reading the session meta line and user messages from response items.

Nothing is modified unless you explicitly delete a session. Recall is read-only by default.

Pressing Enter exits the TUI and launches the appropriate CLI tool (`copilot --resume` or `codex --resume`) to pick up where you left off.

## License

MIT
