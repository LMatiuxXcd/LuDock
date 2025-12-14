use anyhow::Result;
use schemars::schema_for;
use crate::core::datamodel::Instance;
use crate::core::analysis::DiagnosticsReport;
use crate::core::diff::DiffReport;
use std::fs;
use std::path::Path;

pub fn generate_schemas() -> Result<()> {
    let schema_dir = Path::new("schemas");
    if !schema_dir.exists() {
        fs::create_dir_all(schema_dir)?;
    }

    // World Schema
    let world_schema = schema_for!(Instance);
    fs::write(
        schema_dir.join("world.schema.json"),
        serde_json::to_string_pretty(&world_schema)?,
    )?;

    // Diagnostics Schema
    let diag_schema = schema_for!(DiagnosticsReport);
    fs::write(
        schema_dir.join("diagnostics.schema.json"),
        serde_json::to_string_pretty(&diag_schema)?,
    )?;

    // Diff Schema
    let diff_schema = schema_for!(DiffReport);
    fs::write(
        schema_dir.join("diff.schema.json"),
        serde_json::to_string_pretty(&diff_schema)?,
    )?;

    println!("Schemas generated in `schemas/`");
    Ok(())
}
