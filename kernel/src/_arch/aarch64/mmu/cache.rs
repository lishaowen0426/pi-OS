use super::{address::*, config};
use crate::{cpu::registers::*, type_enum, type_enum_with_error, utils::bitfields::Bitfields};
use aarch64_cpu::registers::*;
use core::{arch::asm, fmt};
use tock_registers::interfaces::{ReadWriteable, Readable};

// Ways
// +------+  +------+  +------+  +------+
// |      |  |      |  |      |  |      |
// |      |  |      |  |      |  |      |
// |      |  |      |  |      |  |      |  sets
// |      |  |      |  |      |  |      |
// |      |  |      |  |      |  |      |
// |      |  |      |  |      |  |      |
// +------+  +------+  +------+  +------+
// One row is a set
// Each memory address is mapped to a fixed set(s).
// One set has multiple(here 4) ways an address can be mapped
// Use tags to differentiate among them
// ARMv8, data caches are usually Physically Indexed, Physically Tagged

// Cache Clean and Invalidate instructions
//
// Clean
//   Take dirt items in the cache and write those out to the memory(i.e.,flush).
//   This ensures updates made by an observer that controls the CACHE are made visible to other
// obser   -vers that can access the MEMORY.
//
//
// Invalidate
//   Invalidate does not care whether there are dirty items or not, it prepares the cahce line as
//   nothing is present. Invalidate may result in the loss of updates if those have not been
//   cleaned.
//   This ensures updates made by an observer that controls the MEMORY are made visible to other
//   observers that can access the CACHE.
//
// These are particularly relevent for cpu and devices where CPU can access the cache and the device
// can only access the memory. When the CPU updates the cache, they needs to be cleaned so that the
// device can read. When the device updates the memory, it needs to inalidate the corresponding
// address so that the next time the CPU can read the latest content.
//
//
// Point of Coherency (PoC)
//   The point at which all observers(CPUs, DMA, GPUs) are guaranteed to see the same copy of a
//   memory location. CLIDR_EL1 contains a file LoC, Level of Coherence that defines the last level
//   of cache that must be cleaned or invalidated to realize PoC(i.e., ensure all observers see the
//   same copy).
//
// Point of Unification (PoU)
//   This is for a single PE. PoU is the point at which the I- and D- caches and the TLB are
//   guarenteed to see the same copy. CLIDR_EL1 contains a filed LoUU, Level of Unification,
//   Uniprocessor that defines the last level of data cache that must be cleaned or invalidated to
//   realize PoU.
//
//   PoU can also be applied to a inner shareable domain.
//
// PoC and PoU are for cache maintenance instructions operating on VA. For instructions operating
// by set/way, the point is the next level of caching for clean and the current specified level for
// i* nvalidate. For example, cleaning the L1 cache flushes to level 2 cache.

type_enum!(
    pub enum CacheType {
        NoCache = 0b000,
        InstCacheOnly = 0b001,
        DataCacheOnly = 0b010,
        SeparateInstAndDataCache = 0b011,
        UnifiedCache = 0b100,
    }
);

type_enum!(
    pub enum L1InstPolicy {
        VPIPT = 0b00,
        AIVIVT = 0b01,
        VIPT = 0b10,
        PIPT = 0b11,
    }
);

#[derive(Default)]
pub struct A64Cache {
    level: u8,
    cache_type: CacheType,
    num_of_sets_data_unified: u64,
    num_of_associativity_data_unified: u64,
    num_of_sets_inst: u64,
    num_of_associativity_inst: u64,
}

impl fmt::Display for A64Cache {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.cache_type {
            CacheType::InstCacheOnly => write!(
                f,
                "Level {} {}, {}KB, associativity = {}",
                self.level,
                self.cache_type,
                self.num_of_sets_inst * self.num_of_associativity_inst * 64 / 1024,
                self.num_of_associativity_inst
            ),
            CacheType::DataCacheOnly | CacheType::UnifiedCache => write!(
                f,
                "Level {} {}, {}KB, associativity = {}",
                self.level,
                self.cache_type,
                self.num_of_sets_data_unified * self.num_of_associativity_data_unified * 64 / 1024,
                self.num_of_associativity_data_unified
            ),
            CacheType::SeparateInstAndDataCache => {
                write!(
                    f,
                    "Level {} {}, d-cache: {}KB, associativity = {} i-cache: {}KB, associativity = {}",
                    self.level,
                    self.cache_type,
                    self.num_of_sets_data_unified * self.num_of_associativity_data_unified *64 / 1024 ,
                    self.num_of_associativity_data_unified,
                    self.num_of_sets_inst * self.num_of_associativity_inst * 64 / 1024,
                    self.num_of_associativity_inst,
                )
            }
            _ => Ok(()),
        }
    }
}

#[derive(Default)]
pub struct A64CacheSet {
    l1ip: L1InstPolicy,
    dminline: usize, // in bytes
    iminline: usize,
    louu: u8,
    loc: u8,
    caches: [Option<A64Cache>; 7],
}

