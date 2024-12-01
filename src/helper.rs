use glam::DVec2;

fn cross(v1: DVec2, v2: DVec2) -> f64 {
    v1.x * v2.y - v1.y * v2.x
}

pub fn is_crossing(e1: &(DVec2, DVec2), e2: &(DVec2, DVec2)) -> bool {
    intersection_point(e1, e2).is_some()
}

pub fn intersection_point(e1: &(DVec2, DVec2), e2: &(DVec2, DVec2)) -> Option<DVec2> {
    let (a, b) = e1;
    let (c, d) = e2;

    let ab = b - a;
    let ac = c - a;
    let ad = d - a;
    let cd = d - c;
    let ca = a - c;

    let d1 = cross(ab, ac);
    let d2 = cross(ab, ad);
    let d3 = cross(cd, ca);

    if d1 * d2 < 0.0 && d1 * d2 < 0.0 {
        let t = d1 / (d1 - d2);
        let u = d3 / (d3 - cross(cd, ad));

        if t >= 0.0 && t <= 1.0 && u >= 0.0 && u <= 1.0 {
            return Some(a + ab * t);
        }
    }

    None
}
