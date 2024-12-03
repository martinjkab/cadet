use glam::DVec2;
use intersection_detection::{Intersection, Line};

pub fn is_crossing(e1: &(DVec2, DVec2), e2: &(DVec2, DVec2)) -> bool {
    match intersection_point(e1, e2) {
        Some(p) => {
            // Check if the intersection point is on the start or end of the edges
            // With a small epsilon
            let epsilon = 0.0001;
            let is_on_edge = |e: &(DVec2, DVec2), p: DVec2| {
                let (a, b) = e;
                let ap = p - a;
                let bp = p - b;

                ap.length() > epsilon && bp.length() > epsilon
            };

            is_on_edge(e1, p) && is_on_edge(e2, p)
        }
        _ => false,
    }
}

pub fn intersection_point(e1: &(DVec2, DVec2), e2: &(DVec2, DVec2)) -> Option<DVec2> {
    let (a, b) = e1;
    let (c, d) = e2;

    let line1 = Line::new([a.x, a.y], [b.x, b.y]);
    let line2 = Line::new([c.x, c.y], [d.x, d.y]);

    let computation = line1.intersection(&line2).try_into_intersection().ok();

    match computation {
        Some(Intersection::Point(point)) => Some(DVec2::new(point[0], point[1])),
        _ => None,
    }
}

enum IntersectionType {
    None,
    Collinear,
    Parallel,
    Point(DVec2),
}

fn cross_product(a: DVec2, b: DVec2) -> f64 {
    a.x * b.y - a.y * b.x
}

pub fn is_point_in_triangle(a: &DVec2, b: &DVec2, c: &DVec2, p: &DVec2) -> FaceLocateResult {
    // Compute vectors
    let ab = b - a;
    let bc = c - b;
    let ca = a - c;

    let ap = p - a;
    let bp = p - b;
    let cp = p - c;

    // Cross products
    let cross1 = cross_product(ab, ap);
    let cross2 = cross_product(bc, bp);
    let cross3 = cross_product(ca, cp);

    let epsilon = 0.0001;

    if cross1 > epsilon && cross2 > epsilon && cross3 > epsilon {
        FaceLocateResult::Face
    } else if cross1.abs() < epsilon && cross2.abs() < epsilon && cross3.abs() < epsilon {
        FaceLocateResult::Vertex
    } else if cross1.abs() < epsilon || cross2.abs() < epsilon || cross3.abs() < epsilon {
        FaceLocateResult::Edge
    } else {
        FaceLocateResult::None
    }
}

pub enum FaceLocateResult {
    Face,
    Edge,
    Vertex,
    None,
}
