use std::{cell::RefCell, rc::Rc};

use crate::{edge::Edge, sym_edge::SymEdge, vertex::Vertex};

#[derive(Clone, Debug)]
pub struct Face {
    pub id: usize,
    pub vertices: [Rc<RefCell<Vertex>>; 3],
    pub edges: [(usize, usize); 3],
}

impl Face {
    pub fn edge_indices(&self) -> [(usize, usize); 3] {
        self.edges
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
}
