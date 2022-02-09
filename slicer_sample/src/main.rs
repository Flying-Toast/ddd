use std::io::Read;
use std::fs::File;
use ddd::{
    ConfigProfile,
    parsing::{detect_stl_type, parse_mesh_file, MeshFileUnits},
    slice::Slicer,
    mesh::Scene,
    gcode::slices_to_gcode,
};

fn main() {
    let filename = std::env::args().skip(1).next();
    let mut filebytes = Vec::new();
    if let Some(filename) = filename {
        File::open(&filename).unwrap()
            .read_to_end(&mut filebytes).unwrap();
    } else {
        eprintln!("Need to specify filename.");
        return;
    }

    let mesh = parse_mesh_file(&filebytes, detect_stl_type(&filebytes), MeshFileUnits::Millimeters).unwrap();
    let mut scene = Scene::new();
    scene.add_mesh(mesh);

    let config = ConfigProfile {
        layer_height: 200_000,
        hotend_temperature: 100,
        travel_speed: 5,
    };
    let slicer = Slicer::new(&config);

    let slices = slicer.slice(scene).unwrap();
    let gcode = slices_to_gcode(&config, &slices);

    println!("{gcode}");
}
