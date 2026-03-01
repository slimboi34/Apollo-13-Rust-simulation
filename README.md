# 🚀 Apollo 11 Rust Simulation

A hyper-detailed, real-time 3D simulation of the Apollo 11 mission built in **Rust** using the **Bevy game engine**. Trace the complete journey from Kennedy Space Center to the Moon and back — with physically accurate trajectories, procedural spacecraft models, and cinematic visuals.

![Rust](https://img.shields.io/badge/Rust-000000?style=flat&logo=rust&logoColor=white)
![Bevy](https://img.shields.io/badge/Bevy-232326?style=flat&logo=bevy&logoColor=white)
![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)

## ✨ Features

- **NASA-Accurate Trajectory** — Procedurally generated orbital path matching the real Apollo 11 flight plan:
  - 1.5 Earth parking orbit revolutions
  - Translunar coast arc
  - 10 lunar orbit revolutions
  - Trans-Earth return arc
  - Re-entry and Pacific splashdown

- **Physics-Based Collision Avoidance** — Trajectory points are constrained to never penetrate planetary surfaces

- **Hyper-Detailed Procedural Spacecraft** — 20+ individual mesh components:
  - Service Module (silver hull, SPS engine bell, RCS quad thrusters, high-gain antenna)
  - Command Module (gumdrop cone, heat shield, docking probe)
  - SLA Adapter panels (jettisoned after TLI)
  - Lunar Module (gold foil descent stage, ascent stage, windows, 4 landing legs with pads)

- **Mission Phase Events** — Real-time spacecraft configuration changes:
  - Engine glow during TLI, LOI, and TEI burns
  - SLA adapter jettison
  - LM undocking and independent descent to lunar surface

- **Cinematic Visuals** — 4K/8K planet textures, PBR materials, bloom lighting, starfield skybox

- **Dynamic Camera** — Starts at astronaut-perspective zoom on the KSC launchpad, smoothly pulls out to show the full Earth-Moon system

- **3D Reference Grid** — Spatial tracking grid with color-coded mission event markers

## 🎮 Controls

| Key | Action |
|-----|--------|
| `Space` | Pause / Play |
| `Arrow Up` | Increase speed |
| `Arrow Down` | Decrease speed (supports reverse) |
| `T` | Toggle camera tracking |
| `R` | Reset to KSC launchpad |
| `Mouse Drag` | Orbit camera |
| `Scroll Wheel` | Zoom in/out |

## 🛠️ Requirements

- [Rust](https://www.rust-lang.org/tools/install) (stable toolchain)
- macOS, Linux, or Windows

## 🚀 Quick Start

```bash
# Clone the repository
git clone git@github.com:slimboi34/Apollo-13-Rust-simulation.git
cd Apollo-13-Rust-simulation/apollo11_bevy

# Run the simulation (release mode recommended for performance)
cargo run --release --bin apollo11_bevy
```

## 📁 Project Structure

```
apollo11_bevy/
├── src/
│   ├── main.rs          # App entry point, UI setup, plugin registration
│   ├── constants.rs     # Physical constants, scale factors
│   ├── components.rs    # ECS components and resources
│   ├── spline.rs        # Arc-length sampling, collision avoidance
│   ├── setup.rs         # Solar system, spacecraft, trajectory generation
│   └── systems.rs       # Physics, input, UI update systems
├── assets/              # Planet textures (4K/8K), starfield
├── Cargo.toml           # Dependencies (Bevy, bevy_panorbit_camera)
└── README.md
```

## 🔧 Tech Stack

- **Language:** Rust
- **Engine:** [Bevy](https://bevyengine.org/) v0.14
- **Camera:** [bevy_panorbit_camera](https://crates.io/crates/bevy_panorbit_camera)
- **Rendering:** PBR materials, DirectionalLight, bloom post-processing
- **Trajectory:** Procedural orbital segment generation with physics-based constraints

## 📜 License

MIT License — see [LICENSE](LICENSE) for details.
