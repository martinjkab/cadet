use std::{
    cell::RefCell,
    collections::{HashMap, HashSet, VecDeque},
    rc::Rc,
};

use glam::{DVec2, DVec4};

use rand::seq::IteratorRandom;

#[derive(Clone, PartialEq, Debug)]
struct Vertex {
    index: usize,       // Index of the vertex
    position: DVec2,    // Position of the vertex
    constraints: usize, // Number of constraints referencing this vertex
}

impl std::fmt::Display for Vertex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Vertex {{ index: {}, x: {}, y: {}, constraints: {} }}",
            self.index, self.position.x, self.position.y, self.constraints
        )
    }
}

#[derive(Debug)]
struct Edge {
    a: Rc<RefCell<Vertex>>,
    b: Rc<RefCell<Vertex>>,
    crep: HashSet<usize>, // Constraints represented by this edge
}

impl Edge {
    pub fn edge_indices(&self) -> (usize, usize) {
        (self.a.borrow().index, self.b.borrow().index)
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

#[derive(Clone, Debug)]
struct Face {
    id: usize,                          // Index of the face
    vertices: [Rc<RefCell<Vertex>>; 3], // Indices of the vertices
    edges: [Rc<RefCell<Edge>>; 3],      // Indices of the edges
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

#[derive(Debug, Default)]
struct CDT {
    vertices: Vec<Rc<RefCell<Vertex>>>,
    edges: Vec<Rc<RefCell<Edge>>>,
    faces: Vec<Rc<RefCell<Face>>>,
    sym_edges_by_edges: HashMap<(usize, usize), Rc<RefCell<SymEdge>>>,
    sym_edges_by_vertices: HashMap<usize, Vec<Rc<RefCell<SymEdge>>>>,
    constraints: HashMap<usize, Vec<usize>>,
}

/// Represents a SymEdge in the data structure
struct SymEdge {
    vertex: Rc<RefCell<Vertex>>,
    edge: Rc<RefCell<Edge>>,
    face: Rc<RefCell<Face>>,
    nxt: Option<Rc<RefCell<SymEdge>>>,
    rot: Option<Rc<RefCell<SymEdge>>>,
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

trait SymmetricCompare {
    fn symmetric_compare(&self, other: &Self) -> bool;
    fn inverse_compare(&self, other: &Self) -> bool;
}

impl SymmetricCompare for (usize, usize) {
    fn symmetric_compare(&self, other: &(usize, usize)) -> bool {
        self == other || self.inverse_compare(other)
    }

    fn inverse_compare(&self, other: &(usize, usize)) -> bool {
        self == &(other.1, other.0)
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

    fn pretty_print(&self) {
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

impl CDT {
    fn from_gltf(model_path: &str) -> Self {
        let scenes = easy_gltf::load(model_path).expect("Failed to load gltf file");
        let first_scene = scenes.into_iter().next().expect("No scenes in gltf file");
        let vertices = first_scene
            .models
            .iter()
            .flat_map(|m| m.vertices().clone())
            .map(|v| (v.position.extend(1.), v.normal))
            .enumerate()
            .map(|(i, v)| {
                let (p, _) = v;

                Vertex {
                    position: DVec2 {
                        x: p.x as f64,
                        y: p.z as f64,
                    },
                    index: i,
                    constraints: 0,
                }
            })
            .map(|v| Rc::new(RefCell::new(v)))
            .collect::<Vec<_>>();
        println!(
            "vertices: {:?}",
            vertices
                .iter()
                .map(|v| (
                    v.borrow().index,
                    (v.borrow().position.x, v.borrow().position.y)
                ))
                .collect::<Vec<_>>()
        );
        let indices = first_scene
            .models
            .iter()
            .flat_map(|m| m.indices())
            .flatten()
            .collect::<Vec<_>>();
        println!("indices: {:?}", indices);
        let mut edges = Vec::new();
        let faces: Vec<Rc<RefCell<Face>>> = indices
            .chunks(3)
            .enumerate()
            .map(|(i, c)| {
                let v = c.iter().map(|&x| *x as usize).collect::<Vec<_>>();

                let mut face_edges = Vec::new();
                for i in 0..3 {
                    let edge = Edge {
                        a: vertices[v[i]].clone(),
                        b: vertices[v[(i + 1) % 3]].clone(),
                        crep: HashSet::new(),
                    };
                    let edge = Rc::new(RefCell::new(edge));
                    face_edges.push(edge);
                }
                edges.extend(face_edges.iter().cloned());

                Face {
                    id: i,
                    vertices: [
                        vertices[v[0]].clone(),
                        vertices[v[1]].clone(),
                        vertices[v[2]].clone(),
                    ],
                    edges: [
                        face_edges[0].clone(),
                        face_edges[1].clone(),
                        face_edges[2].clone(),
                    ],
                }
            })
            .map(|f| Rc::new(RefCell::new(f)))
            .collect();
        println!("triangles: {:?}", faces);
        println!("edges: {:?}", edges);

        CDT {
            vertices,
            edges,
            faces,
            ..Default::default()
        }
    }

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

    fn build_sym_edges(&mut self) -> Result<(), String> {
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

    fn ccw(a: &Vertex, b: &Vertex, c: &Vertex) -> f64 {
        let ab = a.position - b.position;
        let ac = a.position - c.position;

        ab.x * ac.y - ab.y * ac.x
    }

    fn is_ccw(a: &Vertex, b: &Vertex, c: &Vertex) -> Orientation {
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

    fn is_point_on_edge(p: &Vertex, triangle: &Face) -> bool {
        for i in 0..3 {
            let a = triangle.vertices[i].borrow();
            let b = triangle.vertices[(i + 1) % 3].borrow();

            let is_ccw = Self::is_ccw(&a, &b, p);
            if is_ccw == Orientation::Collinear {
                return true;
            }
        }
        false
    }

    fn locate_point(&mut self, p: &Vertex) -> LocateResult {
        let epsilon = 1e-6;
        // Step 1: Jump - Select a random vertex sample and find the closest one
        let num_vertices = self.faces.len();
        let sample_size = (num_vertices as f64).powf(1.0 / 3.0).ceil() as usize;
        let mut rng = rand::thread_rng();

        let random_sample = self.faces.iter().choose_multiple(&mut rng, sample_size);

        let mut closest_face = random_sample[0];
        let mut min_distance = f64::MAX;

        for triangle in random_sample {
            for vertex in &triangle.borrow().vertices {
                let vertex = vertex.borrow();
                let distance = (vertex.position.x - p.position.x).powi(2)
                    + (vertex.position.y - p.position.y).powi(2);
                if distance < min_distance {
                    min_distance = distance;
                    closest_face = triangle;
                }
            }
        }

        println!("Closest face: {:?}", closest_face.borrow().vertex_indices());

        // Step 2: Walk - Oriented walk to locate p
        let mut visited = vec![false; self.faces.len()];

        loop {
            let vertices = closest_face.borrow().vertices.clone();
            let vertices = [
                vertices[0].borrow(),
                vertices[1].borrow(),
                vertices[2].borrow(),
            ];

            let centroid = Vertex {
                position: DVec2 {
                    x: (vertices[0].position.x + vertices[1].position.x + vertices[2].position.x)
                        / 3.0,
                    y: (vertices[0].position.y + vertices[1].position.y + vertices[2].position.y)
                        / 3.0,
                },
                index: 0,
                constraints: 0,
            };

            let mut selected_edge_index = None;

            // Find the edge that separates p and the centroid
            for (i, edge) in closest_face.borrow().edges.iter().enumerate() {
                let edge_ref = edge.clone();
                let edge_borrowed = edge_ref.borrow();
                let vertex = edge_borrowed.a.borrow();
                let next_vertex = edge_borrowed.b.borrow();

                let is_point_ccw = Self::is_ccw(&vertex, &next_vertex, p);
                let is_centroid_ccw = Self::is_ccw(&vertex, &next_vertex, &centroid);
                let is_separating_edge = is_point_ccw != is_centroid_ccw;

                println!(
                    "Edge: {:?}, is_point_ccw: {:?}, is_centroid_ccw: {:?}",
                    edge_borrowed.edge_indices(),
                    is_point_ccw,
                    is_centroid_ccw,
                );

                if is_point_ccw == Orientation::Collinear {
                    return LocateResult::Edge(edge.clone());
                }

                if is_separating_edge {
                    selected_edge_index = Some(i);
                    break;
                }
            }

            if let Some(edge_index) = selected_edge_index {
                // Move to the adjacent triangle across the selected edge
                if let Some(neighbor) =
                    self.find_neighboring_face(&closest_face.borrow(), edge_index)
                {
                    if visited[neighbor.borrow().id] {
                        // A loop is detected; fallback to epsilon-based checks
                        if Self::is_point_on_edge(p, &closest_face.borrow()) {
                            return LocateResult::Edge(
                                closest_face.borrow().edges[edge_index].clone(),
                            );
                        }
                        return LocateResult::Face(closest_face.clone());
                    }

                    visited[closest_face.borrow().id] = true;
                    closest_face = &self.faces[neighbor.borrow().id];
                } else {
                    return LocateResult::Face(closest_face.clone());
                }
            } else {
                // The point is inside the current triangle
                return LocateResult::Face(closest_face.clone());
            }
        }
    }

    fn find_neighboring_face(&self, face: &Face, edge_index: usize) -> Option<Rc<RefCell<Face>>> {
        // Get the SymEdge corresponding to the edge
        let edge = &face.edges[edge_index].borrow();
        let a_index = edge.a.borrow().index;
        let b_index = edge.b.borrow().index;
        let sym_edge = self.sym_edges_by_edges.get(&(a_index, b_index))?.borrow();

        sym_edge.neighbor_face()
    }

    // Check if an edge is Delaunay using the in-circle test
    fn is_delaunay(p: DVec2, a: DVec2, b: DVec2, o: DVec2) -> bool {
        let matrix = glam::DMat4::from_cols(
            DVec4::new(p.x, p.y, p.length_squared(), 1.0),
            DVec4::new(a.x, a.y, a.length_squared(), 1.0),
            DVec4::new(b.x, b.y, b.length_squared(), 1.0),
            DVec4::new(o.x, o.y, o.length_squared(), 1.0),
        );
        matrix.determinant() <= 0.0 // True if the point is not inside the circumcircle
    }

    // Edge-flipping routine
    fn flip_edges(&mut self, p: Rc<RefCell<Vertex>>, edge_stack: &mut VecDeque<Rc<RefCell<Edge>>>) {
        while let Some(e) = edge_stack.pop_front() {
            let e_borrowed = e.borrow();
            if !e_borrowed.crep.is_empty() {
                continue;
            }

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

                println!("Flipping edge: {:?}", e.borrow().edge_indices());
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

        // Update the faces
        f1.borrow_mut().replace_edge(edge.clone(), new_edge.clone());
        f2.borrow_mut().replace_edge(edge.clone(), new_edge.clone());

        println!(
            "new f1 vertices: {:?}, new f2 vertices: {:?}",
            f1.borrow().vertex_indices(),
            f2.borrow().vertex_indices()
        );
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

    fn insert_point_in_face(&mut self, v: Vertex, face: Rc<RefCell<Face>>) -> Rc<RefCell<Vertex>> {
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

#[derive(Debug)]
enum LocateResult {
    Vertex(Rc<RefCell<Vertex>>),
    Edge(Rc<RefCell<Edge>>),
    Face(Rc<RefCell<Face>>),
    None,
}

#[derive(Debug, PartialEq)]
enum Orientation {
    Clockwise,
    CounterClockwise,
    Collinear,
}

fn main() {
    std::env::set_var("RUST_BACKTRACE", "1");
    println!("{}", std::env::current_dir().unwrap().display());
    let mut cdt = CDT::from_gltf("C:/Projects/Study/cdt/models/model.glb");
    cdt.build_sym_edges().unwrap();
    // cdt.insert_point_on_edge(
    //     Vertex {
    //         position: DVec2 { x: 0.0, y: 0.0 },
    //         index: 0,
    //         constraints: 0,
    //     },
    //     cdt.edges
    //         .iter()
    //         .find(|e| e.borrow().a.borrow().index == 3)
    //         .unwrap()
    //         .clone(),
    // );
    cdt.insert_point_in_face(
        Vertex {
            position: DVec2 { x: 0.5, y: 0.5 },
            index: 0,
            constraints: 0,
        },
        cdt.faces[0].clone(),
    );
}
