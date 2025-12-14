use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use crate::core::datamodel::{Instance, Vec3Wrapper};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
pub struct DiffReport {
    pub schema_version: String,
    pub status: String, // "changed", "unchanged"
    pub changes: DiffChanges,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug, Default)]
pub struct DiffChanges {
    pub added_instances: Vec<String>, // List of full_paths
    pub removed_instances: Vec<String>, // List of full_paths
    pub modified_instances: Vec<InstanceDiff>,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
pub struct InstanceDiff {
    pub path: String,
    pub property_changes: HashMap<String, PropertyChange>,
    pub spatial_change: Option<SpatialChange>,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
pub struct PropertyChange {
    pub old: String, // Stringified for simplicity in diff
    pub new: String,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
pub struct SpatialChange {
    pub old_center: Option<Vec3Wrapper>,
    pub new_center: Option<Vec3Wrapper>,
    pub displacement: f32,
}

pub fn compare_worlds(old: &Instance, new: &Instance) -> DiffReport {
    let mut report = DiffReport {
        schema_version: "1.0".to_string(),
        status: "unchanged".to_string(),
        changes: DiffChanges::default(),
    };

    let mut old_map = HashMap::new();
    flatten_instance(old, &mut old_map);

    let mut new_map = HashMap::new();
    flatten_instance(new, &mut new_map);

    // Detect Added
    for (path, _) in &new_map {
        if !old_map.contains_key(path) {
            report.changes.added_instances.push(path.clone());
        }
    }

    // Detect Removed
    for (path, _) in &old_map {
        if !new_map.contains_key(path) {
            report.changes.removed_instances.push(path.clone());
        }
    }

    // Detect Modifications
    for (path, new_inst) in &new_map {
        if let Some(old_inst) = old_map.get(path) {
            let mut diff = InstanceDiff {
                path: path.clone(),
                property_changes: HashMap::new(),
                spatial_change: None,
            };

            // Compare Properties
            for (k, new_v) in &new_inst.properties {
                if let Some(old_v) = old_inst.properties.get(k) {
                    if new_v != old_v {
                        diff.property_changes.insert(k.clone(), PropertyChange {
                            old: format!("{:?}", old_v),
                            new: format!("{:?}", new_v),
                        });
                    }
                } else {
                    // Property Added (treated as change from None)
                    diff.property_changes.insert(k.clone(), PropertyChange {
                        old: "null".to_string(),
                        new: format!("{:?}", new_v),
                    });
                }
            }

            // Check Spatial
            let old_c = &old_inst.center;
            let new_c = &new_inst.center;
            
            if old_c != new_c {
                let dist = if let (Some(o), Some(n)) = (old_c, new_c) {
                    let v_o = glam::Vec3::new(o.x, o.y, o.z);
                    let v_n = glam::Vec3::new(n.x, n.y, n.z);
                    v_o.distance(v_n)
                } else {
                    0.0
                };
                
                if dist > 0.001 { // Epsilon
                    diff.spatial_change = Some(SpatialChange {
                        old_center: old_c.clone(),
                        new_center: new_c.clone(),
                        displacement: dist,
                    });
                }
            }

            if !diff.property_changes.is_empty() || diff.spatial_change.is_some() {
                report.changes.modified_instances.push(diff);
            }
        }
    }

    if !report.changes.added_instances.is_empty() || !report.changes.removed_instances.is_empty() || !report.changes.modified_instances.is_empty() {
        report.status = "changed".to_string();
    }

    report
}

fn flatten_instance<'a>(root: &'a Instance, map: &mut HashMap<String, &'a Instance>) {
    map.insert(root.full_path.clone(), root);
    for child in &root.children {
        flatten_instance(child, map);
    }
}
