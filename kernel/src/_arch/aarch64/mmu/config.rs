/// Only support 4K granule and the lookup starts from level 1
use crate::bsp::mmio;
use core::ops::Range;

pub const WORD_SIZE: usize = 4; // 32bits

pub const SHIFT_4K: usize = 12;
pub const MASK_4K: usize = (1 << SHIFT_4K) - 1;
pub const ALIGN_4K: usize = !MASK_4K;

pub const SHIFT_16K: usize = 14;
pub const MASK_16K: usize = (1 << SHIFT_16K) - 1;
pub const ALIGN_16K: usize = !MASK_16K;

pub const SHIFT_64K: usize = 16;
pub const MASK_64K: usize = (1 << SHIFT_64K) - 1;
pub const ALIGN_64K: usize = !MASK_64K;

pub const SHIFT_2M: usize = 21;
pub const MASK_2M: usize = (1 << SHIFT_2M) - 1;
pub const ALIGN_2M: usize = !MASK_2M;

pub const SHIFT_1G: usize = 30;
pub const MASK_1G: usize = (1 << SHIFT_1G) - 1;
pub const ALIGN_1G: usize = !MASK_1G;

pub const OFFSET_BITS: usize = 12;
pub const OFFSET_MASK: usize = (1 << OFFSET_BITS) - 1;

pub const INDEX_BITS: usize = 9;
pub const INDEX_MASK: usize = (1 << INDEX_BITS) - 1;

pub const PAGE_SIZE: usize = 1 << SHIFT_4K;
pub const FRAME_SIZE: usize = 1 << SHIFT_4K;
pub const ENTRIES_PER_TABLE: usize = PAGE_SIZE / 8;

pub const OFFSET_RANGE: Range<usize> = 0..12;
pub const L3_RANGE: Range<usize> = 12..21;
pub const L2_RANGE: Range<usize> = 21..30;
pub const L1_RANGE: Range<usize> = 30..39;

pub const OFFSET_SHIFT: usize = 0;
pub const L3_INDEX_SHIFT: usize = 12;
pub const L2_INDEX_SHIFT: usize = L3_INDEX_SHIFT + INDEX_BITS;
pub const L1_INDEX_SHIFT: usize = L2_INDEX_SHIFT + INDEX_BITS;

#[cfg(not(feature = "build_qemu"))]
pub const KERNEL_BASE: usize = 0xFFFFFF8000000000;
#[cfg(feature = "build_qemu")]
pub const KERNEL_BASE: usize = 0;

pub const RECURSIVE_L1_INDEX: usize = ENTRIES_PER_TABLE - 1;
pub const LOWER_L1_VIRTUAL_ADDRESS: usize = (RECURSIVE_L1_INDEX << L1_INDEX_SHIFT)
    | (RECURSIVE_L1_INDEX << L2_INDEX_SHIFT)
    | (RECURSIVE_L1_INDEX << L3_INDEX_SHIFT);
pub const HIGHER_L1_VIRTUAL_ADDRESS: usize = KERNEL_BASE
    | (RECURSIVE_L1_INDEX << L1_INDEX_SHIFT)
    | (RECURSIVE_L1_INDEX << L2_INDEX_SHIFT)
    | (RECURSIVE_L1_INDEX << L3_INDEX_SHIFT);

pub const STACK_MMIO_L1_INDEX: usize = RECURSIVE_L1_INDEX - 1;

pub const PHYSICAL_PERIPHERAL_START: usize = mmio::PHYSICAL_PERIPHERAL_START;

pub const PHYSICAL_MEMORY_END_INCLUSIVE: usize = mmio::PHYSICAL_MEMORY_END_INCLUSIVE;

pub const PHYSICAL_MEMORY_END_EXCLUSIVE: usize = mmio::PHYSICAL_MEMORY_END_EXCLUSIVE;

pub const NUMBER_OF_FRAMES: usize = (PHYSICAL_MEMORY_END_INCLUSIVE >> SHIFT_4K) + 1;
pub const NUMBER_OF_PAGES: usize = (0xFFFF_FFFF_FFFF >> SHIFT_4K) + 1;

const fn get_level2_index(va: usize) -> usize {
    (va >> L2_INDEX_SHIFT) & INDEX_MASK
}

#[cfg(any(feature = "build_qemu", feature = "build_chainloader"))]
pub const VIRTUAL_PERIPHERAL_START: usize = PHYSICAL_PERIPHERAL_START;
#[cfg(feature = "bsp_rpi4")]
pub const VIRTUAL_PERIPHERAL_START: usize = KERNEL_BASE
    | (STACK_MMIO_L1_INDEX << L1_INDEX_SHIFT)
    | (get_level2_index(PHYSICAL_PERIPHERAL_START)) << L2_INDEX_SHIFT;

#[cfg(test)]
mod tests {
    use super::*;
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
