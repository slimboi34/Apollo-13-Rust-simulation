use bevy::prelude::*;
use bevy_panorbit_camera::PanOrbitCamera;
use crate::constants::*;
use crate::components::*;
use crate::spline::*;

// At PLANET_SCALE=30, DISTANCE_SCALE=350:
// r_e ≈ 173.9, r_m ≈ 47.4
// earth_pos.x ≈ -4.2, moon_pos.x ≈ 345.8

/// Build CSM trajectory procedurally from orbital segments matching NASA diagram:
/// 1) Earth parking orbit (1.5 revolutions)
/// 2) TLI escape + translunar coast (arc ABOVE Earth-Moon line)
/// 3) Lunar orbit (10 revolutions)
/// 4) TEI + transearth coast (arc BELOW Earth-Moon line)
/// 5) Re-entry and splashdown
fn build_csm_trajectory(earth_pos: Vec3, moon_pos: Vec3) -> Vec<Vec3> {
    let r_e: f32 = (6371e3 / L_STAR) as f32 * DISTANCE_SCALE * PLANET_SCALE;
    let r_m: f32 = (1737e3 / L_STAR) as f32 * DISTANCE_SCALE * PLANET_SCALE;
    let parking_alt = r_e + 5.0;  // ~185km parking orbit altitude (scaled)
    let lunar_orbit_alt = r_m + 5.0;  // ~110km lunar orbit altitude (scaled)
    
    let mut pts: Vec<Vec3> = Vec::with_capacity(2000);
    
    // === PHASE 1: Earth Parking Orbit (1.5 revolutions, CCW in XY plane) ===
    let parking_steps = 300;
    let parking_revs = 1.5;
    let parking_start = 200.0_f32.to_radians(); // Start angle ~ Florida longitude
    for i in 0..=parking_steps {
        let frac = i as f32 / parking_steps as f32;
        let angle = parking_start + frac * parking_revs * 2.0 * std::f32::consts::PI;
        pts.push(earth_pos + Vec3::new(
            parking_alt * angle.cos(),
            parking_alt * angle.sin(),
            0.0,  // Planar — matching NASA diagram
        ));
    }
    
    // === PHASE 2: TLI + Translunar Coast (gentle arc ABOVE the Earth-Moon line) ===
    let coast_steps = 200;
    let depart_pos = *pts.last().unwrap();
    let arrive_pos = moon_pos + Vec3::new(-lunar_orbit_alt, lunar_orbit_alt * 0.5, 0.0);
    for i in 1..=coast_steps {
        let frac = i as f32 / coast_steps as f32;
        // Smooth cubic Hermite — arc above the line (positive Y bulge)
        let x = depart_pos.x + (arrive_pos.x - depart_pos.x) * frac;
        let base_y = depart_pos.y + (arrive_pos.y - depart_pos.y) * frac;
        let arc_height = 60.0 * (frac * std::f32::consts::PI).sin(); // Gentle arc above
        pts.push(Vec3::new(x, base_y + arc_height, 0.0));
    }
    
    // === PHASE 3: Lunar Orbit (10 revolutions, CW when viewed from above) ===
    let lunar_steps = 800;
    let lunar_revs = 10.0;
    let lunar_start_angle = (arrive_pos - moon_pos).y.atan2((arrive_pos - moon_pos).x);
    for i in 1..=lunar_steps {
        let frac = i as f32 / lunar_steps as f32;
        let angle = lunar_start_angle - frac * lunar_revs * 2.0 * std::f32::consts::PI;
        pts.push(moon_pos + Vec3::new(
            lunar_orbit_alt * angle.cos(),
            lunar_orbit_alt * angle.sin(),
            0.0,
        ));
    }
    
    // === PHASE 4: TEI + Transearth Coast (gentle arc BELOW the Earth-Moon line) ===
    let return_steps = 200;
    let tei_pos = *pts.last().unwrap();
    let reentry_pos = earth_pos + Vec3::new(parking_alt * 0.7, -parking_alt, 0.0);
    for i in 1..=return_steps {
        let frac = i as f32 / return_steps as f32;
        let x = tei_pos.x + (reentry_pos.x - tei_pos.x) * frac;
        let base_y = tei_pos.y + (reentry_pos.y - tei_pos.y) * frac;
        let arc_depth = -50.0 * (frac * std::f32::consts::PI).sin(); // Arc below
        pts.push(Vec3::new(x, base_y + arc_depth, 0.0));
    }
    
    // === PHASE 5: Re-entry spiral to splashdown ===
    let reentry_steps = 100;
    let splash = earth_pos + Vec3::new(-parking_alt, -10.0, 0.0); // Pacific
    for i in 1..=reentry_steps {
        let frac = i as f32 / reentry_steps as f32;
        let pos = reentry_pos.lerp(splash, frac);
        pts.push(pos);
    }
    
    pts
}

