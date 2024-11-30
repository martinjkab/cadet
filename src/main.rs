use cdt::cdt::CDT;
use glam::DVec2;

use crate::vertex::Vertex;

pub mod cdt;
pub mod edge;
pub mod face;
pub mod locate_result;
pub mod orientation;
pub mod sym_edge;
pub mod symmetric_compare;
pub mod vertex;

fn main() {
    std::env::set_var("RUST_BACKTRACE", "1");
    println!("{}", std::env::current_dir().unwrap().display());
    let mut cdt = CDT::from_gltf("./models/model.glb");
    cdt.build_sym_edges().unwrap();

    cdt.insert_constraint(
        [Vertex {
            position: DVec2 { x: 0.5, y: 0.5 },
            index: 0,
            constraints: 0,
        }]
        .to_vec(),
        0,
    );

    cdt.export_to_obj("./models/output.obj");
}
