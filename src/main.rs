use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
};

use rand::seq::IteratorRandom;

#[derive(Clone, PartialEq, Debug)]
struct Vertex {
    index: usize, // Index of the vertex
    x: f64,
    y: f64,
    constraints: usize, // Number of constraints referencing this vertex
}

impl std::fmt::Display for Vertex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Vertex {{ index: {}, x: {}, y: {}, constraints: {} }}",
            self.index, self.x, self.y, self.constraints
        )
    }
}

#[derive(Debug)]
struct Edge {
    a: Rc<RefCell<Vertex>>,
    b: Rc<RefCell<Vertex>>,
    crep: HashSet<usize>, // Constraints represented by this edge
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
    face: Option<Rc<RefCell<Face>>>,
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

impl SymEdge {
    pub fn bare_to_string(&self) -> String {
        format!(
            "SymEdge {{ vertex: {:?}, edge: {:?}, face: {:?}}}",
            self.vertex.borrow(),
            self.edge.borrow(),
            self.face,
        )
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
            prettytable::Cell::new(&self.vertex.borrow().to_string()),
        ]));

        table.add_row(prettytable::Row::new(vec![
            prettytable::Cell::new("edge"),
            prettytable::Cell::new(&self.edge.borrow().to_string()),
        ]));

        match &self.face {
            Some(face) => table.add_row(prettytable::Row::new(vec![
                prettytable::Cell::new("face"),
                prettytable::Cell::new(&face.borrow().id.to_string()),
            ])),
            None => table.add_row(prettytable::Row::new(vec![
                prettytable::Cell::new("face"),
                prettytable::Cell::new("None"),
            ])),
        };

        match &self.nxt {
            Some(nxt) => table.add_row(prettytable::Row::new(vec![
                prettytable::Cell::new("nxt"),
                prettytable::Cell::new(&nxt.borrow().bare_to_string()),
            ])),
            None => table.add_row(prettytable::Row::new(vec![
                prettytable::Cell::new("nxt"),
                prettytable::Cell::new("None"),
            ])),
        };

        match &self.rot {
            Some(rot) => table.add_row(prettytable::Row::new(vec![
                prettytable::Cell::new("rot"),
                prettytable::Cell::new(&rot.borrow().bare_to_string()),
            ])),
            None => table.add_row(prettytable::Row::new(vec![
                prettytable::Cell::new("rot"),
                prettytable::Cell::new("None"),
            ])),
        };

        match &self.nxt {
            Some(nxt) => {
                match &self.rot {
                    Some(_) => {
                        let neighbor = nxt.borrow().rot.clone().unwrap();
                        table.add_row(prettytable::Row::new(vec![
                            prettytable::Cell::new("neighbor"),
                            prettytable::Cell::new(&neighbor.borrow().bare_to_string()),
                        ]));
                    }
                    _ => (),
                };
            }
            None => (),
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
                    x: p.x as f64,
                    y: p.y as f64,
                    index: i,
                    constraints: 0,
                }
            })
            .map(|v| Rc::new(RefCell::new(v)))
            .collect::<Vec<_>>();
        println!("vertices: {:?}", vertices);
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
                edges.extend(face_edges.iter().map(|e| e.clone()));

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

    fn build_sym_edges(&mut self) -> Result<(), String> {
        self.validate_faces()?;

        // Create SymEdges for each face
        for face_ref in self.faces.iter() {
            let face = face_ref.borrow();
            let mut face_symedges = Vec::new();
            // let mut face_symedges_other = Vec::new();

            for (i, &ref edge) in face.edges.iter().enumerate() {
                let vertex = face.vertices[i].clone();

                // Create a new SymEdge
                let sym = Rc::new(RefCell::new(SymEdge {
                    vertex: vertex.clone(),
                    edge: edge.clone(),
                    face: Some(face_ref.clone()),
                    nxt: None,
                    rot: None,
                }));
                face_symedges.push(sym.clone());

                // Track SymEdges per vertex
                self.sym_edges_by_vertices
                    .entry(vertex.borrow().index)
                    .or_insert_with(Vec::new)
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
        }

        // Step 2: Link `rot` pointers between symmetrical SymEdges
        // based on the vertex they share

        for (index, sym_edges) in self.sym_edges_by_vertices.iter() {
            let vertex = self.vertices[*index].borrow();
            let mut angle_to_sym_edges = std::vec::Vec::new();
            //Iterate over the symmetrical edges
            for sym in sym_edges {
                let sym_ref = sym.borrow();
                let edge = sym_ref.edge.borrow();
                let other_vertex = if edge.a.borrow().index == *index {
                    edge.b.borrow()
                } else {
                    edge.a.borrow()
                };

                let angle = (other_vertex.y - vertex.y).atan2(other_vertex.x - vertex.x);

                angle_to_sym_edges.push((angle, sym.clone()));
            }

            //Sort by angle
            angle_to_sym_edges.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

            //Link the `rot` pointers to the first sym that is counter-clockwise
            for i in 0..angle_to_sym_edges.len() {
                let current = angle_to_sym_edges[i].1.clone();
                let next = angle_to_sym_edges[(i + 1) % angle_to_sym_edges.len()]
                    .1
                    .clone();
                current.borrow_mut().rot = Some(next.clone());
            }
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
                LocateResult::Edge(edge) => Self::insert_point_on_edge(point.clone(), edge),
                LocateResult::Face(face) => Self::insert_point_in_face(point.clone(), face),
                LocateResult::None => {
                    continue;
                }
            };

            // Step 3: Add the vertex to the list
            vertex_list.push(vertex);
        }

        // Step 4: Insert segments between successive vertices
        for i in 0..vertex_list.len() - 1 {
            let v = vertex_list[i].clone();
            let vs = vertex_list[i + 1].clone();
            Self::insert_segment(v, vs, constraint_id);
        }
    }

    fn ccw(a: &Vertex, b: &Vertex, c: &Vertex) -> f64 {
        (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x)
    }

    fn is_ccw(a: &Vertex, b: &Vertex, c: &Vertex) -> bool {
        Self::ccw(a, b, c) > 0.0
    }

    fn is_point_on_edge(p: &Vertex, triangle: &Face, epsilon: f64) -> bool {
        for i in 0..3 {
            let a = triangle.vertices[i].borrow();
            let b = triangle.vertices[(i + 1) % 3].borrow();

            let distance =
                Self::ccw(&a, &b, &p).abs() / ((b.x - a.x).powi(2) + (b.y - a.y).powi(2)).sqrt();
            if distance < epsilon {
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

        let mut closest_triangle = random_sample[0];
        let mut min_distance = f64::MAX;

        for triangle in random_sample {
            for vertex in &triangle.borrow().vertices {
                let vertex = vertex.borrow();
                let distance = (vertex.x - p.x).powi(2) + (vertex.y - p.y).powi(2);
                if distance < min_distance {
                    min_distance = distance;
                    closest_triangle = triangle;
                }
            }
        }

        // Step 2: Walk - Oriented walk to locate p
        let mut current_face = closest_triangle;
        let mut visited = vec![false; self.faces.len()];

        loop {
            let vertices = current_face.borrow().vertices.clone();
            let vertices = [
                vertices[0].borrow(),
                vertices[1].borrow(),
                vertices[2].borrow(),
            ];
            let centroid = Vertex {
                x: (vertices[0].x + vertices[1].x + vertices[2].x) / 3.0,
                y: (vertices[0].y + vertices[1].y + vertices[2].y) / 3.0,
                index: 0,
                constraints: 0,
            };

            let mut selected_edge_index = None;

            // Find the edge that separates p and the centroid
            for (i, edge) in current_face.borrow().edges.iter().enumerate() {
                let edge_ref = edge.clone();
                let edge_borrowed = edge_ref.borrow();
                let vertex = edge_borrowed.a.borrow();
                let next_vertex = edge_borrowed.b.borrow();

                if Self::is_ccw(&vertex, &next_vertex, &p)
                    != Self::is_ccw(&vertex, &next_vertex, &centroid)
                {
                    selected_edge_index = Some(i);
                    break;
                }
            }

            if let Some(edge_index) = selected_edge_index {
                // Move to the adjacent triangle across the selected edge
                if let Some(neighbor) =
                    self.find_neighboring_face(&current_face.borrow(), edge_index)
                {
                    if visited[neighbor.borrow().id] {
                        // A loop is detected; fallback to epsilon-based checks
                        if Self::is_point_on_edge(p, &current_face.borrow(), epsilon) {
                            return LocateResult::Edge(
                                current_face.borrow().edges[edge_index].clone(),
                            );
                        }
                        return LocateResult::Face(current_face.clone());
                    }

                    visited[current_face.borrow().id] = true;
                    current_face = &self.faces[neighbor.borrow().id];
                } else {
                    return LocateResult::Face(current_face.clone());
                }
            } else {
                // The point is inside the current triangle
                return LocateResult::Face(current_face.clone());
            }
        }
    }

    fn find_neighboring_face(&self, face: &Face, edge_index: usize) -> Option<Rc<RefCell<Face>>> {
        // Get the SymEdge corresponding to the edge
        let edge = &face.edges[edge_index].borrow();
        let a_index = edge.a.borrow().index;
        let b_index = edge.b.borrow().index;
        let sym_edge = self.sym_edges_by_edges.get(&(a_index, b_index))?.borrow();

        sym_edge
            .nxt
            .clone()?
            .borrow()
            .rot
            .clone()?
            .borrow()
            .face
            .clone()
    }

    fn insert_point_on_edge(point: Vertex, edge: Rc<RefCell<Edge>>) -> Rc<RefCell<Vertex>> {
        todo!()
    }

    fn insert_point_in_face(point: Vertex, face: Rc<RefCell<Face>>) -> Rc<RefCell<Vertex>> {
        todo!()
    }

    fn insert_segment(v: Rc<RefCell<Vertex>>, vs: Rc<RefCell<Vertex>>, constraint_id: usize) {
        todo!()
    }
}

enum LocateResult {
    Vertex(Rc<RefCell<Vertex>>),
    Edge(Rc<RefCell<Edge>>),
    Face(Rc<RefCell<Face>>),
    None,
}

fn main() {
    std::env::set_var("RUST_BACKTRACE", "1");
    println!("{}", std::env::current_dir().unwrap().display());
    let mut cdt = CDT::from_gltf("C:/Projects/Study/cdt/models/model.glb");
    cdt.build_sym_edges().unwrap();
}
