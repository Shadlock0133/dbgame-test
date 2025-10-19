#![cfg_attr(not(feature = "std"), no_std)]
#![deny(clippy::as_conversions)]

mod alloc;
pub mod audio;
pub mod clock;
pub mod db;
mod db_internal;
pub mod gamepad;
pub mod io;
pub mod math;
pub mod sound_driver;
pub mod vdp;

pub use dbsdk_vu_asm as vu_asm;

use core::cell::UnsafeCell;

struct SyncUnsafeCell<T>(UnsafeCell<T>);
unsafe impl<T: Sync> Sync for SyncUnsafeCell<T> {}

impl<T> SyncUnsafeCell<T> {
    const fn new(value: T) -> Self {
        Self(UnsafeCell::new(value))
    }

    const fn get(&self) -> *mut T {
        self.0.get()
    }
}
