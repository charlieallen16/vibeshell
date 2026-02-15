//! MCP (Model Context Protocol) Server Module
//!
//! This module implements an MCP server that exposes VibeShell's SSH/SFTP
//! functionality as tools for AI assistants.
//!
//! Two transport modes are supported:
//! - **HTTP** (`server.rs`) — axum-based JSON-RPC server on a local port
//! - **Stdio** (`stdio.rs`) — stdin/stdout JSON-RPC for AI tool integration (Claude Code, Codex, Cursor, etc.)

pub mod server;
pub mod stdio;
pub mod tools;

pub use server::McpServer;
