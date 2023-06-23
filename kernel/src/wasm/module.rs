use core::include_bytes;
const WASM_MODULE: &[u8; 230] = include_bytes!("module.wasm");

pub struct WasmModule {}

impl WasmModule {
    pub fn parse(input: &[u8]) -> Self {
        todo!()
    }
}

#[cfg(test)]
#[allow(unused_imports, unused_variables, dead_code)]
mod tests {
    use super::*;
    use crate::println;
    use test_macros::kernel_test;
    #[kernel_test]
    fn test_module() {}
}
