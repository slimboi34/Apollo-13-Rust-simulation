use bevy::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ShipType {
    CommandModule,
    LunarModule,
}

#[derive(Component)]
pub struct Spacecraft {
    pub ship_type: ShipType,
}

#[derive(Component)]
pub struct AttachedLM;

#[derive(Component)]
pub struct AttachedSLA;

#[derive(Component)]
pub struct AttachedSM;

#[derive(Component)]
pub struct EngineGlow;

#[derive(Resource)]
pub struct SimSettings {
    pub paused: bool,
    pub speed: f32,
    pub tracking: bool,
    pub mission_time: f32, // 0.0 to 1.0
}

// Struct to hold purely geometric splines (now to be indexed by Arc-Length)
#[derive(Resource)]
pub struct OrbitPaths {
    pub csm_points: Vec<Vec3>,
    pub lm_points: Vec<Vec3>,
    pub csm_distances: Vec<f32>,
    pub lm_distances: Vec<f32>,
}

pub fn get_total_distance(distances: &Vec<f32>) -> f32 {
    *distances.last().unwrap_or(&0.0)
}

// UI Tag Components
#[derive(Component)] pub struct PhaseText;
#[derive(Component)] pub struct TimeText;
#[derive(Component)] pub struct VelText;
#[derive(Component)] pub struct AltEText;
#[derive(Component)] pub struct AltMText;
#[derive(Component)] pub struct ControlText;
