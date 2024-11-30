use std::{cell::RefCell, rc::Rc};

use crate::{edge::Edge, face::Face, sym_edge::SymEdge, vertex::Vertex};

use super::cdt::CDT;

impl CDT {
    pub fn add_triangle(&mut self, vertices: [Rc<RefCell<Vertex>>; 3]) -> Rc<RefCell<Face>> {
        let edges = (0..vertices.len())
            .map(|i| {
                let a = vertices[i].clone();
                let b = vertices[(i + 1) % vertices.len()].clone();
                let edge = Edge {
                    a: a.clone(),
                    b: b.clone(),
                    crep: std::collections::HashSet::new(),
                };
                Rc::new(RefCell::new(edge))
            })
            .collect::<Vec<_>>();

        let face = Rc::new(RefCell::new(Face {
            id: self.faces.len(),
            vertices,
            edges: [edges[0].clone(), edges[1].clone(), edges[2].clone()],
        }));

        self.build_symedges_for_face(face.clone()).unwrap();

        face.borrow()
            .vertex_indices()
            .iter()
            .for_each(|&vertex_idx| {
                self.build_rot_pointers_for_vertex_sym_edges(self.vertices[vertex_idx].clone());
            });

        self.faces.push(face.clone());

        face.clone()
    }

    pub fn remove_face(&mut self, face: Rc<RefCell<Face>>) {
        let face = face.borrow();
        let face_edges = face.edge_indices();
        let face_vertices = face.vertex_indices();

        // Remove face from faces
        self.faces.retain(|x| x.borrow().id != face.id);

        // Remove face from sym_edges_by_edges
        for edge in face_edges.iter() {
            self.sym_edges_by_edges.remove_entry(edge);
        }

        // Remove face from sym_edges_by_vertices
        for vertex in face_vertices.iter() {
            self.sym_edges_by_vertices
                .get_mut(vertex)
                .unwrap()
                .retain(|x| x.borrow().face.borrow().id != face.id);
        }
    }

    pub fn build_symedges_for_face(&mut self, face: Rc<RefCell<Face>>) -> Result<(), String> {
        let mut face_symedges = Vec::new();

        for (i, edge) in face.borrow().edges.iter().enumerate() {
            let vertex = face.borrow().vertices[i].clone();

            // Create a new SymEdge
            let sym = Rc::new(RefCell::new(SymEdge {
                vertex: vertex.clone(),
                edge: edge.clone(),
                face: face.clone(),
                nxt: None,
                rot: None,
            }));
            face_symedges.push(sym.clone());

            // Track SymEdges per vertex
            self.sym_edges_by_vertices
                .entry(vertex.borrow().index)
                .or_default()
                .push(sym.clone());

            // Track SymEdges per edge
            self.sym_edges_by_edges
                .entry((
                    edge.borrow().a.borrow().index,
                    edge.borrow().b.borrow().index,
                ))
                .insert_entry(sym.clone());
        }

        // Link `nxt` pointers within the face
        for i in 0..3 {
            let nxt = face_symedges[(i + 1) % 3].clone();
            face_symedges[i].borrow_mut().nxt = Some(nxt);
        }

        Ok(())
    }

    pub fn build_rot_pointers_for_vertex_sym_edges(&mut self, vertex: Rc<RefCell<Vertex>>) {
        let sym_edges = self
            .sym_edges_by_vertices
            .get(&vertex.borrow().index)
            .unwrap();
        let mut angle_to_sym_edges = std::vec::Vec::new();
        //Iterate over the symmetrical edges
        for sym in sym_edges {
            let sym_ref = sym.borrow();
            let edge = sym_ref.edge.borrow();
            let other_vertex = if edge.a.borrow().index == vertex.borrow().index {
                edge.b.borrow()
            } else {
                edge.a.borrow()
            };

            let angle = (other_vertex.position - vertex.borrow().position).to_angle();

            angle_to_sym_edges.push((angle, sym.clone()));
        }

        //Sort by angle
        angle_to_sym_edges.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());

        if angle_to_sym_edges.len() < 2 {
            return;
        }

        //Link the `rot` pointers to the first sym that is counter-clockwise
        for i in 0..angle_to_sym_edges.len() {
            let current = angle_to_sym_edges[i].1.clone();
            let next = angle_to_sym_edges[(i + 1) % angle_to_sym_edges.len()]
                .1
                .clone();

            current.borrow_mut().rot = Some(next.clone());
        }
    }

    pub fn build_sym_edges(&mut self) -> Result<(), String> {
        self.validate_faces()?;

        // Create SymEdges for each face
        let faces = self.faces.to_vec();
        for face_ref in faces {
            self.build_symedges_for_face(face_ref)?;
        }

        // Step 2: Link `rot` pointers between symmetrical SymEdges
        // based on the vertex they share
        let vertices = self
            .sym_edges_by_vertices
            .keys()
            .map(|index| self.vertices[*index].clone())
            .collect::<Vec<_>>();

        for vertex in vertices {
            self.build_rot_pointers_for_vertex_sym_edges(vertex);
        }

        Ok(())
    }

    fn validate_faces(&mut self) -> Result<(), String> {
        for (face_idx, face_data) in self.faces.iter().enumerate() {
            if face_data.borrow().vertices.len() < 3 {
                return Err(format!("Face {} has fewer than 3 vertices", face_idx));
            }
        }
        Ok(())
    }
}
