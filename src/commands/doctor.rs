use std::process::Command;
use std::path::Path;
use anyhow::Result;

pub fn check_environment() -> Result<()> {
    println!("LuDock Doctor");
    println!("=============");

    // 1. LuDock Version
    println!("LuDock Version: {}", env!("CARGO_PKG_VERSION"));

    // 2. Check luau-analyze
    print!("luau-analyze: ");
    if check_command("luau-analyze") {
        println!("Found (PATH) ✅");
    } else if Path::new("./luau-analyze").exists() || Path::new("./luau-analyze.exe").exists() {
        println!("Found (Local) ✅");
    } else {
        println!("Not Found ❌");
        println!("  -> Tip: Install luau-analyze or place it in the project root.");
    }

    // 3. Renderer Backend
    println!("Renderer Backend: Software (CPU) ✅");

    // 4. Strict Mode Default
    println!("Strict Mode: Enabled by default (use --relaxed to disable) ✅");

    Ok(())
}

fn check_command(cmd: &str) -> bool {
    Command::new(cmd).arg("--version").output().is_ok()
}
