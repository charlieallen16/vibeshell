//! AI tool detection for VibeShell skill installation.
//!
//! This module detects installed AI coding tools and checks whether
//! the VibeShell SKILL.md is installed in each tool's skills directory.
//!
//! Detection is based on SKILL.md presence only — we do NOT check
//! MCP config files (mcp.json, etc.).

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Represents an AI coding tool that can have the VibeShell skill installed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiTool {
    /// Unique identifier for the tool (e.g., "claude-code", "cursor")
    pub id: String,
    /// Human-readable name of the tool
    pub name: String,
    /// Path to the tool's configuration file (kept for API compat)
    pub config_path: PathBuf,
    /// Whether the AI tool is detected/installed on the system
    pub installed: bool,
    /// Whether the VibeShell SKILL.md is installed in the tool
    pub vibeshell_installed: bool,
}

impl AiTool {
    /// Create a new AiTool instance.
    ///
    /// Detection:
    /// - `installed`: true if any of the candidate directories exist
    /// - `vibeshell_installed`: true if SKILL.md exists in the skills dir
    fn from_candidates(id: &str, name: &str, candidates: Vec<PathBuf>) -> Self {
        let config_path = select_preferred_config_path(&candidates)
            .unwrap_or_else(|| PathBuf::from("mcp.json"));

        let installed = candidates
            .iter()
            .any(|path| path.parent().map(|p| p.exists()).unwrap_or(false));

        let vibeshell_installed = check_skill_installed(id);

        Self {
            id: id.to_string(),
            name: name.to_string(),
            config_path,
            installed,
            vibeshell_installed,
        }
    }
}

/// Get the user's home directory.
fn get_home_dir() -> Option<PathBuf> {
    directories::BaseDirs::new().map(|dirs| dirs.home_dir().to_path_buf())
}

/// Select preferred config path from candidates.
fn select_preferred_config_path(candidates: &[PathBuf]) -> Option<PathBuf> {
    if let Some(existing) = candidates.iter().find(|path| path.exists()) {
        return Some(existing.clone());
    }

    candidates.first().cloned()
}

/// Check if the VibeShell SKILL.md is installed for the given tool.
///
/// Only checks the skills directory — does NOT inspect MCP configs.
fn check_skill_installed(tool_id: &str) -> bool {
    let home = match get_home_dir() {
        Some(h) => h,
        None => return false,
    };

    let skills_dir = match tool_id {
        "claude-code" => home.join(".claude").join("skills"),
        "cursor" => home.join(".cursor").join("skills"),
        "codex" => home.join(".codex").join("skills"),
        "opencode" => home.join(".opencode").join("skills"),
        "gemini-cli" => home.join(".gemini").join("skills"),
        "openclaw" => home.join(".openclaw").join("skills"),
        _ => return false,
    };

    skills_dir.join("vibeshell").join("SKILL.md").exists()
}

/// Detect all supported AI tools and their skill installation status.
///
/// Returns a list of all known AI tools with their current status,
/// including whether they are installed and whether the VibeShell skill is installed.
pub fn detect_ai_tools() -> Vec<AiTool> {
    let mut tools = Vec::new();

    if let Some(home) = get_home_dir() {
        // Claude Code: multi-candidate paths (existing path first, fallback to first)
        tools.push(AiTool::from_candidates(
            "claude-code",
            "Claude Code",
            vec![
                home.join(".claude").join("mcp.json"),
                home.join(".claude.json"),
                home.join(".config").join("claude").join("mcp.json"),
                home.join(".config").join("claude-code").join("mcp.json"),
            ],
        ));

        // Cursor: support both mcp.json and mcpServers.json variants
        tools.push(AiTool::from_candidates(
            "cursor",
            "Cursor",
            vec![
                home.join(".cursor").join("mcp.json"),
                home.join(".cursor").join("mcpServers.json"),
                home.join(".config").join("Cursor").join("User").join("mcp.json"),
                home.join(".config").join("Cursor").join("User").join("mcpServers.json"),
            ],
        ));

        // Codex: support both config.json and mcp.json variants
        tools.push(AiTool::from_candidates(
            "codex",
            "Codex",
            vec![
                home.join(".codex").join("config.json"),
                home.join(".codex").join("mcp.json"),
                home.join(".config").join("codex").join("config.json"),
                home.join(".config").join("codex").join("mcp.json"),
            ],
        ));

        // Open Code: legacy path
        tools.push(AiTool::from_candidates(
            "opencode",
            "Open Code",
            vec![home.join(".opencode").join("mcp.json")],
        ));

        // Gemini CLI: Google's Gemini CLI uses settings.json with mcpServers
        tools.push(AiTool::from_candidates(
            "gemini-cli",
            "Gemini CLI",
            vec![
                home.join(".gemini").join("settings.json"),
                home.join(".config").join("gemini-cli").join("settings.json"),
            ],
        ));

        // OpenClaw: AI agent gateway with MCP support via openclaw.json
        tools.push(AiTool::from_candidates(
            "openclaw",
            "OpenClaw",
            vec![
                home.join(".openclaw").join("openclaw.json"),
                home.join(".config").join("openclaw").join("openclaw.json"),
            ],
        ));
    }

    tools
}

/// Find a specific AI tool by its ID.
pub fn find_tool(tool_id: &str) -> Option<AiTool> {
    detect_ai_tools().into_iter().find(|t| t.id == tool_id)
}

/// Get all installed AI tools (tools whose config directory exists).
pub fn get_installed_tools() -> Vec<AiTool> {
    detect_ai_tools()
        .into_iter()
        .filter(|t| t.installed)
        .collect()
}

/// Get all tools that have the VibeShell skill installed.
pub fn get_configured_tools() -> Vec<AiTool> {
    detect_ai_tools()
        .into_iter()
        .filter(|t| t.vibeshell_installed)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_detect_ai_tools() {
        let tools = detect_ai_tools();
        assert_eq!(tools.len(), 6);

        let tool_ids: Vec<&str> = tools.iter().map(|t| t.id.as_str()).collect();
        assert!(tool_ids.contains(&"claude-code"));
        assert!(tool_ids.contains(&"cursor"));
        assert!(tool_ids.contains(&"codex"));
        assert!(tool_ids.contains(&"opencode"));
        assert!(tool_ids.contains(&"gemini-cli"));
        assert!(tool_ids.contains(&"openclaw"));
    }

    #[test]
    fn test_find_tool() {
        let tool = find_tool("claude-code");
        assert!(tool.is_some());
        assert_eq!(tool.unwrap().name, "Claude Code");

        let unknown = find_tool("unknown-tool");
        assert!(unknown.is_none());
    }

    #[test]
    fn test_select_preferred_config_path_prefers_existing_file() {
        let dir = TempDir::new().unwrap();
        let missing = dir.path().join("missing.json");
        let existing = dir.path().join("existing.json");
        fs::write(&existing, "{}").unwrap();

        let selected = select_preferred_config_path(&[missing, existing.clone()]);
        assert_eq!(selected, Some(existing));
    }

    #[test]
    fn test_select_preferred_config_path_falls_back_to_default() {
        let dir = TempDir::new().unwrap();
        let default_path = dir.path().join("default.json");
        let second = dir.path().join("second.json");

        let selected = select_preferred_config_path(&[default_path.clone(), second]);
        assert_eq!(selected, Some(default_path));
    }
}