impl fmt::Display for A64CacheSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "A64CacheSet: LoU = {}, LoC = {}, dminline = {}, iminline= {}, L1InstPolicy = {}",
            self.louu, self.loc, self.dminline, self.iminline, self.l1ip
        )?;
        for c in &self.caches {
            if let Some(cc) = c.as_ref() {
                writeln!(f, "")?;
                write!(f, "{}", cc)?;
            }
        }
        Ok(())
    }
}

impl A64CacheSet {
    pub fn new() -> Option<A64CacheSet> {
        let l1ip = L1InstPolicy::try_from(CTR_EL0.read(CTR_EL0::L1Ip) as u8).unwrap();
        let dminline = (1 << (CTR_EL0.read(CTR_EL0::DminLine))) * config::WORD_SIZE;
        let iminline = (1 << (CTR_EL0.read(CTR_EL0::IminLine))) * config::WORD_SIZE;

        // Cache Level ID Register: CLIDR_EL1
        let louu = CLIDR_EL1.read(CLIDR_EL1::LoUU);
        let loc = CLIDR_EL1.read(CLIDR_EL1::LoC);

        let mut a64_caches = A64CacheSet::default();
        a64_caches.l1ip = l1ip;
        a64_caches.louu = louu as u8;
        a64_caches.loc = loc as u8;
        a64_caches.dminline = dminline;
        a64_caches.iminline = iminline;
        let clidr_el1_raw = CLIDR_EL1.get();

        for n in 1..8 {
            let ctype =
                CacheType::try_from(clidr_el1_raw.get_bits((3 * (n - 1))..(3 * (n - 1) + 3)) as u8)
                    .unwrap();
            if ctype == CacheType::NoCache {
                break;
            } else if ctype == CacheType::SeparateInstAndDataCache {
                let mut c = A64Cache::default();

                // select level
                CSSELR_EL1.modify(CSSELR_EL1::Level.val((n - 1) as u64));
                c.level = n as u8;
                c.cache_type = ctype;

                // select d-/i- cache
                CSSELR_EL1.modify(CSSELR_EL1::InD::Data);
                c.num_of_sets_data_unified = CCSIDR_EL1.get_num_sets() + 1;
                c.num_of_associativity_data_unified = CCSIDR_EL1.get_associativity() + 1;
                CSSELR_EL1.modify(CSSELR_EL1::InD::Instruction);
                c.num_of_sets_inst = CCSIDR_EL1.get_num_sets() + 1;
                c.num_of_associativity_inst = CCSIDR_EL1.get_associativity() + 1;

                // insert into the cache set
                a64_caches.caches[n - 1] = Some(c);
            } else if (ctype == CacheType::DataCacheOnly) | (ctype == CacheType::UnifiedCache) {
                let mut c = A64Cache::default();
                CSSELR_EL1.modify(CSSELR_EL1::Level.val((n - 1) as u64));
                c.level = n as u8;
                c.cache_type = ctype;

                CSSELR_EL1.modify(CSSELR_EL1::InD::Data);
                c.num_of_sets_data_unified = CCSIDR_EL1.get_num_sets() + 1;
                c.num_of_associativity_data_unified = CCSIDR_EL1.get_associativity() + 1;

                a64_caches.caches[n - 1] = Some(c);
            } else if ctype == CacheType::InstCacheOnly {
                let mut c = A64Cache::default();
                CSSELR_EL1.modify(CSSELR_EL1::Level.val((n - 1) as u64));
                c.level = n as u8;
                c.cache_type = ctype;

                CSSELR_EL1.modify(CSSELR_EL1::InD::Instruction);
                c.num_of_sets_inst = CCSIDR_EL1.get_num_sets() + 1;
                c.num_of_associativity_inst = CCSIDR_EL1.get_associativity() + 1;

                a64_caches.caches[n - 1] = Some(c);
            } else {
                return None;
            }
        }

        Some(a64_caches)
    }

    #[inline(always)]
    pub fn dc_align_to_this_cacheline(&self, va: VirtualAddress) -> VirtualAddress {
        VirtualAddress::from(va.align_up(self.dminline as usize))
    }
    #[inline(always)]
    pub fn dc_align_to_next_cacheline(&self, va: VirtualAddress) -> VirtualAddress {
        VirtualAddress::from(va.align_down(self.dminline as usize))
    }

    #[inline(always)]
    pub fn ic_align_to_this_cacheline(&self, va: VirtualAddress) -> VirtualAddress {
        VirtualAddress::from(va.align_up(self.iminline as usize))
    }
    #[inline(always)]
    pub fn ic_align_to_next_cacheline(&self, va: VirtualAddress) -> VirtualAddress {
        VirtualAddress::from(va.align_up(self.iminline as usize))
    }

    #[inline(always)]
    pub fn ic_invalidate_all_pou_is(&self) {
        unsafe {
            asm!("IC IALLUIS", "DSB ISH", "ISB");
        }
    }
    #[inline(always)]
    pub fn ic_invalidate_all_pou(&self) {
        unsafe {
            asm!("IC IALLU", "DSB SY", "ISB");
        }
    }

