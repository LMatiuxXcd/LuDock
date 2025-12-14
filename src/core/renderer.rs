use crate::core::datamodel::{Instance, PropertyValue};
use anyhow::Result;
use glam::{Mat4, Vec3, Vec4};
use image::{ImageBuffer, Rgb, RgbImage};
use imageproc::drawing::draw_line_segment_mut;
use std::path::Path;

// Constants for render
const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;

pub struct RenderContext {
    buffer: RgbImage,
    z_buffer: Vec<f32>,
    width: u32,
    height: u32,
}

impl RenderContext {
    pub fn new(width: u32, height: u32) -> Self {
        RenderContext {
            buffer: ImageBuffer::new(width, height),
            z_buffer: vec![f32::INFINITY; (width * height) as usize],
            width,
            height,
        }
    }

    pub fn clear(&mut self, color: Rgb<u8>) {
        for pixel in self.buffer.pixels_mut() {
            *pixel = color;
        }
        for z in self.z_buffer.iter_mut() {
            *z = f32::INFINITY;
        }
    }

    pub fn draw_pixel(&mut self, x: u32, y: u32, z: f32, color: Rgb<u8>) {
        if x >= self.width || y >= self.height {
            return;
        }
        let idx = (y * self.width + x) as usize;
        if z < self.z_buffer[idx] {
            self.z_buffer[idx] = z;
            self.buffer.put_pixel(x, y, color);
        }
    }
}

// Simple bounding box for auto-framing
struct Aabb {
    min: Vec3,
    max: Vec3,
}

impl Aabb {
    fn empty() -> Self {
        Aabb {
            min: Vec3::splat(f32::INFINITY),
            max: Vec3::splat(f32::NEG_INFINITY),
        }
    }
    fn extend(&mut self, point: Vec3) {
        self.min = self.min.min(point);
        self.max = self.max.max(point);
    }
    fn center(&self) -> Vec3 {
        (self.min + self.max) * 0.5
    }
    fn size(&self) -> Vec3 {
        self.max - self.min
    }
}

pub struct RenderOptions {
    pub debug_bounds: bool,
    pub debug_origin: bool,
    pub debug_axes: bool,
}

pub fn render_scene(root: &Instance, output_path: &Path, options: RenderOptions) -> Result<()> {
    let mut ctx = RenderContext::new(WIDTH, HEIGHT);
    ctx.clear(Rgb([200, 230, 255])); // Sky blue background

    // 1. Collect all renderable parts (BaseParts)
    let mut parts = Vec::new();
    collect_parts(root, &mut parts, Mat4::IDENTITY);

    if parts.is_empty() {
        // println!("No 3D parts to render."); // Silence this to avoid user confusion if they only have UI
        // Or make it debug only?
        // User complained it says "No parts" but they exist. This implies they DO exist but weren't picked up.
        // I will keep it but add info.
        // println!("Info: No 3D parts found in DataModel.");
    }

    // 2. Calculate Scene AABB for Auto-Framing
    let mut aabb = Aabb::empty();
    for (transform, size, _, _) in &parts {
        // Compute the 8 corners of the OBB
        let half_size = *size * 0.5;
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
            let world_pos = transform.transform_point3(corner);
            aabb.extend(world_pos);
        }
    }

    // 3. Setup Camera
    let center = aabb.center();
    let size = aabb.size();
    let max_dim = size.max_element();
    // Distance needed to fit the object
    // FOV 70 deg
    let fov_y = 70.0_f32.to_radians();
    let distance = (max_dim / 2.0) / (fov_y / 2.0).tan();
    // Position camera diagonally
    let dir = Vec3::new(1.0, 0.8, 1.0).normalize();
    let eye = center + dir * (distance * 1.5 + 5.0); // Add margin
    let target = center;
    let up = Vec3::Y;

    let view = Mat4::look_at_rh(eye, target, up);
    let projection = Mat4::perspective_rh(fov_y, WIDTH as f32 / HEIGHT as f32, 0.1, 1000.0);
    let view_proj = projection * view;

    // 4. Rasterize Parts
    for (transform, size, color, shape) in &parts {
        match shape.as_str() {
            "Ball" => draw_sphere(&mut ctx, &view_proj, *transform, *size, *color),
            "Cylinder" => draw_cylinder(&mut ctx, &view_proj, *transform, *size, *color),
            _ => draw_cube(&mut ctx, &view_proj, *transform, *size, *color), // Default to Block/Cube
        }
    }

    // 5. Debug Visuals
    if options.debug_axes {
         draw_axes(&mut ctx, &view_proj, Mat4::IDENTITY, 5.0);
    }
    
    if options.debug_bounds {
        for (transform, size, _, _) in &parts {
             draw_wireframe_box(&mut ctx, &view_proj, *transform, *size, Rgb([255, 255, 0]));
        }
    }
    
    if options.debug_origin {
         // Draw a small cross at 0,0,0
         draw_wireframe_box(&mut ctx, &view_proj, Mat4::IDENTITY, Vec3::splat(0.5), Rgb([0, 0, 0]));
    }

    // 6. UI Overlay
    draw_ui_overlay(&mut ctx, root);

    ctx.buffer.save(output_path)?;
    Ok(())
}

