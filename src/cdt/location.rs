use std::{cell::RefCell, rc::Rc};

use geo::Coord;
use glam::DVec2;

use crate::{face::Face, helper::is_ccw, locate_result::LocateResult, orientation::Orientation};

use super::cdt::CDT;

impl CDT {
    // pub fn locate_point(&self, p: &DVec2) -> LocateResult {
    //     // Step 1: Jump - Select a random vertex sample and find the closest one
    //     let num_vertices = self.faces.len();
    //     let sample_size = (num_vertices as f64).powf(1.0 / 3.0).ceil() as usize;
    //     let mut rng = rand::thread_rng();

    //     let random_sample = self.faces.iter().choose_multiple(&mut rng, sample_size);

    //     let mut closest_face = random_sample[0].clone();

    //     let mut min_distance = f64::MAX;

    //     for triangle in random_sample {
    //         for vertex in &triangle.borrow().vertices {
    //             let vertex = vertex.borrow();
    //             let distance =
    //                 (vertex.position.x - p.x).powi(2) + (vertex.position.y - p.y).powi(2);
    //             if distance < min_distance {
    //                 min_distance = distance;
    //                 closest_face = triangle.clone();
    //             }
    //         }
    //     }

    //     // Step 2: Walk - Oriented walk to locate p
    //     let mut visited = HashMap::<usize, bool>::new();

    //     loop {
    //         assert!(self.faces.iter().any(|f| Rc::ptr_eq(&f, &closest_face)));
    //         let vertices = closest_face.borrow().vertices.clone();
    //         let vertices = [
    //             vertices[0].borrow(),
    //             vertices[1].borrow(),
    //             vertices[2].borrow(),
    //         ];

    //         let centroid = DVec2 {
    //             x: (vertices[0].position.x + vertices[1].position.x + vertices[2].position.x) / 3.0,
    //             y: (vertices[0].position.y + vertices[1].position.y + vertices[2].position.y) / 3.0,
    //         };

    //         let mut selected_edge_index = None;

    //         // Find the edge that separates p and the centroid
    //         for (i, edge) in closest_face.borrow().edges.iter().enumerate() {
    //             let sym_edge = self.get_sym_edge_for_half_edge(edge).unwrap();
    //             let edge_borrowed = sym_edge.borrow();
    //             let a = edge_borrowed.a();
    //             let b = edge_borrowed.b();
    //             let vertex = a.borrow();
    //             let next_vertex = b.borrow();

    //             let is_point_ccw = Self::is_ccw(&vertex.position, &next_vertex.position, p);
    //             let is_centroid_ccw =
    //                 Self::is_ccw(&vertex.position, &next_vertex.position, &centroid);
    //             let is_separating_edge = is_point_ccw != is_centroid_ccw;

    //             if is_point_ccw == Orientation::Collinear {
    //                 return LocateResult::Edge(edge_borrowed.edge.clone());
    //             }

    //             if is_separating_edge {
    //                 selected_edge_index = Some(i);
    //                 break;
    //             }
    //         }

    //         if let Some(edge_index) = selected_edge_index {
    //             // Move to the adjacent triangle across the selected edge
    //             if let Some(neighbor) =
    //                 self.find_neighboring_face(&closest_face.clone().borrow(), edge_index)
    //             {
    //                 if visited.get(&neighbor.borrow().id).copied().unwrap_or(false) {
    //                     // A loop is detected; fallback to epsilon-based checks
    //                     if Self::is_point_on_edge(p, &closest_face.borrow()) {
    //                         let sym_edge = self.get_sym_edge_for_half_edge(
    //                             &closest_face.borrow().edges[edge_index],
    //                         );
    //                         let sym_edge = sym_edge.unwrap();
    //                         let edge = sym_edge.borrow().edge.clone();
    //                         return LocateResult::Edge(edge);
    //                     }
    //                     return LocateResult::Face(closest_face.clone());
    //                 }