/// Build LM trajectory: follows CSM until lunar orbit, then descends to surface
fn build_lm_trajectory(earth_pos: Vec3, moon_pos: Vec3) -> Vec<Vec3> {
    let r_m: f32 = (1737e3 / L_STAR) as f32 * DISTANCE_SCALE * PLANET_SCALE;
    let lunar_orbit_alt = r_m + 5.0;
    
    // LM follows CSM for the first portion
    let csm = build_csm_trajectory(earth_pos, moon_pos);
    
    // LM separates at 50% of the CSM path (during lunar orbit)
    let sep_idx = csm.len() / 2;
    let mut pts: Vec<Vec3> = csm[..=sep_idx].to_vec();
    
    // Descent from lunar orbit to surface (Sea of Tranquility)
    let sep_pos = pts[sep_idx];
    let descent_start_angle = ((sep_pos - moon_pos).y).atan2((sep_pos - moon_pos).x);
    
    let descent_steps = 100;
    for i in 1..=descent_steps {
        let frac = i as f32 / descent_steps as f32;
        let alt = lunar_orbit_alt + (r_m + 0.5 - lunar_orbit_alt) * frac;
        let angle = descent_start_angle + frac * 1.5;
        pts.push(moon_pos + Vec3::new(alt * angle.cos(), alt * angle.sin(), 0.0));
    }
    
    // Surface stay (stationary)
    let surface_pos = *pts.last().unwrap();
    for _ in 0..200 {
        pts.push(surface_pos);
    }
    
    pts
}

