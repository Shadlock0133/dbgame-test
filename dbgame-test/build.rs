use std::{
    env,
    fs::{self, File},
    io::Write,
    path::Path,
};

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let out_dir = Path::new(&out_dir);

    obj(out_dir, Path::new("../assets/untitled.obj"));
    obj(out_dir, Path::new("../assets/floor.obj"));
}

struct Vert {
    pos: [f32; 3],
    color: [u8; 4],
}

fn obj(out_dir: &Path, obj_path: &Path) {
    let obj = fs::read_to_string(obj_path).unwrap();

    let mut verts: Vec<Vert> = vec![];
    let mut vert_texs: Vec<[f32; 2]> = vec![];
    let mut faces: Vec<[(u32, u32); 3]> = vec![];

    let mut has_vert_colors = false;

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
                let (y, rest) = rest.split_once(' ').unwrap();
                if let Some((z, colors)) = rest.split_once(' ') {
                    has_vert_colors = true;
                    let (r, rest) = colors.split_once(' ').unwrap();
                    let (g, b) = rest.split_once(' ').unwrap();
                    let pos = [x, y, z].map(|v| v.parse().unwrap());
                    let [r, g, b] = [r, g, b]
                        .map(|v| v.parse::<f32>().unwrap())
                        .map(|x| (x * 255.0) as u8);
                    verts.push(Vert {
                        pos,
                        color: [r, g, b, 255],
                    });
                } else {
                    let z = rest;
                    let pos = [x, y, z].map(|v| v.parse().unwrap());
                    verts.push(Vert { pos, color: [0; 4] });
                }
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

    let mut out_file = File::create(
        out_dir.join(obj_path.with_extension("3d").file_name().unwrap()),
    )
    .unwrap();
    out_file.write_all(&[has_vert_colors as u8]).unwrap();
    let verts_len = u32::try_from(verts.len()).unwrap();
    out_file.write_all(&verts_len.to_le_bytes()).unwrap();
    let vert_texs_len = u32::try_from(vert_texs.len()).unwrap();
    out_file.write_all(&vert_texs_len.to_le_bytes()).unwrap();
    let faces_len = u32::try_from(faces.len()).unwrap();
    out_file.write_all(&faces_len.to_le_bytes()).unwrap();
    for Vert { pos, color } in verts {
        out_file
            .write_all(pos.map(|v| v.to_le_bytes()).as_flattened())
            .unwrap();
        if has_vert_colors {
            out_file.write_all(&color).unwrap();
        }
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
