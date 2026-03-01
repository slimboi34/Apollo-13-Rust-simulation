// Physics and scale constants for the Apollo 11 Simulation

pub const M_E: f64 = 5.97219e24;      // Mass of Earth (kg)
pub const M_M: f64 = 7.34767309e22;    // Mass of Moon (kg)

pub const M_STAR: f64 = M_E + M_M;
pub const L_STAR: f64 = 384400e3;      // Mean Earth-Moon distance (m)

pub const MU: f64 = M_M / M_STAR;      // Moon mass ratio

pub const X_E: f64 = -MU;              // Earth position in barycentric frame
pub const X_M: f64 = 1.0 - MU;         // Moon position in barycentric frame

pub const DISTANCE_SCALE: f32 = 350.0; // World units per L_STAR
pub const PLANET_SCALE: f32 = 30.0;    // Visual planet radius multiplier
