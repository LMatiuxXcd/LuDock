# LuDock

LuDock is a headless CLI infrastructure for AI agents to develop Roblox projects. It simulates a Roblox Studio environment, allowing agents to create scripts and declarative instances, validate them, and visualize the world through a software renderer.

## Features

- **100% Rust**: Single binary, no heavy dependencies.
- **Headless Runtime**: Runs in CI/CD or agent containers.
- **Virtual DataModel**: Mimics Roblox's instance hierarchy.
- **Luau Analysis**: Integrates with `luau-analyze` for type checking.
- **3D Renderer**: Generates `.png` feedback for agents using a pure software renderer (no GPU required).

## Installation

```bash
cargo build --release
cp target/release/ludock /usr/local/bin/
```

### Dependencies

- **luau-analyze**: For script validation, ensure `luau-analyze` is in your PATH. You can download it from the [Luau repository](https://github.com/roblox/luau/releases).

## Usage

### Create a Project

```bash
ludock create "MyGame"
```

This creates a directory structure mirroring Roblox Studio services.

### Edit Project

- Add `.server.lua`, `.local.lua`, `.module.lua` for scripts.
- Add `.basepart`, `.model`, etc., for declarative instances.

Example `part.basepart`:
```lua
ClassName = Part
Name = MyPart
Size = Vector3.new(10, 1, 10)
CFrame = CFrame.new(0, 5, 0)
Color = Color3.fromRGB(255, 0, 0)
Anchored = true
```

### Run & Render

```bash
cd MyGame
ludock run --3d
```

Outputs are saved in `results/`:
- `world.json`: The complete instance tree.
- `diagnostics.json`: Script errors and warnings.
- `render.png`: A visual snapshot of the workspace.
