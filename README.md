# 🚀 Apollo 11 Rust Simulation

A real-time 3D simulation of the **Apollo 11** mission built in **Rust** using the **Bevy game engine**. Trace the complete journey from Kennedy Space Center to the Moon and back — with physically accurate trajectories, procedural spacecraft models, and cinematic visuals.

![Rust](https://img.shields.io/badge/Rust-000000?style=flat&logo=rust&logoColor=white)
![Bevy](https://img.shields.io/badge/Bevy_0.14-232326?style=flat&logo=bevy&logoColor=white)
![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)

---

## ✨ Features

### NASA-Accurate Trajectory
The trajectory is **procedurally generated from orbital segments**, not hand-drawn splines. Each phase is built from real orbital mechanics:

| Phase | Points | Description |
|-------|--------|-------------|
| Earth Parking Orbit | 300 | 1.5 revolutions at ~185km altitude (CCW) |
| Translunar Coast | 200 | Gentle arc above the Earth-Moon line |
| Lunar Orbit | 800 | 10 revolutions at ~110km altitude (CW) |
| Trans-Earth Coast | 200 | Return arc below the Earth-Moon line |
| Re-entry & Splashdown | 100 | Final approach to the Pacific Ocean |

A **physics-based collision constraint** ensures no trajectory point ever penetrates a planet surface — points inside a body are pushed radially outward to the surface + minimum altitude.

### Procedural Spacecraft (20+ Meshes)
Every part of the Apollo stack is individually modeled:

- **Service Module** — Silver cylindrical hull with SPS engine bell, high-gain antenna, and 4 RCS quad thruster blocks
- **Command Module** — Gumdrop cone with ablative heat shield and docking probe
- **SLA Adapter** — Grey panels connecting SM to LM (jettisoned at t=0.12)
- **Lunar Module** — Gold foil descent stage, ascent stage with dark windows, descent engine bell, 4 landing legs with foot pads
- **Engine Glow** — Orange emissive sphere behind SPS bell, visible during burns

### Mission Phase Events
The spacecraft configuration changes in real-time as the mission progresses:

| Time (t) | Event | Visual Effect |
|----------|-------|---------------|
| 0.02–0.08 | TLI Burn | 🔥 Engine glow ON |
| 0.12 | SLA Jettison | Grey adapter panels disappear |
| 0.33–0.38 | LOI Burn | 🔥 Engine glow ON |
| 0.50 | LM Undocking | Gold LM separates into independent entity |
| 0.55–0.60 | LM Descent | LM spirals down to lunar surface |
| 0.68–0.73 | TEI Burn | 🔥 Engine glow ON |
| 0.85–1.0 | Re-entry | CSM returns to Earth |

### Dynamic Camera System
- Starts at **astronaut-perspective zoom** (radius=18) right on the KSC launchpad
- **Quadratic ease zoom-out** as the mission progresses: `radius = 18 + t² × 332`
- Smooth camera tracking with lerp interpolation (no jarring snaps)
- Free orbit with mouse drag + scroll wheel zoom at any time

### 3D Reference Grid
A translucent spatial grid is drawn between Earth and Moon for depth perception, with color-coded 3D cross markers at each mission event.

---

## 🎮 Controls

| Key | Action | Details |
|-----|--------|---------|
| `Space` | **Play / Pause** | Simulation starts paused at T-0 on the KSC launchpad |
| `⬆ Arrow Up` | **Increase speed** | Hold to accelerate (max 2.0x) |
| `⬇ Arrow Down` | **Decrease speed** | Hold to slow down. Goes negative for **reverse playback** (min -2.0x) |
| `T` | **Toggle tracking** | ON = camera follows spacecraft. OFF = free camera |
| `R` | **Reset** | Snaps back to T-0 at KSC with astronaut zoom, paused |
| `Mouse Drag` | **Orbit camera** | Left-click and drag to rotate around the focus point |
| `Scroll Wheel` | **Zoom** | Scroll to zoom in/out. Overrides auto-zoom when tracking |

### HUD Display
The overlay shows real-time mission data:
- **Phase name** (Earth Parking, Translunar Coast, Lunar Orbit, etc.)
- **Mission elapsed time** (T+ hours and minutes, scaled to 195-hour mission)
- **Velocity** (km/s)
- **Altitude above Earth** and **Altitude above Moon** (km, with comma formatting)
- **Current controls state** (paused/live, tracking on/off, speed multiplier)

---

## 🔧 How It Works

### Architecture

The simulation uses the **Bevy Entity-Component-System (ECS)** architecture, split into 6 modules:

```
main.rs         → App setup, plugin registration, UI layout
constants.rs    → Physical constants (masses, distances, scale factors)
components.rs   → ECS components: Spacecraft, SimSettings, OrbitPaths, UI tags
spline.rs       → Arc-length sampling, collision avoidance math
setup.rs        → Solar system spawning, trajectory generation, spacecraft meshes
systems.rs      → Per-frame update: physics, input handling, UI updates
```

### Trajectory Generation (`setup.rs`)
Instead of using a mathematical spline (which freely interpolates and can clip through planets), the trajectory is **built procedurally from 5 orbital segments**:

1. **Circular orbit points** are generated around Earth using `sin`/`cos` at the correct altitude
2. **Coast arcs** are computed as smooth curves with sinusoidal height offsets above/below the Earth-Moon line
3. **Lunar orbit points** are generated around the Moon with a configurable number of revolutions
4. All points are concatenated into one path of ~1,600 points
5. `enforce_planet_clearance()` post-processes every point, pushing any that fall inside a planet radially outward
6. Arc-length distances are computed so velocity is physically proportional to distance traveled

### Rendering
- **PBR materials** on all meshes (metallic, roughness, emissive)
- **Directional light** as the Sun with 50,000 lux intensity
- **Ambient light** at 150 lux for shadow detail
- **Bloom post-processing** for engine glow and star highlights
- **Gizmos** for trajectory trails, grid, and event markers (no mesh overhead)

### Scaling
Real distances are impractical for visualization, so the simulation uses two scale factors:
- `DISTANCE_SCALE = 350` — World units per Earth-Moon distance (~384,400 km)
- `PLANET_SCALE = 30` — Visual radius multiplier (Earth appears 30x larger than to-scale)

---

## 🛠️ Requirements

- [Rust](https://www.rust-lang.org/tools/install) (stable toolchain)
- macOS, Linux, or Windows
- GPU with Metal, Vulkan, or DX12 support

## 🚀 Quick Start

```bash
# Clone the repository
git clone https://github.com/slimboi34/Apollo-13-Rust-simulation.git
cd Apollo-13-Rust-simulation/apollo11_bevy

# Run the simulation (release mode recommended for performance)
cargo run --release --bin apollo11_bevy
```

The first build takes ~2 minutes to compile Bevy. Subsequent runs are fast.

---

## 🖼️ Texture Assets

The simulation uses 4K planet textures. These are included in the repo but originally sourced from:

| Texture | Source | License |
|---------|--------|---------|
| `earth_4k.jpg` | [Solar System Scope](https://www.solarsystemscope.com/textures/) | CC BY 4.0 |
| `earth_normal_4k.jpg` | [Solar System Scope](https://www.solarsystemscope.com/textures/) | CC BY 4.0 |
| `earth_specular_4k.jpg` | [Solar System Scope](https://www.solarsystemscope.com/textures/) | CC BY 4.0 |
| `earth_clouds_4k.jpg` | [Solar System Scope](https://www.solarsystemscope.com/textures/) | CC BY 4.0 |
| `earth_night_4k.jpg` | [Solar System Scope](https://www.solarsystemscope.com/textures/) | CC BY 4.0 |
| `moon_4k.jpg` | [Solar System Scope](https://www.solarsystemscope.com/textures/) | CC BY 4.0 |
| `sun_4k.jpg` | [Solar System Scope](https://www.solarsystemscope.com/textures/) | CC BY 4.0 |
| `stars_4k.jpg` | [Solar System Scope](https://www.solarsystemscope.com/textures/) | CC BY 4.0 |

### Downloading Higher-Resolution Textures (Optional)
For 8K textures (not included due to file size), run the included download script:
```bash
cd apollo11_bevy
python3 download_4k_textures.py
```

---

## 📁 Project Structure

```
Apollo-13-Rust-simulation/
├── README.md
├── LICENSE
├── .gitignore
└── apollo11_bevy/
    ├── Cargo.toml                    # Dependencies
    ├── Cargo.lock
    ├── src/
    │   ├── main.rs                   # App entry, UI layout
    │   ├── constants.rs              # Physics constants, scale factors
    │   ├── components.rs             # ECS components & resources
    │   ├── spline.rs                 # Arc-length math, collision avoidance
    │   ├── setup.rs                  # Trajectory builder, mesh spawning
    │   └── systems.rs                # Physics, input, UI systems
    ├── assets/                       # 4K planet textures
    ├── download_4k_textures.py       # Texture download helper
    └── download_normal_textures.py   # Normal/specular map helper
```

## 🔗 Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| [bevy](https://bevyengine.org/) | 0.14 | Game engine (ECS, rendering, windowing) |
| [bevy_panorbit_camera](https://crates.io/crates/bevy_panorbit_camera) | 0.19 | Orbital camera with smooth zoom/pan |

## 📜 License

MIT License — see [LICENSE](LICENSE) for details.

Texture assets are licensed under [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/) by [Solar System Scope](https://www.solarsystemscope.com/).
