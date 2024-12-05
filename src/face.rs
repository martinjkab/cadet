use std::{cell::RefCell, rc::Rc};

use crate::{edge::Edge, vertex::Vertex};

#[derive(Clone, Debug)]
pub struct Face {
    pub id: usize,
    pub vertices: [Rc<RefCell<Vertex>>; 3],
}

impl Face {
    pub fn edges(&self) -> [(Rc<RefCell<Vertex>>, Rc<RefCell<Vertex>>); 3] {
        [
            (self.vertices[0].clone(), self.vertices[1].clone()),
            (self.vertices[1].clone(), self.vertices[2].clone()),
            (self.vertices[2].clone(), self.vertices[0].clone()),
        ]
    }
    pub fn edge_indices(&self) -> [(usize, usize); 3] {
        [
            (
                self.vertices[0].borrow().index,
                self.vertices[1].borrow().index,
            ),
            (
                self.vertices[1].borrow().index,
                self.vertices[2].borrow().index,
            ),
            (
                self.vertices[2].borrow().index,
                self.vertices[0].borrow().index,
            ),
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
}

pub trait ToIndices<T> {
    fn to_indices(&self) -> T;
}

impl ToIndices<(usize, usize)> for (Rc<RefCell<Vertex>>, Rc<RefCell<Vertex>>) {
    fn to_indices(&self) -> (usize, usize) {
        (self.0.borrow().index, self.1.borrow().index)
    }
}
