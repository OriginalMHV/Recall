# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- OpenAI Codex CLI provider (`~/.codex/sessions/`)

## [0.1.0] - 2026-03-24

### Added
- TUI session browser with split-panel layout (session list + preview)
- GitHub Copilot CLI provider (`~/.copilot/session-state/`)
- Claude Code provider (`~/.claude/projects/`)
- Provider filtering with Tab key (All / Copilot / Claude Code)
- Fuzzy search across summaries, directories, and message content
- Session preview with checkpoints, messages, and completed tasks
- Resume sessions directly from TUI (Enter key)
- Delete sessions with confirmation dialog
- CLI mode: `--list`, `--count`, `--provider` flags
- Provider badges with color coding in session list
- Scroll preview with Shift+Arrow or Left/Right keys
- Distribution via crates.io, Homebrew, shell installer, and GitHub Releases
