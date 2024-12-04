use std::{cell::RefCell, collections::VecDeque, rc::Rc};

use glam::{DMat3, DVec2, DVec3};

use crate::{
    cdt::location::FastLocate,
    edge::Edge,
    symmetric_compare::{Flipped, SymmetricCompare},
    vertex::Vertex,
};

use super::cdt::CDT;

impl CDT {
    // Check if an edge is Delaunay using the in-circle test
    pub fn is_delaunay(a: DVec2, b: DVec2, c: DVec2, d: DVec2) -> bool {
        let matrix: DMat3 = DMat3::from_cols(
            DVec3::new(a.x - d.x, a.y - d.y, (a - d).dot(a - d)),
            DVec3::new(b.x - d.x, b.y - d.y, (b - d).dot(b - d)),
            DVec3::new(c.x - d.x, c.y - d.y, (c - d).dot(c - d)),
        );
        let det = matrix.determinant();

        det >= -1e-8
    }

    // Edge-flipping routine
    pub fn flip_edges(
        &mut self,
        p: Rc<RefCell<Vertex>>,
        edge_stack: &mut VecDeque<Rc<RefCell<Edge>>>,
    ) {
        while let Some(e) = edge_stack.pop_front() {
            {
                let e_borrowed = e.borrow();
                if e_borrowed.is_constrained() {
                    continue;
                }

                let sym_edge_rc = self.get_sym_edge_for_half_edge(&e_borrowed.edge_indices());

                let sym_edge_rc = match sym_edge_rc {
                    Some(sym_edge) => Some(sym_edge),
                    None => self.get_sym_edge_for_half_edge(&e_borrowed.edge_indices().flipped()),
                };

                let sym_edge_rc = match sym_edge_rc {
                    Some(sym_edge) => sym_edge,
                    None => {
                        continue;
                    }
                };

                let sym_edge = sym_edge_rc.borrow();

                let neighbor_face_rc = match sym_edge.neighbor_face() {
                    Some(face) => face,
                    None => {
                        continue;
                    }
                };

                let mut face_borrowed = sym_edge.face.borrow_mut();
                let mut neighbor_face = neighbor_face_rc.borrow_mut();

                let tri = geo::Triangle::new(
                    geo::Coord {
                        x: face_borrowed.vertices[0].borrow().position.x,
                        y: face_borrowed.vertices[0].borrow().position.y,
                    },
                    geo::Coord {
                        x: face_borrowed.vertices[1].borrow().position.x,
                        y: face_borrowed.vertices[1].borrow().position.y,
                    },
                    geo::Coord {
                        x: face_borrowed.vertices[2].borrow().position.x,
                        y: face_borrowed.vertices[2].borrow().position.y,
                    },
                );

                //If point is inside face, swap the faces
                if tri.locate_point(&p.borrow().position) {
                    std::mem::swap(&mut face_borrowed, &mut neighbor_face);
                }

                let o_vertex = neighbor_face.opposite_vertex(&e_borrowed);
                let o = o_vertex.borrow();
                let is_delanuay = Self::is_delaunay(
                    face_borrowed.vertices[0].borrow().position,
                    face_borrowed.vertices[1].borrow().position,
                    face_borrowed.vertices[2].borrow().position,
                    o.position,
                );

                if is_delanuay {
                    continue;
                }

                let different_edges = face_borrowed
                    .edges
                    .iter()
                    .filter(|x| !(**x).symmetric_compare(&e_borrowed.edge_indices()))
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
        let f2 = sym_edge.borrow().neighbor_face().unwrap();

        // Az e élt nem tartalmazó csúcsok mindkét háromszögben
        let v1 = f1.borrow().opposite_vertex(&edge.borrow());
        let v2 = f2.borrow().opposite_vertex(&edge.borrow());

        // Deleting the old faces
        self.remove_face(f1.clone());
        self.remove_face(f2.clone());

        // Create two completely new faces
        let f1 = self.add_face([v2.clone(), v1.clone(), edge.borrow().a.clone()]);

        let f2 = self.add_face([v1.clone(), v2.clone(), edge.borrow().b.clone()]);

        self.export_to_obj("./models/output.obj");

        //Waiting for user input
        // let mut input = String::new();
        // std::io::stdin().read_line(&mut input).unwrap();
    }
}
