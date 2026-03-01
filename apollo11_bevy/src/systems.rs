use bevy::prelude::*;
use bevy::render::view::screenshot::ScreenshotManager;
use bevy::window::PrimaryWindow;
use bevy_panorbit_camera::PanOrbitCamera;
use crate::constants::*;
use crate::components::*;
use crate::spline::*;

pub fn physics_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Spacecraft, &mut Transform)>,
    query_attached_lm: Query<Entity, With<AttachedLM>>,
    query_attached_sla: Query<Entity, With<AttachedSLA>>,
    mut query_glow: Query<&mut Visibility, With<EngineGlow>>,
    mut cam_query: Query<&mut PanOrbitCamera>,
    mut settings: ResMut<SimSettings>,
    time: Res<Time>,
    mut gizmos: Gizmos,
    paths: Res<OrbitPaths>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if !settings.paused {
        settings.mission_time += settings.speed * time.delta_seconds();
        if settings.mission_time > 1.0 { settings.mission_time = 1.0; }
        if settings.mission_time < 0.0 { settings.mission_time = 0.0; }
    }

    let t = settings.mission_time;
    
    // ARC-LENGTH PARAMETERIZATION: 
    let total_csm_dist = get_total_distance(&paths.csm_distances);
    let total_lm_dist = get_total_distance(&paths.lm_distances);
    
    let target_csm_dist = total_csm_dist * t;
    let target_lm_dist = total_lm_dist * t;
    
    let current_csm = sample_pos_by_distance(&paths.csm_points, &paths.csm_distances, target_csm_dist);
    let current_lm = sample_pos_by_distance(&paths.lm_points, &paths.lm_distances, target_lm_dist);

    // === ENGINE GLOW: visible during major burn phases ===
    let is_burning = (t > 0.02 && t < 0.08)   // TLI burn
                   || (t > 0.33 && t < 0.38)   // LOI burn
                   || (t > 0.68 && t < 0.73);   // TEI burn
    for mut vis in &mut query_glow {
        *vis = if is_burning { Visibility::Visible } else { Visibility::Hidden };
    }
    
    // === SLA JETTISON at t=0.12 (after transposition & docking) ===
    if t >= 0.12 {
        if let Ok(sla) = query_attached_sla.get_single() {
            commands.entity(sla).despawn_recursive();
        }
    }

    // === LM UNDOCKING at t=0.5 ===
    let is_detached = t >= 0.5;
    for (_entity, sc, mut transform) in &mut query {
        if sc.ship_type == ShipType::CommandModule {
            
            if is_detached {
                if let Ok(lm_child) = query_attached_lm.get_single() {
                    commands.entity(lm_child).despawn_recursive();
                    
                    // Spawn detailed independent LM
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
                    let silver = materials.add(StandardMaterial {
                        base_color: Color::srgb(0.85, 0.85, 0.88),
                        metallic: 0.95,
                        perceptual_roughness: 0.1,
                        ..default()
                    });
                    
                    commands.spawn((
                        PbrBundle {
                            mesh: meshes.add(Cuboid::new(0.6, 0.35, 0.6)),
                            material: gold_foil.clone(),
                            transform: Transform::from_translation(current_lm).with_scale(Vec3::splat(2.0)),
                            ..default()
                        },
                        Spacecraft { ship_type: ShipType::LunarModule },
                    )).with_children(|lm| {
                        // Ascent Stage
                        lm.spawn(PbrBundle {
                            mesh: meshes.add(Cuboid::new(0.4, 0.35, 0.4)),
                            material: gold_foil.clone(),
                            transform: Transform::from_xyz(0.0, 0.35, 0.0),
                            ..default()
                        });
                        // Windows
                        lm.spawn(PbrBundle {
                            mesh: meshes.add(Cuboid::new(0.42, 0.12, 0.08)),
                            material: dark_metal.clone(),
                            transform: Transform::from_xyz(0.0, 0.45, 0.22),
                            ..default()
                        });
                        // Descent Engine
                        lm.spawn(PbrBundle {
                            mesh: meshes.add(bevy::math::primitives::Cone { radius: 0.15, height: 0.2 }),
                            material: dark_metal.clone(),
                            transform: Transform::from_xyz(0.0, -0.27, 0.0)
                                .with_rotation(Quat::from_rotation_x(std::f32::consts::PI)),
                            ..default()
                        });
                        // Landing Legs + Pads
                        for angle in [0.0_f32, 90.0, 180.0, 270.0] {
                            let rad = angle.to_radians();
                            lm.spawn(PbrBundle {
                                mesh: meshes.add(Cuboid::new(0.03, 0.4, 0.03)),
                                material: silver.clone(),
                                transform: Transform::from_xyz(0.4 * rad.cos(), -0.35, 0.4 * rad.sin())
                                    .with_rotation(Quat::from_rotation_z(0.3 * if angle < 180.0 { 1.0 } else { -1.0 })),
                                ..default()
                            });
                            lm.spawn(PbrBundle {
                                mesh: meshes.add(Cylinder::new(0.06, 0.02)),
                                material: gold_foil.clone(),
                                transform: Transform::from_xyz(0.5 * rad.cos(), -0.55, 0.5 * rad.sin()),
                                ..default()
                            });
                        }
                    });
                }
            }

            // Look-ahead prediction using true distance (e.g. 1 unit ahead)
            let next_dist = (target_csm_dist + 1.0).min(total_csm_dist);
            let next_pos = sample_pos_by_distance(&paths.csm_points, &paths.csm_distances, next_dist);
            let direction = (next_pos - current_csm).normalize_or_zero();
            
            transform.translation = current_csm;
            if direction != Vec3::ZERO {
                transform.rotation = Quat::from_rotation_arc(Vec3::Y, direction);
            }


            if settings.tracking {
                if let Ok(mut cam) = cam_query.get_single_mut() {
                    // Smooth camera tracking with lerp
                    cam.target_focus = current_csm;
                    cam.focus = cam.focus.lerp(current_csm, 0.1);
                    
                    // Dynamic zoom: astronaut POV at launch → wide overview in deep space
                    // t=0.0: radius=18 (right on the capsule, Earth fills the view)
                    // t=0.1: radius=80 (pulling away from Earth orbit)
                    // t=0.5: radius=250 (full Earth-Moon transit view)
                    // t=1.0: radius=350 (maximum overview)
                    let zoom = 18.0 + t * t * 332.0; // Quadratic ease - slow start, fast pull-back
                    cam.target_radius = zoom;
                }
            }
        } else if sc.ship_type == ShipType::LunarModule {
            // Retrograde decent logic uses looking behind
            let prev_dist = (target_lm_dist - 1.0).max(0.0);
            let prev_pos = sample_pos_by_distance(&paths.lm_points, &paths.lm_distances, prev_dist);
            let direction = (current_lm - prev_pos).normalize_or_zero(); 
            
            transform.translation = current_lm;
            if direction != Vec3::ZERO {
                transform.rotation = Quat::from_rotation_arc(Vec3::Y, direction);
            }
        }
    }

    // ============================================
    // 3D REFERENCE GRID (aids spatial tracking)
    // ============================================
    let earth_pos = Vec3::new((X_E as f32) * DISTANCE_SCALE, 0.0, 0.0);
    let moon_pos = Vec3::new((X_M as f32) * DISTANCE_SCALE, 0.0, 0.0);
    let grid_center = (earth_pos + moon_pos) * 0.5;
    let grid_half = 200.0_f32; // Half-extent of grid
    let grid_spacing = 40.0_f32;
    let grid_color = Color::srgba(0.2, 0.3, 0.5, 0.15);
    
    // XZ plane grid lines (horizontal reference plane)
    let steps = (grid_half * 2.0 / grid_spacing) as i32;
    for i in 0..=steps {
        let offset = -grid_half + i as f32 * grid_spacing;
        // Lines along X
        gizmos.line(
            grid_center + Vec3::new(-grid_half, 0.0, offset),
            grid_center + Vec3::new(grid_half, 0.0, offset),
            grid_color,
        );
        // Lines along Z
        gizmos.line(
            grid_center + Vec3::new(offset, 0.0, -grid_half),
            grid_center + Vec3::new(offset, 0.0, grid_half),
            grid_color,
        );
    }
    
    // Axis lines through grid center (brighter, thicker feel via double draw)
    let axis_color = Color::srgba(0.3, 0.5, 0.8, 0.3);
    gizmos.line(grid_center + Vec3::new(-grid_half, 0.0, 0.0), grid_center + Vec3::new(grid_half, 0.0, 0.0), axis_color);
    gizmos.line(grid_center + Vec3::new(0.0, 0.0, -grid_half), grid_center + Vec3::new(0.0, 0.0, grid_half), axis_color);
    gizmos.line(grid_center + Vec3::new(0.0, -grid_half, 0.0), grid_center + Vec3::new(0.0, grid_half, 0.0), axis_color);

    // Earth-Moon line (direct reference)
    gizmos.line(earth_pos, moon_pos, Color::srgba(0.4, 0.4, 0.6, 0.2));

    // ============================================
    // DYNAMIC TRAJECTORY TRAILS
    // ============================================
    let mut current_csm_pts = Vec::new();
    for (i, &d) in paths.csm_distances.iter().enumerate() {
        if d <= target_csm_dist {
            current_csm_pts.push(paths.csm_points[i]);
        } else {
            break;
        }
    }
    current_csm_pts.push(current_csm);
    if current_csm_pts.len() > 1 {
        gizmos.linestrip(current_csm_pts, Color::srgb(0.0, 1.0, 0.8));
    }
    
    let mut current_lm_pts = Vec::new();
    for (i, &d) in paths.lm_distances.iter().enumerate() {
        if d <= target_lm_dist {
            current_lm_pts.push(paths.lm_points[i]);
        } else {
            break;
        }
    }
    current_lm_pts.push(current_lm);
    if current_lm_pts.len() > 1 {
        gizmos.linestrip(current_lm_pts, Color::srgb(1.0, 0.8, 0.0));
    }

    // ============================================
    // MODULE SEPARATION EVENT MARKERS  
    // ============================================
    // These mark the exact positions where key separation events happen
    struct SepEvent { t: f32, color: Color }
    let events = [
        SepEvent { t: 0.08, color: Color::srgb(1.0, 0.3, 0.3) },  // S-IVB jettison
        SepEvent { t: 0.15, color: Color::srgb(0.3, 1.0, 0.3) },  // LM extraction
        SepEvent { t: 0.35, color: Color::srgb(0.3, 0.6, 1.0) },  // LOI burn
        SepEvent { t: 0.50, color: Color::srgb(1.0, 1.0, 0.0) },  // LM undocking
        SepEvent { t: 0.55, color: Color::srgb(1.0, 0.5, 0.0) },  // Powered descent
        SepEvent { t: 0.60, color: Color::srgb(0.0, 1.0, 1.0) },  // Touchdown
        SepEvent { t: 0.70, color: Color::srgb(0.8, 0.3, 1.0) },  // TEI burn
    ];
    
    for ev in &events {
        let ev_dist = total_csm_dist * ev.t;
        let ev_pos = sample_pos_by_distance(&paths.csm_points, &paths.csm_distances, ev_dist);
        
        // Draw a small cross marker at each event point
        let arm = 3.0;
        gizmos.line(ev_pos + Vec3::X * arm, ev_pos - Vec3::X * arm, ev.color);
        gizmos.line(ev_pos + Vec3::Y * arm, ev_pos - Vec3::Y * arm, ev.color);
        gizmos.line(ev_pos + Vec3::Z * arm, ev_pos - Vec3::Z * arm, ev.color);
        
        // Draw a circle around the event point
        gizmos.circle(ev_pos, Dir3::Y, 4.0, ev.color);
    }
}

