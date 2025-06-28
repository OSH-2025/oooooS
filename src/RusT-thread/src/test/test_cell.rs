use lazy_static::lazy_static;   
use crate::rtthread_rt::kservice::cell::RTIntrFreeCell;
use cortex_m_semihosting::hprintln;

struct TestCell {
    value: u32,
}
lazy_static! {
    static ref TEST_CELL: RTIntrFreeCell<TestCell> = unsafe { RTIntrFreeCell::new(TestCell { value: 0 }) };
} 

/// 测试RTIntrFreeCell的exclusive_access方法
pub fn test_cell_exclusive_access() {
    TEST_CELL.exclusive_access().value = 1;
    hprintln!("TEST_CELL.value: {:?}", TEST_CELL.exclusive_access().value);
    TEST_CELL.exclusive_access().value = 2;
    hprintln!("TEST_CELL.value: {:?}", TEST_CELL.exclusive_access().value);
    TEST_CELL.exclusive_access().value = 3;
    hprintln!("TEST_CELL.value: {:?}", TEST_CELL.exclusive_access().value);
    TEST_CELL.exclusive_access().value = 4;
    hprintln!("TEST_CELL.value: {:?}", TEST_CELL.exclusive_access().value);
}

/// 测试RTIntrFreeCell的exclusive_session方法
pub fn test_cell_exclusive_session() {
    TEST_CELL.exclusive_session(|cell| {
        cell.value = 1;
        hprintln!("TEST_CELL.value: {:?}", cell.value);
    });
}

/// 测试RTIntrFreeCell的field_ptr方法
pub fn test_cell_field_ptr() {
    let value_ptr = TEST_CELL.field_mut_ptr(|cell| &mut cell.value);
    change(value_ptr);
    hprintln!("TEST_CELL.value: {:?}", TEST_CELL.exclusive_access().value);
}

fn change(value: *mut u32) {
    unsafe {
        *value *= 2;
    }
}

pub fn test_cell() {
    test_cell_exclusive_access();
    test_cell_exclusive_session();
    test_cell_field_ptr();
}