    //                 visited.insert(closest_face.borrow().id, true);
    //                 closest_face = neighbor.clone();
    //             } else {
    //                 // Check if point lies outside the convex hull
    //                 let edge = &closest_face.borrow().edges[edge_index];
    //                 let edge = self.get_sym_edge_for_half_edge(edge).unwrap();
    //                 let edge_borrowed = edge.borrow();
    //                 let is_ccw = Self::is_ccw(
    //                     &edge_borrowed.a().borrow().position,
    //                     &edge_borrowed.b().borrow().position,
    //                     p,
    //                 ) == Orientation::CounterClockwise;
    //                 if is_ccw {
    //                     return LocateResult::None;
    //                 }
    //                 return LocateResult::Face(closest_face.clone());
    //             }
    //         } else {
    //             // The point is inside the current triangle
    //             return LocateResult::Face(closest_face.clone());
    //         }
    //     }
    // }

    pub fn locate_point(&self, p: &DVec2) -> LocateResult {
        //Brute force method
        for face in &self.faces {
            let vertices = face.borrow().vertices.clone();
            let vertices = [
                vertices[0].borrow(),
                vertices[1].borrow(),
                vertices[2].borrow(),
            ];

            let tri = geo::Triangle::new(
                Coord {
                    x: vertices[0].position.x,
                    y: vertices[0].position.y,
                },
                Coord {
                    x: vertices[1].position.x,
                    y: vertices[1].position.y,
                },
                Coord {
                    x: vertices[2].position.x,
                    y: vertices[2].position.y,
                },
            );

            let is_point_in_triangle = tri.locate_point(p);
            if is_point_in_triangle {
                for edge in &face.borrow().edge_indices() {
                    let sym_edge = self.get_sym_edge_for_half_edge(edge).unwrap();
                    let edge = sym_edge.borrow().edge.clone();

                    let a = edge.borrow().a.clone();
                    let b = edge.borrow().b.clone();

                    let pa = a.borrow().position.distance(*p);
                    let pb = b.borrow().position.distance(*p);
                    let ab = a.borrow().position.distance(b.borrow().position);

                    if pa + pb - ab < 1e-6 {
                        if pa < 0.0001 {
                            return LocateResult::Vertex(a);
                        }

                        if pb < 0.0001 {
                            return LocateResult::Vertex(b);
                        }
                        return LocateResult::Edge(edge);
                    }
                }
                return LocateResult::Face(face.clone());
            }
        }

        LocateResult::None
    }

    pub fn is_point_on_edge(p: &DVec2, triangle: &Face) -> bool {
        for i in 0..3 {
            let a = triangle.vertices[i].borrow();
            let b = triangle.vertices[(i + 1) % 3].borrow();

            let is_ccw = is_ccw(&a.position, &b.position, p);
            if is_ccw == Orientation::Collinear {
                return true;
            }
        }
        false
    }

    pub fn find_neighboring_face(
        &self,
        face: &Face,
        edge_index: usize,
    ) -> Option<Rc<RefCell<Face>>> {
        // Get the SymEdge corresponding to the edge
        let edge = &face.edge_indices()[edge_index];
        let binding = self.get_sym_edge_for_half_edge(edge)?;
        let sym_edge = binding.borrow();

        sym_edge.neighbor_face()
    }
}

pub trait FastLocate {
    fn locate_point(&self, p: &DVec2) -> bool;
}

impl<T> FastLocate for geo::Triangle<T>
where
    T: geo::GeoNum,
{
    fn locate_point(&self, coord: &DVec2) -> bool {
        let p0x = self.0.x.to_f64().unwrap();
        let p0y = self.0.y.to_f64().unwrap();
        let p1x = self.1.x.to_f64().unwrap();
        let p1y = self.1.y.to_f64().unwrap();
        let p2x = self.2.x.to_f64().unwrap();
        let p2y = self.2.y.to_f64().unwrap();

        let px = coord.x;
        let py = coord.y;

        let a = 0.5 * (-p1y * p2x + p0y * (-p1x + p2x) + p0x * (p1y - p2y) + p1x * p2y);

        let sign = a.signum();
        let epsilon = 1e-9;

        let s = (p0y * p2x - p0x * p2y + (p2y - p0y) * px + (p0x - p2x) * py) * sign;
        let t = (p0x * p1y - p0y * p1x + (p0y - p1y) * px + (p1x - p0x) * py) * sign;

        s > -epsilon && t > -epsilon && (s + t) < 2. * a * sign + epsilon
    }
}
