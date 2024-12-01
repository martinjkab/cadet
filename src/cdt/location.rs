use std::{cell::RefCell, collections::HashMap, rc::Rc};

use glam::DVec2;
use rand::seq::IteratorRandom;

use crate::{face::Face, locate_result::LocateResult, orientation::Orientation};

use super::cdt::CDT;

impl CDT {
    pub fn locate_point(&self, p: &DVec2) -> LocateResult {
        // Step 1: Jump - Select a random vertex sample and find the closest one
        let num_vertices = self.faces.len();
        let sample_size = (num_vertices as f64).powf(1.0 / 3.0).ceil() as usize;
        let mut rng = rand::thread_rng();

        let random_sample = self.faces.iter().choose_multiple(&mut rng, sample_size);

        let mut closest_face = random_sample[0].clone();
        let mut min_distance = f64::MAX;

        for triangle in random_sample {
            for vertex in &triangle.borrow().vertices {
                let vertex = vertex.borrow();
                let distance =
                    (vertex.position.x - p.x).powi(2) + (vertex.position.y - p.y).powi(2);
                if distance < min_distance {
                    min_distance = distance;
                    closest_face = triangle.clone();
                }
            }
        }

        // Step 2: Walk - Oriented walk to locate p
        let mut visited = HashMap::<usize, bool>::new();

        loop {
            let vertices = closest_face.borrow().vertices.clone();
            let vertices = [
                vertices[0].borrow(),
                vertices[1].borrow(),
                vertices[2].borrow(),
            ];

            let centroid = DVec2 {
                x: (vertices[0].position.x + vertices[1].position.x + vertices[2].position.x) / 3.0,
                y: (vertices[0].position.y + vertices[1].position.y + vertices[2].position.y) / 3.0,
            };

            let mut selected_edge_index = None;

            // Find the edge that separates p and the centroid
            for (i, edge) in closest_face.borrow().edges.iter().enumerate() {
                let edge_ref = edge.clone();
                let edge_borrowed = edge_ref.borrow();
                let vertex = edge_borrowed.a.borrow();
                let next_vertex = edge_borrowed.b.borrow();

                let is_point_ccw = Self::is_ccw(&vertex.position, &next_vertex.position, p);
                let is_centroid_ccw =
                    Self::is_ccw(&vertex.position, &next_vertex.position, &centroid);
                let is_separating_edge = is_point_ccw != is_centroid_ccw;

                if is_point_ccw == Orientation::Collinear {
                    return LocateResult::Edge(edge.clone());
                }

                if is_separating_edge {
                    selected_edge_index = Some(i);
                    break;
                }
            }

            if let Some(edge_index) = selected_edge_index {
                // Move to the adjacent triangle across the selected edge
                if let Some(neighbor) =
                    self.find_neighboring_face(&closest_face.clone().borrow(), edge_index)
                {
                    if visited.get(&neighbor.borrow().id).copied().unwrap_or(false) {
                        // A loop is detected; fallback to epsilon-based checks
                        if Self::is_point_on_edge(p, &closest_face.borrow()) {
                            return LocateResult::Edge(
                                closest_face.borrow().edges[edge_index].clone(),
                            );
                        }
                        return LocateResult::Face(closest_face.clone());
                    }

                    visited.insert(closest_face.borrow().id, true);
                    closest_face = neighbor.clone();
                } else {
                    // Check if point lies outside the convex hull
                    let edge = &closest_face.borrow().edges[edge_index];
                    let edge_borrowed = edge.borrow();
                    let is_ccw = Self::is_ccw(
                        &edge_borrowed.a.borrow().position,
                        &edge_borrowed.b.borrow().position,
                        p,
                    ) == Orientation::CounterClockwise;
                    if is_ccw {
                        return LocateResult::None;
                    }
                    return LocateResult::Face(closest_face.clone());
                }
            } else {
                // The point is inside the current triangle
                return LocateResult::Face(closest_face.clone());
            }
        }
    }

    pub fn is_point_on_edge(p: &DVec2, triangle: &Face) -> bool {
        for i in 0..3 {
            let a = triangle.vertices[i].borrow();
            let b = triangle.vertices[(i + 1) % 3].borrow();

            let is_ccw = Self::is_ccw(&a.position, &b.position, p);
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
        let edge = &face.edges[edge_index].borrow();
        let a_index = edge.a.borrow().index;
        let b_index = edge.b.borrow().index;
        let sym_edge = self.sym_edges_by_edges.get(&(a_index, b_index))?.borrow();

        sym_edge.neighbor_face()
    }
}
