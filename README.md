<div align="center">

<img src="app-icon.svg" width="128" height="128" alt="VibeShell Logo" />

# VibeShell

**The first SSH client built for AI agents. Let Claude Code, Codex, and other CLI agents manage your servers.**

[![Release](https://img.shields.io/github/v/release/veithly/vibeshell?style=flat-square&color=7aa2f7)](https://github.com/veithly/vibeshell/releases)
[![License](https://img.shields.io/badge/license-MIT-9ece6a?style=flat-square)](LICENSE)
[![CI](https://img.shields.io/github/actions/workflow/status/veithly/vibeshell/ci.yml?style=flat-square&label=CI)](https://github.com/veithly/vibeshell/actions)
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-bb9af7?style=flat-square)](#installation)

<br />

<sub>Built with [Tauri 2](https://v2.tauri.app) · [React 18](https://react.dev) · [Xterm.js](https://xtermjs.org) · [russh](https://github.com/warp-tech/russh)</sub>

---

[Why VibeShell](#why-vibeshell) · [Features](#features) · [AI Integration](#-ai-agent-integration) · [Installation](#installation) · [Development](#development)

</div>

<br />

## Why VibeShell

Most SSH clients were built in an era before AI. They have clunky interfaces, zero automation, and treat every session as a purely human activity.

**VibeShell is different.** It's a modern terminal with first-class support for AI coding agents — Claude Code, OpenAI Codex, Cursor, Windsurf, and any MCP-compatible tool. Through a built-in MCP server and auto-installable skill system, your AI agents can SSH into servers, execute commands, transfer files, and manage infrastructure — all while you watch in a beautiful UI or let it run autonomously.

> Think of it as the terminal client that both you *and* your AI actually enjoy using.

<br />

## Features

### 🤖 AI Agent Integration

The killer feature. VibeShell ships with a built-in **MCP (Model Context Protocol) server** and a **skill installer** that teaches AI agents how to use it.

| What your AI can do | MCP Tool |
|----------------------|----------|
| List & manage servers | `server_list`, `server_add`, `server_get` |
| Open SSH sessions | `session_create`, `session_attach` |
| Execute remote commands | `exec` |
| Browse & transfer files | `sftp_ls`, `sftp_upload`, `sftp_download` |
| Create directories, move/delete files | `sftp_mkdir`, `sftp_mv`, `sftp_rm` |

**One-click install** — VibeShell auto-detects Claude Code, Codex CLI, Cursor, Windsurf, and other MCP-compatible tools on your machine, then installs its skill (`SKILL.md` + MCP config) so your agent knows exactly how and when to use SSH.

```
You: "Deploy the latest build to production"
Agent: [Uses VibeShell] → connects to prod server → pulls code → restarts service → confirms deployment
```

### 🖥️ Terminal

- **Multi-tab sessions** — Connect to multiple servers simultaneously
- **WebGL rendering** — Buttery-smooth terminal via Xterm.js
- **Local shell** — Built-in local terminal alongside SSH sessions
- **Ghost text completions** — Fish-shell-style inline suggestions
- **Keyboard shortcuts** — `Ctrl+N` new server, `Ctrl+K` quick command, `Ctrl+W` close tab

### 🔐 SSH & Security

- **SSH tunneling** — Local forward, remote forward, and SOCKS5 dynamic forwarding
- **Jump host (ProxyJump)** — Connect through bastion/jump servers
- **SSH agent forwarding** — Forward local SSH agent to remote hosts
- **Host key verification** — TOFU with SHA-256 fingerprint management
- **Encrypted credential storage** — AES-encrypted passwords and key passphrases

### 📁 File Management

- **SFTP browser** — Built-in file manager with upload, download, preview, edit
- **Drag & drop** — Upload files by dragging into the SFTP panel
- **Compression** — Compress/extract archives on remote servers
- **File preview** — View text files, images, and code with syntax highlighting

### ⚡ Productivity

- **Command snippets** — Save, tag, and reuse frequently used commands
- **Session recording** — Record terminal sessions for playback and audit
- **Post-login commands** — Auto-execute commands after SSH connection
- **Quick command palette** — `Ctrl+K` to search and run anything
- **Server groups & tags** — Organize hundreds of servers effortlessly

### 🎨 Design

- **Tokyo Night theme** — A beautiful dark theme, not a generic UI
- **Internationalization** — English and 简体中文
- **Native performance** — Rust backend, ~8 MB binary, instant startup
- **Cross-platform** — Windows, macOS, Linux with native look and feel

<br />

## Installation

### Download

| Platform | Download |
|----------|----------|
| **Windows** x64 | [`.exe` installer](https://github.com/veithly/vibeshell/releases/latest) · [`.msi`](https://github.com/veithly/vibeshell/releases/latest) |
| **macOS** Apple Silicon | [`.dmg`](https://github.com/veithly/vibeshell/releases/latest) |
| **macOS** Intel | [`.dmg`](https://github.com/veithly/vibeshell/releases/latest) |
| **Linux** x64 | [`.deb`](https://github.com/veithly/vibeshell/releases/latest) · [`.AppImage`](https://github.com/veithly/vibeshell/releases/latest) · [`.rpm`](https://github.com/veithly/vibeshell/releases/latest) |

### Build from Source

```bash
# Prerequisites: Node.js 18+, Rust 1.70+, Tauri prerequisites
git clone https://github.com/veithly/vibeshell.git
cd vibeshell
npm install
npx tauri dev      # Development mode with hot reload
npx tauri build    # Build release binary
```

<br />

## Architecture

```
┌──────────────────────────────────────────────────────┐
│                     Frontend                          │
│     React 18 + Zustand + Xterm.js + Tailwind CSS     │
├──────────────────────────────────────────────────────┤
│                  Tauri IPC Bridge                     │
├──────────────────────────────────────────────────────┤
│                      Backend                          │
│         Rust + russh + rusqlite + tokio               │
│  ┌──────┐ ┌──────┐ ┌────────┐ ┌─────────┐ ┌─────┐  │
│  │ SSH  │ │ SFTP │ │ Tunnel │ │ Storage │ │ MCP │  │
│  └──────┘ └──────┘ └────────┘ └─────────┘ └─────┘  │
│                                              ▲       │
│  ┌──────────────────────────────────────────┐│       │
│  │        Skill Installer & Detector        ││       │
│  │  Claude Code · Codex · Cursor · ...      │┘       │
│  └──────────────────────────────────────────┘        │
└──────────────────────────────────────────────────────┘
```

- **Frontend** — React handles UI, Zustand manages state, Xterm.js renders terminals
- **Backend** — Rust provides native SSH/SFTP via `russh`, SQLite for persistence
- **MCP Server** — Exposes 16 tools for AI agents to manage servers, sessions, and files
- **Skill Installer** — Auto-detects AI tools and configures the VibeShell MCP skill

<br />

## Development

```bash
npx tauri dev                                      # Dev server with hot reload
npx tsc --noEmit                                   # Type-check frontend
cargo check --manifest-path src-tauri/Cargo.toml   # Check Rust backend
cargo test --manifest-path src-tauri/Cargo.toml    # Run Rust tests
```

### Project Structure

```
vibeshell/
├── src/                    # React frontend
│   ├── components/         # UI components (Terminal, SFTP, Settings, ...)
│   ├── stores/             # Zustand state stores
│   ├── i18n/               # Internationalization (en, zh)
│   └── lib/                # Utilities (tauri.ts, utils.ts)
├── src-tauri/              # Rust backend
│   └── src/
│       ├── commands/       # Tauri command handlers (60+ commands)
│       ├── ssh/            # SSH client & fingerprints (russh)
│       ├── sftp/           # SFTP operations
│       ├── tunnel/         # SSH tunneling engine
│       ├── mcp/            # MCP server & 16 tool definitions
│       ├── install/        # AI tool detector & skill installer
│       ├── storage/        # SQLite database & encrypted credentials
│       └── logging/        # Session recording
├── cli/                    # CLI companion tool
└── .github/workflows/      # CI + auto-release pipeline
```

<br />

## Contributing

Contributions are welcome!

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

<br />

## License

[MIT](LICENSE) — Made with care by [veithly](https://github.com/veithly).
