use crate::{errno::*, println, BootInfo};
use aarch64_cpu::registers::*;
use spin::{mutex::SpinMutex, once::Once};
use tock_registers::interfaces::{Readable, Writeable};

#[path = "mmu/address.rs"]
pub mod address;
#[path = "mmu/cache.rs"]
mod cache;
#[path = "mmu/config.rs"]
pub mod config;
#[path = "mmu/translation_entry.rs"]
pub mod translation_entry;
#[path = "mmu/translation_table.rs"]
pub mod translation_table;

#[path = "mmu/allocator.rs"]
mod allocator;
#[path = "mmu/heap.rs"]
pub mod heap;

use address::*;
use allocator::*;
use cache::*;
use translation_entry::*;
use translation_table::*;

extern "C" {
    fn clear_memory_range(start: usize, end_exclusive: usize);
}
fn config_registers_el1() -> Result<(), ErrorCode> {
    // let t0sz: u64 = (64 - (PHYSICAL_MEMORY_END_INCLUSIVE + 1).trailing_zeros()) as u64; //
    // currently just identity map
    let t0sz: u64 = 16 + 9; // start from level 1
    let t1sz: u64 = 16 + 9; // start from level 1

    println!("TTBR0: 0x0 - {:#x}", u64::pow(2, (64 - t0sz) as u32) - 1);

    let is_4kb_page_supported = || -> bool { ID_AA64MMFR0_EL1.read(ID_AA64MMFR0_EL1::TGran4) == 0 };

    if !is_4kb_page_supported() {
        return Err(EAGAIN);
    }

    // Support physical memory up to 64GB
    TCR_EL1.write(
        TCR_EL1::IPS::Bits_32 /*pi4 has 4GB memory*/
        + TCR_EL1::T0SZ.val(t0sz)
        + TCR_EL1::T1SZ.val(t1sz)
        //+ TCR_EL1::TBI0::Used /*Memory Taggging Extension(MTE) is not supported on pi */
        //+ TCR_EL1::TBI1::Used /*Memory Taggging Extension(MTE) is not supported on pi */
        + TCR_EL1::AS::ASID8Bits /* Sizeof ASID = 8 bits*/
        + TCR_EL1::A1::TTBR0
        + TCR_EL1::TG0::KiB_4
        + TCR_EL1::TG1::KiB_4
        + TCR_EL1::SH0::Inner /*AArch64 assumes all PEs use the same OS are in the same Inner Shareable domain*/
        + TCR_EL1::SH1::Inner /*AArch64 assumes all PEs use the same OS are in the same Inner Shareable domain*/
        + TCR_EL1::ORGN0::WriteBack_ReadAlloc_WriteAlloc_Cacheable
        + TCR_EL1::IRGN0::WriteBack_ReadAlloc_WriteAlloc_Cacheable
        + TCR_EL1::ORGN1::WriteBack_ReadAlloc_WriteAlloc_Cacheable
        + TCR_EL1::IRGN1::WriteBack_ReadAlloc_WriteAlloc_Cacheable
        + TCR_EL1::EPD1::EnableTTBR1Walks
        + TCR_EL1::EPD0::EnableTTBR0Walks,
    );

    // Be careful when change this!
    // We use the attribute index in some places when we set the block/page table entry AttrIdx
    // Remember to change those if MAIR_EL1 is modified.
    MAIR_EL1.write(
        MAIR_EL1::Attr1_Normal_Outer::WriteBack_NonTransient_ReadWriteAlloc
            + MAIR_EL1::Attr1_Normal_Inner::WriteBack_NonTransient_ReadWriteAlloc
            + MAIR_EL1::Attr0_Device::nonGathering_nonReordering_EarlyWriteAck,
    );

    Ok(())
}

