use cdt::cdt::CDT;
use constraints::constraints::{ConstraintSegment, Constraints};
use glam::DVec2;
use std::io::{self, BufRead};

pub mod cdt;
pub mod constraints;
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

    let constraints = Constraints {
        constraint_segments: vec![
            ConstraintSegment::generate_circle(DVec2 { x: -0.45, y: 0.1 }, 0.2, 64, 0),
            ConstraintSegment::generate_circle(DVec2 { x: 0., y: 0.1 }, 0.2, 64, 0),
            ConstraintSegment::generate_circle(DVec2 { x: 0.45, y: 0.1 }, 0.2, 64, 0),
            ConstraintSegment::generate_circle(DVec2 { x: -0.25, y: -0.1 }, 0.2, 64, 0),
            ConstraintSegment::generate_circle(DVec2 { x: 0.25, y: -0.1 }, 0.2, 64, 0),
        ],
    };

    // constraints.export("./constraints/olympics.ct");

    // return;

    let constraints = Constraints::load("./constraints/circle_0-25.ct");

    let mut cdt = CDT::from_gltf(&input_path);
    cdt.build_sym_edges().unwrap();
    cdt.add_constraints(&constraints);

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

    cdt.export_to_obj(&output_path);
}
