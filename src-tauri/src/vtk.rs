use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug)]
pub struct VtkVolume {
    pub dims: Vec<usize>,
    pub spacing: (f64, f64, f64),
    pub origin: (f64, f64, f64),
    pub data: Vec<f64>,
}

// This function is not production grade or anything and is not safe at all
// This is just to make sense of the vtk file but the format is not rigorously tested by any means
pub fn read_vtk(path: &String) -> Result<VtkVolume, String> {
    let file = File::open(path).map_err(|e| e.to_string())?;
    let reader = BufReader::new(file);

    let mut dims = None;
    let mut spacing = None;
    let mut origin = None;

    let mut lines = reader.lines();

    while let Some(Ok(line)) = lines.next() {
        let line = line.trim();

        if line.starts_with("DIMENSIONS") {
            let parts: Vec<_> = line.split_whitespace().collect();
            dims = Some([
                parts[1].parse().unwrap(),
                parts[2].parse().unwrap(),
                parts[3].parse().unwrap(),
            ]);
        }

        if line.starts_with("SPACING") {
            let parts: Vec<_> = line.split_whitespace().collect();
            spacing = Some((
                parts[1].parse().unwrap(),
                parts[2].parse().unwrap(),
                parts[3].parse().unwrap(),
            ));
        }

        if line.starts_with("ORIGIN") {
            let parts: Vec<_> = line.split_whitespace().collect();
            origin = Some((
                parts[1].parse().unwrap(),
                parts[2].parse().unwrap(),
                parts[3].parse().unwrap(),
            ));
        }

        if line.starts_with("FIELD") {
            break;
        }
    }

    let [nx, ny, nz] = dims.ok_or("Missing DIMENSIONS")?;
    let spacing = spacing.ok_or("Missing SPACING")?;
    let origin = origin.ok_or("Missing ORIGIN")?;

    let mut total_points = 0usize;

    if let Some(Ok(line)) = lines.next() {
        let parts: Vec<_> = line.split_whitespace().collect();

        total_points = parts[2]
        .parse::<usize>()
        .map_err(|_| "Invalid FIELD size")?;
    }

    let mut data = Vec::with_capacity(total_points);

    for line in lines {
        let line = line.map_err(|e| e.to_string())?;
        for val in line.split_whitespace() {
            if !val.is_empty() {
                data.push(val.parse::<f64>().map_err(|_| "Parse error")?);
            }
        }

        if data.len() >= total_points {
            break;
        }
    }

    if data.len() != total_points {
        return Err(format!(
            "Expected {}, got {} values",
            total_points,
            data.len()
        ));
    }

    Ok(VtkVolume {
        dims: vec![nx, ny, nz],
        spacing,
        origin,
        data,
    })
}