fn draw_ui_overlay(ctx: &mut RenderContext, root: &Instance) {
    // Traverse for StarterGui -> ScreenGui -> Frame
    for child in &root.children {
        if child.class_name == "StarterGui" {
            for screen_gui in &child.children {
                if screen_gui.class_name == "ScreenGui" {
                    draw_gui_recursive(ctx, screen_gui, 0.0, 0.0, ctx.width as f32, ctx.height as f32);
                }
            }
        }
    }
}

fn draw_gui_recursive(ctx: &mut RenderContext, instance: &Instance, parent_x: f32, parent_y: f32, parent_w: f32, parent_h: f32) {
    let mut my_x = parent_x;
    let mut my_y = parent_y;
    let mut my_w = parent_w;
    let mut my_h = parent_h;

    if instance.class_name == "Frame" {
        // Position
        if let Some(PropertyValue::UDim2(pos)) = instance.properties.get("Position") {
            my_x = parent_x + (pos.xs * parent_w) + (pos.xo as f32);
            my_y = parent_y + (pos.ys * parent_h) + (pos.yo as f32);
        }
        
        // Size
        if let Some(PropertyValue::UDim2(size)) = instance.properties.get("Size") {
            my_w = (size.xs * parent_w) + (size.xo as f32);
            my_h = (size.ys * parent_h) + (size.yo as f32);
        }

        // AnchorPoint (Optional, assume 0,0 for now if missing)
        // BackgroundColor3
        let color = if let Some(PropertyValue::Color3(c)) = instance.properties.get("BackgroundColor3") {
            Rgb([(c.r * 255.0) as u8, (c.g * 255.0) as u8, (c.b * 255.0) as u8])
        } else {
            Rgb([255, 255, 255]) // Default white
        };

        // Draw Rect using imageproc
        let rect = imageproc::rect::Rect::at(my_x as i32, my_y as i32).of_size(my_w as u32, my_h as u32);
        imageproc::drawing::draw_filled_rect_mut(&mut ctx.buffer, rect, color);
    }

    for child in &instance.children {
        draw_gui_recursive(ctx, child, my_x, my_y, my_w, my_h);
    }
}

fn collect_parts(
    instance: &Instance,
    parts: &mut Vec<(Mat4, Vec3, Rgb<u8>, String)>,
    _parent_transform: Mat4,
) {
    let mut current_transform = Mat4::IDENTITY; // Placeholder if we don't find CFrame
    let mut found_cframe = false;

    if instance.class_name == "Part" || instance.class_name == "BasePart" {
        // Extract Size
        let size = if let Some(PropertyValue::Vector3(v)) = instance.properties.get("Size") {
            v.clone().into()
        } else {
            Vec3::new(4.0, 1.0, 2.0) // Default part size
        };

        // Extract Color
        let color = if let Some(PropertyValue::Color3(c)) = instance.properties.get("Color") {
            Rgb([
                (c.r * 255.0) as u8,
                (c.g * 255.0) as u8,
                (c.b * 255.0) as u8,
            ])
        } else {
            Rgb([163, 162, 165]) // Medium stone grey
        };

        // Extract Shape
        let shape = if let Some(PropertyValue::Enum(s)) = instance.properties.get("Shape") {
            // "Enum.PartType.Ball" -> "Ball"
            s.split('.').last().unwrap_or("Block").to_string()
        } else {
            "Block".to_string()
        };

        // Extract CFrame
        if let Some(PropertyValue::CFrame(cf)) = instance.properties.get("CFrame") {
            let c = &cf.components;
            let col0 = Vec4::new(c[3], c[6], c[9], 0.0);
            let col1 = Vec4::new(c[4], c[7], c[10], 0.0);
            let col2 = Vec4::new(c[5], c[8], c[11], 0.0);
            let col3 = Vec4::new(c[0], c[1], c[2], 1.0); // Translation

            current_transform = Mat4::from_cols(col0, col1, col2, col3);
            found_cframe = true;
        } else if let Some(PropertyValue::Vector3(pos)) = instance.properties.get("Position") {
            current_transform = Mat4::from_translation(pos.clone().into());
            found_cframe = true;
        } else {
            // Default CFrame if missing? 
            // If explicit CFrame is missing, check if it's just meant to be at 0,0,0
            // But usually DSL specifies it.
            // Let's print a warning if we found a Part but no position.
            // eprintln!("Warning: Part {} has no CFrame/Position", instance.name);
        }

        if found_cframe {
            parts.push((current_transform, size, color, shape));
        } else {
             // Try to render at identity if missing?
             // Or maybe user didn't specify CFrame? 
             // "CFrame = CFrame.new(0, 0, 0)" is common.
             // If I failed to parse it, `found_cframe` is false.
             // Debug print:
             // println!("Debug: Part {} skipped (no CFrame)", instance.name);
        }
    }

    // Recurse
    for child in &instance.children {
        collect_parts(child, parts, Mat4::IDENTITY);
    }
}

