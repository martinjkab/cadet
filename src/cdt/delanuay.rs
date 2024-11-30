use std::{cell::RefCell, collections::VecDeque, rc::Rc};

use glam::{DMat4, DVec2, DVec4};

use crate::{edge::Edge, vertex::Vertex};

use super::cdt::CDT;

impl CDT {
    // Check if an edge is Delaunay using the in-circle test
    pub fn is_delaunay(p: DVec2, a: DVec2, b: DVec2, o: DVec2) -> bool {
        let matrix = DMat4::from_cols(
            DVec4::new(p.x, p.y, p.length_squared(), 1.0),
            DVec4::new(a.x, a.y, a.length_squared(), 1.0),
            DVec4::new(b.x, b.y, b.length_squared(), 1.0),
            DVec4::new(o.x, o.y, o.length_squared(), 1.0),
        );
        matrix.determinant() <= 0.0 // True if the point is not inside the circumcircle
    }

    // Edge-flipping routine
    pub fn flip_edges(
        &mut self,
        p: Rc<RefCell<Vertex>>,
        edge_stack: &mut VecDeque<Rc<RefCell<Edge>>>,
    ) {
        while let Some(e) = edge_stack.pop_front() {
            let e_borrowed = e.borrow();
            if !e_borrowed.crep.is_empty() {
                continue;
            }

            //If doesnt have neighbor face, skip
            let sym_edge = self
                .sym_edges_by_edges
                .get(&(e_borrowed.edge_indices()))
                .unwrap();

            // Get edge endpoints
            let a = e_borrowed.a.borrow();
            let b = e_borrowed.b.borrow();

            let sym_edge = self.sym_edges_by_edges.get(&(a.index, b.index)).unwrap();
            let face = sym_edge.borrow().face.clone();

            let o = face.borrow().opposite_vertex(&e_borrowed);
            let o = o.borrow();
            let is_delanuay =
                Self::is_delaunay(p.borrow().position, a.position, b.position, o.position);

            println!(
                "Checking edge: {:?}, face vertices: {:?}, is Delaunay: {}",
                e.borrow().edge_indices(),
                face.borrow().vertex_indices(),
                is_delanuay
            );

            if is_delanuay {
                continue; // Skip if the edge is already Delaunay
            }

            {
                let face_borrowed = face.borrow();
                let different_edges = face_borrowed
                    .edges
                    .iter()
                    .filter(|x| x.borrow().edge_indices() != (a.index, b.index))
                    .collect::<Vec<_>>();

                edge_stack.push_back(different_edges[0].clone());
                edge_stack.push_back(different_edges[1].clone());
            }

            self.flip_edge(e.clone());
        }
    }

    fn flip_edge(&mut self, edge: Rc<RefCell<Edge>>) {
        let sym_edge = self
            .sym_edges_by_edges
            .get(&(edge.borrow().edge_indices()))
            .unwrap();
        let sym_edge_borrowed = sym_edge.borrow();

        let f1 = sym_edge_borrowed.face.clone();
        let f2 = match sym_edge_borrowed.neighbor_face() {
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

        let a = edge.borrow().a.clone();
        let b = edge.borrow().b.clone();

        let new_edge = Edge {
            a: v1.clone(),
            b: v2.clone(),
            crep: edge.borrow().crep.clone(),
        };
        let new_edge = Rc::new(RefCell::new(new_edge));

        println!(
            "Flipping edge: {:?} to {:?}",
            edge.borrow().edge_indices(),
            new_edge.borrow().edge_indices()
        );

        println!(
            "new f1 vertices: {:?}, new f2 vertices: {:?}",
            f1.borrow().vertex_indices(),
            f2.borrow().vertex_indices()
        );
        println!(
            "new f1 edges: {:?}, new f2 edges: {:?}",
            f1.borrow().edge_indices(),
            f2.borrow().edge_indices()
        );
    }
}
