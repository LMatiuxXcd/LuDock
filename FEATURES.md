# LuDock Features

LuDock is a robust, agent-first infrastructure for simulating the Roblox Studio environment.

## 📦 Core Capabilities

### 1. Virtual DataModel
LuDock maintains an in-memory graph of instances that mimics the Roblox DataModel.
*   **Hierarchy**: Strict parent-child relationships (e.g., `game/Workspace`, `game/Lighting`).
*   **Properties**: Supports `Vector3`, `CFrame`, `Color3`, `UDim2`, `Enum`, `String`, `Number`, `Bool`.
*   **Determinism**: Guaranteed identical JSON/PNG outputs for identical inputs (V5 UUIDs, sorted loading).

### 2. Declarative Instance DSL
Custom format for defining instances.
```lua
ClassName = Part
Name = MyPart
Shape = Enum.PartType.Ball
Size = Vector3.new(10, 1, 10)
Color = Color3.fromRGB(255, 0, 0)
```

### 3. Luau Analysis Integration
*   **Strict Mode**: Fails if `luau-analyze` reports errors.
*   **Relaxed Mode**: Warns only.
*   **Auto-detection**: Finds binary in PATH or local dir.

### 4. Software Renderer (3D & 2D)
*   **3D**: Pure Rust rasterizer for `Block`, `Ball`, `Cylinder`.
*   **2D UI**: Renders `StarterGui` layouts (`Frame`, `UDim2` positioning/sizing).
*   **Debug**: Wireframe AABBs, Axes, Origins.

---

## 🚀 CLI Commands

### `ludock create <Name>`
Scaffolds a new project with standard services and plugin infrastructure.

### `ludock run [OPTIONS]`
Compiles and generates artifacts.

**Presets:**
*   `--preset agent`: Strict + Render + Diff + Debug Flags (Best for AI).
*   `--preset ci`: Strict + No Render + Diff (Best for pipelines).
*   `--preset debug`: Relaxed + Render + Debug Flags (Best for humans).

**Flags:**
*   `--relaxed`: Disable strict checks.
*   `--diff`: Generate `results/diff.json`.
*   `--3d`: Enable rendering.
*   `--debug-bounds`, `--debug-origin`, `--debug-axes`: Visual overlays.

### `ludock doctor`
Diagnoses environment (version, binaries, settings).

### `ludock schema`
Generates JSON Schemas for `world.json`, `diagnostics.json`, and `diff.json` into `schemas/`.

---

## 📂 Artifacts & Contracts

### Stable Schemas (`schemas/`)
Versioning support via `schemaVersion`.
*   `world.schema.json`
*   `diagnostics.schema.json`
*   `diff.schema.json`

### `results/diff.json`
Structured comparison of runs:
*   `added_instances`
*   `removed_instances`
*   `modified_instances` (property changes, spatial displacement)

### `results/render.png`
800x600 visualization of the world + UI.

---

## 🧩 Plugin System
Projects include a `.ludock/plugins/manifest.json` for future extensibility hooks.
