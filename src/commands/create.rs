use anyhow::{Context, Result};
use serde_json::json;
use std::fs;
use std::path::Path;

pub fn create_project(name: &str) -> Result<()> {
    let root = Path::new(name);

    if root.exists() {
        return Err(anyhow::anyhow!("Directory '{}' already exists", name));
    }

    // Create directories
    let dirs = vec![
        "game/Workspace",
        "game/Lighting",
        "game/ReplicatedFirst",
        "game/ReplicatedStorage",
        "game/ServerScriptService",
        "game/ServerStorage",
        "game/StarterGui",
        "game/StarterPack",
        "game/StarterPlayer/StarterPlayerScripts",
        "game/StarterPlayer/StarterCharacterScripts",
        "game/SoundService",
        "results",
    ];

    for dir in dirs {
        fs::create_dir_all(root.join(dir))
            .with_context(|| format!("Failed to create directory {}", dir))?;
    }

    // Create plugins directory
    let plugins_dir = root.join(".ludock/plugins");
    fs::create_dir_all(&plugins_dir).with_context(|| "Failed to create plugins directory")?;
    
    // Create placeholder manifest
    let manifest = json!({
        "manifestVersion": "1.0",
        "plugins": []
    });
    fs::write(plugins_dir.join("manifest.json"), serde_json::to_string_pretty(&manifest)?)?;

    // Create ludock.json
    let config = json!({
        "name": name,
        "version": "0.1.0",
        "created_at": chrono::Utc::now().to_rfc3339()
    });

    let config_path = root.join("ludock.json");
    fs::write(&config_path, serde_json::to_string_pretty(&config)?)
        .with_context(|| "Failed to write ludock.json")?;

    println!("Created LuDock project: {}", name);
    Ok(())
}
