fn derivatives(state: &[f64; 6]) -> [f64; 6] {
    let G: f64 = 6.67430e-11;
    let M_E: f64 = 5.972e24;
    let M_M: f64 = 7.348e22;
    let D_EM: f64 = 3.844e8;
    let M_STAR = M_E + M_M;
    let MU = M_M / M_STAR;
    let MU_E = 1.0 - MU;
    let X_E = -MU;
    let X_M = 1.0 - MU;

    let x = state[0]; let y = state[1]; let z = state[2];
    let vx = state[3]; let vy = state[4]; let vz = state[5];

    let re_sq = (x - X_E)*(x - X_E) + y*y + z*z;
    let rm_sq = (x - X_M)*(x - X_M) + y*y + z*z;
    let re = re_sq.sqrt(); let rm = rm_sq.sqrt();

    let ax = 2.0*vy + x - MU_E*(x - X_E)/(re*re_sq) - MU*(x - X_M)/(rm*rm_sq);
    let ay = -2.0*vx + y - MU_E*y/(re*re_sq) - MU*y/(rm*rm_sq);
    let az = -MU_E*z/(re*re_sq) - MU*z/(rm*rm_sq);

    [vx, vy, vz, ax, ay, az]
}

fn rk4_step(mut state: [f64; 6], dt: f64) -> [f64; 6] {
    let k1 = derivatives(&state);
    let mut s2 = [0.0; 6];
    for i in 0..6 { s2[i] = state[i] + 0.5 * dt * k1[i]; }
    let k2 = derivatives(&s2);
    let mut s3 = [0.0; 6];
    for i in 0..6 { s3[i] = state[i] + 0.5 * dt * k2[i]; }
    let k3 = derivatives(&s3);
    let mut s4 = [0.0; 6];
    for i in 0..6 { s4[i] = state[i] + dt * k3[i]; }
    let k4 = derivatives(&s4);
    for i in 0..6 { state[i] += (dt / 6.0) * (k1[i] + 2.0*k2[i] + 2.0*k3[i] + k4[i]); }
    state
}

fn main() {
    let G: f64 = 6.67430e-11;
    let M_E: f64 = 5.972e24;
    let M_M: f64 = 7.348e22;
    let L_STAR: f64 = 3.844e8;
    let M_STAR = M_E + M_M;
    let MU = M_M / M_STAR;
    let X_E = -MU;
    let X_M = 1.0 - MU;

    let t_star_sq = (L_STAR * L_STAR * L_STAR) / (G * M_STAR);
    let t_star = t_star_sq.sqrt();

    let mut best_angle = 0.0;
    let mut best_v = 0.0;
    let mut best_dist = f64::MAX;

    let dt = 0.0005;

    for a_deg in (-90..90).step_by(5) {
        let angle = (a_deg as f64) * std::f64::consts::PI / 180.0;
        let r0 = (6371e3 + 200e3) / L_STAR;
        
        for v_km in vec![10.85, 10.9, 10.91, 10.92, 10.93, 10.95, 11.0] {
            let v0 = (v_km * 1e3 * t_star) / L_STAR;
            
            let mut state = [
                X_E + r0 * angle.cos(), r0 * angle.sin(), 0.0,
                v0 * (-angle.sin()), v0 * angle.cos(), 0.0
            ];

            let mut min_m_dist = f64::MAX;
            let mut min_e_return = f64::MAX;
            
            for step in 0..200000 {
                state = rk4_step(state, dt);
                
                let dist_m = ((state[0]-X_M).powi(2) + state[1].powi(2)).sqrt() * L_STAR;
                if dist_m < min_m_dist { min_m_dist = dist_m; }
                
                // If we passed the moon (reached min dist) and are coming back...
                if step > 20000 && min_m_dist < 200_000e3 {
                    let dist_e = ((state[0]-X_E).powi(2) + state[1].powi(2)).sqrt() * L_STAR;
                    if dist_e < min_e_return { min_e_return = dist_e; }
                }

                if ((state[0]-X_E).powi(2) + state[1].powi(2)).sqrt() * L_STAR > 500_000e3 {
                    break; // escaped
                }
            }

            // Ideal free return: pass moon close (e.g. 50k - 100k) and return to earth (< 20k)
            if min_m_dist < 100_000e3 && min_m_dist > 1737e3 && min_e_return < 100_000e3 {
                println!("GOOD TLI: Angle {} deg, V = {} km/s -> Min Moon Dist: {:.0} km, Return Earth Dist: {:.0} km", 
                    a_deg, v_km, min_m_dist/1000.0, min_e_return/1000.0);
                if min_m_dist < best_dist {
                    best_dist = min_m_dist;
                    best_angle = angle;
                    best_v = v_km;
                }
            }
        }
    }
    println!("BEST: Angle {}, V {}", best_angle, best_v);
}
