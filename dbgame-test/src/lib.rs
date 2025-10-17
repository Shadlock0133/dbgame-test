use std::{
    f32::consts::{FRAC_PI_2, TAU},
    sync::{LazyLock, Mutex},
};

use byteorder::{LE, ReadBytesExt};
use image::{ImageBuffer, ImageFormat, Rgba};
use sdk::{
    db::{log, register_panic}, gamepad::{Gamepad, GamepadButton, GamepadSlot, GamepadState}, logfmt, math::{Matrix4x4, Quaternion, Vector2, Vector3}, vdp::{
        self, BlendEquation, BlendFactor, Color32, Texture, TextureFormat,
        TextureUnit, Topology, VertexSlotFormat,
    }
};

const BG: Color32 = Color32::new(20, 30, 42, 255);
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
static MODEL_TEXTURE_PNG: &[u8] = include_bytes!("../../assets/untitled.png");
static MODEL_TEXTURE: LazyLock<ImageBuffer<Rgba<u8>, Vec<u8>>> =
    LazyLock::new(|| {
        image::load_from_memory_with_format(MODEL_TEXTURE_PNG, ImageFormat::Png)
            .unwrap()
            .flipv()
            .into_rgba8()
    });

static FLOOR: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/floor.3d"));
static FLOOR_DATA: LazyLock<Vec<f32>> = LazyLock::new(|| decode_data(FLOOR));
static FLOOR_TEXTURE_PNG: &[u8] = include_bytes!("../../assets/floor.png");
static FLOOR_TEXTURE: LazyLock<ImageBuffer<Rgba<u8>, Vec<u8>>> =
    LazyLock::new(|| {
        image::load_from_memory_with_format(FLOOR_TEXTURE_PNG, ImageFormat::Png)
            .unwrap()
            .flipv()
            .into_rgba8()
    });

fn decode_data(mut model: &[u8]) -> Vec<f32> {
    let has_vert_colors = match model.read_u8().unwrap() {
        0 => false,
        1 => true,
        _ => unimplemented!(),
    };
    let verts_count = model.read_u32::<LE>().unwrap();
    let vert_texs_count = model.read_u32::<LE>().unwrap();
    let faces_count = model.read_u32::<LE>().unwrap();
    let verts: Vec<[f32; 3]> = (0..verts_count)
        .map(|_| {
            let x = model.read_f32::<LE>().unwrap();
            let y = model.read_f32::<LE>().unwrap();
            let z = model.read_f32::<LE>().unwrap();
            if has_vert_colors {
                let _r = model.read_u8().unwrap();
                let _g = model.read_u8().unwrap();
                let _b = model.read_u8().unwrap();
                let _a = model.read_u8().unwrap();
            }
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
            [(); _].map(|()| {
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
                out[1] = [1.0; 4];
                out[3][..2].copy_from_slice(&vert_texs[vti as usize]);

                out
            })
        })
        .flatten()
        .flatten()
        .collect()
}

