#![no_std]

pub struct UnitTest {
    pub name: &'static str,
    pub test_func: fn(),
}
