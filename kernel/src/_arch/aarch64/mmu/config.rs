/// Only support 4K granule and the lookup starts from level 1
#[allow(dead_code)]
pub(super) mod config {
    use core::ops::Range;
    pub const OFFSET_BITS: usize = 12;
    pub const OFFSET_MASK: usize = (1 << OFFSET_BITS) - 1;

    pub const INDEX_BITS: usize = 9;
    pub const INDEX_MASK: usize = (1 << INDEX_BITS) - 1;

    pub const PAGE_SIZE: usize = 1 << OFFSET_BITS;
    pub const ENTRIES_PER_TABLE: usize = PAGE_SIZE / 8;

    pub const OFFSET_SHIFT: usize = 0;
    pub const L3_INDEX_SHIFT: usize = OFFSET_BITS;
    pub const L2_INDEX_SHIFT: usize = OFFSET_BITS + INDEX_BITS;
    pub const L1_INDEX_SHIFT: usize = OFFSET_BITS + 2 * INDEX_BITS;

    pub const OFFSET_RANGE: Range<usize> = 0..12;
    pub const L3_RANGE: Range<usize> = 12..21;
    pub const L2_RANGE: Range<usize> = 21..30;
    pub const L1_RANGE: Range<usize> = 30..39;

    pub const RECURSIVE_L1_INDEX: usize = ENTRIES_PER_TABLE - 1;
    pub const L1_VIRTUAL_ADDR: usize = (RECURSIVE_L1_INDEX << L1_INDEX_SHIFT)
        | (RECURSIVE_L1_INDEX << L2_INDEX_SHIFT)
        | (RECURSIVE_L1_INDEX << L3_INDEX_SHIFT);
}

#[cfg(test)]
mod tests {
    use super::*;
    use config::*;
    #[allow(unused_imports)]
    use test_macros::kernel_test;
    #[kernel_test]
    fn test_mmu_config() {
        assert_eq!(OFFSET_MASK, 0b1111_1111_1111);
        assert_eq!(INDEX_MASK, 0b1_1111_1111);
        assert_eq!(PAGE_SIZE, 4096);
        assert_eq!(ENTRIES_PER_TABLE, 512);
        assert_eq!(L3_INDEX_SHIFT, 12);
        assert_eq!(L2_INDEX_SHIFT, 12 + 9);
        assert_eq!(L1_INDEX_SHIFT, 12 + 2 * 9);
        assert_eq!(RECURSIVE_L1_INDEX, 511);
    }
}
