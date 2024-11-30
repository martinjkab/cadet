use std::{cell::RefCell, rc::Rc};

use crate::{edge::Edge, face::Face, vertex::Vertex};

#[derive(Debug)]
pub enum LocateResult {
    Vertex(Rc<RefCell<Vertex>>),
    Edge(Rc<RefCell<Edge>>),
    Face(Rc<RefCell<Face>>),
    None,
}
