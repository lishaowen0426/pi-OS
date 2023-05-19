// The Non-shareable Normal memory is likely to be accessed only by a sinlge PE.
// AArch64 does not require the hardware to make data accesses by different observers to a
// location in such a memory region coherent, unless the memory is NON-CACHEABLE.
//
//
// The means, the presence of caches might lead to coherency issues when communication
// between the observers. Therefore, software must use cache maintenance instructions,
// plus any barriers, to share the memory coherently.
//

// The abstraction model for the use of a Load-Exclusive/Store-Exclusive instruction pair
// accessing a non-aborting memory address x is:
//   1. The Load-Exclusive instruction reads a value from memory address x.
//   2. The corresponding Store-Exclusive instruction succeeds in writing back to x
//   ONLY IF no other observer, process, or thread has performed a more recent store to address x.
//   The Store-Exclusive instruction returns a status bit that indicates whether the memory write
//   succeeded.
