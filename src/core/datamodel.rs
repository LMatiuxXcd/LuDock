use glam::Vec3;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Instance {
    pub id: Uuid,
    pub name: String,
    pub class_name: String,
    pub properties: HashMap<String, PropertyValue>,
    pub children: Vec<Instance>,
    
    // Computed fields (enriched DataModel)
    #[serde(default)]
    pub full_path: String,
    #[serde(default)]
    pub world_bounds: Option<AabbWrapper>,
    #[serde(default)]
    pub center: Option<Vec3Wrapper>,
}

impl Instance {
    // Generate deterministic UUID based on the path
    pub fn new(name: &str, class_name: &str, path_hash_seed: &str) -> Self {
        let namespace = Uuid::NAMESPACE_OID;
        let id = Uuid::new_v5(&namespace, path_hash_seed.as_bytes());

        Instance {
            id,
            name: name.to_string(),
            class_name: class_name.to_string(),
            properties: HashMap::new(),
            children: Vec::new(),
            full_path: String::new(),
            world_bounds: None,
            center: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
#[serde(untagged)]
pub enum PropertyValue {
    String(String),
    Bool(bool),
    Number(f64),
    Vector3(Vec3Wrapper),
    CFrame(CFrameWrapper),
    Color3(Color3Wrapper),
    UDim2(UDim2Wrapper),
    Enum(String), // e.g. "Enum.PartType.Block"
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
pub struct AabbWrapper {
    pub min: Vec3Wrapper,
    pub max: Vec3Wrapper,
}

// Wrapper structs to implement Serde for external types or custom formatting

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
pub struct Vec3Wrapper {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl From<Vec3> for Vec3Wrapper {
    fn from(v: Vec3) -> Self {
        Vec3Wrapper {
            x: v.x,
            y: v.y,
            z: v.z,
        }
    }
}

impl From<Vec3Wrapper> for Vec3 {
    fn from(v: Vec3Wrapper) -> Self {
        Vec3::new(v.x, v.y, v.z)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
pub struct CFrameWrapper {
    pub position: Vec3Wrapper,
    // For now, simple position + identity rotation or basic lookAt could be stored.
    // Storing full matrix in a simple way.
    pub components: [f32; 12],
}

impl CFrameWrapper {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        CFrameWrapper {
            position: Vec3Wrapper { x, y, z },
            components: [x, y, z, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
pub struct Color3Wrapper {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl Color3Wrapper {
    pub fn from_rgb(r: f32, g: f32, b: f32) -> Self {
        Color3Wrapper {
            r: r / 255.0,
            g: g / 255.0,
            b: b / 255.0,
        }
    }
    pub fn new(r: f32, g: f32, b: f32) -> Self {
        Color3Wrapper { r, g, b }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
pub struct UDim2Wrapper {
    pub xs: f32,
    pub xo: i32,
    pub ys: f32,
    pub yo: i32,
}
