use crate::core::datamodel::{Instance, PropertyValue};
use crate::core::parser::parse_instance_dsl;
use anyhow::Result;
use std::fs;
use std::path::Path;

pub fn load_project(root_path: &Path) -> Result<Instance> {
    let game_path = root_path.join("game");
    if !game_path.exists() {
        return Err(anyhow::anyhow!(
            "Game directory not found at {:?}",
            game_path
        ));
    }

    // Root is "game" (DataModel)
    let mut datamodel = Instance::new("DataModel", "DataModel", "game");
    datamodel.full_path = "game".to_string();

    // We need to reconstruct the hierarchy.
    // The strategy is to walk the directory, create instances for files and folders.
    // However, "folders" in FS might just be organizational or represent a parent.
    // The prompt says: "GameName/game/Workspace/..."

    // We will build a temporary map of Path -> Instance to link parents and children.
    // But since `Instance` owns its children, we might need a recursive approach or a multi-pass approach.
    // Recursive approach is cleaner for directory walking.

    datamodel.children = load_directory(&game_path, "game")?;

    // Post-process derived data (AABB, Center) for Root
    // Actually, AABB for root should encompass all children.
    // We can do a recursive pass or compute it during load.
    // Let's compute it during load by returning the AABB from load_directory or computing it here.
    // For simplicity, we can do a bottom-up pass now.
    compute_derived_data(&mut datamodel);

    Ok(datamodel)
}

fn load_directory(dir: &Path, parent_full_path: &str) -> Result<Vec<Instance>> {
    let mut children = Vec::new();

    // Read directory entries and collect them into a vector
    let mut entries = fs::read_dir(dir)?
        .collect::<Result<Vec<_>, std::io::Error>>()?;

    // Sort entries by file name to ensure deterministic order
    entries.sort_by_key(|entry| entry.file_name());

    for entry in entries {
        let path = entry.path();
        let name = path.file_stem().unwrap().to_string_lossy().to_string();
        // Create a stable path identifier string for UUID generation
        // using forward slashes for cross-platform consistency
        let path_str = path.to_string_lossy().replace('\\', "/");

        if path.is_dir() {
            let class_name = infer_class_from_name(&name, true);
            let clean_name = clean_name(&name);
            let current_full_path = format!("{}/{}", parent_full_path, clean_name);

            let mut instance = Instance::new(&clean_name, &class_name, &path_str);
            instance.full_path = current_full_path.clone();
            instance.children = load_directory(&path, &current_full_path)?;
            children.push(instance);
        } else {
            // It's a file
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

            // Skip non-instance files (like .DS_Store)
            if ext == "json" || ext.is_empty() {
                continue;
            }

            // Handle Scripts
            if name.ends_with(".server") && ext == "lua" {
                let clean_name = name.trim_end_matches(".server").to_string();
                let mut inst = Instance::new(&clean_name, "Script", &path_str);
                inst.properties.insert(
                    "Source".to_string(),
                    PropertyValue::String(fs::read_to_string(&path)?),
                );
                children.push(inst);
                continue;
            }
            if name.ends_with(".local") && ext == "lua" {
                let clean_name = name.trim_end_matches(".local").to_string();
                let mut inst = Instance::new(&clean_name, "LocalScript", &path_str);
                inst.properties.insert(
                    "Source".to_string(),
                    PropertyValue::String(fs::read_to_string(&path)?),
                );
                children.push(inst);
                continue;
            }
            if name.ends_with(".module") && ext == "lua" {
                let clean_name = name.trim_end_matches(".module").to_string();
                let mut inst = Instance::new(&clean_name, "ModuleScript", &path_str);
                inst.properties.insert(
                    "Source".to_string(),
                    PropertyValue::String(fs::read_to_string(&path)?),
                );
                children.push(inst);
                continue;
            }

            // Handle Declarative Instances
            let class_name = map_extension_to_class(ext);
            let mut instance = Instance::new(&name, &class_name, &path_str);

            // Parse DSL
            let content = fs::read_to_string(&path)?;
            match parse_instance_dsl(&content) {
                Ok((_, props)) => {
                    // Override properties from file
                    for (k, v) in props {
                        // If file specifies ClassName, use it
                        if k == "ClassName" 
                            && let PropertyValue::String(ref s) = v 
                        {
                            instance.class_name = s.clone();
                        }
                        // If file specifies Name, use it
                        if k == "Name" 
                            && let PropertyValue::String(ref s) = v 
                        {
                            instance.name = s.clone();
                        }
                        instance.properties.insert(k, v);
                    }
                }
                Err(e) => {
                    eprintln!("Failed to parse DSL for {:?}: {}", path, e);
                }
            }
            // Update full path after Name might have changed? 
            // Name is property. But instance.name is struct field.
            // We updated instance.name above if found.
            instance.full_path = format!("{}/{}", parent_full_path, instance.name);

            children.push(instance);
        }
    }

    Ok(children)
}

use crate::core::datamodel::{AabbWrapper, Vec3Wrapper};
use glam::{Mat4, Vec3};

