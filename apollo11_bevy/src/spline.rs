use bevy::prelude::*;

/// Compute cumulative arc-length distances for a sequence of 3D points.
/// Returns a Vec where distances[i] = total distance from points[0] to points[i].
pub fn calculate_arc_lengths(pts: &[Vec3]) -> Vec<f32> {
    let mut distances = Vec::with_capacity(pts.len());
    let mut current_dist = 0.0;
    distances.push(current_dist);

    for i in 1..pts.len() {
        current_dist += pts[i].distance(pts[i - 1]);
        distances.push(current_dist);
    }
    distances
}

/// Sample a position along a polyline by physical distance traveled.
/// Uses linear interpolation between the two bounding points.
pub fn sample_pos_by_distance(pts: &[Vec3], distances: &[f32], target_distance: f32) -> Vec3 {
    if target_distance <= 0.0 {
        return pts[0];
    }
    let max_dist = *distances.last().unwrap();
    if target_distance >= max_dist {
        return *pts.last().unwrap();
    }

    for i in 0..distances.len() - 1 {
        let d0 = distances[i];
        let d1 = distances[i + 1];

        if target_distance >= d0 && target_distance <= d1 {
            let seg_len = d1 - d0;
            if seg_len < f32::EPSILON {
                return pts[i];
            }
            let local_t = (target_distance - d0) / seg_len;
            return pts[i].lerp(pts[i + 1], local_t);
        }
    }

    *pts.last().unwrap()
}

/// Physics constraint: push any trajectory point outside of gravitational body radii.
/// Each point inside a body is projected radially outward to surface + min_altitude.
pub fn enforce_planet_clearance(
    points: &mut [Vec3],
    bodies: &[(Vec3, f32)],
    min_altitude: f32,
) {
    for pt in points.iter_mut() {
        for &(center, radius) in bodies {
            let to_point = *pt - center;
            let dist = to_point.length();
            let min_dist = radius + min_altitude;

            if dist < min_dist && dist > f32::EPSILON {
                *pt = center + to_point.normalize() * min_dist;
            }
        }
    }
}
