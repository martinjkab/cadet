use std::{
    cell::RefCell,
    collections::{HashMap, HashSet, VecDeque},
    rc::Rc,
};

use crate::{
    edge::Edge, face::Face, locate_result::LocateResult, orientation::Orientation,
    sym_edge::SymEdge, vertex::Vertex,
};

#[derive(Debug, Default)]
pub struct CDT {
    pub vertices: Vec<Rc<RefCell<Vertex>>>,
    pub edges: Vec<Rc<RefCell<Edge>>>,
    pub faces: Vec<Rc<RefCell<Face>>>,
    pub sym_edges_by_edges: HashMap<(usize, usize), Rc<RefCell<SymEdge>>>,
    pub sym_edges_by_vertices: HashMap<usize, Vec<Rc<RefCell<SymEdge>>>>,
    pub constraints: HashMap<usize, Vec<usize>>,
}

impl CDT {
    fn validate_faces(&mut self) -> Result<(), String> {
        for (face_idx, face_data) in self.faces.iter().enumerate() {
            if face_data.borrow().vertices.len() < 3 {
                return Err(format!("Face {} has fewer than 3 vertices", face_idx));
            }
        }
        Ok(())
    }

    fn build_symedges_for_face(&mut self, face: Rc<RefCell<Face>>) -> Result<(), String> {
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

    fn build_rot_pointers_for_vertex_sym_edges(&mut self, vertex: Rc<RefCell<Vertex>>) {
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

        println!(
            "Angle to SymEdges ({:?}): {:?}",
            vertex.borrow().index,
            angle_to_sym_edges
                .iter()
                .map(|(a, b)| (a, b.borrow().edge.borrow().edge_indices()))
                .collect::<Vec<_>>()
        );

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

        println!("SymEdges:");
        for sym_edge in self.sym_edges_by_edges.values() {
            sym_edge.borrow().pretty_print();
        }

        Ok(())
    }

    fn insert_constraint(
        &mut self,
        constraint_points: Vec<Vertex>, // List of points in the constraint
        constraint_id: usize,           // ID of the constraint
    ) {
        let mut vertex_list = Vec::new();

        for point in constraint_points.iter() {
            // Step 1: Locate the point in the triangulation
            let locate_result = self.locate_point(point);

            // Step 2: Handle the locate result
            let vertex = match locate_result {
                LocateResult::Vertex(v) => v,
                LocateResult::Edge(edge) => self.insert_point_on_edge(point.clone(), edge),
                LocateResult::Face(face) => self.insert_point_in_face(point.clone(), face),
                LocateResult::None => {
                    continue;
                }
            };

            // Step 3: Add the vertex to the list
            vertex_list.push(vertex);
        }

        // // Step 4: Insert segments between successive vertices
        // for i in 0..vertex_list.len() - 1 {
        //     let v = vertex_list[i].clone();
        //     let vs = vertex_list[i + 1].clone();
        //     Self::insert_segment(v, vs, constraint_id);
        // }
    }

    pub fn ccw(a: &Vertex, b: &Vertex, c: &Vertex) -> f64 {
        let ab = a.position - b.position;
        let ac = a.position - c.position;

        ab.x * ac.y - ab.y * ac.x
    }

    pub fn is_ccw(a: &Vertex, b: &Vertex, c: &Vertex) -> Orientation {
        println!("CCW: {:?}, {:?}, {:?}, {:?}", a, b, c, Self::ccw(a, b, c));
        let ccw = Self::ccw(a, b, c);
        let distance = ccw.abs()
            / ((b.position.x - a.position.x).powi(2) + (b.position.y - a.position.y).powi(2))
                .sqrt();
        if distance < 1e-6 {
            return Orientation::Collinear;
        }
        if ccw > 0.0 {
            return Orientation::CounterClockwise;
        }
        Orientation::Clockwise
    }

    fn insert_point_on_edge(
        &mut self,
        point: Vertex,
        edge: Rc<RefCell<Edge>>,
    ) -> Rc<RefCell<Vertex>> {
        let edge = edge.borrow();
        let a = edge.a.borrow();
        let b = edge.b.borrow();

        let ab = b.position - a.position;
        let ap = point.position - a.position;

        let t = ap.dot(ab) / ab.dot(ab);
        let t = t.clamp(0.0, 1.0);

        let v = Vertex {
            position: a.position + ab * t,
            index: 0,
            constraints: 0,
        };

        let v = Rc::new(RefCell::new(v));

        // Set the crep list of the two created sub edges of e to be orig
        let orig = edge.crep.clone();
        let edge1 = Edge {
            a: edge.a.clone(),
            b: v.clone(),
            crep: orig.clone(),
        };
        let edge2 = Edge {
            a: v.clone(),
            b: edge.b.clone(),
            crep: orig.clone(),
        };

        let edge1 = Rc::new(RefCell::new(edge1));
        let edge2 = Rc::new(RefCell::new(edge2));

        let sym_edge = self.sym_edges_by_edges.get(&(a.index, b.index)).unwrap();
        let face_1 = sym_edge.borrow().face.clone();
        let face_2 = sym_edge.borrow().neighbor_face().unwrap();

        // These edges are the outlines of the face_1 and face_2 (does not include the shared edge)
        let face_1_edges = face_1.borrow().edges.clone();
        let face_2_edges = face_2.borrow().edges.clone();
        let mut all_edges = vec![
            face_1_edges[0].clone(),
            face_1_edges[1].clone(),
            face_1_edges[2].clone(),
            face_2_edges[0].clone(),
            face_2_edges[1].clone(),
            face_2_edges[2].clone(),
        ];

        println!(
            "All edges: {:?}",
            all_edges
                .iter()
                .map(|x| x.borrow().edge_indices())
                .collect::<Vec<_>>()
        );

        //Remove the shared edge
        all_edges.retain(|x| {
            x.borrow().edge_indices() != (a.index, b.index)
                && x.borrow().edge_indices() != (b.index, a.index)
        });

        println!(
            "All edges after removal: {:?}",
            all_edges
                .iter()
                .map(|x| x.borrow().edge_indices())
                .collect::<Vec<_>>()
        );

        let mut edge_stack = VecDeque::new();
        edge_stack.extend(all_edges);

        self.flip_edges(v.clone(), &mut edge_stack);

        v
    }

    pub fn insert_point_in_face(
        &mut self,
        v: Vertex,
        face: Rc<RefCell<Face>>,
    ) -> Rc<RefCell<Vertex>> {
        //New vertex
        let v = Vertex {
            position: v.position,
            index: self.vertices.len(),
            constraints: 1,
        };
        let v = Rc::new(RefCell::new(v));
        self.vertices.push(v.clone());
        {
            // New edges
            let face_borrowed = face.borrow();

            // New faces
            let faces_count = self.faces.len();
            let new_faces: Vec<_> = face_borrowed
                .edges
                .iter()
                .enumerate()
                .map(|(i, edge)| {
                    let vertices = [edge.borrow().a.clone(), edge.borrow().b.clone(), v.clone()];
                    let mut edges = (1..3)
                        .map(|i| {
                            let a = vertices[i].clone();
                            let b = vertices[(i + 1) % 3].clone();
                            let edge = Edge {
                                a: a.clone(),
                                b: b.clone(),
                                crep: HashSet::new(),
                            };
                            Rc::new(RefCell::new(edge))
                        })
                        .collect::<Vec<_>>();
                    edges.insert(0, edge.clone());
                    self.edges.extend(edges.clone());

                    Face {
                        id: faces_count + i,
                        vertices,
                        edges: [edges[0].clone(), edges[1].clone(), edges[2].clone()],
                    }
                })
                .map(|face| Rc::new(RefCell::new(face)))
                .collect();

            println!(
                "New faces: {:?}",
                new_faces
                    .iter()
                    .map(|x| x.borrow().edge_indices())
                    .collect::<Vec<_>>()
            );

            self.faces.extend(new_faces.clone());
            self.faces.retain(|x| x.borrow().id != face_borrowed.id);

            for face in new_faces.iter() {
                self.build_symedges_for_face(face.clone()).unwrap();
            }

            for face in new_faces.iter() {
                let vertices = face.borrow().vertices.clone();
                for vertex in vertices.iter() {
                    self.build_rot_pointers_for_vertex_sym_edges(vertex.clone());
                }
            }
        }

        println!("SymEdges:");
        let mut sym_edges = self.sym_edges_by_edges.values().collect::<Vec<_>>();
        sym_edges.sort_by(|a, b| {
            a.borrow()
                .face
                .borrow()
                .vertex_indices()
                .cmp(&b.borrow().face.borrow().vertex_indices())
                .then(
                    a.borrow()
                        .edge
                        .borrow()
                        .edge_indices()
                        .cmp(&b.borrow().edge.borrow().edge_indices()),
                )
        });
        for sym_edge in sym_edges {
            sym_edge.borrow().pretty_print();
        }

        let mut edge_stack = VecDeque::new();
        edge_stack.extend(face.borrow().edges.clone());

        self.flip_edges(v.clone(), &mut edge_stack);

        v
    }

    // fn insert_segment(v: Rc<RefCell<Vertex>>, vs: Rc<RefCell<Vertex>>, constraint_id: usize) {}
}
