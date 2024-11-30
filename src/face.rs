use std::{cell::RefCell, rc::Rc};

use crate::{edge::Edge, vertex::Vertex};

#[derive(Clone, Debug)]
pub struct Face {
    pub id: usize,                          // Index of the face
    pub vertices: [Rc<RefCell<Vertex>>; 3], // Indices of the vertices
    pub edges: [Rc<RefCell<Edge>>; 3],      // Indices of the edges
}

impl Face {
    pub fn edge_indices(&self) -> [(usize, usize); 3] {
        [
            self.edges[0].borrow().edge_indices(),
            self.edges[1].borrow().edge_indices(),
            self.edges[2].borrow().edge_indices(),
        ]
    }

    pub fn vertex_indices(&self) -> [usize; 3] {
        [
            self.vertices[0].borrow().index,
            self.vertices[1].borrow().index,
            self.vertices[2].borrow().index,
        ]
    }

    pub fn opposite_vertex(&self, edge: &Edge) -> Rc<RefCell<Vertex>> {
        let edge = edge.edge_indices();
        self.vertices
            .iter()
            .find(|vertex| {
                let index = vertex.borrow().index;
                index != edge.0 && index != edge.1
            })
            .cloned()
            .expect("Edge not found in face")
    }

    pub fn replace_edge(&mut self, old_edge: Rc<RefCell<Edge>>, new_edge: Rc<RefCell<Edge>>) {
        for i in 0..3 {
            if self.edges[i].borrow().edge_indices() == old_edge.borrow().edge_indices() {
                self.edges[i] = new_edge.clone();
                break;
            }
        }

        // Update vertices
        let old_edge = old_edge.borrow();
        let new_edge = new_edge.borrow();

        for i in 0..3 {
            if self.vertices[i].borrow().index == old_edge.a.borrow().index {
                self.vertices[i] = new_edge.a.clone();
            } else if self.vertices[i].borrow().index == old_edge.b.borrow().index {
                self.vertices[i] = new_edge.b.clone();
            }
        }
    }
}
