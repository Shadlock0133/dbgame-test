use core::{
    alloc::Layout,
    ffi::c_void,
    mem::{align_of, size_of},
};

const SIZE_SIZE: usize = size_of::<i64>();
const MAX_ALIGN: usize = align_of::<i64>();

#[unsafe(no_mangle)]
#[cfg(feature = "std")]
pub fn malloc(size: i32) -> *mut c_void {
    // basically we just allocate a block of memory with an 8-byte preamble that stores the length (we use 8 bytes to maintain alignment)
    // that way, we can pass the raw pointer to C, and then when we get the pointer back we do some arithmetic to get at the original preamble
    // and then we can reconstruct the Layout that was passed to alloc

    // NOTE: we align to align_of::<i64>() which is the equivalent of C's max_align_t for wasm32
    // this matches the behavior of C's malloc

    // NOTE: removed write_unaligned b/c it is no longer necessary - malloc is already 8-byte aligned

    let actual_size = SIZE_SIZE + usize::try_from(size).unwrap();
    let layout = Layout::array::<u8>(actual_size)
        .unwrap()
        .align_to(MAX_ALIGN)
        .unwrap();
    let mem = unsafe { std::alloc::alloc(layout) };
    if !mem.is_null() {
        unsafe { mem.cast::<i64>().write(size.into()) };
    }
    unsafe { mem.add(SIZE_SIZE) }.cast()
}

#[unsafe(no_mangle)]
#[cfg(feature = "std")]
pub fn free(ptr: *mut c_void) {
    // back up by 8 bytes to get at the preamble, which contains the allocated size

    // NOTE: removed read_unaligned b/c it is no longer necessary - malloc is already 8-byte aligned

    let ptr = unsafe { ptr.sub(SIZE_SIZE) }.cast::<u8>();
    let size = unsafe { ptr.cast::<i64>().read() };
    let actual_size = SIZE_SIZE + usize::try_from(size).unwrap();
    let layout = Layout::array::<u8>(actual_size)
        .unwrap()
        .align_to(MAX_ALIGN)
        .unwrap();
    unsafe { std::alloc::dealloc(ptr, layout) };
}
