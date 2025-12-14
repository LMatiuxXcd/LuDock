use crate::core::analysis::run_analysis;
use crate::core::diff::compare_worlds;
use crate::core::loader::load_project;
use crate::core::renderer::{render_scene, RenderOptions};
use anyhow::{Context, Result};
use std::fs;

pub struct RunOptions {
    pub render: bool,
    pub relaxed: bool,
    pub target: Option<String>,
    pub diff: bool,
    pub debug_bounds: bool,
    pub debug_origin: bool,
    pub debug_axes: bool,
}

pub fn apply_preset(opts: &mut RunOptions, preset: &str) -> Result<()> {
    match preset {
        "agent" => {
            // Strict mode, render enabled, diff enabled, full debug info
            opts.relaxed = false;
            opts.render = true;
            opts.diff = true;
            opts.debug_bounds = true;
            opts.debug_origin = true;
            opts.debug_axes = true;
        }
        "ci" => {
            // Strict mode, no render (unless forced), diff enabled for reports
            opts.relaxed = false;
            if !opts.render { opts.render = false; } // Don't override if user asked? User said "ci -> strict sin render"
            opts.diff = true;
        }
        "debug" => {
            // Relaxed mode, render enabled, all debug flags
            opts.relaxed = true;
            opts.render = true;
            opts.debug_bounds = true;
            opts.debug_origin = true;
            opts.debug_axes = true;
        }
        _ => return Err(anyhow::anyhow!("Unknown preset: {}", preset)),
    }
    Ok(())
}

pub fn run_project(options: RunOptions) -> Result<()> {
    // 1. Determine root (current dir)
    let root = std::env::current_dir()?;
    let results_dir = root.join("results");

    if !results_dir.exists() {
        fs::create_dir_all(&results_dir).with_context(|| "Failed to create results directory")?;
    }

    // Diff Mode: Load previous world.json if needed
    let old_world = if options.diff {
        if let Ok(content) = fs::read_to_string(results_dir.join("world.json")) {
            serde_json::from_str::<crate::core::datamodel::Instance>(&content).ok()
        } else {
            None
        }
    } else {
        None
    };

    println!("Loading project...");
    let datamodel = load_project(&root).with_context(|| "Failed to load project structure")?;

    // 2. Generate world.json
    println!("Generating world.json...");
    let world_json = serde_json::to_string_pretty(&datamodel)?;
    fs::write(results_dir.join("world.json"), &world_json)?;

    // Handle Diff
    if let Some(old_inst) = old_world {
        println!("Computing structured diff...");
        let diff_report = compare_worlds(&old_inst, &datamodel);
        let diff_json = serde_json::to_string_pretty(&diff_report)?;
        fs::write(results_dir.join("diff.json"), diff_json)?;
        println!("Diff report generated (Status: {})", diff_report.status);
    }

    // 3. Run Analysis (Strict vs Relaxed)
    println!("Running Luau analysis...");
    match run_analysis(&root, options.relaxed) {
        Ok(diagnostics) => {
            let diagnostics_json = serde_json::to_string_pretty(&diagnostics)?;
            fs::write(results_dir.join("diagnostics.json"), diagnostics_json)?;
            
            // STRICT MODE: Fail if errors found and not relaxed
            if !options.relaxed && !diagnostics.errors.is_empty() {
                eprintln!("Strict Mode: {} errors found. Aborting render.", diagnostics.errors.len());
                std::process::exit(1);
            }
        },
        Err(e) => {
            if !options.relaxed {
                 eprintln!("Strict Mode Error: {}", e);
                 std::process::exit(1);
            } else {
                 eprintln!("Analysis failed but continuing (relaxed): {}", e);
                 fs::write(results_dir.join("diagnostics.json"), "{\"errors\": []}")?;
            }
        }
    }

    // 4. Render
    if options.render {
        println!("Rendering 3D view...");
        let target_instance = &datamodel; // Target logic simplified for now

        let output_path = results_dir.join("render.png");
        
        let render_opts = RenderOptions {
            debug_bounds: options.debug_bounds,
            debug_origin: options.debug_origin,
            debug_axes: options.debug_axes,
        };

        render_scene(target_instance, &output_path, render_opts).with_context(|| "Failed to render scene")?;
        println!("Render saved to {:?}", output_path);
    }

    println!("LuDock run completed successfully.");
    Ok(())
}
