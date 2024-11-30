use std::{cell::RefCell, collections::VecDeque, rc::Rc};

use glam::{DMat3, DVec2, DVec3};

use crate::{edge::Edge, vertex::Vertex};

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
    pub fn flip_edges(
        &mut self,
        p: Rc<RefCell<Vertex>>,
        edge_stack: &mut VecDeque<Rc<RefCell<Edge>>>,
    ) {
        while let Some(e) = edge_stack.pop_front() {
            {
                let e_borrowed = e.borrow();
                if !e_borrowed.crep.is_empty() {
                    continue;
                }

                //If doesnt have neighbor face, skip
                let sym_edge = self
                    .sym_edges_by_edges
                    .get(&(e_borrowed.edge_indices()))
                    .unwrap()
                    .borrow();

                let face = sym_edge.face.borrow();

                let neighbor_face = match sym_edge.neighbor_face() {
                    Some(face) => face,
                    None => continue,
                };
                let neighbor_face = neighbor_face.borrow();

                let o = neighbor_face
                    .vertices
                    .iter()
                    .find(|&x| {
                        let x = x.borrow();
                        x.index != e_borrowed.a.borrow().index
                            && x.index != e_borrowed.b.borrow().index
                    })
                    .unwrap()
                    .borrow();
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
                    .filter(|x| x.borrow().edge_indices() != e_borrowed.edge_indices())
                    .collect::<Vec<_>>();

                edge_stack.push_back(different_edges[0].clone());
                edge_stack.push_back(different_edges[1].clone());
            }

            self.flip_edge(e.clone());
        }
    }

    fn flip_edge(&mut self, edge: Rc<RefCell<Edge>>) {
        let edge_indices = edge.borrow().edge_indices();
        let sym_edge = self.sym_edges_by_edges.get(&edge_indices).unwrap();

        let f1 = sym_edge.borrow().face.clone();
        let f2 = match sym_edge.borrow().neighbor_face() {
            Some(face) => face,
            None => return,
        };

        println!(
            "f1 vertices: {:?}, f2 vertices: {:?}",
            f1.borrow().vertex_indices(),
            f2.borrow().vertex_indices()
        );
        println!(
            "f1 edges: {:?}, f2 edges: {:?}",
            f1.borrow().edge_indices(),
            f2.borrow().edge_indices()
        );

        // Az e élt nem tartalmazó csúcsok mindkét háromszögben
        let v1 = f1.borrow().opposite_vertex(&edge.borrow());
        let v2 = f2.borrow().opposite_vertex(&edge.borrow());

        let new_edge = Edge {
            a: v1.clone(),
            b: v2.clone(),
            crep: edge.borrow().crep.clone(),
        };
        let new_edge = Rc::new(RefCell::new(new_edge));
        let new_edge_borrowed = new_edge.borrow();

        println!(
            "Flipping edge: {:?} to {:?}",
            edge.borrow().edge_indices(),
            new_edge_borrowed.edge_indices()
        );

        // Deleting the old faces
        self.remove_face(f1.clone());
        self.remove_face(f2.clone());

        // Create two completely new faces
        let new_f1 = self.add_triangle([
            new_edge_borrowed.b.clone(),
            new_edge_borrowed.a.clone(),
            edge.borrow().a.clone(),
        ]);

        let new_f2 = self.add_triangle([
            new_edge_borrowed.a.clone(),
            new_edge_borrowed.b.clone(),
            edge.borrow().b.clone(),
        ]);

        println!(
            "new f1 vertices: {:?}, new f2 vertices: {:?}",
            new_f1.borrow().vertex_indices(),
            new_f2.borrow().vertex_indices()
        );
        println!(
            "new f1 edges: {:?}, new f2 edges: {:?}",
            new_f1.borrow().edge_indices(),
            new_f2.borrow().edge_indices()
        );
    }
}
