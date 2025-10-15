use std::{
    env,
    fs::{self, File},
    io::Write,
    path::Path,
};

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let out_dir = Path::new(&out_dir);

    let obj = fs::read_to_string("../assets/untitled.obj").unwrap();

    let mut verts: Vec<[f32; 3]> = vec![];
    let mut vert_texs: Vec<[f32; 2]> = vec![];
    let mut faces: Vec<[(u32, u32); 3]> = vec![];

    for line in obj.lines() {
        if line.is_empty() {
            continue;
        }
        let (kind, data) = line.split_once(' ').unwrap();
        match kind {
            "#" => (), // comment
            "o" => (), // object name
            "v" => {
                let (x, rest) = data.split_once(' ').unwrap();
                let (y, z) = rest.split_once(' ').unwrap();
                verts.push([x, y, z].map(|v| v.parse().unwrap()));
            }
            "vt" => {
                let (u, v) = data.split_once(' ').unwrap();
                vert_texs.push([u, v].map(|vt| vt.parse().unwrap()));
            }
            "f" => {
                let (a, rest) = data.split_once(' ').unwrap();
                let (b, c) = rest.split_once(' ').unwrap();
                faces.push([a, b, c].map(|f| {
                    let (i, it) = f.split_once("/").unwrap();
                    (
                        i.parse::<u32>().unwrap() - 1,
                        it.parse::<u32>().unwrap() - 1,
                    )
                }));
            }
            "s" => (), // smoothing
            _ => todo!("unknown line kind: {kind}"),
        }
    }

    let mut out_file = File::create(out_dir.join("untitled.3d")).unwrap();
    out_file
        .write_all(&u32::try_from(verts.len()).unwrap().to_le_bytes())
        .unwrap();
    out_file
        .write_all(&u32::try_from(vert_texs.len()).unwrap().to_le_bytes())
        .unwrap();
    out_file
        .write_all(&u32::try_from(faces.len()).unwrap().to_le_bytes())
        .unwrap();
    for v in verts {
        out_file
            .write_all(v.map(|v| v.to_le_bytes()).as_flattened())
            .unwrap();
    }
    for vt in vert_texs {
        out_file
            .write_all(vt.map(|vt| vt.to_le_bytes()).as_flattened())
            .unwrap();
    }
    for f in faces {
        out_file
            .write_all(
                f.map(|(v, vt)| [v.to_le_bytes(), vt.to_le_bytes()])
                    .as_flattened()
                    .as_flattened(),
            )
            .unwrap();
    }
}
