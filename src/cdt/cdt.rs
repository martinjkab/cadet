use std::{
    cell::RefCell,
    collections::{HashMap, HashSet, VecDeque},
    rc::Rc,
};

use glam::DVec2;

use crate::{
    edge::Edge,
    face::Face,
    helper::{intersection_point, is_crossing},
    locate_result::LocateResult,
    orientation::Orientation,
    sym_edge::SymEdge,
    symmetric_compare::SymmetricCompare,
    vertex::Vertex,
};

#[derive(Debug, Default)]
pub struct CDT {
    pub vertices: Vec<Rc<RefCell<Vertex>>>,
    pub edges: Vec<Rc<RefCell<Edge>>>,
    pub faces: Vec<Rc<RefCell<Face>>>,
    pub sym_edges_by_half_edges: HashMap<(usize, usize), Rc<RefCell<SymEdge>>>,
    pub sym_edges_by_vertices: HashMap<usize, Vec<Rc<RefCell<SymEdge>>>>,
    pub constraints: HashMap<usize, Vec<usize>>,
    pub face_id_counter: usize,
}

impl CDT {
    pub fn insert_constraint(
        &mut self,
        constraint_points: Vec<DVec2>, // List of points in the constraint
        constraint_id: usize,          // ID of the constraint
    ) {
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
        for i in 0..vertex_list.len() - 1 {
            let v = vertex_list[i].clone();
            let vs = vertex_list[i + 1].clone();
            self.insert_segment(v, vs, constraint_id);
        }
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
        let edge_rc = edge.clone();
        let edge = edge.borrow();
        let a = edge.a.borrow();
        let b = edge.b.borrow();

        let ab = b.position - a.position;
        let ap = point - a.position;

        let t = ap.dot(ab) / ab.dot(ab);
        let t = t.clamp(0.0, 1.0);

        let v = self.add_vertex(a.position + ab * t, 1);

        let edge_indices = edge.edge_indices();

        let sym_edge = self.get_sym_edge_for_half_edge(&edge_indices).unwrap();
        let face_1 = sym_edge.borrow().face.clone();
        let face_2 = sym_edge.borrow().neighbor_face();

        let face_2 = match face_2 {
            Some(face) => face,
            None => {
                return v;
            }
        };

        assert!(face_1.borrow().vertex_indices() != face_2.borrow().vertex_indices());
        assert!(face_1.as_ptr() != face_2.as_ptr());

        // Remove the old faces
        self.remove_face(face_1.clone());
        self.remove_face(face_2.clone());

        // Get the edges that are diffferent from e
        let face_1_edges = face_1.borrow().edges.clone();
        let face_2_edges = face_2.borrow().edges.clone();

        let face_1_edges = face_1_edges
            .iter()
            .filter(|face_edge| !face_edge.symmetric_compare(&edge.edge_indices()))
            .map(|edge| (self.vertices[edge.0].clone(), self.vertices[edge.1].clone()))
            .collect::<Vec<_>>();

        let face_2_edges = face_2_edges
            .iter()
            .filter(|face_edge| !face_edge.symmetric_compare(&edge.edge_indices()))
            .map(|edge| (self.vertices[edge.0].clone(), self.vertices[edge.1].clone()))
            .collect::<Vec<_>>();

        assert!(face_1_edges.len() == 2);
        assert!(face_2_edges.len() == 2);

        let new_faces = [
            self.add_face([
                face_1_edges[0].0.clone(),
                face_1_edges[0].1.clone(),
                v.clone(),
            ]),
            self.add_face([
                face_2_edges[0].0.clone(),
                face_2_edges[0].1.clone(),
                v.clone(),
            ]),
            self.add_face([
                face_1_edges[1].0.clone(),
                face_1_edges[1].1.clone(),
                v.clone(),
            ]),
            self.add_face([
                face_2_edges[1].0.clone(),
                face_2_edges[1].1.clone(),
                v.clone(),
            ]),
        ];

        let edges = new_faces
            .iter()
            .map(|face| face.borrow().edges.clone())
            .flatten()
            .filter(|edge| {
                [face_1_edges.clone(), face_2_edges.clone()]
                    .iter()
                    .flatten()
                    .any(|face_edge| {
                        (face_edge.0.borrow().index, face_edge.1.borrow().index)
                            .symmetric_compare(&edge)
                    })
            })
            .map(|edge| self.get_sym_edge_for_half_edge(&edge).unwrap())
            .map(|edge| edge.borrow().edge.clone())
            .collect::<Vec<_>>();

        assert!(edges.len() == 4);

        let mut edge_stack = VecDeque::new();
        edge_stack.extend(edges.clone());

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
        self.remove_face(face.clone());

        let face_borrowed = face.borrow();

        let new_faces = face_borrowed
            .edges
            .iter()
            .map(|edge| {
                let edge = (self.vertices[edge.0].clone(), self.vertices[edge.1].clone());
                let vertices = [edge.0.clone(), edge.1.clone(), v.clone()];

                self.add_face(vertices)
            })
            .collect::<Vec<_>>();

        let edges = new_faces
            .iter()
            .map(|face| face.borrow().edges.clone())
            .flatten()
            .filter(|edge| {
                face_borrowed
                    .edges
                    .iter()
                    .any(|face_edge| face_edge.symmetric_compare(&edge))
            })
            .map(|edge| self.get_sym_edge_for_half_edge(&edge).unwrap())
            .map(|edge| edge.borrow().edge.clone())
            .collect::<Vec<_>>();

        assert!(edges.len() == 3);

        let mut edge_stack = VecDeque::new();
        edge_stack.extend(edges.clone());

        self.flip_edges(&mut edge_stack);

        v
    }

