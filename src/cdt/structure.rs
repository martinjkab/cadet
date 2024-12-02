use std::{cell::RefCell, rc::Rc};

use glam::DVec2;

use crate::{
    edge::Edge,
    face::Face,
    sym_edge::SymEdge,
    symmetric_compare::{Flipped, TupleOrdered},
    vertex::Vertex,
};

use super::cdt::CDT;

impl CDT {
    pub fn add_face(&mut self, vertices: [Rc<RefCell<Vertex>>; 3]) -> Rc<RefCell<Face>> {
        let edges = (0..vertices.len())
            .map(|i| {
                let a = vertices[i].borrow().index;
                let b = vertices[(i + 1) % vertices.len()].clone().borrow().index;
                (a, b)
            })
            .collect::<Vec<_>>();

        let face = Rc::new(RefCell::new(Face {
            id: self.face_id_counter,
            vertices,
            edges: [edges[0].clone(), edges[1].clone(), edges[2].clone()],
        }));

        edges.iter().for_each(|edge| {
            let index = self
                .edges
                .binary_search_by(|x| x.borrow().edge_indices().ordered().cmp(&edge.ordered()));

            match index {
                Err(index) => {
                    let edge = Rc::new(RefCell::new(Edge {
                        a: self.vertices[edge.0].clone(),
                        b: self.vertices[edge.1].clone(),
                        crep: Default::default(),
                    }));

                    self.edges.insert(index, edge.clone());
                }
                _ => {}
            }
        });

        self.build_symedges_for_face(face.clone()).unwrap();

        face.borrow()
            .vertex_indices()
            .iter()
            .for_each(|&vertex_idx| {
                self.build_rot_pointers_for_vertex_sym_edges(self.vertices[vertex_idx].clone());
            });

        self.faces.push(face.clone());

        self.face_id_counter += 1;

        face.clone()
    }

    pub fn remove_face(&mut self, face: Rc<RefCell<Face>>) {
        let face_borrowed = face.borrow();

        let len_before = self.faces.len();

        // Remove face from faces
        self.faces.retain(|x| x.as_ptr() != face.as_ptr());

        assert_eq!(self.faces.len(), len_before - 1);

        // Remove face from sym_edges_by_edges
        for edge in face_borrowed.edges.iter() {
            let to_remove = self.get_sym_edge_for_half_edge(&edge).unwrap();

            self.remove_sym_edge(to_remove);
        }
    }

    pub fn add_vertex(&mut self, position: DVec2, constraints: usize) -> Rc<RefCell<Vertex>> {
        let vertex = Vertex {
            position,
            index: self.vertices.len(),
            constraints,
        };
        let vertex = Rc::new(RefCell::new(vertex));
        self.vertices.push(vertex.clone());
        vertex
    }

    pub fn build_symedges_for_face(&mut self, face: Rc<RefCell<Face>>) -> Result<(), String> {
        let mut face_symedges = Vec::new();

        for (i, edge) in face.borrow().edges.iter().enumerate() {
            let vertex = face.borrow().vertices[i].clone();
            let edge = self
                .edges
                .iter()
                .find(|x| x.borrow().edge_indices().ordered() == edge.ordered())
                .unwrap()
                .clone();
            let sym = Rc::new(RefCell::new(SymEdge {
                vertex: vertex.clone(),
                edge: edge.clone(),
                face: face.clone(),
                nxt: None,
                rot: None,
            }));
            println!("Adding sym edge: {:?}", sym.borrow().edge_indices());
            face_symedges.push(sym.clone());

            self.sym_edges_by_vertices
                .entry(vertex.borrow().index)
                .or_default()
                .push(sym.clone());

            self.sym_edges_by_half_edges
                .insert(sym.borrow().edge_indices(), sym.clone());
        }

        for i in 0..3 {
            let nxt = face_symedges[(i + 1) % 3].clone();
            face_symedges[i].borrow_mut().nxt = Some(nxt);
        }

        Ok(())
    }

    pub fn get_sym_edge_for_half_edge(
        &self,
        edge: &(usize, usize),
    ) -> Option<Rc<RefCell<SymEdge>>> {
        self.sym_edges_by_half_edges.get(&edge).cloned()
    }

    pub fn get_all_sym_edges_for_edge(&self, edge: Rc<RefCell<Edge>>) -> Vec<Rc<RefCell<SymEdge>>> {
        [
            self.get_sym_edge_for_half_edge(&edge.borrow().edge_indices()),
            self.get_sym_edge_for_half_edge(&edge.borrow().edge_indices().flipped()),
        ]
        .iter()
        .filter_map(|x| x.clone())
        .collect()
    }

    pub fn remove_sym_edge(&mut self, sym_edge: Rc<RefCell<SymEdge>>) {
        let edge = sym_edge.borrow().edge.clone();
        {
            let sym_edge_borrowed = sym_edge.borrow();
            let edge_indices = sym_edge_borrowed.edge_indices();
            let vertex_index = sym_edge_borrowed.vertex.borrow().index;

            self.sym_edges_by_half_edges.remove(&edge_indices);

            let all_symedges_for_edge =
                self.get_all_sym_edges_for_edge(sym_edge_borrowed.edge.clone());

            if all_symedges_for_edge.is_empty() {
                let len_before = self.edges.len();
                self.edges
                    .retain(|x| x.borrow().edge_indices().ordered() != edge_indices.ordered());
                assert_eq!(self.edges.len(), len_before - 1);
            }

            let vertex_entry = self.sym_edges_by_vertices.get_mut(&vertex_index).unwrap();

            let len_before = vertex_entry.len();

            vertex_entry.retain(|x| !Rc::ptr_eq(x, &sym_edge));

            assert_eq!(vertex_entry.len(), len_before - 1);

            if vertex_entry.is_empty() {
                self.sym_edges_by_vertices.remove(&vertex_index);
            }
        }

        //Update rot pointers
        self.build_rot_pointers_for_vertex_sym_edges(edge.borrow().a.clone());
        self.build_rot_pointers_for_vertex_sym_edges(edge.borrow().b.clone());

        let which_is_rot_of_this = self
            .sym_edges_by_half_edges
            .iter()
            .filter(|(_, sym)| {
                let sym = sym.borrow();
                let rot = sym.rot.clone();
                match rot {
                    Some(rot) => Rc::ptr_eq(&rot, &sym_edge),
                    None => false,
                }
            })
            .collect::<Vec<_>>();

        let is_any_rot_of_this = which_is_rot_of_this.clone().len() > 0;

        println!("Removing sym edge: {:?}", sym_edge.borrow().edge_indices());

        assert!(!is_any_rot_of_this);
    }

    pub fn build_rot_pointers_for_vertex_sym_edges(&mut self, vertex: Rc<RefCell<Vertex>>) {
        let sym_edges = self.sym_edges_by_vertices.get(&vertex.borrow().index);
        let sym_edges = match sym_edges {
            Some(sym_edges) => sym_edges,
            None => return,
        };
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
            angle_to_sym_edges[0].1.borrow_mut().rot = None;
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
