use std::{cell::RefCell, collections::HashSet, rc::Rc};

use crate::vertex::Vertex;

#[derive(Debug)]
pub struct Edge {
    pub a: Rc<RefCell<Vertex>>,
    pub b: Rc<RefCell<Vertex>>,
    pub crep: HashSet<usize>, // Constraints represented by this edge
}

impl Edge {
    pub fn edge_indices(&self) -> (usize, usize) {
        (self.a.borrow().index, self.b.borrow().index)
    }

    pub fn insert_constraint(&mut self, constraint: usize) {
        self.crep.insert(constraint);
    }

    pub fn is_constrained(&self) -> bool {
        !self.crep.is_empty()
    }
}

impl std::fmt::Display for Edge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Edge {{ a: {}, b: {}, crep: {:?} }}",
            self.a.borrow().index,
            self.b.borrow().index,
            self.crep
        )
    }
}
