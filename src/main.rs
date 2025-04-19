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

    let constraint_path = if args.len() > 2 {
        args[2].clone()
    } else {
        get_path_from_stdin("Enter constraint path:")
    };

    let output_path = if args.len() > 3 {
        args[3].clone()
    } else {
        get_path_from_stdin("Enter output path:")
    };

    let constraints = Constraints {
        constraint_segments: vec![ConstraintSegment::generate_square(
            DVec2 { x: 0., y: 0. },
            0.75 * 2.,
            0,
        )],
    };

    // constraints.export("./constraints/square_0-75.ct");

    // return;

    let constraints = Constraints::load(&constraint_path);

    let mut cdt = CDT::from_gltf(&input_path);
    cdt.build_sym_edges().unwrap();

    // cdt.export_to_obj("./models/output.obj");

    // // Wait 100ms
    // std::thread::sleep(std::time::Duration::from_millis(1000));

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
