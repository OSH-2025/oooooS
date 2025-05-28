extern crate alloc;
use alloc::alloc::{alloc, dealloc,Layout};
use cortex_m_semihosting::hprintln;
use alloc::vec::Vec;
use alloc::boxed::Box;

//-----------------测试：alloc/dealloc-----------------

pub fn test_alloc_dealloc() {
    hprintln!("test_alloc_dealloc");

    let size = 1024;
    let p1 = unsafe { alloc(Layout::from_size_align(size, size).unwrap()) as usize };
    hprintln!("p1: {}", p1);
    
    unsafe { dealloc(p1 as *mut u8, Layout::from_size_align(size, size).unwrap()) };
    hprintln!("test_alloc_dealloc done\n\n\n");
}
    
//--------------测试：collectiion-----------------

// vec

pub fn test_vec() {
    hprintln!("test_vec");
    let mut vec = Vec::new();
    vec.push(1);
    vec.push(2);
    vec.push(3);
    vec.push(4);
    vec.push(5);
    vec.push(6);
    vec.push(7);

    for &i in &vec {
        hprintln!("{}", i);
    }
    hprintln!("test_vec done\n\n\n");
}

// Box

pub fn test_box() {
    hprintln!("test_box");
    let xbox = Box::new(15678);
    hprintln!("Box value: {}", *xbox);
    hprintln!("test_box done\n\n\n");
}