    fn insert_segment(
        &mut self,
        start: Rc<RefCell<Vertex>>,
        end: Rc<RefCell<Vertex>>,
        constraint_id: usize,
    ) {
        let edge_list = self.find_crossed_edges(start.clone(), end.clone());

        for edge in edge_list.iter() {
            let intersection_point = intersection_point(
                &(start.borrow().position, end.borrow().position),
                &(
                    edge.borrow().a.borrow().position,
                    edge.borrow().b.borrow().position,
                ),
            );

            if let Some(intersection_point) = intersection_point {
                self.insert_point_on_edge(intersection_point, edge.clone());
            }
        }
    }

    fn find_crossed_edges(
        &self,
        start: Rc<RefCell<Vertex>>,
        end: Rc<RefCell<Vertex>>,
    ) -> Vec<Rc<RefCell<Edge>>> {
        let mut edge_list = Vec::new();

        for edge in self.edges.iter() {
            let edge_borrowed = edge.borrow();

            let is_crossing = is_crossing(
                &(start.borrow().position, end.borrow().position),
                &(
                    edge_borrowed.a.borrow().position,
                    edge_borrowed.b.borrow().position,
                ),
            );

            if is_crossing {
                edge_list.push(edge.clone());
            }
        }

        // // Get the initial half-edge from the starting vertex
        // let sym_edge = self
        //     .sym_edges_by_vertices
        //     .get(&start.borrow().index)
        //     .and_then(|edges| edges.first().cloned());

        // if sym_edge.is_none() {
        //     return edge_list; // If no half-edges are associated with the vertex, return empty
        // }

        // let mut sym_edge = sym_edge.unwrap();
        // let initial_sym_edge = sym_edge.clone(); // Keep track of the initial half-edge to detect cycles
        // let mut visited = std::collections::HashSet::new(); // Prevent re-checking edges

        // loop {
        //     let current_edge = sym_edge.clone();

        //     println!(
        //         "Current edge: {:?}",
        //         (
        //             current_edge.borrow().edge.borrow().a.borrow().position,
        //             current_edge.borrow().edge.borrow().b.borrow().position
        //         )
        //     );

        //     // If the edge is already visited, break
        //     let current_index = current_edge.as_ptr() as usize;
        //     if !visited.insert(current_index) {
        //         break;
        //     }

        //     // Check if the current edge crosses the segment
        //     let is_crossing = is_crossing(
        //         &(start.borrow().position, end.borrow().position),
        //         &(
        //             current_edge.borrow().edge.borrow().a.borrow().position,
        //             current_edge.borrow().edge.borrow().b.borrow().position,
        //         ),
        //     );

        //     if is_crossing {
        //         edge_list.push(current_edge.borrow().edge.clone());
        //     }

        //     // Traverse the `nxt` edge first
        //     let mut next_edge = current_edge.borrow().nxt.clone();

        //     // If `nxt` is None or already visited, traverse the `rot` edge
        //     if next_edge.is_none()
        //         || visited.contains(&(next_edge.as_ref().unwrap().as_ptr() as usize))
        //     {
        //         next_edge = current_edge.borrow().rot.clone();
        //     }

        //     // If no new edge to traverse, or we've cycled back to the starting edge, stop
        //     if next_edge.is_none()
        //         || next_edge.as_ref().map(|e| e.as_ptr()) == Some(initial_sym_edge.as_ptr())
        //     {
        //         break;
        //     }

        //     sym_edge = next_edge.unwrap();
        // }

        edge_list
    }
}
