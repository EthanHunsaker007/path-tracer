use std::fs::File;
use std::io::{BufRead, BufReader};

pub fn parse_obj(path: &str) -> (Vec<[f32; 4]>, Vec<[u32; 4]>) {
    let file = File::open(path).expect("OBJ load failed");
    let reader = BufReader::new(file);

    let mut vertices: Vec<[f32; 4]> = Vec::new();
    let mut triangles: Vec<[u32; 4]> = Vec::new();

    for line in reader.lines() {
        let line = line.unwrap();
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        match parts[0] {
            "v" => {
                let x: f32 = parts[1].parse().unwrap();
                let y: f32 = parts[2].parse().unwrap();
                let z: f32 = parts[3].parse().unwrap();
                vertices.push([x * 10.0, -y * 10.0, z * 10.0, 1.0]);
            }
            "f" => {
                let indices: Vec<u32> = parts[1..]
                .iter()
                .map(|s| s.split('/').next().unwrap().parse::<u32>().unwrap() - 1)
                .collect();
            
                for i in 1..indices.len() - 1 {
                    triangles.push([indices[0], indices[i], indices[i + 1], 0]);
                }
            }
            _ => {}
        }
    }

    (vertices, triangles)

}