pub fn init(boot_info: &BootInfo) -> Result<(), ErrorCode> {
    MMU.call_once(|| {
        MemoryManagementUnit::new(
            UnsafeTranslationTable::new(config::LOWER_L1_VIRTUAL_ADDRESS as *mut L1Entry),
            UnsafeTranslationTable::new(config::HIGHER_L1_VIRTUAL_ADDRESS as *mut L1Entry),
        )
    });

    //  give 4k va and pa to the heap allocator so that it can distribute memory to the page and
    //  frame allocator
    //  note that we cannot allocate new page table at the moment since the frame allocator is not
    // ready to use

    let mut boot_info_copy = *boot_info; // make a copy of boot info for later update
    boot_info_copy.free_frame.align_to_4K();
    boot_info_copy.higher_free_page.align_to_4K();

    let pa = boot_info_copy.free_frame.pop_4K_front().unwrap();
    let va = boot_info_copy.higher_free_page.pop_4K_front().unwrap();

    println!("Allocated to heap allocator..");
    println!("va: {:?}, pa: {}", va, pa);

    MMU.get().unwrap().map(va, pa, RWNORMAL, BLOCK_4K).unwrap();
    // let l3 = 0x90000 as *const u64;
    unsafe {
        // println!("using lower address");
        // let e = core::ptr::read_volatile(l3.offset((509) as isize));
        // println!("address = {:#018x}", ((e >> 12) << 12) & ((1 << 48) - 1));
        // println!("e {:#066b}", e);
        // let e = core::ptr::read_volatile(l3.offset((va.level3()) as isize));
        // println!("address = {:#018x}", ((e >> 12) << 12) & ((1 << 48) - 1));
        // println!("e {:#066b}", e);
    }
    unsafe {
        core::ptr::read_volatile(va.value() as *mut u64);
    }
    println!(" map success");
    heap::heap_init(VaRange::new(va, va + VirtualAddress::_4K)).unwrap();
    println!(" Updated boot info {}", boot_info_copy);

    allocator::init(&boot_info_copy).unwrap();

    Ok(())
}

#[derive(Copy, Clone)]
pub enum BlockSize {
    _4K,
    _2M,
    _1G,
}

pub static BLOCK_4K: &BlockSize = &BlockSize::_4K;
pub static BLOCK_2M: &BlockSize = &BlockSize::_2M;
pub static BLOCK_1G: &BlockSize = &BlockSize::_1G;

pub enum MemoryRegion {
    Higher,
    Lower,
}

pub static HIGHER_PAGE: &MemoryRegion = &MemoryRegion::Higher;
pub static LOWER_PAGE: &MemoryRegion = &MemoryRegion::Lower;

pub struct MemoryManagementUnit {
    lower_l1: SpinMutex<UnsafeTranslationTable<Level1>>,
    higher_l1: SpinMutex<UnsafeTranslationTable<Level1>>,
    cache: A64CacheSet,
}
impl MemoryManagementUnit {
    pub fn new(
        lower_l1_table: UnsafeTranslationTable<Level1>,
        higher_l1_table: UnsafeTranslationTable<Level1>,
    ) -> Self {
        Self {
            lower_l1: SpinMutex::new(lower_l1_table),
            higher_l1: SpinMutex::new(higher_l1_table),
            cache: A64CacheSet::new().unwrap(),
        }
    }

    fn map(
        &self,
        va: VirtualAddress,
        pa: PhysicalAddress,
        mt: &MemoryType,
        sz: &BlockSize,
    ) -> Result<(), ErrorCode> {
        if va.is_lower() {
            self.lower_l1.lock().map(va, pa, mt, sz)
        } else {
            self.higher_l1.lock().map(va, pa, mt, sz)
        }
    }
    fn translate(&self, va: VirtualAddress) -> Option<PhysicalAddress> {
        if va.is_lower() {
            self.lower_l1.lock().translate(va)
        } else {
            self.higher_l1.lock().translate(va)
        }
    }
    // kzalloc will zero the allocated memory
    pub fn kzalloc(
        &self,
        sz: &BlockSize,
        mt: &MemoryType,
        region: &MemoryRegion,
    ) -> Result<Mapped, ErrorCode> {
        let va = allocator::PAGE_ALLOCATOR
            .get()
            .unwrap()
            .allocate(sz, region)?;
        let pa = allocator::FRAME_ALLOCATOR.get().unwrap().allocate(sz)?;
        self.map(va, pa, mt, sz)?;

        match *sz {
            BlockSize::_4K => {
                unsafe {
                    clear_memory_range(va.value(), (va + VirtualAddress::_4K).value());
                }
                Ok(Mapped {
                    va: VaRange::new(va, va + VirtualAddress::_4K),
                    pa: PaRange::new(pa, pa + PhysicalAddress::_4K),
                })
            }
            BlockSize::_2M => {
                unsafe {
                    clear_memory_range(va.value(), (va + VirtualAddress::_2M).value());
                }
                Ok(Mapped {
                    va: VaRange::new(va, va + VirtualAddress::_2M),
                    pa: PaRange::new(pa, pa + PhysicalAddress::_2M),
                })
            }
            _ => Err(ESUPPORTED),
        }
    }
}

pub static MMU: Once<MemoryManagementUnit> = Once::new();

#[cfg(test)]
#[allow(unused_imports, unused_variables, dead_code)]
mod tests {
    use super::*;
    use test_macros::kernel_test;
    #[kernel_test]
    fn test_mmu() {}
}
