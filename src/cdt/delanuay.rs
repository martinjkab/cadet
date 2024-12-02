use std::{cell::RefCell, collections::VecDeque, rc::Rc};

use glam::{DMat3, DVec2, DVec3};

use crate::{edge::Edge, sym_edge, symmetric_compare::TupleOrdered};

use super::cdt::CDT;

impl CDT {
    // Check if an edge is Delaunay using the in-circle test
    fn is_delaunay(a: DVec2, b: DVec2, c: DVec2, d: DVec2) -> bool {
        let matrix: DMat3 = DMat3::from_cols(
            DVec3::new(a.x - d.x, a.y - d.y, (a - d).dot(a - d)),
            DVec3::new(b.x - d.x, b.y - d.y, (b - d).dot(b - d)),
            DVec3::new(c.x - d.x, c.y - d.y, (c - d).dot(c - d)),
        );
        let det = matrix.determinant();
        det >= 0.0 // True if the point is not inside the circumcircle
    }

    // Edge-flipping routine
    pub fn flip_edges(&mut self, edge_stack: &mut VecDeque<Rc<RefCell<Edge>>>) {
        while let Some(e) = edge_stack.pop_front() {
            {
                println!("Flipping edge: {:?}", e.borrow().edge_indices());
                let e_borrowed = e.borrow();
                if !e_borrowed.crep.is_empty() {
                    continue;
                }

                let sym_edge_rc = self.get_sym_edge_for_half_edge(&e_borrowed.edge_indices());

                let sym_edge_rc = match sym_edge_rc {
                    Some(sym_edge) => sym_edge,
                    None => continue,
                };

                let sym_edge = sym_edge_rc.borrow();

                let face = sym_edge.face.borrow();

                let neighbor_face = match sym_edge.neighbor_face() {
                    Some(face) => face,
                    None => continue,
                };
                let neighbor_face = neighbor_face.borrow();

                let o_vertex = neighbor_face.opposite_vertex(&e_borrowed);
                let o = o_vertex.borrow();
                let is_delanuay = Self::is_delaunay(
                    face.vertices[0].borrow().position,
                    face.vertices[1].borrow().position,
                    face.vertices[2].borrow().position,
                    o.position,
                );

                if is_delanuay {
                    continue; // Skip if the edge is already Delaunay
                }

                let different_edges = face
                    .edges
                    .iter()
                    .filter(|x| **x != e_borrowed.edge_indices())
                    .map(|x| self.get_sym_edge_for_half_edge(x).unwrap())
                    .map(|x| x.borrow().edge.clone())
                    .collect::<Vec<_>>();

                assert_eq!(different_edges.len(), 2);

                edge_stack.push_back(different_edges[0].clone());
                edge_stack.push_back(different_edges[1].clone());
            }

            self.flip_edge(e.clone());
        }
    }

    fn flip_edge(&mut self, edge: Rc<RefCell<Edge>>) {
        let sym_edge = self
            .get_sym_edge_for_half_edge(&edge.borrow().edge_indices())
            .unwrap();

        let f1 = sym_edge.borrow().face.clone();
        let f2 = match sym_edge.borrow().neighbor_face() {
            Some(face) => face,
            None => return,
        };

        // Az e élt nem tartalmazó csúcsok mindkét háromszögben
        let v1 = f1.borrow().opposite_vertex(&edge.borrow());
        let v2 = f2.borrow().opposite_vertex(&edge.borrow());

        // Deleting the old faces
        self.remove_face(f1.clone());
        self.remove_face(f2.clone());

        // Create two completely new faces
        self.add_face([v2.clone(), v1.clone(), edge.borrow().a.clone()]);

        self.add_face([v1.clone(), v2.clone(), edge.borrow().b.clone()]);
    }
}