pub fn input_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut settings: ResMut<SimSettings>,
    mut cam_query: Query<&mut PanOrbitCamera>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        settings.paused = !settings.paused;
    }
    if keyboard_input.just_pressed(KeyCode::KeyT) {
        settings.tracking = !settings.tracking;
    }
    if keyboard_input.just_pressed(KeyCode::KeyR) {
        // Reset to Kennedy Space Center launchpad (astronaut POV)
        settings.mission_time = 0.0;
        settings.speed = 0.02;
        settings.paused = true;
        settings.tracking = true;
        
        if let Ok(mut cam) = cam_query.get_single_mut() {
            // Reset to astronaut-level zoom on the launch pad
            cam.radius = Some(18.0);
            cam.target_radius = 18.0;
        }
    }
    
    // Change speed
    if keyboard_input.pressed(KeyCode::ArrowUp) {
        settings.speed += 0.005;
        if settings.speed > 2.0 { settings.speed = 2.0; }
    }
    if keyboard_input.pressed(KeyCode::ArrowDown) {
        settings.speed -= 0.005;
        if settings.speed < -2.0 { settings.speed = -2.0; } // Support reverse play!
    }
}

pub fn update_ui_with_settings(
    query_sc: Query<&Spacecraft>,
    mut q_phase: Query<&mut Text, (With<PhaseText>, Without<TimeText>, Without<VelText>, Without<AltEText>, Without<AltMText>, Without<ControlText>)>,
    mut q_time: Query<&mut Text, (With<TimeText>, Without<PhaseText>, Without<VelText>, Without<AltEText>, Without<AltMText>, Without<ControlText>)>,
    mut q_vel: Query<&mut Text, (With<VelText>, Without<PhaseText>, Without<TimeText>, Without<AltEText>, Without<AltMText>, Without<ControlText>)>,
    mut q_alt_e: Query<&mut Text, (With<AltEText>, Without<PhaseText>, Without<TimeText>, Without<VelText>, Without<AltMText>, Without<ControlText>)>,
    mut q_alt_m: Query<&mut Text, (With<AltMText>, Without<PhaseText>, Without<TimeText>, Without<VelText>, Without<AltEText>, Without<ControlText>)>,
    mut q_ctrl: Query<&mut Text, (With<ControlText>, Without<PhaseText>, Without<TimeText>, Without<VelText>, Without<AltEText>, Without<AltMText>)>,
    settings: Res<SimSettings>,
    paths: Res<OrbitPaths>,
) {
    let mut csm_sc: Option<&Spacecraft> = None;
    for sc in &query_sc {
        if sc.ship_type == ShipType::CommandModule {
             csm_sc = Some(sc);
             break;
        }
    }
    
    if csm_sc.is_some() { 
        let t = settings.mission_time; 

        let hours = (t * 195.0).floor() as i32;
        let mins = ((t * 195.0 * 60.0).abs() % 60.0).floor() as i32;
        
        let earth_pos = Vec3::new((X_E as f32) * DISTANCE_SCALE, 0.0, 0.0);
        let moon_pos = Vec3::new((X_M as f32) * DISTANCE_SCALE, 0.0, 0.0);
        
        let total_dist = get_total_distance(&paths.csm_distances);
        let current_target = total_dist * t;
        let next_target = (total_dist * (t + 0.001)).min(total_dist);
        
        let current_csm = sample_pos_by_distance(&paths.csm_points, &paths.csm_distances, current_target);
        let next_csm = sample_pos_by_distance(&paths.csm_points, &paths.csm_distances, next_target);
        
        let v_world = current_csm.distance(next_csm) / 0.001;
        let v_km = (v_world / DISTANCE_SCALE) * 11.0; 

        let dist_e_real = (current_csm.distance(earth_pos) as f64 / DISTANCE_SCALE as f64) * L_STAR;
        let dist_m_real = (current_csm.distance(moon_pos) as f64 / DISTANCE_SCALE as f64) * L_STAR;
        
        // Accurate altitudes accounting for Planet Scale 30x and Distance Scale 200x
        let alt_e = (dist_e_real - (6371e3 * PLANET_SCALE as f64)).max(0.0) / 1000.0;
        let alt_m = (dist_m_real - (1737e3 * PLANET_SCALE as f64)).max(0.0) / 1000.0;
        
        // Add rudimentary comma grouping for hundreds of thousands of km
        fn format_km(val: f64) -> String {
            let mut s = format!("{:.0}", val);
            let len = s.len();
            if len > 3 { s.insert(len - 3, ','); }
            if len > 6 { s.insert(len - 6, ','); }
            s
        }

        let phase = if t < 0.1 { "Earth Parking" } else if t < 0.3 { "Translunar Coast" } else if t < 0.5 { "Lunar Orbit Insertion" } else if t < 0.6 { "LM Descent / Surface Ops" } else if t < 0.7 { "Ascent & TEI" } else if t < 0.95 { "Transearth Coast" } else { "Re-Entry & Splashdown" };

        if let Ok(mut text) = q_phase.get_single_mut() { text.sections[0].value = format!("Phase: {}", phase); }
        if let Ok(mut text) = q_time.get_single_mut() { text.sections[0].value = format!("T+ {:02}h {:02}m", hours, mins); }
        if let Ok(mut text) = q_vel.get_single_mut() { text.sections[0].value = format!("Velocity: {:.2} km/s", v_km); }
        if let Ok(mut text) = q_alt_e.get_single_mut() { text.sections[0].value = format!("Alt (Earth): {} km", format_km(alt_e)); }
        if let Ok(mut text) = q_alt_m.get_single_mut() { text.sections[0].value = format!("Alt (Moon): {} km", format_km(alt_m)); }
        if let Ok(mut text) = q_ctrl.get_single_mut() {
            let pause_str = if settings.paused { "PAUSED" } else { "LIVE" };
            let track_str = if settings.tracking { "ON" } else { "OFF" };
            text.sections[0].value = format!("SPACE: Play/Pause [{}] | T: Track [{}] | R: Reset | P: Screenshot | ARROWS: Speed [{:.2}X]", pause_str, track_str, settings.speed);
        }
    }
}

