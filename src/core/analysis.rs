use anyhow::Result;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command;
use walkdir::WalkDir;

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct Diagnostic {
    pub file: String,
    pub line: usize,
    pub message: String,
    pub severity: String, // "error" or "warning"
    #[serde(default)]
    pub code: Option<String>, // e.g., "UnknownProperty"
    #[serde(default)]
    pub hint: Option<String>, // e.g., "Did you mean Size?"
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct DiagnosticsReport {
    pub errors: Vec<Diagnostic>,
    #[serde(default = "default_schema_version")]
    pub schema_version: String,
}

fn default_schema_version() -> String {
    "1.0".to_string()
}

pub fn run_analysis(root_path: &Path, skip_analysis: bool) -> Result<DiagnosticsReport> {
    if skip_analysis {
        return Ok(DiagnosticsReport { errors: Vec::new(), schema_version: "1.0".to_string() });
    }

    // Check if luau-analyze exists
    let analyze_cmd = if Command::new("luau-analyze")
        .arg("--version")
        .output()
        .is_ok()
    {
        "luau-analyze".to_string()
    } else if Path::new("./luau-analyze").exists() {
        "./luau-analyze".to_string()
    } else if Path::new("./luau-analyze.exe").exists() {
        "./luau-analyze.exe".to_string()
    } else {
        return Err(anyhow::anyhow!("`luau-analyze` not found in PATH or current directory. Install it or use --relaxed to skip."));
    };

    // Find all Lua files
    let mut lua_files = Vec::new();
    let game_path = root_path.join("game");

    if game_path.exists() {
        for entry in WalkDir::new(game_path) {
            let entry = entry?;
            if entry.path().extension().is_some_and(|e| e == "lua") {
                lua_files.push(entry.path().to_owned());
            }
        }
    }

    let mut diagnostics = Vec::new();

    // Check if luau-analyze is available
    // For this environment, we might not have it installed. We will try to run it.
    // If it fails, we report a warning but don't crash.
    // In a real agent environment, this tool would be pre-installed.

    // We run luau-analyze on each file individually for now
    for file in lua_files {
        let output = Command::new(&analyze_cmd).arg(&file).output()?; // Propagate error if execution fails strangely, though we checked existence

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr); // luau-analyze usually prints to stdout/stderr

        // Parse output
        // Format usually: "file.lua:10:5: TypeError: ..."
        // or just human readable.
        // We'll try to parse basic "File:Line: Msg" format

        let combined = format!("{}\n{}", stdout, stderr);
        for line in combined.lines() {
            if let Some(diag) = parse_luau_line(line, file.to_str().unwrap_or("unknown")) {
                diagnostics.push(diag);
            }
        }
    }

    Ok(DiagnosticsReport {
        errors: diagnostics,
        schema_version: "1.0".to_string(),
    })
}

fn parse_luau_line(line: &str, filepath: &str) -> Option<Diagnostic> {
    // Example: "path/to/script.lua:5:16: TypeError: ..."
    // Or sometimes just "path/to/script.lua:5: TypeError: ..."

    // Simple heuristic: look for first colon after the filepath suffix
    // Since filepath might be absolute or relative, checking if line starts with it is tricky if output format differs.
    // But usually it starts with the filename passed as arg.

    let parts: Vec<&str> = line.splitn(4, ':').collect();
    if parts.len() >= 3 {
        // parts[0] is file (mostly)
        // parts[1] is line
        // parts[2] is col (maybe) or message start

        let line_num = parts[1].trim().parse::<usize>().ok()?;

        // Check if parts[2] is a number (column)
        let message_start_idx = if parts[2].trim().parse::<usize>().is_ok() {
            3
        } else {
            2
        };

        if parts.len() > message_start_idx {
            let raw_message = parts[message_start_idx..].join(":").trim().to_string();

            // Try to infer code/hint from message (naive heuristic)
            // e.g. "Key 'Szie' not found in class 'Part'. Did you mean 'Size'?"
            let mut code = None;
            let mut hint = None;

            if raw_message.contains("not found in class") {
                code = Some("UnknownProperty".to_string());
            } else if raw_message.contains("Type mismatch") {
                code = Some("TypeMismatch".to_string());
            }

            if let Some(idx) = raw_message.find("Did you mean") {
                let h = raw_message[idx..].to_string();
                hint = Some(
                    h.trim_matches(|c| c == '\'' || c == '"' || c == '?' || c == '.')
                        .to_string(),
                );
            }

            return Some(Diagnostic {
                file: filepath.to_string(),
                line: line_num,
                message: raw_message,
                severity: "error".to_string(), // Assume error mostly
                code,
                hint,
            });
        }
    }

    None
}