// Helper to draw a mesh
fn draw_mesh(ctx: &mut RenderContext, view_proj: &Mat4, model: Mat4, vertices: &[Vec3], indices: &[u32], color: Rgb<u8>) {
    let mvp = *view_proj * model;
    let clip_coords: Vec<Vec4> = vertices
        .iter()
        .map(|p| mvp * Vec4::new(p.x, p.y, p.z, 1.0))
        .collect();

    for i in 0..(indices.len() / 3) {
        let idx0 = indices[i * 3] as usize;
        let idx1 = indices[i * 3 + 1] as usize;
        let idx2 = indices[i * 3 + 2] as usize;

        rasterize_triangle(
            ctx,
            &clip_coords[idx0],
            &clip_coords[idx1],
            &clip_coords[idx2],
            color,
        );
    }
}

fn draw_cube(ctx: &mut RenderContext, view_proj: &Mat4, model: Mat4, size: Vec3, color: Rgb<u8>) {
    let half = size * 0.5;
    let corners = [
        Vec3::new(-half.x, -half.y, -half.z), // 0
        Vec3::new(half.x, -half.y, -half.z),  // 1
        Vec3::new(-half.x, half.y, -half.z),  // 2
        Vec3::new(half.x, half.y, -half.z),   // 3
        Vec3::new(-half.x, -half.y, half.z),  // 4
        Vec3::new(half.x, -half.y, half.z),   // 5
        Vec3::new(-half.x, half.y, half.z),   // 6
        Vec3::new(half.x, half.y, half.z),    // 7
    ];
    let indices = [
        4, 5, 7, 4, 7, 6, // Front
        1, 0, 2, 1, 2, 3, // Back
        0, 4, 6, 0, 6, 2, // Left
        5, 1, 3, 5, 3, 7, // Right
        6, 7, 3, 6, 3, 2, // Top
        0, 1, 5, 0, 5, 4, // Bottom
    ];
    draw_mesh(ctx, view_proj, model, &corners, &indices, color);
}

fn draw_sphere(ctx: &mut RenderContext, view_proj: &Mat4, model: Mat4, size: Vec3, color: Rgb<u8>) {
    // Generate sphere mesh (icosphere or UV sphere). Using simple UV sphere.
    let lat_segments = 12;
    let lon_segments = 12;
    let radius = size.min_element() * 0.5; // Approximation: use min dimension as radius

    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    for lat in 0..=lat_segments {
        let theta = lat as f32 * std::f32::consts::PI / lat_segments as f32;
        let sin_theta = theta.sin();
        let cos_theta = theta.cos();

        for lon in 0..=lon_segments {
            let phi = lon as f32 * 2.0 * std::f32::consts::PI / lon_segments as f32;
            let sin_phi = phi.sin();
            let cos_phi = phi.cos();

            let x = cos_phi * sin_theta;
            let y = cos_theta;
            let z = sin_phi * sin_theta;

            vertices.push(Vec3::new(x * radius, y * radius, z * radius));
        }
    }

    for lat in 0..lat_segments {
        for lon in 0..lon_segments {
            let first = (lat * (lon_segments + 1)) + lon;
            let second = first + lon_segments + 1;

            indices.push(first);
            indices.push(second);
            indices.push(first + 1);

            indices.push(second);
            indices.push(second + 1);
            indices.push(first + 1);
        }
    }
    
    draw_mesh(ctx, view_proj, model, &vertices, &indices, color);
}