    // Cache maintenance instructions operating on VA have no alignment restrictions
    #[inline(always)]
    pub fn ic_inalidate_va_pou(&self, va: VirtualAddress) {
        unsafe {
            asm!("IC IVAU {}", "DSB ISH", "ISB", in(reg) va.value() as u64);
        }
    }
    #[inline(always)]
    pub fn ic_inalidate_va_range_pou(&self, start: VirtualAddress, end: VirtualAddress) {
        start.iter_to(self.iminline, end).unwrap().for_each(|va| {
            self.ic_inalidate_va_pou(va);
        });
    }

    // Cache maintenance instructions operating on VA might fault since
    // they involve a VA-PA translation. See the manual: Cache Support
    #[inline(always)]
    pub fn dc_invalidate_va_poc(&self, va: VirtualAddress) {
        unsafe {
            asm!("DC IVAC {}", "DSB ISH", in(reg) va.value() as u64);
        }
    }
    #[inline(always)]
    pub fn dc_invalidate_va_range_poc(&self, start: VirtualAddress, end: VirtualAddress) {
        start.iter_to(self.dminline, end).unwrap().for_each(|va| {
            self.dc_invalidate_va_poc(va);
        });
    }
    #[inline(always)]
    pub fn dc_clean_va_poc(&self, va: VirtualAddress) {
        unsafe {
            asm!("DC CVAC {}", "DSB ISH", in(reg) va.value() as u64);
        }
    }
    #[inline(always)]
    pub fn dc_clean_va_range_poc(&self, start: VirtualAddress, end: VirtualAddress) {
        start.iter_to(self.dminline, end).unwrap().for_each(|va| {
            self.dc_clean_va_poc(va);
        });
    }
    #[inline(always)]
    pub fn dc_clean_va_pou(&self, va: VirtualAddress) {
        unsafe {
            asm!("DC CVAU {}", "DSB ISH", in(reg) va.value() as u64);
        }
    }
    #[inline(always)]
    pub fn dc_clean_va_range_pou(&self, start: VirtualAddress, end: VirtualAddress) {
        start.iter_to(self.dminline, end).unwrap().for_each(|va| {
            self.dc_clean_va_pou(va);
        });
    }

    #[inline(always)]
    pub fn dc_clean_invalidate_va_poc(&self, va: VirtualAddress) {
        unsafe {
            asm!("DC CIVAC {}", "DSB ISH", in(reg) va.value() as u64);
        }
    }
    #[inline(always)]
    pub fn dc_clean_invalidate_va_range_poc(&self, start: VirtualAddress, end: VirtualAddress) {
        start.iter_to(self.dminline, end).unwrap().for_each(|va| {
            self.dc_clean_invalidate_va_poc(va);
        });
    }

    // Sometimes we don't know or care whether the address is data or instruction
    #[inline(always)]
    pub fn caches_clean_invalidate_va(&self, va: VirtualAddress) {
        self.dc_clean_invalidate_va_poc(va);
        self.ic_inalidate_va_pou(va);
    }
    #[inline(always)]
    pub fn caches_clean_invalidate_va_range(&self, start: VirtualAddress, end: VirtualAddress) {
        self.dc_clean_invalidate_va_range_poc(start, end);
        self.ic_inalidate_va_range_pou(start, end);
    }
}

// All TLBs start at an IMPLEMENTATION DEFINED but UNKNOWN state
// TLBs are disabled from reset.
//
// TLB maintenance instructions ensure that changes to the translation tables are reflected
// correctly in those TLB caching structures.
//
// All entries that does not itself cause a fault can be cached. These include table entries
// pointing to subsequent tables used in the translation.
//
// Whenever translation tables entries associated with a particular ASID are changed, the
// corresponding entries must be invalidated from the TLB to ensure that these changes are visiable
// to subsequent execution.
//
// Because a TLB never holds any entry that generates a fault, therefore, a change from an entry
// that causes a fault to one that does not fault, does not require any TLB invalidation.

pub struct A64TLB;

impl A64TLB {
    #[inline(always)]
    fn concat_asid_va(va: VirtualAddress, asid: u8) -> u64 {
        ((asid as u64) << 48) | ((va.value() as u64) >> 12)
    }
    pub fn invalidate_all() {
        unsafe {
            asm!("DSB ISHST", "TLBI VMALLE1", "DSB ISH", "ISB");
        }
    }
    pub fn invalidate_asid(asid: u8) {
        unsafe {
            asm!("DSB ISHST", "TLBI ASIDE1, {}", "DSB ISH", "ISB", in(reg) ((asid as u64) << 48));
        }
    }
    pub fn invalidate_va_asid(_va: VirtualAddress, _asid: u8) {
        core::todo!();
    }
    pub fn invalidate_va(_va: VirtualAddress) {
        core::todo!();
    }
}

#[cfg(test)]
#[allow(unused_imports, unused_variables, dead_code)]
mod tests {
    use super::*;
    use test_macros::kernel_test;
    //#[kernel_test]
    fn test_a64cache_tlb() {
        let a64_caches = A64CacheSet::new().unwrap();
    }
}
