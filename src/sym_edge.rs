use std::{cell::RefCell, rc::Rc};

use crate::{edge::Edge, face::Face, symmetric_compare::SymmetricCompare, vertex::Vertex};

/// Represents a SymEdge in the data structure
pub struct SymEdge {
    pub vertex: Rc<RefCell<Vertex>>,
    pub edge: Rc<RefCell<Edge>>,
    pub face: Rc<RefCell<Face>>,
    pub nxt: Option<Rc<RefCell<SymEdge>>>,
    pub rot: Option<Rc<RefCell<SymEdge>>>,
}

impl std::fmt::Debug for SymEdge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "SymEdge {{ vertex: {:?}, edge: {:?}, face: {:?}, nxt: {:?}, rot: {:?} }}",
            self.vertex.borrow(),
            self.edge.borrow(),
            self.face,
            self.nxt.clone().map(|nxt| nxt.borrow().bare_to_string()),
            self.rot.clone().map(|rot| rot.borrow().bare_to_string())
        )
    }
}

impl SymEdge {
    pub fn bare_to_string(&self) -> String {
        format!(
            "SymEdge {{ vertex: {:?}, edge: {:?}, face: {:?}}}",
            self.vertex.borrow(),
            self.edge.borrow(),
            self.face,
        )
    }

    pub fn neighbor(&self) -> Option<Rc<RefCell<SymEdge>>> {
        let neighbor = self.nxt.clone()?.borrow().rot.clone()?;
        let edge_indices = self.edge.borrow().edge_indices();
        let neighbor_edge_indices = neighbor.borrow().edge.borrow().edge_indices();
        let is_inverse = edge_indices.symmetric_compare(&neighbor_edge_indices);
        if is_inverse {
            return Some(neighbor);
        }
        None
    }

    pub fn neighbor_face(&self) -> Option<Rc<RefCell<Face>>> {
        Some(self.neighbor()?.borrow().face.clone())
    }

    pub fn pretty_print(&self) {
        // Create a table
        let mut table = prettytable::Table::new();

        // Set table titles
        table.add_row(prettytable::Row::new(vec![
            prettytable::Cell::new("Field"),
            prettytable::Cell::new("Value"),
        ]));

        // Add each field in the struct as a table row
        table.add_row(prettytable::Row::new(vec![
            prettytable::Cell::new("vertex"),
            prettytable::Cell::new(&format!("{:?}", &self.vertex.borrow().index)),
        ]));

        table.add_row(prettytable::Row::new(vec![
            prettytable::Cell::new("edge"),
            prettytable::Cell::new(&format!("{:?}", &self.edge.borrow().edge_indices())),
        ]));

        table.add_row(prettytable::Row::new(vec![
            prettytable::Cell::new("face"),
            prettytable::Cell::new(&format!("{:?}", self.face.borrow().vertex_indices())),
        ]));

        match &self.nxt {
            Some(nxt) => table.add_row(prettytable::Row::new(vec![
                prettytable::Cell::new("nxt"),
                prettytable::Cell::new(&format!("{:?}", nxt.borrow().edge.borrow().edge_indices())),
            ])),
            None => table.add_row(prettytable::Row::new(vec![
                prettytable::Cell::new("nxt"),
                prettytable::Cell::new("None"),
            ])),
        };

        match &self.rot {
            Some(rot) => table.add_row(prettytable::Row::new(vec![
                prettytable::Cell::new("rot"),
                prettytable::Cell::new(&format!("{:?}", rot.borrow().edge.borrow().edge_indices())),
            ])),
            None => table.add_row(prettytable::Row::new(vec![
                prettytable::Cell::new("rot"),
                prettytable::Cell::new("None"),
            ])),
        };

        let neighbor = self.neighbor();
        if let Some(neighbor) = neighbor {
            table.add_row(prettytable::Row::new(vec![
                prettytable::Cell::new("neighbor"),
                prettytable::Cell::new(&format!(
                    "{:?}",
                    neighbor.borrow().edge.borrow().edge_indices()
                )),
            ]));
        };

        // Print the table
        table.printstd();
    }
}
