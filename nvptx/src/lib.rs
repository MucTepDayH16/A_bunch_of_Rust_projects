#![no_std]
#![feature(abi_ptx, stdarch_nvptx, alloc_error_handler, panic_info_message)]

extern crate alloc;

use alloc::{
    alloc::{GlobalAlloc, Layout},
    string::String,
};
use core::{arch::nvptx::*, fmt::Write, marker::PhantomData, slice};

pub struct PTXAllocator;

unsafe impl GlobalAlloc for PTXAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        malloc(layout.size()) as *mut u8
    }
    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        free(ptr as *mut _);
    }
}

#[global_allocator]
static GLOBAL_ALLOCATOR: PTXAllocator = PTXAllocator;

#[panic_handler]
fn panic(info: &::core::panic::PanicInfo) -> ! {
    let mut panic_msg = String::new();
    if let Some(message) = info.message() {
        let _ = panic_msg.write_fmt(*message);
    }
    unsafe { ::core::arch::nvptx::trap() }
}

#[alloc_error_handler]
fn alloc_error_handler(_: ::core::alloc::Layout) -> ! {
    unsafe { ::core::arch::nvptx::trap() }
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct PtxSlice<T: 'static> {
    data: *const T,
    len: usize,
    _phantom: PhantomData<&'static T>,
}

impl<T> PtxSlice<T> {
    pub unsafe fn as_slice(&self) -> &[T] {
        slice::from_raw_parts(self.data, self.len)
    }
}

#[repr(C)]
pub struct PtxSliceMut<T: 'static> {
    data: *mut T,
    len: usize,
    _phantom: PhantomData<&'static T>,
}

impl<T> From<PtxSliceMut<T>> for PtxSlice<T> {
    fn from(slice: PtxSliceMut<T>) -> Self {
        Self {
            data: slice.data,
            len: slice.len,
            _phantom: PhantomData,
        }
    }
}

impl<T> PtxSliceMut<T> {
    pub unsafe fn as_slice(&self) -> &[T] {
        slice::from_raw_parts(self.data, self.len)
    }

    pub unsafe fn as_slice_mut(&mut self) -> &mut [T] {
        slice::from_raw_parts_mut(self.data, self.len)
    }
}

#[no_mangle]
pub unsafe extern "ptx-kernel" fn add(
    a: PtxSlice<u64>,
    b: PtxSlice<u64>,
    mut c: PtxSliceMut<u64>,
) {
    let a = a.as_slice();
    let b = b.as_slice();
    let c = c.as_slice_mut();

    let idx = _block_dim_x() as usize;
    if idx < c.len() && idx < a.len() && idx < b.len() {
        c[idx] = a[idx] + b[idx];
    }
}
