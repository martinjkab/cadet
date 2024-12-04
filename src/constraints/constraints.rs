use std::io::BufRead;

use glam::DVec2;

struct Constraints {
    pub constraint_segments: Vec<ConstraintSegment>,
}

struct ConstraintSegment {
    pub constraints: Vec<Constraint>,
    pub id: usize,
}

struct Constraint {
    pub position: DVec2,
}

impl Constraints {
    pub fn load(model_path: &str) -> Self {
        let file = std::fs::File::open(model_path).expect("Failed to open file");
        let reader = std::io::BufReader::new(file);
        let lines = reader.lines();
        let mut constraint_lists = Vec::new();
        let mut constraints = Vec::new();
        let mut id = 0;

        for line in lines {
            let line = line.expect("Failed to read line");
            let parts = line.split_whitespace().collect::<Vec<_>>();

            if parts.is_empty() {
                constraint_lists.push(ConstraintSegment { constraints, id });
                constraints = Vec::new();
                id += 1;
                break;
            }

            let x = parts[0].parse::<f64>().expect("Failed to parse x");
            let y = parts[1].parse::<f64>().expect("Failed to parse y");

            constraints.push(Constraint {
                position: DVec2 { x, y },
            });
        }

        Constraints {
            constraint_segments: constraint_lists,
        }
    }

    pub fn export_to_obj(&self, model_path: &str) {}
}
