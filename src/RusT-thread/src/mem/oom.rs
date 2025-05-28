// in src/lib.rs
#![feature(alloc_error_handler)] // at the top of the file
extern crate alloc;
#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}