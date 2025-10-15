use std::sync::{LazyLock, Mutex};

use byteorder::{LE, ReadBytesExt};
use image::ImageFormat;
use sdk::{
    db::{log, register_panic},
    gamepad::{Gamepad, GamepadSlot},
    logfmt,
    math::{Matrix4x4, Quaternion, Vector3},
    vdp::{
        self, BlendEquation, Color32, Texture, TextureFormat, TextureUnit,
        Topology, VertexSlotFormat,
    },
};

const BG: Color32 = Color32::new(128, 128, 255, 255);
const _BLACK: Color32 = Color32::new(0, 0, 0, 0);

#[unsafe(no_mangle)]
pub fn main(_: i32, _: i32) -> i32 {
    log(c"main start");
    register_panic();
    vdp::set_vsync_handler(Some(vsync_handler));
    log(c"main end");
    0
}

static MODEL: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/untitled.3d"));
static MODEL_DATA: LazyLock<Vec<f32>> = LazyLock::new(|| decode_data(MODEL));
static TEXTURE: &[u8] = include_bytes!("../../assets/untitled.png");
static TEXTURE_DATA: LazyLock<Vec<u8>> = LazyLock::new(|| {
    image::load_from_memory_with_format(TEXTURE, ImageFormat::Png)
        .unwrap()
        .flipv()
        .into_rgba8()
        .into_raw()
});

fn decode_data(mut model: &[u8]) -> Vec<f32> {
    log(c"decoding");
    let verts_count = model.read_u32::<LE>().unwrap();
    let vert_texs_count = model.read_u32::<LE>().unwrap();
    let faces_count = model.read_u32::<LE>().unwrap();
    let verts: Vec<[f32; 3]> = (0..verts_count)
        .map(|_| {
            let x = model.read_f32::<LE>().unwrap();
            let y = model.read_f32::<LE>().unwrap();
            let z = model.read_f32::<LE>().unwrap();
            [x, y, z]
        })
        .collect();
    let vert_texs: Vec<[f32; 2]> = (0..vert_texs_count)
        .map(|_| {
            let u = model.read_f32::<LE>().unwrap();
            let v = model.read_f32::<LE>().unwrap();
            [u, v]
        })
        .collect();
    let faces: Vec<[(u32, u32); 3]> = (0..faces_count)
        .map(|_| {
            [(); 3].map(|()| {
                let v = model.read_u32::<LE>().unwrap();
                let vt = model.read_u32::<LE>().unwrap();
                (v, vt)
            })
        })
        .collect();
    faces
        .into_iter()
        .flat_map(|f| {
            f.map(|(vi, vti)| {
                let mut out = [[0.0; 4]; 4];

                out[0][..3].copy_from_slice(&verts[vi as usize]);
                out[0][3] = 1.0;
                out[1] = [0.0, 1.0, 1.0, 1.0];
                out[3][..2].copy_from_slice(&vert_texs[vti as usize]);

                out
            })
        })
        .flatten()
        .flatten()
        .collect()
}

static PRG_PROJ: &[u32] = &sdk::vu_asm::vu_asm!(
    ld r0 0     // slot 0 = position
    ld r1 1     // slot 1 = color
    ld r2 2     // slot 2 = ocolor
    ld r3 3     // slot 3 = texcoord

    ldc r4 0    // constant 0 = transform column 0
    ldc r5 1    // constant 1 = transform column 1
    ldc r6 2    // constant 2 = transform column 2
    ldc r7 3    // constant 3 = transform column 3

    mulm r0 r4  // transform position with matrix

    st pos r0
    st col r1
    st ocol r2
    st tex r3
);
static PRG_STAT: &[u32] = &sdk::vu_asm::vu_asm!(
    ld r0 0     // slot 0 = position
    ld r1 1     // slot 1 = color
    ld r2 2     // slot 2 = ocolor
    ld r3 3     // slot 3 = texcoord

    st pos r0
    st col r1
    st ocol r2
    st tex r3
);
static STATE: Mutex<State> = Mutex::new(State::new());

struct State {
    rot: f32,
}

impl State {
    const fn new() -> Self {
        Self { rot: 0.0 }
    }
}

fn vsync_handler() {
    let mut state = STATE.lock().unwrap();
    let input = Gamepad::new(GamepadSlot::SlotA).read_state();
    let rx = input.right_stick_x as f32 / i16::MAX as f32;
    state.rot += rx * 0.1;
    draw(&state);
}

fn draw(state: &State) {
    vdp::clear_color(BG);
    vdp::clear_depth(1.0);
    vdp::blend_equation(BlendEquation::Add);
    vdp::blend_func(vdp::BlendFactor::One, vdp::BlendFactor::Zero);
    vdp::depth_write(true);
    vdp::depth_func(vdp::Compare::LessOrEqual);

    vdp::set_vu_stride(size_of::<f32>() * 4 * 4);
    let projection =
        Matrix4x4::projection_ortho_aspect(640.0 / 480.0, 1.0, 0.0, 1.0)
            * Matrix4x4::scale(Vector3::new(0.2, 0.2, 0.2)).transposed()
            * Matrix4x4::translation(Vector3::new(0.0, -1.0, 0.0)).transposed()
            * Matrix4x4::rotation(Quaternion::from_euler(Vector3::new(
                0.0, state.rot, 0.0,
            )))
            .transposed();
    vdp::set_vu_cdata(0, &projection.get_row(0));
    vdp::set_vu_cdata(1, &projection.get_row(1));
    vdp::set_vu_cdata(2, &projection.get_row(2));
    vdp::set_vu_cdata(3, &projection.get_row(3));
    vdp::set_vu_layout(0, 0, VertexSlotFormat::FLOAT4);
    vdp::set_vu_layout(1, 16, VertexSlotFormat::FLOAT4);
    vdp::set_vu_layout(2, 32, VertexSlotFormat::FLOAT4);
    vdp::set_vu_layout(3, 48, VertexSlotFormat::FLOAT4);
    let texture =
        Texture::new(256, 256, false, TextureFormat::RGBA8888).unwrap();
    texture.set_texture_data(0, &TEXTURE_DATA);
    vdp::bind_texture_slot(TextureUnit::TU0, Some(&texture));
    vdp::upload_vu_program(PRG_PROJ);
    vdp::submit_vu::<f32>(Topology::TriangleList, &MODEL_DATA);
    // vdp::upload_vu_program(PRG_STAT);
    // vdp::submit_vu::<f32>(
    //     Topology::TriangleList,
    //     [
    //         [0.0, 0.0, 0.0, 1.0],
    //         [0.0; 4],
    //         [0.0; 4],
    //         [0.0; 4],
    //         [0.0, 1.0, 0.0, 1.0],
    //         [0.0; 4],
    //         [0.0; 4],
    //         [0.0; 4],
    //         [1.0, 1.0, 0.0, 1.0],
    //         [0.0; 4],
    //         [0.0; 4],
    //         [0.0; 4],
    //     ]
    //     .as_flattened(),
    // );
}