fn set_vu_cdata_matrix4x4(offset: usize, matrix: Matrix4x4) {
    for i in 0..4 {
        vdp::set_vu_cdata(offset + i, &matrix.get_column(i));
    }
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

struct State {
    camera_pos: Vector3,
    camera_rot: Vector2,
    player_pos: Vector3,
    player_rot: f32,
    rainbow: f32,
}

impl State {
    const fn new() -> Self {
        Self {
            camera_pos: Vector3::zero(),
            camera_rot: Vector2::zero(),
            player_pos: Vector3::zero(),
            player_rot: 0.0,
            rainbow: 0.0,
        }
    }
}

fn vsync_handler() {
    static STATE: Mutex<State> = Mutex::new(State::new());
    let mut state = STATE.lock().unwrap();

    let input = Gamepad::new(GamepadSlot::SlotA).read_state();

    update(&mut state, input);
    draw(&state);
}

fn check_input_axis(
    input: GamepadState,
    button1: GamepadButton,
    button2: GamepadButton,
) -> f32 {
    match (
        input.button_mask.contains(button1),
        input.button_mask.contains(button2),
    ) {
        (true, false) => 1.0,
        (false, true) => -1.0,
        _ => 0.0,
    }
}

fn i16_to_f32(value: i16) -> f32 {
    (value as f32 / i16::MAX as f32).clamp(-1.0, 1.0)
}

fn update(state: &mut State, input: GamepadState) {
    let lx = i16_to_f32(input.left_stick_x);
    let ly = i16_to_f32(input.left_stick_y);
    let rx = i16_to_f32(input.right_stick_x);
    let ry = i16_to_f32(input.right_stick_y);

    let height_delta =
        check_input_axis(input, GamepadButton::L1, GamepadButton::R1);
    let player_rot_delta =
        check_input_axis(input, GamepadButton::L2, GamepadButton::R2);

    state.player_rot =
        (state.player_rot + player_rot_delta * 0.03).rem_euclid(TAU);

    let player_dir =
        Vector2::new(state.player_rot.sin(), state.player_rot.cos());
    let player_dir_side =
        Vector2::new(state.player_rot.cos(), -state.player_rot.sin());

    state.player_pos.z += (ly * -0.06 * player_dir).y;
    state.player_pos.y += height_delta * 0.06;
    state.player_pos.x += (lx * 0.06 * player_dir_side).x;

    state.camera_pos = state.player_pos;

    state.camera_rot.x += rx * 0.03;
    state.camera_rot.y =
        (state.camera_rot.y - ry * 0.03).clamp(-0.01, FRAC_PI_2);

    let rainbow_delta =
        check_input_axis(input, GamepadButton::A, GamepadButton::B);
    state.rainbow = (state.rainbow + rainbow_delta * 0.03).clamp(0.0, 1.0);
}

fn draw(state: &State) {
    vdp::clear_color(BG);
    vdp::clear_depth(1.0);
    vdp::set_culling(true);
    vdp::blend_equation(BlendEquation::Add);
    vdp::blend_func(BlendFactor::One, BlendFactor::Zero);
    vdp::depth_write(true);
    vdp::depth_func(vdp::Compare::LessOrEqual);

    vdp::set_vu_stride(size_of::<f32>() * 4 * 4);
    vdp::set_vu_layout(0, 0, VertexSlotFormat::FLOAT4);
    vdp::set_vu_layout(1, 16, VertexSlotFormat::FLOAT4);
    vdp::set_vu_layout(2, 32, VertexSlotFormat::FLOAT4);
    vdp::set_vu_layout(3, 48, VertexSlotFormat::FLOAT4);
    vdp::upload_vu_program(PRG_PROJ);

    let model_texture = Texture::new(
        MODEL_TEXTURE.width().try_into().unwrap(),
        MODEL_TEXTURE.height().try_into().unwrap(),
        false,
        TextureFormat::RGBA8888,
    )
    .unwrap();
    model_texture.set_texture_data(0, &MODEL_TEXTURE);

    let floor_texture = Texture::new(
        FLOOR_TEXTURE.width().try_into().unwrap(),
        FLOOR_TEXTURE.height().try_into().unwrap(),
        false,
        TextureFormat::RGBA8888,
    )
    .unwrap();
    floor_texture.set_texture_data(0, &FLOOR_TEXTURE);

    let projection =
        Matrix4x4::projection_ortho_aspect(640.0 / 480.0, 4.0, 0.0, 1.0);
    let mat = Matrix4x4::translation(state.player_pos - state.camera_pos)
        * Matrix4x4::rotation(Quaternion::from_euler(Vector3::new(
            0.0,
            state.player_rot + state.camera_rot.x,
            0.0,
        )))
        * Matrix4x4::rotation(Quaternion::from_euler(Vector3::new(
            state.camera_rot.y,
            0.0,
            0.0,
        )))
        * Matrix4x4::scale(Vector3::new(0.2, 0.2, 0.2))
        * projection;
    set_vu_cdata_matrix4x4(0, mat);

    vdp::bind_texture_slot(TextureUnit::TU0, Some(&model_texture));
    vdp::submit_vu::<f32>(Topology::TriangleList, &MODEL_DATA);

    let mat = Matrix4x4::translation(state.camera_pos)
        * Matrix4x4::rotation(Quaternion::from_euler(Vector3::new(
            0.0,
            state.camera_rot.x,
            0.0,
        )))
        * Matrix4x4::rotation(Quaternion::from_euler(Vector3::new(
            state.camera_rot.y,
            0.0,
            0.0,
        )))
        * Matrix4x4::scale(Vector3::new(0.2, 0.2, 0.2))
        * projection;
    set_vu_cdata_matrix4x4(0, mat);

    vdp::bind_texture_slot(TextureUnit::TU0, Some(&floor_texture));
    vdp::submit_vu::<f32>(Topology::TriangleList, &FLOOR_DATA);
    vdp::bind_texture_slot(TextureUnit::TU0, None::<&Texture>);

    set_vu_cdata_matrix4x4(0, Matrix4x4::identity());
    vdp::blend_equation(BlendEquation::Add);
    vdp::blend_func(BlendFactor::SrcAlpha, BlendFactor::DstAlpha);
    let amount = state.rainbow;
    vdp::submit_vu::<f32>(
        Topology::TriangleList,
        [
            [-1.0, 1.0, 0.0, 1.0],
            [1.0, 0.0, 0.0, amount],
            [0.0; 4],
            [0.0; 4],
            [-1.0, -1.0, 0.0, 1.0],
            [0.0, 1.0, 0.0, amount],
            [0.0; 4],
            [0.0; 4],
            [1.0, 1.0, 0.0, 1.0],
            [0.0, 0.0, 1.0, amount],
            [0.0; 4],
            [0.0; 4],
            [-1.0, -1.0, 0.0, 1.0],
            [0.0, 1.0, 0.0, amount],
            [0.0; 4],
            [0.0; 4],
            [1.0, -1.0, 0.0, 1.0],
            [1.0, 0.0, 0.0, amount],
            [0.0; 4],
            [0.0; 4],
            [1.0, 1.0, 0.0, 1.0],
            [0.0, 0.0, 1.0, amount],
            [0.0; 4],
            [0.0; 4],
        ]
        .as_flattened(),
    );
}
