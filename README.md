<div align="center">

<img src="https://capsule-render.vercel.app/api?type=waving&color=0:2E4057,50:048A81,100:54C6EB&height=200&text=RECALL&fontSize=80&fontColor=E8F1F2&fontAlignY=35&desc=find%20your%20way%20back.&descAlignY=55&descSize=22&descAlign=50&animation=fadeIn" width="100%" alt="Recall" />

[![Rust](https://img.shields.io/badge/Rust-048A81?style=for-the-badge&logo=rust&logoColor=E8F1F2&labelColor=1C1C1C)](https://rust-lang.org)
[![License](https://img.shields.io/badge/License-MIT-2E4057?style=for-the-badge&labelColor=1C1C1C)](LICENSE)

</div>

---

## What is Recall?

A TUI session browser for [GitHub Copilot CLI](https://github.com/github/copilot-cli). Browse, search, preview, and resume your past sessions without memorizing UUIDs or scrolling through `/resume`.

Think of it as the conversation sidebar that ChatGPT and Claude have — but for your terminal.

### Features

| Feature | Description |
|---------|-------------|
| **Session list** | All sessions sorted by last activity, with summary and age |
| **Live preview** | Checkpoints, user messages, completed tasks — at a glance |
| **Search** | Fuzzy filter across summaries, directories, and message content |
| **Resume** | Press Enter to launch `copilot --resume <id>` directly |
| **Delete** | Clean up old sessions with confirmation |
| **CLI mode** | `--list` and `--count` flags for scripting |

## Install

```bash
# from source
git clone https://github.com/OriginalMHV/recall.git
cd recall && cargo install --path .
```

Requires Rust >= 1.85 and an existing [Copilot CLI](https://github.com/github/copilot-cli) installation.

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
| `g` / `G` | Jump to first / last session |
| `q` / `Esc` | Quit |

### Search mode

| Key | Action |
|-----|--------|
| Type | Filter sessions |
| `Enter` / `Esc` | Exit search |
| `Ctrl+U` | Clear search |

## How it works

Recall reads directly from `~/.copilot/session-state/` — the same directory Copilot CLI uses to persist sessions. It parses `workspace.yaml` for metadata (summary, timestamps, working directory) and `events.jsonl` for conversation content (user messages, task completions). Checkpoint history is pulled from `checkpoints/index.md`.

Nothing is modified unless you explicitly delete a session. Recall is read-only by default.

When you hit Enter on a session, Recall exits cleanly and launches `copilot --resume <session-id>`, handing control back to the Copilot CLI.

## License

MIT. See [LICENSE](LICENSE).

<img src="https://capsule-render.vercel.app/api?type=waving&color=0:54C6EB,50:048A81,100:2E4057&height=120&section=footer&reversal=true" width="100%" alt="" />