fn draw_cylinder(ctx: &mut RenderContext, view_proj: &Mat4, model: Mat4, size: Vec3, color: Rgb<u8>) {
    let segments = 16;
    let radius = size.x.min(size.z) * 0.5; // X/Z determines radius usually
    let half_height = size.y * 0.5;

    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    // Side vertices
    for i in 0..=segments {
        let theta = i as f32 * 2.0 * std::f32::consts::PI / segments as f32;
        let x = theta.cos() * radius;
        let z = theta.sin() * radius;
        
        vertices.push(Vec3::new(x, -half_height, z)); // Bottom ring
        vertices.push(Vec3::new(x, half_height, z));  // Top ring
    }
    
    // Center points for caps
    let bottom_center_idx = vertices.len() as u32;
    vertices.push(Vec3::new(0.0, -half_height, 0.0));
    let top_center_idx = vertices.len() as u32;
    vertices.push(Vec3::new(0.0, half_height, 0.0));

    for i in 0..segments {
        let base = i * 2;
        let next_base = (i + 1) * 2;
        
        // Side quads (2 tris)
        indices.push(base);
        indices.push(base + 1);
        indices.push(next_base);
        
        indices.push(next_base);
        indices.push(base + 1);
        indices.push(next_base + 1);
        
        // Bottom Cap
        indices.push(bottom_center_idx);
        indices.push(next_base);
        indices.push(base);
        
        // Top Cap
        indices.push(top_center_idx);
        indices.push(base + 1);
        indices.push(next_base + 1);
    }

    // Reorient to align with Roblox Cylinder (which lies on X axis by default? No, usually Y up).
    // Actually Roblox Cylinder is usually along X axis. 
    // Wait, Roblox BasePart Cylinder: "The cylinder is aligned with the X-axis."
    // My generation above is along Y axis.
    // I need to rotate it 90 deg around Z.
    let rotation = Mat4::from_rotation_z(90.0_f32.to_radians());
    
    // We can apply this rotation to the vertices or the model matrix. 
    // Applying to vertices here for simplicity.
    let vertices: Vec<Vec3> = vertices.iter().map(|v| rotation.transform_point3(*v)).collect();

    draw_mesh(ctx, view_proj, model, &vertices, &indices, color);
}

fn rasterize_triangle(ctx: &mut RenderContext, v0: &Vec4, v1: &Vec4, v2: &Vec4, color: Rgb<u8>) {
    // Homogeneous divide
    if v0.w <= 0.0 || v1.w <= 0.0 || v2.w <= 0.0 {
        return;
    } // Very basic near plane clipping (discard)

    let p0 = ndc_to_screen(v0, ctx.width, ctx.height);
    let p1 = ndc_to_screen(v1, ctx.width, ctx.height);
    let p2 = ndc_to_screen(v2, ctx.width, ctx.height);

    // Bounding box of triangle
    let min_x = p0.0.min(p1.0).min(p2.0).max(0.0) as u32;
    let max_x = p0.0.max(p1.0).max(p2.0).min((ctx.width - 1) as f32) as u32;
    let min_y = p0.1.min(p1.1).min(p2.1).max(0.0) as u32;
    let max_y = p0.1.max(p1.1).max(p2.1).min((ctx.height - 1) as f32) as u32;

    // Edge functions
    let edge = |a: (f32, f32), b: (f32, f32), c: (f32, f32)| {
        (c.0 - a.0) * (b.1 - a.1) - (c.1 - a.1) * (b.0 - a.0)
    };

    let p0_2d = (p0.0, p0.1);
    let p1_2d = (p1.0, p1.1);
    let p2_2d = (p2.0, p2.1);

    // Use full p0, p1, p2 for interpolation but edge only cares about x,y
    let area = edge(p0_2d, p1_2d, p2_2d);
    if area == 0.0 {
        return;
    }

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let p = (x as f32 + 0.5, y as f32 + 0.5);

            let w0 = edge(p1_2d, p2_2d, p);
            let w1 = edge(p2_2d, p0_2d, p);
            let w2 = edge(p0_2d, p1_2d, p);

            // Check if inside
            if w0 >= 0.0 && w1 >= 0.0 && w2 >= 0.0 {
                let w0 = w0 / area;
                let w1 = w1 / area;
                let w2 = w2 / area;

                // Interpolate Z
                let _z = 1.0 / (w0 / v0.w + w1 / v1.w + w2 / v2.w);
                // Actually, standard perspective correct interpolation for Z buffer is just Z in NDC?
                // Z in screen space:
                let z_depth = w0 * p0.2 + w1 * p1.2 + w2 * p2.2;

                ctx.draw_pixel(x, y, z_depth, color);
            }
        }
    }
}

