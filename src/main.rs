use cdt::cdt::CDT;
use glam::DVec2;
use helper::is_crossing;
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

    let a = DVec2 { x: -0.75, y: -0.75 };
    let b = DVec2 { x: 0.75, y: -0.75 };

    let c = DVec2 { x: -1., y: -1. };
    let d = DVec2 { x: -1., y: -0.5 };

    println!("Is crossing: {}", is_crossing(&(a, b), &(c, d)));

    assert_eq!(is_crossing(&(a, b), &(c, d)), false);

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
            // DVec2 { x: -0.5, y: -0.25 },
            // DVec2 { x: 0.5, y: -0.25 },
            // DVec2 { x: 0.5, y: 0.25 },
            // DVec2 { x: -0.5, y: 0.25 },
            DVec2 { x: -0.75, y: -0.75 },
            DVec2 { x: 0.75, y: -0.75 },
        ]
        .to_vec(),
        0,
    );

    cdt.export_to_obj(&output_path);
}
