mod commands;
mod core;

use anyhow::Result;

fn main() -> Result<()> {
    commands::main()
}
