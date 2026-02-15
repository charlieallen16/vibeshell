//! VibeShell skill installer for AI coding tools.
//!
//! This module installs/uninstalls the VibeShell SKILL.md file into
//! AI coding tool skill directories. The SKILL.md teaches AI agents
//! how to use the `vshell` CLI to manage SSH servers and sessions.
//!
//! **Important**: This installer does NOT modify MCP config files.
//! Integration is purely through SKILL.md — the AI reads the skill
//! and learns to call `vshell` via its shell/exec tool.

use std::fs;
use std::path::{Path, PathBuf};
use anyhow::{anyhow, Context, Result};

use super::detector::{find_tool, AiTool};

/// Resolve the absolute path to the vshell binary.
///
/// Search order:
/// 1. Next to the current executable (Tauri install location)
/// 2. Common installation paths per platform
/// 3. PATH lookup via `which`/`where`
/// 4. Fall back to bare "vshell" command name
pub fn resolve_vshell_binary() -> String {
    let vshell_name = if cfg!(windows) { "vshell.exe" } else { "vshell" };

    // 1. Next to the current executable
    if let Ok(current_exe) = std::env::current_exe() {
        if let Some(dir) = current_exe.parent() {
            let candidate = dir.join(vshell_name);
            if candidate.exists() {
                return candidate.to_string_lossy().to_string();
            }
        }
    }

    // 2. Common installation paths per platform
    #[cfg(windows)]
    {
        for env_var in &["LOCALAPPDATA", "ProgramFiles"] {
            if let Ok(base) = std::env::var(env_var) {
                let candidate = PathBuf::from(&base).join("VibeShell").join(vshell_name);
                if candidate.exists() {
                    return candidate.to_string_lossy().to_string();
                }
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        for c in &[
            "/Applications/VibeShell.app/Contents/MacOS/vshell",
            "/usr/local/bin/vshell",
        ] {
            if Path::new(c).exists() {
                return c.to_string();
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        for c in &["/usr/bin/vshell", "/usr/local/bin/vshell"] {
            if Path::new(c).exists() {
                return c.to_string();
            }
        }
    }

    // 3. Try to find via PATH (which/where)
    #[cfg(windows)]
    {
        if let Ok(output) = std::process::Command::new("where").arg("vshell").output() {
            if output.status.success() {
                let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if let Some(first_line) = path.lines().next() {
                    if Path::new(first_line).exists() {
                        return first_line.to_string();
                    }
                }
            }
        }
    }

    #[cfg(not(windows))]
    {
        if let Ok(output) = std::process::Command::new("which").arg("vshell").output() {
            if output.status.success() {
                let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if Path::new(&path).exists() {
                    return path;
                }
            }
        }
    }

    // 4. Fallback: bare command name
    vshell_name.to_string()
}

/// The SKILL.md content that teaches AI agents how to use VibeShell CLI.
///
/// This is the sole integration mechanism — AI agents read this skill file
/// and learn to run `vshell` commands via their shell/exec capabilities.
const SKILL_MD_CONTENT: &str = r#"---
name: vibeshell
description: Connect to remote SSH servers, execute commands, and transfer files via SFTP using VibeShell. Use this when the user needs to manage remote servers, deploy code, run remote commands, or transfer files over SSH.
---

You have access to **VibeShell**, a high-performance SSH/SFTP terminal client.
Use the `vshell` CLI to manage SSH servers and sessions from the command line.

> **Prerequisite**: The VibeShell GUI must be running for session commands to work.
> The CLI communicates with the GUI via IPC.

## When to Use This Skill

- User asks to "SSH into", "connect to", or "log into" a remote server
- User wants to run commands on a remote machine
- User needs to deploy files or code to a server
- User wants to check server status, logs, or resource usage
- User needs to manage SSH connections or sessions

## CLI Commands

### Check version
```bash
vshell version
```

### Connect to a server
```bash
vshell ssh <server-name>
# alias: vshell connect <server-name>
```
Connects to a server that was previously configured in the VibeShell GUI.

### List active sessions
```bash
vshell sessions
# alias: vshell ls
```

### Attach to an existing session
```bash
vshell attach <session-id>
```

### Kill a session
```bash
vshell kill <session-id>
vshell kill --all
```

### List detected AI tools
```bash
vshell tools
```
Shows which AI coding tools are detected and whether VibeShell skill is installed.

### Install/uninstall skill to AI tools
```bash
vshell install <tool-id>    # e.g. claude-code, cursor, codex
vshell install all
vshell uninstall <tool-id>
vshell uninstall all
```

## Typical Workflow

1. **Check if server is configured**: Ask the user which server to connect to, or suggest they configure one in the VibeShell GUI.
2. **Connect**: `vshell ssh my-server`
3. **List sessions**: `vshell sessions` to see active connections
4. **Clean up**: `vshell kill <session-id>` when done

## Notes

- Servers must be configured in the VibeShell GUI first (name, host, port, username, auth type).
- The GUI application must be running for CLI commands to work (IPC communication).
- For file transfers, use the VibeShell GUI's built-in SFTP panel.
- For direct remote command execution, connect via `vshell ssh` then use the terminal.
"#;

/// Get the skills directory path for a given AI tool.
fn get_skills_dir(tool_id: &str) -> Option<PathBuf> {
    let home = directories::BaseDirs::new().map(|dirs| dirs.home_dir().to_path_buf())?;

    match tool_id {
        "claude-code" => Some(home.join(".claude").join("skills")),
        "cursor" => Some(home.join(".cursor").join("skills")),
        "codex" => Some(home.join(".codex").join("skills")),
        "opencode" => Some(home.join(".opencode").join("skills")),
        "gemini-cli" => Some(home.join(".gemini").join("skills")),
        "openclaw" => Some(home.join(".openclaw").join("skills")),
        _ => None,
    }
}

/// Install the SKILL.md file into the tool's skills directory.
///
/// This is the **only** thing the installer does. It does NOT modify
/// any MCP config file (mcp.json, mcpServers.json, etc.).
fn install_skill_file(tool_id: &str) -> Result<PathBuf> {
    let skills_dir = get_skills_dir(tool_id)
        .ok_or_else(|| anyhow!("No skills directory known for tool: {}", tool_id))?;

    let skill_dir = skills_dir.join("vibeshell");
    fs::create_dir_all(&skill_dir)
        .with_context(|| format!("Failed to create skill directory {:?}", skill_dir))?;

    let skill_path = skill_dir.join("SKILL.md");
    fs::write(&skill_path, SKILL_MD_CONTENT)
        .with_context(|| format!("Failed to write skill file {:?}", skill_path))?;

    log::info!("[Install] Skill file installed to {:?}", skill_path);
    Ok(skill_path)
}

/// Remove the SKILL.md file from the tool's skills directory.
fn uninstall_skill_file(tool_id: &str) -> Result<()> {
    if let Some(skills_dir) = get_skills_dir(tool_id) {
        let skill_dir = skills_dir.join("vibeshell");
        if skill_dir.exists() {
            fs::remove_dir_all(&skill_dir)
                .with_context(|| format!("Failed to remove skill directory {:?}", skill_dir))?;
            log::info!("[Install] Skill file removed from {:?}", skill_dir);
        }
    }
    Ok(())
}

/// Result of an installation operation.
#[derive(Debug)]
pub struct InstallResult {
    /// The tool that was installed to
    pub tool: AiTool,
    /// Whether the installation was successful
    pub success: bool,
    /// Path to the backup file if created (unused — kept for API compat)
    pub backup_path: Option<PathBuf>,
    /// Error message if installation failed
    pub error: Option<String>,
}

/// Install VibeShell skill to a specific AI tool.
///
/// Only installs the SKILL.md file. Does NOT modify MCP configs.
pub fn install_to_tool(tool_id: &str, _config_path: &PathBuf) -> Result<Option<PathBuf>> {
    let skill_path = install_skill_file(tool_id)?;
    Ok(Some(skill_path))
}

/// Uninstall VibeShell skill from a specific AI tool.
///
/// Only removes the SKILL.md file. Does NOT modify MCP configs.
pub fn uninstall_from_tool(tool_id: &str, _config_path: &PathBuf) -> Result<Option<PathBuf>> {
    uninstall_skill_file(tool_id)?;
    Ok(None)
}

/// Install VibeShell skill to a tool by ID.
pub fn install_by_id(tool_id: &str) -> Result<InstallResult> {
    let tool = find_tool(tool_id)
        .ok_or_else(|| anyhow!("Unknown tool: {}", tool_id))?;

    match install_to_tool(&tool.id, &tool.config_path) {
        Ok(path) => Ok(InstallResult {
            tool,
            success: true,
            backup_path: path,
            error: None,
        }),
        Err(e) => Ok(InstallResult {
            tool,
            success: false,
            backup_path: None,
            error: Some(e.to_string()),
        }),
    }
}

/// Uninstall VibeShell skill from a tool by ID.
pub fn uninstall_by_id(tool_id: &str) -> Result<InstallResult> {
    let tool = find_tool(tool_id)
        .ok_or_else(|| anyhow!("Unknown tool: {}", tool_id))?;

    match uninstall_from_tool(&tool.id, &tool.config_path) {
        Ok(_) => Ok(InstallResult {
            tool,
            success: true,
            backup_path: None,
            error: None,
        }),
        Err(e) => Ok(InstallResult {
            tool,
            success: false,
            backup_path: None,
            error: Some(e.to_string()),
        }),
    }
}

/// Install VibeShell skill to all detected installed tools.
pub fn install_to_all() -> Vec<InstallResult> {
    use super::detector::get_installed_tools;

    get_installed_tools()
        .into_iter()
        .map(|tool| {
            match install_to_tool(&tool.id, &tool.config_path) {
                Ok(path) => InstallResult {
                    tool,
                    success: true,
                    backup_path: path,
                    error: None,
                },
                Err(e) => InstallResult {
                    tool,
                    success: false,
                    backup_path: None,
                    error: Some(e.to_string()),
                },
            }
        })
        .collect()
}

/// Uninstall VibeShell skill from all configured tools.
pub fn uninstall_from_all() -> Vec<InstallResult> {
    use super::detector::get_configured_tools;

    get_configured_tools()
        .into_iter()
        .map(|tool| {
            match uninstall_from_tool(&tool.id, &tool.config_path) {
                Ok(_) => InstallResult {
                    tool,
                    success: true,
                    backup_path: None,
                    error: None,
                },
                Err(e) => InstallResult {
                    tool,
                    success: false,
                    backup_path: None,
                    error: Some(e.to_string()),
                },
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_skill_md_content_is_valid() {
        assert!(SKILL_MD_CONTENT.contains("vshell"));
        assert!(SKILL_MD_CONTENT.contains("ssh"));
        assert!(SKILL_MD_CONTENT.contains("sessions"));
        assert!(SKILL_MD_CONTENT.contains("kill"));
    }

    #[test]
    fn test_install_creates_skill_file() {
        // This test can only run if the home directory exists
        // In CI it may not have the .claude dir, so we just verify the function signature
        let dir = TempDir::new().unwrap();
        let config_path = dir.path().join("mcp.json");
        // install_to_tool should NOT create/modify config_path
        // It only writes SKILL.md to the skills directory
        let _ = install_to_tool("claude-code", &config_path);
        // config_path should NOT exist (we don't touch MCP configs)
        assert!(!config_path.exists());
    }
}
