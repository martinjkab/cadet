use cdt::cdt::CDT;
use glam::DVec2;
use std::io::{self, BufRead};

pub mod cdt;
pub mod edge;
pub mod face;
pub mod helper;
pub mod locate_result;
pub mod orientation;
pub mod sym_edge;
pub mod symmetric_compare;
pub mod vertex;

fn get_path_from_stdin(prompt: &str) -> String {
    println!("{}", prompt);
    let mut input = String::new();
    io::stdin()
        .lock()
        .read_line(&mut input)
        .expect("Failed to read line");
    input.trim().to_string()
}

fn main() {
    std::env::set_var("RUST_BACKTRACE", "1");

    let args: Vec<String> = std::env::args().collect();

    let input_path = if args.len() > 1 {
        args[1].clone()
    } else {
        get_path_from_stdin("Enter input path:")
    };

    let output_path = if args.len() > 2 {
        args[2].clone()
    } else {
        get_path_from_stdin("Enter output path:")
    };

    let mut cdt = CDT::from_gltf(&input_path);
    cdt.build_sym_edges().unwrap();

    cdt.insert_constraint(
        [
            DVec2 { x: -0.5, y: -0.5 },
            DVec2 { x: 0.5, y: -0.5 },
            DVec2 { x: 0.5, y: 0.5 },
            DVec2 { x: -0.5, y: 0.5 },
            DVec2 { x: -0.5, y: -0.5 },
            // DVec2 { x: 0., y: 0. },
            // DVec2 { x: -0.5, y: -0.5 },
            // DVec2 { x: 0.5, y: -0.5 },
            // DVec2 { x: 0.5, y: 0.5 },
            // DVec2 { x: -0.5, y: 0.5 },
            // DVec2 { x: -0.5, y: -0.5 },
            // DVec2 { x: 0., y: 0. },
        ]
        .to_vec(),
        0,
    );

    // cdt.insert_constraint([DVec2 { x: 0., y: 0. }].to_vec(), 0);

    cdt.insert_constraint(
        generate_circle(DVec2 { x: 0., y: 0.125 }, 0.25, 32).to_vec(),
        1,
    );

    cdt.insert_constraint(
        generate_circle(DVec2 { x: 0., y: -0.125 }, 0.25, 32).to_vec(),
        2,
    );

    // cdt.insert_constraint(generate_circle(DVec2 { x: 0., y: 0. }, 0.5, 8).to_vec(), 0);

    // cdt.insert_constraint(
    //     generate_line(100, DVec2 { x: -1.0, y: 1.0 }, DVec2 { x: 1.0, y: -1.0 }),
    //     0,d
    // );

    // cdt.insert_constraint(
    //     [DVec2 { x: -0.5, y: -0.5 }, DVec2 { x: 0.5, y: -0.5 }].to_vec(),
    //     0,
    // );

    println!("Number of faces: {}", cdt.faces.len());

    //Verify that for every sym_edge, the face exists
    for sym_edge in cdt.sym_edges_by_half_edges.values() {
        let sym_edge = sym_edge.borrow();

        let is_any_face = cdt
            .faces
            .iter()
            .any(|face| face.as_ptr() == sym_edge.face.as_ptr());

        assert!(is_any_face);
    }

    //Verify that every edge is delaunay
    // for sym_edge in cdt.sym_edges_by_half_edges.values() {
    //     let sym_edge = sym_edge.borrow();
    //     let neighbor_face = sym_edge.neighbor_face();

    //     let neighbor_face = match neighbor_face {
    //         Some(neighbor_face) => neighbor_face,
    //         None => continue,
    //     };

    //     let is_delanuay = CDT::is_delaunay(
    //         sym_edge.face.borrow().vertices[0].borrow().position,
    //         sym_edge.face.borrow().vertices[1].borrow().position,
    //         sym_edge.face.borrow().vertices[2].borrow().position,
    //         neighbor_face
    //             .borrow()
    //             .opposite_vertex(&sym_edge.edge.borrow())
    //             .borrow()
    //             .position,
    //     );

    //     assert!(is_delanuay);
    // }

    cdt.export_to_obj(&output_path);
}

//Generate a circle from DVec2 with radius r
fn generate_circle(center: DVec2, r: f64, n: usize) -> Vec<DVec2> {
    let mut circle = Vec::new();
    let step = 2. * std::f64::consts::PI / n as f64;
    for i in 0..n {
        let x = center.x + r * f64::cos(i as f64 * step);
        let y = center.y + r * f64::sin(i as f64 * step);
        circle.push(DVec2 { x, y });
    }
    circle
}

fn generate_line(n: usize, from: DVec2, to: DVec2) -> Vec<DVec2> {
    let mut line = Vec::new();
    let step = (to - from) / n as f64;
    for i in 0..n {
        let x = from.x + i as f64 * step.x;
        let y = from.y + i as f64 * step.y;
        line.push(DVec2 { x, y });
    }
    line
}
