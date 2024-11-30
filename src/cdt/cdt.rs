use std::{
    cell::RefCell,
    collections::{HashMap, VecDeque},
    rc::Rc,
};

use glam::DVec2;

use crate::{
    edge::Edge, face::Face, locate_result::LocateResult, orientation::Orientation,
    sym_edge::SymEdge, symmetric_compare::SymmetricCompare, vertex::Vertex,
};

#[derive(Debug, Default)]
pub struct CDT {
    pub vertices: Vec<Rc<RefCell<Vertex>>>,
    pub edges: Vec<Rc<RefCell<Edge>>>,
    pub faces: Vec<Rc<RefCell<Face>>>,
    pub faces_by_ids: HashMap<usize, Rc<RefCell<Face>>>,
    pub sym_edges_by_edges: HashMap<(usize, usize), Rc<RefCell<SymEdge>>>,
    pub sym_edges_by_vertices: HashMap<usize, Vec<Rc<RefCell<SymEdge>>>>,
    pub constraints: HashMap<usize, Vec<usize>>,
    pub face_id_counter: usize,
}

impl CDT {
    pub fn insert_constraint(
        &mut self,
        constraint_points: Vec<DVec2>, // List of points in the constraint
        _constraint_id: usize,         // ID of the constraint
    ) {
        println!("Inserting constraint: {:?}", constraint_points);
        let mut vertex_list = Vec::new();

        for point in constraint_points.iter() {
            // Step 1: Locate the point in the triangulation
            let locate_result = self.locate_point(point);

            // Step 2: Handle the locate result
            let vertex = match locate_result {
                LocateResult::Vertex(v) => v,
                LocateResult::Edge(edge) => self.insert_point_on_edge(*point, edge),
                LocateResult::Face(face) => self.insert_point_in_face(*point, face),
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

    pub fn ccw(a: &DVec2, b: &DVec2, c: &DVec2) -> f64 {
        let ab = a - b;
        let ac = a - c;

        ab.x * ac.y - ab.y * ac.x
    }

    pub fn is_ccw(a: &DVec2, b: &DVec2, c: &DVec2) -> Orientation {
        let ccw = Self::ccw(a, b, c);
        let distance = ccw.abs() / ((b.x - a.x).powi(2) + (b.y - a.y).powi(2)).sqrt();
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
        point: DVec2,
        edge: Rc<RefCell<Edge>>,
    ) -> Rc<RefCell<Vertex>> {
        let edge = edge.borrow();
        let a = edge.a.borrow();
        let b = edge.b.borrow();

        let ab = b.position - a.position;
        let ap = point - a.position;

        let t = ap.dot(ab) / ab.dot(ab);
        let t = t.clamp(0.0, 1.0);

        let v = self.add_vertex(a.position + ab * t, 1);

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

        // Remove the old faces
        // self.remove_face(face_1.clone());
        // self.remove_face(face_2.clone());

        // Get the edges that are diffferent from e
        let face_1_edges = face_1.borrow().edges.clone();
        let face_2_edges = face_2.borrow().edges.clone();

        println!("Edge indices: {:?}", edge.edge_indices());

        println!(
            "Face 1 edges: {:?}",
            face_1_edges.clone().map(|x| x.borrow().edge_indices())
        );
        println!(
            "Face 2 edges: {:?}",
            face_2_edges.clone().map(|x| x.borrow().edge_indices())
        );

        let face_1_edges = face_1_edges
            .iter()
            .filter(|face_edge| {
                !face_edge
                    .borrow()
                    .edge_indices()
                    .symmetric_compare(&edge.edge_indices())
            })
            .cloned()
            .collect::<Vec<_>>();

        let face_2_edges = face_2_edges
            .iter()
            .filter(|face_edge| {
                !face_edge
                    .borrow()
                    .edge_indices()
                    .symmetric_compare(&edge.edge_indices())
            })
            .cloned()
            .collect::<Vec<_>>();

        let new_faces = [
            self.add_face([
                face_1_edges[0].borrow().a.clone(),
                face_1_edges[0].borrow().b.clone(),
                v.clone(),
            ]),
            self.add_face([
                face_2_edges[0].borrow().a.clone(),
                face_2_edges[0].borrow().b.clone(),
                v.clone(),
            ]),
            self.add_face([
                face_1_edges[1].borrow().a.clone(),
                face_1_edges[1].borrow().b.clone(),
                v.clone(),
            ]),
            self.add_face([
                face_2_edges[1].borrow().a.clone(),
                face_2_edges[1].borrow().b.clone(),
                v.clone(),
            ]),
        ];

        println!(
            "New faces: {:?}",
            new_faces.map(|x| x.borrow().vertex_indices())
        );

        let mut edge_stack = VecDeque::new();
        edge_stack.extend(face_1_edges.clone());
        edge_stack.extend(face_2_edges.clone());

        self.flip_edges(&mut edge_stack);

        v
    }

    pub fn insert_point_in_face(
        &mut self,
        v: DVec2,
        face: Rc<RefCell<Face>>,
    ) -> Rc<RefCell<Vertex>> {
        //New vertex
        let v = self.add_vertex(v, 1);
        self.vertices.push(v.clone());

        {
            self.remove_face(face.clone());
            // New edges
            let face_borrowed = face.borrow();

            face_borrowed.edges.iter().for_each(|edge| {
                let vertices = [edge.borrow().a.clone(), edge.borrow().b.clone(), v.clone()];

                self.add_face(vertices);
            });
        }

        let mut edge_stack = VecDeque::new();
        edge_stack.extend(face.borrow().edges.clone());

        self.flip_edges(&mut edge_stack);

        v
    }

    // fn insert_segment(v: Rc<RefCell<Vertex>>, vs: Rc<RefCell<Vertex>>, constraint_id: usize) {}
}
