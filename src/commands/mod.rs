use clap::{Parser, Subcommand};
use anyhow::Result;

pub mod create;
pub mod run;
pub mod doctor;
pub mod schema;

#[derive(Parser)]
#[command(name = "ludock")]
#[command(about = "A headless Roblox-like runtime for AI agents")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Create a new project
    Create {
        /// Name of the project
        name: String,
    },
    /// Run analysis and render
    Run {
        /// Enable 3D rendering
        #[arg(long = "3d")]
        render: bool,

        /// Relaxed mode: Skip strict analysis checks, warn only
        #[arg(long = "relaxed")]
        relaxed: bool,

        /// Specific instance to render (optional path)
        #[arg(long = "target")]
        target: Option<String>,

        /// Enable diff mode (compare against previous results)
        #[arg(long = "diff")]
        diff: bool,

        /// Draw bounding boxes in render
        #[arg(long = "debug-bounds")]
        debug_bounds: bool,

        /// Draw origin point in render
        #[arg(long = "debug-origin")]
        debug_origin: bool,

        /// Draw axes in render
        #[arg(long = "debug-axes")]
        debug_axes: bool,

        /// Execution preset (agent, ci, debug)
        #[arg(long = "preset")]
        preset: Option<String>,
    },
    /// Check environment status
    Doctor,
    /// Generate JSON schemas
    Schema,
}

pub fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Create { name } => {
            create::create_project(name)?;
        }
        Commands::Run { 
            render, 
            relaxed, 
            target, 
            diff, 
            debug_bounds, 
            debug_origin, 
            debug_axes,
            preset,
        } => {
            let mut opts = run::RunOptions {
                render: *render,
                relaxed: *relaxed,
                target: target.clone(),
                diff: *diff,
                debug_bounds: *debug_bounds,
                debug_origin: *debug_origin,
                debug_axes: *debug_axes,
            };
            
            if let Some(p) = preset {
                run::apply_preset(&mut opts, p)?;
            }

            run::run_project(opts)?;
        }
        Commands::Doctor => {
            doctor::check_environment()?;
        }
        Commands::Schema => {
            schema::generate_schemas()?;
        }
    }

    Ok(())
}
