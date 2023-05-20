//! The Non-shareable Normal memory is likely to be accessed only by a sinlge PE.
//! AArch64 does not require the hardware to make data accesses by different observers to a
//! location in such a memory region coherent, unless the memory is NON-CACHEABLE.
//!
//!
//! The means, the presence of caches might lead to coherency issues when communication
//! between the observers. Therefore, software must use cache maintenance instructions,
//! plus any barriers, to share the memory coherently.

//! The abstraction model for the use of a Load-Exclusive/Store-Exclusive instruction pair
//! accessing a non-aborting memory address x is:
//!   1. The Load-Exclusive instruction reads a value from memory address x.
//!   2. The corresponding Store-Exclusive instruction succeeds in writing back to x
//!   ONLY IF no other observer, process, or thread has performed a more recent store to address x.
//!   The Store-Exclusive instruction returns a status bit that indicates whether the memory write
//!   succeeded.
//!
//! A Load-Exclusive instruction basically marks a small block of memory for exclusive access,
//! which is then cleared by a Store-Exclusive instruction.

//! Non-shareable memory locations
//!     
//!     The exclusive access instruction rely on a local Exclusive monitor, or local monitor.
//!
//!     Load-Exclusive:
//!         1. The executing PE marks the PA
//!         2. The local monitor of the executing PE transitions to the Exclusive Access state.
//!
//!     Store-Exclusive:
//!         if the local monitor is in the exclusive access state && the address of the
//!         store-exclusive matches the one that has been marked by an earlier load-exclusive, then
//!         the store occurs, and a status value is returned to a register, and the local monitor
//!         transitions to the open access state
//!
//!         if the local monitor is in the open access state, then, no store takes place, a status
//!         value of 1 is returned, the local monitor remains in the open access state
//!
//!
//!
//!     Things that are IMPLEMENTATION DEFINED and shouldn't be relied upon are
//!         1. Store-Exclusive to an unmarked address
//!         2. Load-Exclusive to multiple addresses without intermediate Store-Exclusive
//!         3. Using Store instead of Store-Exclusive for accessing either marked or unmarked
//!            addresses
//!
//!
//! Shareable memory locations
//!     
//!     The exclusive access relies on both a local monitor for each PE and a global monitor.
//!
//!     The global monitor supporting the marking of ONLY ONE PA for each PE. Any situation from
//!     the marking of multiple addresses on a single PE is UNPREDICTABLE
//!
//!     
//!     Load-Exclusive:
//!         1. The PA of the access is marked as exclusive access for the requesting PE. This
//!            access also causes the exclusive access mark to be REMOVED from any other PA that
//!            has been marked before by the requesting PE.
//!         2. The Load-Exclusive by one PE has no effect on the global monitor state for any other
//!            PE
//!         3. The local monitor is also updated according to the non-shareable description.
//!
//!     Store-Exclusive:
//!         1. The store is guaranteed to succeed only if the PA is marked as exclusive access for
//!            the requesting PE and both the global monitor and the local monitor for the
//!            requesting PE are in the Exclusive Access state
//!         2. If the PA is marked for exclusive in the global monitor for ANY OTHER PE, then that
//!            PE's global monitor state transitions to Open Access state.
//!         3. If no address is marked, the store does not succeed.
//!
//!
//!     Things that are IMPLEMENTATION DEFINED and shouldn't be relied upon are
//!         1. Store-Exclusive to a different PA marked by the global monitor.
//!         2. Using Store instead of Store-Exclusive for accessing either marked or unmarked
//!
//!
//! Marking Granule
//!     When an address is marked, the actually marked block is created by ignoring the least
//!     significant bits of the memory address. Any address within this marked block is marked.
//!     The size, i.e., the number of ignored bits, is called the Exclusive reservation granule,
//!     which can be identified from CTR_EL0.