pub fn setup_solar_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // Balanced ambient light - enough to see detail in shadow but still dramatic
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 150.0,
    });
    
    commands.insert_resource(SimSettings { paused: true, speed: 0.02, tracking: true, mission_time: 0.0 });
    
    let earth_pos = Vec3::new((X_E as f32) * DISTANCE_SCALE, 0.0, 0.0);
    let moon_pos  = Vec3::new((X_M as f32) * DISTANCE_SCALE, 0.0, 0.0);

    // Build procedural trajectories from orbital segments (not splines!)
    let mut raw_csm = build_csm_trajectory(earth_pos, moon_pos);
    let mut raw_lm = build_lm_trajectory(earth_pos, moon_pos);
    
    // Physics constraint: ensure NO trajectory point penetrates planet surfaces
    let r_e = (6371e3 / L_STAR) as f32 * DISTANCE_SCALE * PLANET_SCALE;
    let r_m = (1737e3 / L_STAR) as f32 * DISTANCE_SCALE * PLANET_SCALE;
    let bodies = [(earth_pos, r_e), (moon_pos, r_m)];
    enforce_planet_clearance(&mut raw_csm, &bodies, 3.0);
    enforce_planet_clearance(&mut raw_lm, &bodies, 3.0);
    
    // Compute arc lengths for distance-based sampling
    let csm_distances = calculate_arc_lengths(&raw_csm);
    let lm_distances = calculate_arc_lengths(&raw_lm);
    
    let initial_pos = raw_csm[0];
    
    commands.insert_resource(OrbitPaths { 
        csm_points: raw_csm, 
        lm_points: raw_lm,
        csm_distances,
        lm_distances
    });
    
    // Sunlight - bright but not blinding, coming from behind the camera
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: true,
            illuminance: 50000.0,
            ..default()
        },
        transform: Transform::from_xyz(0.0, 200.0, 500.0).looking_at(earth_pos, Vec3::Y),
        ..default()
    });

    // Sun Sphere (far away, large, glowing)
    let sun_tex = asset_server.load("sun_4k.jpg");
    commands.spawn(PbrBundle {
        mesh: meshes.add(bevy::math::primitives::Sphere::new(200.0).mesh().uv(64, 32)),
        material: materials.add(StandardMaterial {
            base_color_texture: Some(sun_tex.clone()),
            emissive: Color::WHITE.into(),
            emissive_texture: Some(sun_tex),
            unlit: true,
            ..default()
        }),
        transform: Transform::from_xyz(0.0, 200.0, 4000.0),
        ..default()
    });

    // == EARTH ==
    // Clean, simple approach: just use the color texture. No broken normal/specular stubs.
    let r_e = (6371e3 / L_STAR) as f32 * DISTANCE_SCALE * PLANET_SCALE;
    let earth_tex = asset_server.load("earth_4k.jpg");
    
    commands.spawn(PbrBundle {
        mesh: meshes.add(bevy::math::primitives::Sphere::new(r_e).mesh().uv(128, 64)),
        material: materials.add(StandardMaterial {
            base_color_texture: Some(earth_tex),
            perceptual_roughness: 0.5,
            metallic: 0.0,
            ..default()
        }),
        transform: Transform::from_translation(earth_pos)
            .with_rotation(Quat::from_rotation_z(23.5_f32.to_radians())),
        ..default()
    }).with_children(|parent| {
        // Cloud layer using the PNG with proper alpha channel
        let clouds: Handle<Image> = asset_server.load("earth_clouds_4k.png");
        parent.spawn(PbrBundle {
            mesh: meshes.add(bevy::math::primitives::Sphere::new(r_e * 1.005).mesh().uv(128, 64)),
            material: materials.add(StandardMaterial {
                base_color: Color::srgba(1.0, 1.0, 1.0, 0.5),
                base_color_texture: Some(clouds),
                alpha_mode: AlphaMode::Blend,
                unlit: true,
                ..default()
            }),
            ..default()
        });

        // Night lights on dark side
        let night_tex = asset_server.load("earth_night_4k.jpg");
        parent.spawn(PbrBundle {
            mesh: meshes.add(bevy::math::primitives::Sphere::new(r_e * 1.001).mesh().uv(128, 64)),
            material: materials.add(StandardMaterial {
                base_color_texture: Some(night_tex.clone()),
                emissive_texture: Some(night_tex),
                emissive: Color::srgb(1.5, 1.2, 0.6).into(),
                alpha_mode: AlphaMode::Add,
                unlit: true,
                ..default()
            }),
            ..default()
        });
    });

    // == MOON ==
    let r_m = (1737e3 / L_STAR) as f32 * DISTANCE_SCALE * PLANET_SCALE;
    let moon_tex = asset_server.load("moon_4k.jpg");
    commands.spawn(PbrBundle {
        mesh: meshes.add(bevy::math::primitives::Sphere::new(r_m).mesh().uv(128, 64)),
        material: materials.add(StandardMaterial {
            base_color_texture: Some(moon_tex),
            perceptual_roughness: 0.85,
            metallic: 0.0,
            ..default()
        }),
        transform: Transform::from_xyz((X_M as f32) * DISTANCE_SCALE, 0.0, 0.0),
        ..default()
    });

    // == STARFIELD ==
    let star_tex = asset_server.load("starmap_8k.jpg");
    commands.spawn(PbrBundle {
        mesh: meshes.add(bevy::math::primitives::Sphere::new(9000.0).mesh().uv(64, 32)),
        material: materials.add(StandardMaterial {
            base_color_texture: Some(star_tex),
            unlit: true,
            cull_mode: None,
            ..default()
        }),
        transform: Transform::IDENTITY,
        ..default()
    });

    // ================================================================
    // == HYPER-DETAILED APOLLO 11 SPACECRAFT STACK ==
    // ================================================================
    // Real Apollo stack (bottom to top): SPS Engine Bell → Service Module → 
    // Heat Shield → Command Module → SLA Adapter → Lunar Module
    
    let silver = materials.add(StandardMaterial {
        base_color: Color::srgb(0.85, 0.85, 0.88),
        metallic: 0.95,
        perceptual_roughness: 0.1,
        ..default()
    });
    let white_thermal = materials.add(StandardMaterial {
        base_color: Color::srgb(0.95, 0.95, 0.95),
        metallic: 0.3,
        perceptual_roughness: 0.6,
        ..default()
    });
    let gold_foil = materials.add(StandardMaterial {
        base_color: Color::srgb(0.82, 0.62, 0.04),
        metallic: 1.0,
        perceptual_roughness: 0.12,
        ..default()
    });
    let dark_metal = materials.add(StandardMaterial {
        base_color: Color::srgb(0.25, 0.25, 0.28),
        metallic: 0.9,
        perceptual_roughness: 0.4,
        ..default()
    });
    let heat_shield = materials.add(StandardMaterial {
        base_color: Color::srgb(0.35, 0.20, 0.08),
        metallic: 0.2,
        perceptual_roughness: 0.9,
        ..default()
    });
    let sla_panel = materials.add(StandardMaterial {
        base_color: Color::srgb(0.75, 0.75, 0.72),
        metallic: 0.5,
        perceptual_roughness: 0.5,
        ..default()
    });

    commands.spawn((
        PbrBundle {
            // Root: Service Module body (main cylindrical hull)
            mesh: meshes.add(Cylinder::new(0.5, 1.8)),
            material: silver.clone(),
            transform: Transform::from_translation(initial_pos).with_scale(Vec3::splat(2.0)),
            ..default()
        },
        Spacecraft { ship_type: ShipType::CommandModule },
        AttachedSM,
    )).with_children(|parent| {
        parent.spawn(PbrBundle {
            mesh: meshes.add(bevy::math::primitives::Cone { radius: 0.35, height: 0.5 }),
            material: dark_metal.clone(),
            transform: Transform::from_xyz(0.0, -1.15, 0.0)
                .with_rotation(Quat::from_rotation_x(std::f32::consts::PI)),
            ..default()
        });
        
        // Engine Glow (emissive sphere behind the SPS bell, visible during burns)
        parent.spawn((
            PbrBundle {
                mesh: meshes.add(bevy::math::primitives::Sphere::new(0.2).mesh().uv(16, 8)),
                material: materials.add(StandardMaterial {
                    base_color: Color::srgb(1.0, 0.6, 0.1),
                    emissive: Color::srgb(5.0, 3.0, 0.5).into(),
                    unlit: true,
                    ..default()
                }),
                transform: Transform::from_xyz(0.0, -1.5, 0.0),
                visibility: Visibility::Hidden,
                ..default()
            },
            EngineGlow,
        ));
        
        // High Gain Antenna dish (small cylinder on SM side)
        parent.spawn(PbrBundle {
            mesh: meshes.add(Cylinder::new(0.15, 0.02)),
            material: white_thermal.clone(),
            transform: Transform::from_xyz(0.55, 0.0, 0.0),
            ..default()
        });

        // RCS Quad Thruster blocks (4 around the SM)
        for angle in [0.0_f32, 90.0, 180.0, 270.0] {
            let rad = angle.to_radians();
            parent.spawn(PbrBundle {
                mesh: meshes.add(Cuboid::new(0.08, 0.15, 0.08)),
                material: white_thermal.clone(),
                transform: Transform::from_xyz(
                    0.52 * rad.cos(),
                    0.3,
                    0.52 * rad.sin(),
                ),
                ..default()
            });
        }

        // Heat Shield (dark disc between SM and CM)
        parent.spawn(PbrBundle {
            mesh: meshes.add(Cylinder::new(0.48, 0.06)),
            material: heat_shield.clone(),
            transform: Transform::from_xyz(0.0, 0.93, 0.0),
            ..default()
        });
        
        // Command Module (gumdrop cone)
        parent.spawn(PbrBundle {
            mesh: meshes.add(bevy::math::primitives::Cone { radius: 0.48, height: 0.7 }),
            material: silver.clone(),
            transform: Transform::from_xyz(0.0, 1.31, 0.0),
            ..default()
        });

        // Docking Probe (thin cylinder on top of CM)
        parent.spawn(PbrBundle {
            mesh: meshes.add(Cylinder::new(0.03, 0.2)),
            material: dark_metal.clone(),
            transform: Transform::from_xyz(0.0, 1.76, 0.0),
            ..default()
        });
        
        // SLA Adapter (frustum connecting SM to LM, simulated with cylinder)
        parent.spawn((
            PbrBundle {
                mesh: meshes.add(Cylinder::new(0.55, 0.6)),
                material: sla_panel.clone(),
                transform: Transform::from_xyz(0.0, -1.2, 0.0),
                ..default()
            },
            AttachedSLA,
        ));

        // ================================================
        // ATTACHED LUNAR MODULE (stowed inside SLA)
        // ================================================
        parent.spawn((
            PbrBundle {
                // LM Descent Stage (gold foil octagonal box)
                mesh: meshes.add(Cuboid::new(0.6, 0.35, 0.6)),
                material: gold_foil.clone(),
                transform: Transform::from_xyz(0.0, -1.7, 0.0),
                ..default()
            },
            AttachedLM,
        )).with_children(|lm| {
            // LM Ascent Stage (smaller box on top of descent)
            lm.spawn(PbrBundle {
                mesh: meshes.add(Cuboid::new(0.4, 0.35, 0.4)),
                material: gold_foil.clone(),
                transform: Transform::from_xyz(0.0, 0.35, 0.0),
                ..default()
            });
            // LM Ascent windows (dark panels)
            lm.spawn(PbrBundle {
                mesh: meshes.add(Cuboid::new(0.42, 0.12, 0.08)),
                material: dark_metal.clone(),
                transform: Transform::from_xyz(0.0, 0.45, 0.22),
                ..default()
            });
            // Descent Engine Bell
            lm.spawn(PbrBundle {
                mesh: meshes.add(bevy::math::primitives::Cone { radius: 0.15, height: 0.2 }),
                material: dark_metal.clone(),
                transform: Transform::from_xyz(0.0, -0.27, 0.0)
                    .with_rotation(Quat::from_rotation_x(std::f32::consts::PI)),
                ..default()
            });
            // Landing Legs (4 struts extending outward and downward)
            for angle in [0.0_f32, 90.0, 180.0, 270.0] {
                let rad = angle.to_radians();
                lm.spawn(PbrBundle {
                    mesh: meshes.add(Cuboid::new(0.03, 0.4, 0.03)),
                    material: silver.clone(),
                    transform: Transform::from_xyz(
                        0.4 * rad.cos(),
                        -0.35,
                        0.4 * rad.sin(),
                    ).with_rotation(Quat::from_rotation_z(0.3 * if angle < 180.0 { 1.0 } else { -1.0 })),
                    ..default()
                });
                // Landing pad (small flat disc at end of each leg)
                lm.spawn(PbrBundle {
                    mesh: meshes.add(Cylinder::new(0.06, 0.02)),
                    material: gold_foil.clone(),
                    transform: Transform::from_xyz(
                        0.5 * rad.cos(),
                        -0.55,
                        0.5 * rad.sin(),
                    ),
                    ..default()
                });
            }
        });
    });

    // Camera — start right at the spacecraft on the Kennedy Space Center launchpad
    // This gives the astronaut perspective: the Earth is MASSIVE, filling the view
    let cam_offset = Vec3::new(5.0, 8.0, 15.0); // Slightly above and behind the capsule
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_translation(initial_pos + cam_offset)
                .looking_at(initial_pos, Vec3::Y),
            projection: Projection::Perspective(PerspectiveProjection {
                fov: 60.0_f32.to_radians(), // Wide human FOV
                near: 0.1,
                far: 20000.0,
                ..default()
            }),
            ..default()
        },
        PanOrbitCamera {
            focus: initial_pos,
            target_focus: initial_pos,
            radius: Some(18.0),   // Tight astronaut zoom
            target_radius: 18.0,
            ..default()
        },
        bevy::core_pipeline::bloom::BloomSettings {
            intensity: 0.08,
            ..default()
        },
    ));
}