/// Automatically captures screenshots at key mission moments for portfolio use.
/// Also supports manual capture with the P key.
pub fn auto_screenshot_system(
    settings: Res<SimSettings>,
    mut tracker: ResMut<ScreenshotTracker>,
    main_window: Query<Entity, With<PrimaryWindow>>,
    mut screenshot_manager: ResMut<ScreenshotManager>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    // Portfolio-perfect mission moments
    let moments: [(f32, &str); 8] = [
        (0.01, "01_ksc_launch"),          // On the launchpad, Earth filling view
        (0.05, "02_tli_burn"),            // Engine glow during TLI
        (0.15, "03_translunar_coast"),    // Earth receding, deep space ahead
        (0.35, "04_lunar_approach"),      // Moon growing large ahead
        (0.45, "05_lunar_orbit"),         // Orbiting the Moon
        (0.52, "06_lm_undocking"),        // LM just separated from CSM
        (0.70, "07_tei_homeward"),        // TEI burn, heading home
        (0.95, "08_reentry_splashdown"),  // Approaching Earth for splashdown
    ];

    let t = settings.mission_time;
    let Ok(window) = main_window.get_single() else { return };

    // Auto-capture at mission moments
    for (i, (trigger_t, name)) in moments.iter().enumerate() {
        if i < tracker.captured.len() && !tracker.captured[i] {
            // Trigger within a small window around the target time
            if (t - trigger_t).abs() < 0.005 && !settings.paused {
                tracker.captured[i] = true;
                let path = format!("screenshots/{}.png", name);
                std::fs::create_dir_all("screenshots").ok();
                screenshot_manager.save_screenshot_to_disk(window, path.clone()).unwrap();
                info!("📸 Portfolio screenshot saved: {}", path);
            }
        }
    }

    // Manual screenshot with P key
    if keyboard_input.just_pressed(KeyCode::KeyP) {
        let timestamp = (t * 1000.0) as i32;
        let path = format!("screenshots/manual_t{:04}.png", timestamp);
        std::fs::create_dir_all("screenshots").ok();
        screenshot_manager.save_screenshot_to_disk(window, path.clone()).unwrap();
        info!("📸 Manual screenshot saved: {}", path);
    }
}
