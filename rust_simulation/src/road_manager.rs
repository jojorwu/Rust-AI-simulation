use crate::road::{Road, Point};

pub struct RoadManager {
    pub roads: Vec<Road>,
}

impl RoadManager {
    pub fn new() -> Self {
        RoadManager {
            roads: Vec::new(),
        }
    }

    pub fn add_road(&mut self, road: Road) {
        self.roads.push(road);
    }

    /// Finds the nearest point on any road to a given point.
    /// Returns the road and the closest point on that road.
    pub fn find_nearest_road_point(&self, point: Point) -> Option<(&Road, Point)> {
        self.roads
            .iter()
            .filter_map(|road| {
                road.find_closest_point(point)
                    .map(|closest_point| (road, closest_point))
            })
            .min_by(|(_, a), (_, b)| {
                let dist_a = (a.x - point.x).powi(2) + (a.y - point.y).powi(2);
                let dist_b = (b.x - point.x).powi(2) + (b.y - point.y).powi(2);
                dist_a.partial_cmp(&dist_b).unwrap_or(std::cmp::Ordering::Equal)
            })
    }
}
