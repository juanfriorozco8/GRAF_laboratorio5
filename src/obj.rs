use raylib::prelude::Vector3;
use std::{error::Error, fs, io};

use crate::vertex::Vertex;

pub struct Obj {
    pub vertices: Vec<Vector3>,
    pub indices: Vec<usize>,
}

impl Obj {
    pub fn get_vertex_array(&self) -> Vec<Vertex> {
        self.indices
            .iter()
            .map(|&index| Vertex::new_with_color(self.vertices[index], Vector3::new(1.0, 1.0, 1.0)))
            .collect()
    }
}

pub fn load_obj(path: &str) -> Result<Obj, Box<dyn Error>> {
    let content = fs::read_to_string(path)?;
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    for line in content.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        match parts[0] {
            "v" => {
                if parts.len() < 4 {
                    return Err(io::Error::other("Un vertice necesita x, y, z").into());
                }
                let x: f32 = parts[1].parse()?;
                let y: f32 = parts[2].parse()?;
                let z: f32 = parts[3].parse()?;
                if ![x, y, z].into_iter().all(f32::is_finite) {
                    return Err(io::Error::other("Coordenada no finita").into());
                }
                vertices.push(Vector3::new(x, y, z));
            }
            "f" => {
                if parts.len() != 4 {
                    return Err(io::Error::other("El OBJ debe estar triangulado").into());
                }
                for part in &parts[1..4] {
                    let index: usize = part.split('/').next().unwrap_or("").parse()?;
                    indices.push(
                        index.checked_sub(1).ok_or_else(|| {
                            io::Error::other("Los indices OBJ deben comenzar en 1")
                        })?,
                    );
                }
            }
            _ => {}
        }
    }

    if vertices.is_empty() || indices.is_empty() {
        return Err(io::Error::other("El OBJ no contiene vertices y caras").into());
    }
    if indices.iter().any(|&index| index >= vertices.len()) {
        return Err(io::Error::other("Una cara referencia un vertice inexistente").into());
    }
    Ok(Obj { vertices, indices })
}
