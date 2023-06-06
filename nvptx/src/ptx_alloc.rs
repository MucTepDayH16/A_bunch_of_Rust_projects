use core::{
    alloc::{GlobalAlloc, Layout},
    arch::nvptx,
};

pub struct PTXAllocator;

unsafe impl GlobalAlloc for PTXAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        nvptx::malloc(layout.size()) as *mut u8
    }
    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        nvptx::free(ptr as *mut _);
    }
}
