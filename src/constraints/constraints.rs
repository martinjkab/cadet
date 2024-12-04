use std::io::{BufRead, Write};

use glam::DVec2;

pub struct Constraints {
    pub constraint_segments: Vec<ConstraintSegment>,
}

pub struct ConstraintSegment {
    pub constraints: Vec<DVec2>,
    pub id: usize,
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
                continue;
            }

            let x = parts[0].parse::<f64>().expect("Failed to parse x");
            let y = parts[1].parse::<f64>().expect("Failed to parse y");

            constraints.push(DVec2 { x, y });
        }

        Constraints {
            constraint_segments: constraint_lists,
        }
    }

    pub fn export(&self, model_path: &str) {
        let file = std::fs::File::create(model_path).expect("Failed to create file");
        let mut writer = std::io::BufWriter::new(file);

        for segment in &self.constraint_segments {
            for constraint in &segment.constraints {
                writeln!(writer, "{} {}", constraint.x, constraint.y)
                    .expect("Failed to write to file");
            }
            writeln!(writer).expect("Failed to write to file");
        }
    }
}

impl ConstraintSegment {
    pub fn generate_circle(center: DVec2, r: f64, n: usize, id: usize) -> ConstraintSegment {
        let mut circle = Vec::new();
        let step = 2. * std::f64::consts::PI / n as f64;
        for i in 0..n {
            let x = center.x + r * f64::cos(i as f64 * step);
            let y = center.y + r * f64::sin(i as f64 * step);
            circle.push(DVec2 { x, y });
        }
        ConstraintSegment {
            constraints: circle.iter().map(|&position| position).collect(),
            id,
        }
    }

    pub fn generate_line(n: usize, from: DVec2, to: DVec2, id: usize) -> ConstraintSegment {
        let mut line = Vec::new();
        let step = (to - from) / n as f64;
        for i in 0..n {
            let x = from.x + i as f64 * step.x;
            let y = from.y + i as f64 * step.y;
            line.push(DVec2 { x, y });
        }

        ConstraintSegment {
            constraints: line.iter().map(|&position| position).collect(),
            id,
        }
    }

    pub fn generate_square(center: DVec2, side: f64, id: usize) -> ConstraintSegment {
        let mut square = Vec::new();
        let half_side = side / 2.;
        square.push(DVec2 {
            x: center.x - half_side,
            y: center.y - half_side,
        });
        square.push(DVec2 {
            x: center.x + half_side,
            y: center.y - half_side,
        });
        square.push(DVec2 {
            x: center.x + half_side,
            y: center.y + half_side,
        });
        square.push(DVec2 {
            x: center.x - half_side,
            y: center.y + half_side,
        });

        ConstraintSegment {
            constraints: square.iter().map(|&position| position).collect(),
            id,
        }
    }
}
