use core::ffi::{c_char, c_void};

use crate::{
    SyncUnsafeCell,
    audio::AudioVoiceParam,
    clock::DateTime,
    gamepad::{GamepadSlot, GamepadState},
    io::{FileMode, SeekOrigin},
    vdp::{
        BlendEquation, BlendFactor, Color32, Compare, Rectangle, TexCombine,
        TextureFilter, TextureFormat, TextureUnit, TextureWrap, Topology,
        VertexSlotFormat, WindingOrder,
    },
};

#[repr(C)]
pub struct NativeDirectoryInfo {
    pub name: [i8; 32],
    pub created: u64,
    pub modified: u64,
    pub size: i32,
    pub is_directory: u32,
}

unsafe extern "C" {
    pub fn db_log(strptr: *const c_char);
    pub fn vdp_setVsyncHandler(tick: unsafe extern "C" fn());
    pub fn vdp_clearColor(colorptr: *const Color32);
    pub fn vdp_clearDepth(depth: f32);
    pub fn vdp_depthWrite(enable: bool);
    pub fn vdp_depthFunc(compare: Compare);
    pub fn vdp_blendEquation(mode: BlendEquation);
    pub fn vdp_blendFunc(srcFactor: BlendFactor, dstFactor: BlendFactor);
    pub fn vdp_setWinding(winding: WindingOrder);
    pub fn vdp_setCulling(enabled: bool);
    pub fn vdp_allocTexture(
        mipmap: bool,
        format: TextureFormat,
        width: i32,
        height: i32,
    ) -> i32;
    pub fn vdp_releaseTexture(handle: i32);
    pub fn vdp_getUsage() -> i32;
    pub fn vdp_setTextureData(
        handle: i32,
        level: i32,
        data: *const c_void,
        dataLen: i32,
    );
    pub fn vdp_setTextureDataYUV(
        handle: i32,
        yData: *const c_void,
        yDataLen: i32,
        uData: *const c_void,
        uDataLen: i32,
        vData: *const c_void,
        vDataLen: i32,
    );
    pub fn vdp_setTextureDataRegion(
        handle: i32,
        level: i32,
        dstRect: *const Rectangle,
        data: *const c_void,
        dataLen: i32,
    );
    pub fn vdp_copyFbToTexture(
        srcRect: *const Rectangle,
        dstRect: *const Rectangle,
        dstTexture: i32,
    );
    pub fn vdp_setVUCData(offset: i32, data: *const c_void);
    pub fn vdp_setVULayout(slot: i32, offset: i32, format: VertexSlotFormat);
    pub fn vdp_setVUStride(stride: i32);
    pub fn vdp_uploadVUProgram(program: *const c_void, programLen: i32);
    pub fn vdp_submitVU(topology: Topology, data: *const c_void, dataLen: i32);
    pub fn vdp_setSampleParamsSlot(
        slot: TextureUnit,
        filter: TextureFilter,
        wrap_u: TextureWrap,
        wrap_v: TextureWrap,
    );
    pub fn vdp_bindTextureSlot(slot: TextureUnit, handle: i32);
    pub fn vdp_setTexCombine(tex_combine: TexCombine, vtx_combine: TexCombine);
    pub fn vdp_allocRenderTexture(width: i32, height: i32) -> i32;
    pub fn vdp_setRenderTarget(handle: i32);
    pub fn vdp_viewport(x: i32, y: i32, w: i32, h: i32);
    pub fn vdp_submitDepthQuery(
        refVal: f32,
        compare: Compare,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
    );
    pub fn vdp_getDepthQueryResult() -> i32;
    pub fn audio_alloc(data: *const c_void, dataLen: i32, audioFmt: i32)
    -> i32;
    pub fn audio_allocCompressed(
        data: *const c_void,
        dataLen: i32,
        chunkLen: i32,
    ) -> i32;
    pub fn audio_free(handle: i32);
    pub fn audio_getUsage() -> i32;
    pub fn audio_queueSetParam_i(
        slot: i32,
        param: AudioVoiceParam,
        value: i32,
        time: f64,
    );
    pub fn audio_queueSetParam_f(
        slot: i32,
        param: AudioVoiceParam,
        value: f32,
        time: f64,
    );
    pub fn audio_queueStartVoice(slot: i32, time: f64);
    pub fn audio_queueStopVoice(slot: i32, time: f64);
    pub fn audio_getVoiceState(slot: i32) -> bool;
    pub fn audio_getTime() -> f64;
    pub fn audio_setReverbParams(
        roomSize: f32,
        damping: f32,
        width: f32,
        wet: f32,
        dry: f32,
    );
    pub fn audio_initSynth(dataPtr: *const u8, dataLen: i32) -> bool;
    pub fn audio_playMidi(
        dataPtr: *const u8,
        dataLen: i32,
        looping: bool,
    ) -> bool;
    pub fn audio_setMidiReverb(enable: bool);
    pub fn audio_setMidiVolume(volume: f32);
    pub fn gamepad_isConnected(slot: GamepadSlot) -> bool;
    pub fn gamepad_readState(slot: GamepadSlot, ptr: *mut GamepadState);
    pub fn gamepad_setRumble(slot: GamepadSlot, enable: bool);
    pub fn fs_deviceExists(devstr: *const c_char) -> bool;
    pub fn fs_deviceEject(devstr: *const c_char);
    pub fn fs_fileExists(pathstr: *const c_char) -> bool;
    pub fn fs_open(pathstr: *const c_char, mode: FileMode) -> i32;
    pub fn fs_read(handle: i32, buffer: *mut c_void, bufferLen: i32) -> i32;
    pub fn fs_write(handle: i32, buffer: *const c_void, bufferLen: i32) -> i32;
    pub fn fs_seek(handle: i32, position: i32, whence: SeekOrigin) -> i32;
    pub fn fs_tell(handle: i32) -> i32;
    pub fn fs_flush(handle: i32);
    pub fn fs_close(handle: i32);
    pub fn fs_eof(handle: i32) -> bool;
    pub fn fs_openDir(pathstr: *const c_char) -> i32;
    pub fn fs_readDir(dir: i32) -> *const NativeDirectoryInfo;
    pub fn fs_rewindDir(dir: i32);
    pub fn fs_closeDir(dir: i32);
    pub fn fs_allocMemoryCard(
        filenamestr: *const c_char,
        icondata: *const u8,
        iconpalette: *const u16,
        blocks: i32,
    ) -> i32;
    pub fn clock_getTimestamp() -> u64;
    pub fn clock_timestampToDatetime(timestamp: u64, datetime: *mut DateTime);
    // pub fn clock_datetimeToTimestamp(datetime: *const DateTime) -> u64;
}

#[used]
pub static ERRNO: SyncUnsafeCell<i32> = SyncUnsafeCell::new(0);

#[unsafe(no_mangle)]
pub fn __errno_location() -> *mut i32 {
    ERRNO.get()
}
