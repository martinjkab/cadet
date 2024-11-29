use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
};

#[derive(Clone, PartialEq, Debug)]
struct Vertex {
    x: f64,
    y: f64,
    constraints: usize, // Number of constraints referencing this vertex
}

#[derive(Debug)]
struct Edge {
    vertices: (usize, usize), // Indices of the vertices

    crep: HashSet<usize>, // Constraints represented by this edge
}

#[derive(Clone, Debug)]
struct Face {
    vertices: [usize; 3], // Indices of the vertices
    edges: [usize; 3],    // Indices of the edges
}

#[derive(Debug)]
struct CDT {
    vertices: Vec<Rc<RefCell<Vertex>>>,
    edges: Vec<Rc<RefCell<Edge>>>,
    faces: Vec<Rc<RefCell<Face>>>,
    sym_edges: Vec<Rc<RefCell<SymEdge>>>,
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
            self.vertex.borrow().x,
            self.edge.borrow().vertices,
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
            self.vertex.borrow().x,
            self.edge.borrow().vertices,
            self.face,
        )
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
            .map(|v| {
                let (p, n) = v;
                Vertex {
                    x: p.x as f64,
                    y: p.y as f64,
                    constraints: 0,
                }
            })
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
        let faces = indices
            .chunks(3)
            .map(|c| {
                let v = c.iter().map(|&x| *x as usize).collect::<Vec<_>>();

                let mut face_edges = Vec::new();
                for i in 0..3 {
                    let edge = Edge {
                        vertices: (v[i], v[(i + 1) % 3]),

                        crep: HashSet::new(),
                    };
                    face_edges.push(edge);
                }
                edges.extend(face_edges);

                let edge_indices = edges.len() - 3..edges.len();
                let edge_indices = edge_indices.collect::<Vec<_>>();

                Face {
                    vertices: [v[0], v[1], v[2]],
                    edges: [edge_indices[0], edge_indices[1], edge_indices[2]],
                }
            })
            .collect::<Vec<_>>();
        println!("triangles: {:?}", faces);
        println!("edges: {:?}", edges);

        CDT {
            vertices: vertices
                .into_iter()
                .map(|v| Rc::new(RefCell::new(v)))
                .collect(),
            edges: edges
                .into_iter()
                .map(|e| Rc::new(RefCell::new(e)))
                .collect(),
            faces: faces
                .into_iter()
                .map(|f| Rc::new(RefCell::new(f)))
                .collect(),
            sym_edges: Vec::new(),
            constraints: HashMap::new(),
        }
    }

    fn new() -> Self {
        CDT {
            vertices: Vec::new(),
            edges: Vec::new(),
            faces: Vec::new(),
            sym_edges: Vec::new(),
            constraints: HashMap::new(),
        }
    }

    fn validate_faces(&mut self) -> Result<(), String> {
        for (face_idx, face_data) in self.faces.iter().enumerate() {
            if face_data.borrow().vertices.len() < 3 {
                return Err(format!("Face {} has fewer than 3 vertices", face_idx));
            }
            for &v_idx in &face_data.borrow().vertices {
                if v_idx >= self.vertices.len() {
                    return Err(format!(
                        "Face {} references invalid vertex {}",
                        face_idx, v_idx
                    ));
                }
            }
        }
        Ok(())
    }

    fn build_sym_edges(&mut self) -> Result<(), String> {
        self.validate_faces()?;
        let mut vertex_to_symedges = std::collections::HashMap::new();

        // Create SymEdges for each face
        for face_ref in self.faces.iter() {
            let face = face_ref.borrow();
            let mut face_symedges = Vec::new();
            let mut face_symedges_other = Vec::new();

            for (i, &edge_idx) in face.edges.iter().enumerate() {
                let vertex_idx = face.vertices[i];

                // Create a new SymEdge
                let sym = Rc::new(RefCell::new(SymEdge {
                    vertex: self.vertices[vertex_idx].clone(),
                    edge: self.edges[edge_idx].clone(),
                    face: Some(face_ref.clone()),
                    nxt: None,
                    rot: None,
                }));
                face_symedges.push(sym.clone());

                // Track SymEdges per vertex
                vertex_to_symedges
                    .entry(vertex_idx)
                    .or_insert_with(Vec::new)
                    .push(sym.clone());

                let vertex_idx = face.vertices[(i + 1) % 3];

                let sym = Rc::new(RefCell::new(SymEdge {
                    vertex: self.vertices[vertex_idx].clone(),
                    edge: self.edges[edge_idx].clone(),
                    face: Some(face_ref.clone()),
                    nxt: None,
                    rot: None,
                }));
                face_symedges_other.push(sym.clone());

                // Track SymEdges per vertex
                vertex_to_symedges
                    .entry(vertex_idx)
                    .or_insert_with(Vec::new)
                    .push(sym.clone());
            }

            // Link `nxt` pointers within the face
            for i in 0..3 {
                let nxt = face_symedges[(i + 1) % 3].clone();
                face_symedges[i].borrow_mut().nxt = Some(nxt);
            }

            for i in 0..3 {
                let nxt = face_symedges_other[(i + 1) % 3].clone();
                face_symedges_other[i].borrow_mut().nxt = Some(nxt);
            }

            self.sym_edges.extend(face_symedges);
            self.sym_edges.extend(face_symedges_other);
        }

        // Step 2: Link `rot` pointers between symmetrical SymEdges
        // based on the vertex they share

        for (index, sym_edges) in vertex_to_symedges.iter() {
            let vertex = self.vertices[*index].clone();
            let mut angle_to_sym_edges = std::vec::Vec::new();
            //Iterate over the symmetrical edges
            for sym in sym_edges {
                let edge = sym.borrow().edge.clone();
                let other_vertex_index = if edge.borrow().vertices.0 == *index {
                    edge.borrow().vertices.1
                } else {
                    edge.borrow().vertices.0
                };

                let angle = (self.vertices[other_vertex_index].borrow().y - vertex.borrow().y)
                    .atan2(self.vertices[other_vertex_index].borrow().x - vertex.borrow().x);

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
        for sym_edge in &self.sym_edges {
            println!("{:?}", sym_edge.borrow());
        }

        Ok(())
    }
}

fn main() {
    std::env::set_var("RUST_BACKTRACE", "1");
    println!("{}", std::env::current_dir().unwrap().display());
    let mut cdt = CDT::from_gltf("C:/Projects/Study/cdt/models/model.glb");
    cdt.build_sym_edges().unwrap();
}
