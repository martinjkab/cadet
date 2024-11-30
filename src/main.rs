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
    let mut cdt = CDT::from_gltf("C:/Projects/Study/cdt/models/model.glb");
    cdt.build_sym_edges().unwrap();
    // cdt.insert_point_on_edge(
    //     Vertex {
    //         position: DVec2 { x: 0.0, y: 0.0 },
    //         index: 0,
    //         constraints: 0,
    //     },
    //     cdt.edges
    //         .iter()
    //         .find(|e| e.borrow().a.borrow().index == 3)
    //         .unwrap()
    //         .clone(),
    // );
    cdt.insert_point_in_face(
        Vertex {
            position: DVec2 { x: 0.5, y: 0.5 },
            index: 0,
            constraints: 0,
        },
        cdt.faces[0].clone(),
    );
    cdt.export_to_obj("C:/Projects/Study/cdt/models/output.obj");
}
