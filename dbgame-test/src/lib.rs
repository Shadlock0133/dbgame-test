use sdk::{
    db::log, math::Vector4, vdp::{self, BlendEquation, Color32, Topology, VertexSlotFormat}
};

const BG: Color32 = Color32::new(128, 128, 255, 255);
const BLACK: Color32 = Color32::new(0, 0, 0, 0);

#[unsafe(no_mangle)]
pub fn main(_: i32, _: i32) -> i32 {
    // register_panic();
    vdp::set_vsync_handler(Some(vsync_handler));
    log(c"test");
    0
}

fn vsync_handler() {
    vdp::clear_color(BG);
    vdp::clear_depth(1.0);
    vdp::blend_equation(BlendEquation::Add);
    vdp::blend_func(vdp::BlendFactor::One, vdp::BlendFactor::Zero);
    vdp::depth_write(true);
    vdp::depth_func(vdp::Compare::LessOrEqual);

    vdp::set_vu_stride(size_of::<f32>() * 4 * 4);
    vdp::set_vu_cdata(0, &Vector4::unit_x());
    vdp::set_vu_cdata(1, &Vector4::unit_y());
    vdp::set_vu_cdata(2, &Vector4::unit_z());
    vdp::set_vu_cdata(3, &Vector4::unit_w());
    vdp::set_vu_layout(0, 0, VertexSlotFormat::FLOAT4);
    vdp::set_vu_layout(1, 16, VertexSlotFormat::FLOAT4);
    vdp::set_vu_layout(2, 32, VertexSlotFormat::FLOAT4);
    vdp::set_vu_layout(3, 48, VertexSlotFormat::FLOAT4);
    vdp::upload_vu_program(&sdk::vu_asm::vu_asm!(
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
    ));
    vdp::submit_vu::<f32>(
        Topology::TriangleList,
        [
            [0.0, 0.5, 0.0, 1.0],
            [1.0, 0.0, 0.0, 1.0],
            [0.0; 4],
            [0.0; 4],
            [-0.5, -0.5, 0.0, 1.0],
            [0.0, 1.0, 0.0, 1.0],
            [0.0; 4],
            [0.0; 4],
            [0.5, -0.5, 0.0, 1.0],
            [0.0, 0.0, 1.0, 1.0],
            [0.0; 4],
            [0.0; 4],
        ]
        .as_flattened(),
    );
}