fn compute_derived_data(instance: &mut Instance) -> Option<AabbWrapper> {
    // 1. Compute bounds for self if BasePart
    let mut my_min = Vec3::splat(f32::INFINITY);
    let mut my_max = Vec3::splat(f32::NEG_INFINITY);
    let mut has_bounds = false;

    if instance.class_name == "Part" || instance.class_name == "BasePart" {
         // Get Size and CFrame
         let size = if let Some(PropertyValue::Vector3(v)) = instance.properties.get("Size") {
             Vec3::new(v.x, v.y, v.z)
         } else {
             Vec3::new(4.0, 1.0, 2.0)
         };
         
         let transform = if let Some(PropertyValue::CFrame(cf)) = instance.properties.get("CFrame") {
             let c = &cf.components;
             let col0 = glam::Vec4::new(c[3], c[6], c[9], 0.0);
             let col1 = glam::Vec4::new(c[4], c[7], c[10], 0.0);
             let col2 = glam::Vec4::new(c[5], c[8], c[11], 0.0);
             let col3 = glam::Vec4::new(c[0], c[1], c[2], 1.0);
             Mat4::from_cols(col0, col1, col2, col3)
         } else if let Some(PropertyValue::Vector3(pos)) = instance.properties.get("Position") {
             Mat4::from_translation(Vec3::new(pos.x, pos.y, pos.z))
         } else {
             Mat4::IDENTITY
         };

        let half_size = size * 0.5;
        let corners = [
            Vec3::new(-half_size.x, -half_size.y, -half_size.z),
            Vec3::new(half_size.x, -half_size.y, -half_size.z),
            Vec3::new(-half_size.x, half_size.y, -half_size.z),
            Vec3::new(half_size.x, half_size.y, -half_size.z),
            Vec3::new(-half_size.x, -half_size.y, half_size.z),
            Vec3::new(half_size.x, -half_size.y, half_size.z),
            Vec3::new(-half_size.x, half_size.y, half_size.z),
            Vec3::new(half_size.x, half_size.y, half_size.z),
        ];

        for corner in corners {
            let p = transform.transform_point3(corner);
            my_min = my_min.min(p);
            my_max = my_max.max(p);
        }
        has_bounds = true;
    }

    // 2. Aggregate children bounds
    for child in &mut instance.children {
        if let Some(child_bounds) = compute_derived_data(child) {
            let min = Vec3::new(child_bounds.min.x, child_bounds.min.y, child_bounds.min.z);
            let max = Vec3::new(child_bounds.max.x, child_bounds.max.y, child_bounds.max.z);
            my_min = my_min.min(min);
            my_max = my_max.max(max);
            has_bounds = true;
        }
    }

    if has_bounds {
        let center = (my_min + my_max) * 0.5;
        instance.center = Some(Vec3Wrapper { x: center.x, y: center.y, z: center.z });
        let bounds = AabbWrapper {
             min: Vec3Wrapper { x: my_min.x, y: my_min.y, z: my_min.z },
             max: Vec3Wrapper { x: my_max.x, y: my_max.y, z: my_max.z }
        };
        instance.world_bounds = Some(bounds.clone());
        Some(bounds)
    } else {
        None
    }
}

fn infer_class_from_name(name: &str, is_dir: bool) -> String {
    // If name contains dot, maybe it's "cat.model" -> Model
    if let Some(idx) = name.rfind('.') {
        let ext = &name[idx + 1..];
        let class = map_extension_to_class(ext);
        if class != "Folder" {
            // If map returns default, maybe check known services
            return class;
        }
    }

    // Known Services (Direct children of game usually)
    match name {
        "Workspace" => "Workspace".to_string(),
        "Lighting" => "Lighting".to_string(),
        "ReplicatedStorage" => "ReplicatedStorage".to_string(),
        "ReplicatedFirst" => "ReplicatedFirst".to_string(),
        "ServerScriptService" => "ServerScriptService".to_string(),
        "ServerStorage" => "ServerStorage".to_string(),
        "StarterGui" => "StarterGui".to_string(),
        "StarterPack" => "StarterPack".to_string(),
        "StarterPlayer" => "StarterPlayer".to_string(),
        "SoundService" => "SoundService".to_string(),
        _ => {
            if is_dir {
                "Folder".to_string()
            } else {
                "Unknown".to_string()
            }
        }
    }
}

fn clean_name(name: &str) -> String {
    if let Some(idx) = name.rfind('.') {
        // e.g. "cat.model" -> "cat"
        // But "Workspace" -> "Workspace"
        // Check if the suffix is a known class extension-like thing
        let ext = &name[idx + 1..];
        if map_extension_to_class(ext) != "Folder" {
            return name[..idx].to_string();
        }
    }
    name.to_string()
}

fn map_extension_to_class(ext: &str) -> String {
    match ext {
        "basepart" => "Part".to_string(), // Default BasePart is Part
        "part" => "Part".to_string(),
        "model" => "Model".to_string(),
        "folder" => "Folder".to_string(),
        "script" => "Script".to_string(),
        "localscript" => "LocalScript".to_string(),
        "modulescript" => "ModuleScript".to_string(),
        "gui" => "ScreenGui".to_string(),
        "frame" => "Frame".to_string(),
        "button" => "TextButton".to_string(),
        "label" => "TextLabel".to_string(),
        _ => "Folder".to_string(), // Default for directory, or fallback
    }
}
