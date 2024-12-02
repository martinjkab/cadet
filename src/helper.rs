use glam::DVec2;
use intersection_detection::{Intersection, IntersectionResult, Line, PointLike};

fn cross(v1: DVec2, v2: DVec2) -> f64 {
    v1.x * v2.y - v1.y * v2.x
}

pub fn is_crossing(e1: &(DVec2, DVec2), e2: &(DVec2, DVec2)) -> bool {
    intersection_point(e1, e2).is_some()
}

pub fn intersection_point(e1: &(DVec2, DVec2), e2: &(DVec2, DVec2)) -> Option<DVec2> {
    let (a, b) = e1;
    let (c, d) = e2;

    let line1 = Line::new([a.x, a.y], [b.x, b.y]);
    let line2 = Line::new([c.x, c.y], [d.x, d.y]);

    let computation = line1.intersection(&line2).try_into_intersection().ok();

    match computation {
        Some(Intersection::Point(p)) => Some(DVec2::new(p[0], p[1])),
        _ => None,
    }
}