fn ndc_to_screen(v: &Vec4, width: u32, height: u32) -> (f32, f32, f32) {
    let ndc = *v / v.w;
    let x = (ndc.x + 1.0) * 0.5 * width as f32;
    let y = (1.0 - ndc.y) * 0.5 * height as f32; // Flip Y
    let z = ndc.z; // -1 to 1 usually, or 0 to 1 depending on API (WebGPU/DX vs GL). Glam uses GL conventions (-1 to 1)
    (x, y, z)
}

fn draw_line(ctx: &mut RenderContext, p0: (f32, f32), p1: (f32, f32), color: Rgb<u8>) {
    draw_line_segment_mut(&mut ctx.buffer, p0, p1, color);
}

fn draw_wireframe_box(ctx: &mut RenderContext, view_proj: &Mat4, model: Mat4, size: Vec3, color: Rgb<u8>) {
    let half = size * 0.5;
    let corners = [
        Vec3::new(-half.x, -half.y, -half.z), // 0
        Vec3::new(half.x, -half.y, -half.z),  // 1
        Vec3::new(-half.x, half.y, -half.z),  // 2
        Vec3::new(half.x, half.y, -half.z),   // 3
        Vec3::new(-half.x, -half.y, half.z),  // 4
        Vec3::new(half.x, -half.y, half.z),   // 5
        Vec3::new(-half.x, half.y, half.z),   // 6
        Vec3::new(half.x, half.y, half.z),    // 7
    ];
    
    let mvp = *view_proj * model;
    let clip: Vec<Vec4> = corners.iter().map(|p| mvp * Vec4::new(p.x, p.y, p.z, 1.0)).collect();
    
    // Edges
    let edges = [
        (0,1), (1,3), (3,2), (2,0), // Bottom face (if y up) or Back
        (4,5), (5,7), (7,6), (6,4), // Top/Front
        (0,4), (1,5), (2,6), (3,7)  // Connecting
    ];

    for (i, j) in edges {
        let v0 = clip[i];
        let v1 = clip[j];
        // Clip check logic (naive)
        if v0.w > 0.0 && v1.w > 0.0 {
            let s0 = ndc_to_screen(&v0, ctx.width, ctx.height);
            let s1 = ndc_to_screen(&v1, ctx.width, ctx.height);
            draw_line(ctx, (s0.0, s0.1), (s1.0, s1.1), color);
        }
    }
}

fn draw_axes(ctx: &mut RenderContext, view_proj: &Mat4, model: Mat4, length: f32) {
    let origin = Vec3::ZERO;
    let x = Vec3::X * length;
    let y = Vec3::Y * length;
    let z = Vec3::Z * length;

    let pts = [origin, x, y, z];
    let mvp = *view_proj * model;
    let clip: Vec<Vec4> = pts.iter().map(|p| mvp * Vec4::new(p.x, p.y, p.z, 1.0)).collect();

    let o_s = ndc_to_screen(&clip[0], ctx.width, ctx.height);
    
    if clip[0].w > 0.0 {
         // X: Red
        if clip[1].w > 0.0 {
            let x_s = ndc_to_screen(&clip[1], ctx.width, ctx.height);
            draw_line(ctx, (o_s.0, o_s.1), (x_s.0, x_s.1), Rgb([255, 0, 0]));
        }
        // Y: Green
        if clip[2].w > 0.0 {
            let y_s = ndc_to_screen(&clip[2], ctx.width, ctx.height);
            draw_line(ctx, (o_s.0, o_s.1), (y_s.0, y_s.1), Rgb([0, 255, 0]));
        }
        // Z: Blue
        if clip[3].w > 0.0 {
            let z_s = ndc_to_screen(&clip[3], ctx.width, ctx.height);
            draw_line(ctx, (o_s.0, o_s.1), (z_s.0, z_s.1), Rgb([0, 0, 255]));
        }
    }
}
