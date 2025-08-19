use crate::config::RoadSetting;
use noise::{NoiseFn, Perlin};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug)]
pub struct Road {
    pub path: Vec<Point>,
    pub settings: RoadSetting,
}

pub fn generate_road(settings: RoadSetting, start: Point, end: Point) -> Road {
    let mut path = Vec::new();
    let perlin = Perlin::new(0); // Using a seed of 0 for deterministic generation

    let dx = end.x - start.x;
    let dy = end.y - start.y;
    let distance = (dx.powi(2) + dy.powi(2)).sqrt();
    let steps = (distance / 1.0).ceil() as u32;

    let main_direction_x = dx / distance;
    let main_direction_y = dy / distance;

    // Perpendicular direction
    let perp_direction_x = -main_direction_y;
    let perp_direction_y = main_direction_x;

    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        let current_pos_x = start.x + t * dx;
        let current_pos_y = start.y + t * dy;

        // Use a frequency factor for the noise to control the "busyness" of the curve.
        // A smaller value will make the curves longer and smoother.
        let noise_frequency = 0.1;
        let noise_input = i as f64 * noise_frequency;
        let noise_value = perlin.get([noise_input, 0.0]); // 2D noise, but we only need one dimension of change.

        // The displacement is perpendicular to the main path direction.
        // The magnitude of the displacement is controlled by the noise and the curvature setting.
        let displacement = noise_value as f32 * settings.curvature * 10.0; // Multiplier to make curvature effect more visible

        let point = Point {
            x: current_pos_x + perp_direction_x * displacement,
            y: current_pos_y + perp_direction_y * displacement,
        };
        path.push(point);
    }

    Road { path, settings }
}

impl Road {
    /// Finds the closest point on the road's path to a given point.
    pub fn find_closest_point(&self, point: Point) -> Option<Point> {
        self.path
            .iter()
            .min_by(|a, b| {
                let dist_a = (a.x - point.x).powi(2) + (a.y - point.y).powi(2);
                let dist_b = (b.x - point.x).powi(2) + (b.y - point.y).powi(2);
                dist_a.partial_cmp(&dist_b).unwrap_or(std::cmp::Ordering::Equal)
            })
            .copied()
    }
}
