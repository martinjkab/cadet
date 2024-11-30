use glam::DVec2;

#[derive(Clone, PartialEq, Debug)]
pub struct Vertex {
    pub index: usize,       // Index of the vertex
    pub position: DVec2,    // Position of the vertex
    pub constraints: usize, // Number of constraints referencing this vertex
}

impl std::fmt::Display for Vertex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Vertex {{ index: {}, x: {}, y: {}, constraints: {} }}",
            self.index, self.position.x, self.position.y, self.constraints
        )
    }
}
