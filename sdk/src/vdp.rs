use core::convert::TryInto;

use crate::{
    SyncUnsafeCell,
    db_internal::{
        vdp_allocRenderTexture, vdp_allocTexture, vdp_bindTextureSlot,
        vdp_blendEquation, vdp_blendFunc, vdp_clearColor, vdp_clearDepth,
        vdp_copyFbToTexture, vdp_depthFunc, vdp_depthWrite,
        vdp_getDepthQueryResult, vdp_getUsage, vdp_releaseTexture,
        vdp_setCulling, vdp_setRenderTarget, vdp_setSampleParamsSlot,
        vdp_setTexCombine, vdp_setTextureData, vdp_setTextureDataRegion,
        vdp_setTextureDataYUV, vdp_setVUCData, vdp_setVULayout,
        vdp_setVUStride, vdp_setVsyncHandler, vdp_setWinding,
        vdp_submitDepthQuery, vdp_submitVU, vdp_uploadVUProgram, vdp_viewport,
    },
    math::Vector4,
};

static VSYNC_HANDLER: SyncUnsafeCell<Option<fn()>> = SyncUnsafeCell::new(None);

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Color32 {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color32 {
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Color32 {
        Color32 { r, g, b, a }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Rectangle {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl Rectangle {
    pub const fn new(x: i32, y: i32, width: i32, height: i32) -> Rectangle {
        Rectangle {
            x,
            y,
            width,
            height,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum TextureError {
    DimensionsInvalid,
    AllocationFailed,
}

pub trait DBTex {
    fn get_handle(&self) -> i32;
}

#[repr(C)]
pub struct Texture {
    pub format: TextureFormat,
    pub width: i32,
    pub height: i32,
    pub mipmap: bool,
    handle: i32,
}

#[repr(C)]
pub struct RenderTexture {
    pub width: i32,
    pub height: i32,
    handle: i32,
}

impl DBTex for Texture {
    fn get_handle(&self) -> i32 {
        self.handle
    }
}

impl DBTex for RenderTexture {
    fn get_handle(&self) -> i32 {
        self.handle
    }
}

impl Texture {
    pub fn new(
        width: i32,
        height: i32,
        mipmap: bool,
        format: TextureFormat,
    ) -> Result<Texture, TextureError> {
        // dimensions must be power of two (unless this is a YUV420 image)
        if format != TextureFormat::YUV420
            && ((width & (width - 1)) != 0 || (height & (height - 1)) != 0)
        {
            return Result::Err(TextureError::DimensionsInvalid);
        }

        // allocate and check to see if allocation failed
        let handle = unsafe { vdp_allocTexture(mipmap, format, width, height) };
        if handle == -1 {
            return Result::Err(TextureError::AllocationFailed);
        }

        Result::Ok(Texture {
            format,
            mipmap,
            width,
            height,
            handle,
        })
    }

    /// Upload texture data for the given mip level of this texture
    pub fn set_texture_data<T>(&self, level: i32, data: &[T]) {
        unsafe {
            let len_bytes = core::mem::size_of_val(data);
            vdp_setTextureData(
                self.handle,
                level,
                data.as_ptr().cast(),
                len_bytes.try_into().unwrap(),
            )
        }
    }

    /// Upload individual planes for this YUV texture
    pub fn set_texture_data_yuv(
        &self,
        y_data: &[u8],
        u_data: &[u8],
        v_data: &[u8],
    ) {
        unsafe {
            vdp_setTextureDataYUV(
                self.handle,
                y_data.as_ptr().cast(),
                y_data.len().try_into().unwrap(),
                u_data.as_ptr().cast(),
                u_data.len().try_into().unwrap(),
                v_data.as_ptr().cast(),
                v_data.len().try_into().unwrap(),
            )
        }
    }

    /// Upload texture data for the given mip level and region of this texture
    pub fn set_texture_data_region<T>(
        &self,
        level: i32,
        dst_rect: Option<Rectangle>,
        data: &[T],
    ) {
        unsafe {
            match dst_rect {
                Some(v) => {
                    let len_bytes = core::mem::size_of_val(data);
                    vdp_setTextureDataRegion(
                        self.handle,
                        level,
                        &v,
                        data.as_ptr().cast(),
                        len_bytes.try_into().unwrap(),
                    )
                }
                None => vdp_setTextureData(
                    self.handle,
                    level,
                    data.as_ptr().cast(),
                    data.len().try_into().unwrap(),
                ),
            }
        }
    }

    /// Copy a region of the framebuffer into a region of the given texture
    pub fn copy_framebuffer_to_texture(
        target: &Texture,
        src_rect: Rectangle,
        dst_rect: Rectangle,
    ) {
        unsafe { vdp_copyFbToTexture(&src_rect, &dst_rect, target.handle) }
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        unsafe { vdp_releaseTexture(self.handle) }
    }
}

impl RenderTexture {
    pub fn new(width: i32, height: i32) -> Result<RenderTexture, TextureError> {
        // dimensions must be power of two
        if (width & (width - 1)) != 0 || (height & (height - 1)) != 0 {
            return Err(TextureError::DimensionsInvalid);
        }

        // allocate and check to see if allocation failed
        let handle = unsafe { vdp_allocRenderTexture(width, height) };
        if handle == -1 {
            return Err(TextureError::AllocationFailed);
        }

        Ok(RenderTexture {
            width,
            height,
            handle,
        })
    }
}

impl Drop for RenderTexture {
    fn drop(&mut self) {
        unsafe { vdp_releaseTexture(self.handle) }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub enum Compare {
    Never = 0x0200,
    Less = 0x0201,
    Equal = 0x0202,
    LessOrEqual = 0x0203,
    Greater = 0x0204,
    NotEqual = 0x0205,
    GreaterOrEqual = 0x0206,
    Always = 0x0207,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub enum BlendEquation {
    Add = 0x8006,
    Subtract = 0x800A,
    ReverseSubtract = 0x800B,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub enum BlendFactor {
    Zero = 0,
    One = 1,
    SrcColor = 0x0300,
    OneMinusSrcColor = 0x0301,
    SrcAlpha = 0x0302,
    OneMinusSrcAlpha = 0x0303,
    DstAlpha = 0x0304,
    OneMinusDstAlpha = 0x0305,
    DstColor = 0x0306,
    OneMinusDstColor = 0x0307,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub enum WindingOrder {
    Clockwise = 0x0900,
    CounterClockwise = 0x0901,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub enum Topology {
    LineList = 0x0000,
    LineStrip = 0x0001,
    TriangleList = 0x0002,
    TriangleStrip = 0x0003,
}

#[repr(C)]
#[derive(Clone, Copy, PartialEq)]
pub enum TextureFormat {
    RGB565 = 0,
    RGBA4444 = 1,
    RGBA8888 = 2,
    DXT1 = 3,
    DXT3 = 4,
    YUV420 = 5,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub enum TextureFilter {
    Nearest = 0x2600,
    Linear = 0x2601,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub enum TextureWrap {
    Clamp = 0x812F,
    Repeat = 0x2901,
    Mirror = 0x8370,
}

#[repr(C)]
#[derive(Clone, Copy, PartialEq)]
pub enum VertexSlotFormat {
    FLOAT1 = 0,
    FLOAT2 = 1,
    FLOAT3 = 2,
    FLOAT4 = 3,
    UNORM4 = 4,
    SNORM4 = 5,
}

#[repr(C)]
#[derive(Clone, Copy, PartialEq)]
pub enum TexCombine {
    None = 0,
    Mul = 1,
    Add = 2,
    Sub = 3,
    Mix = 4,
    Dot3 = 5,
}

#[repr(C)]
#[derive(Clone, Copy, PartialEq)]
pub enum TextureUnit {
    TU0 = 0,
    TU1 = 1,
}

unsafe extern "C" fn real_vsync_handler() {
    if let Some(handler) = unsafe { *VSYNC_HANDLER.get() } {
        handler()
    }
}

/// Clear the backbuffer to the given color
pub fn clear_color(color: Color32) {
    unsafe { vdp_clearColor(&color) }
}

/// Clear the depth buffer to the given depth value
pub fn clear_depth(depth: f32) {
    unsafe { vdp_clearDepth(depth) }
}

/// Set whether depth writes are enabled
pub fn depth_write(enable: bool) {
    unsafe { vdp_depthWrite(enable) }
}

/// Set the current depth test comparison
pub fn depth_func(compare: Compare) {
    unsafe { vdp_depthFunc(compare) }
}

/// Set the blend equation mode
pub fn blend_equation(mode: BlendEquation) {
    unsafe { vdp_blendEquation(mode) }
}

/// Set the source and destination blend factors
pub fn blend_func(src_factor: BlendFactor, dst_factor: BlendFactor) {
    unsafe { vdp_blendFunc(src_factor, dst_factor) }
}

/// Set the winding order for backface culling
pub fn set_winding(winding: WindingOrder) {
    unsafe { vdp_setWinding(winding) }
}

/// Set backface culling enabled or disabled
pub fn set_culling(enabled: bool) {
    unsafe { vdp_setCulling(enabled) }
}

/// Get total texture memory usage in bytes
pub fn get_usage() -> i32 {
    unsafe { vdp_getUsage() }
}

/// Set the current viewport rect
pub fn viewport(rect: Rectangle) {
    unsafe { vdp_viewport(rect.x, rect.y, rect.width, rect.height) }
}

/// Compare a region of the depth buffer against the given reference value
pub fn submit_depth_query(ref_val: f32, compare: Compare, rect: Rectangle) {
    unsafe {
        vdp_submitDepthQuery(
            ref_val,
            compare,
            rect.x,
            rect.y,
            rect.width,
            rect.height,
        )
    }
}

/// Get the number of pixels which passed the submitted depth query
pub fn get_depth_query_result() -> i32 {
    unsafe { vdp_getDepthQueryResult() }
}

/// Set one of VU's 16 const data slots to given vector
pub fn set_vu_cdata(offset: usize, data: &Vector4) {
    unsafe {
        vdp_setVUCData(offset.try_into().unwrap(), <*const _>::cast(data))
    }
}

/// Configure input vertex element slot layout
pub fn set_vu_layout(slot: usize, offset: usize, format: VertexSlotFormat) {
    unsafe {
        vdp_setVULayout(
            slot.try_into().unwrap(),
            offset.try_into().unwrap(),
            format,
        )
    }
}

/// Set stride of input vertex data (size of each vertex in bytes)
pub fn set_vu_stride(stride: usize) {
    unsafe { vdp_setVUStride(stride.try_into().unwrap()) }
}

/// Upload a new VU program
pub fn upload_vu_program(program: &[u32]) {
    unsafe {
        let program_len = core::mem::size_of_val(program);
        vdp_uploadVUProgram(
            program.as_ptr().cast(),
            program_len.try_into().unwrap(),
        )
    }
}

/// Submit geometry to be processed by the VU
pub fn submit_vu<T>(topology: Topology, data: &[T]) {
    unsafe {
        let data_len = core::mem::size_of_val(data);
        vdp_submitVU(
            topology,
            data.as_ptr().cast(),
            data_len.try_into().unwrap(),
        )
    }
}

/// Set the current render target
pub fn set_render_target(texture: Option<RenderTexture>) {
    unsafe { vdp_setRenderTarget(texture.map_or(-1, |v| v.handle)) }
}

/// Set texture sample params for the given texture unit
pub fn set_sample_params_slot(
    slot: TextureUnit,
    filter: TextureFilter,
    wrap_u: TextureWrap,
    wrap_v: TextureWrap,
) {
    unsafe { vdp_setSampleParamsSlot(slot, filter, wrap_u, wrap_v) }
}

/// Bind a texture to the given texture unit
pub fn bind_texture_slot<T: DBTex>(slot: TextureUnit, texture: Option<&T>) {
    unsafe { vdp_bindTextureSlot(slot, texture.map_or(-1, |v| v.get_handle())) }
}

/// Set texture combiner mode
pub fn set_tex_combine(tex_combine: TexCombine, vtx_combine: TexCombine) {
    unsafe { vdp_setTexCombine(tex_combine, vtx_combine) }
}

/// Set an optional handler for vertical sync
pub fn set_vsync_handler(handler: Option<fn()>) {
    unsafe {
        VSYNC_HANDLER.get().write(handler);
        vdp_setVsyncHandler(real_vsync_handler)
    }
}
