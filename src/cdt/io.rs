use std::{cell::RefCell, collections::HashSet, io::Write, rc::Rc};

use glam::DVec2;

use crate::{edge::Edge, face::Face, vertex::Vertex};

use super::cdt::CDT;

impl CDT {
    pub fn from_gltf(model_path: &str) -> Self {
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

        let indices = first_scene
            .models
            .iter()
            .flat_map(|m| m.indices())
            .flatten()
            .collect::<Vec<_>>();

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

        CDT {
            vertices,
            edges,
            faces,
            ..Default::default()
        }
    }

    pub fn export_to_obj(&self, model_path: &str) {
        let file = std::fs::File::create(model_path).expect("Failed to create file");
        let mut writer = std::io::BufWriter::new(file);

        for vertex in self.vertices.iter() {
            let vertex = vertex.borrow();
            writeln!(writer, "v {} {} 0.0", vertex.position.x, vertex.position.y)
                .expect("Failed to write to file");
        }

        for face in self.faces.iter() {
            let face = face.borrow();
            let indices = face.vertex_indices();
            writeln!(
                writer,
                "f {} {} {}",
                indices[0] + 1,
                indices[1] + 1,
                indices[2] + 1
            )
            .expect("Failed to write to file");
        }

        writer.flush().expect("Failed to flush buffer");

        println!("Exported to {}", model_path);
    }
}
