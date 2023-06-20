KERNEL_BASE = 0x2000000;
LOADER_OFFSET = 0x80000;

ENTRY(LOADER_OFFSET);

/* Flags:
 *     4 == R
 *     5 == RX
 *     6 == RW
 */
PHDRS
{
    segment_boot_core_stack PT_LOAD FLAGS(6);
    segment_code PT_LOAD FLAGS(5);
    segment_bss PT_LOAD FLAGS(6);
    segment_data PT_LOAD FLAGS(6);
}

SECTIONS
{
    . = KERNEL_BASE;

    /***********************************************************************************************
    * Boot Core Stack
    ***********************************************************************************************/
    .boot_core_stack(NOLOAD): AT (ADDR(.boot_core_stack) - KERNEL_BASE)
    {
        . += LOADER_OFFSET;
        __boot_core_stack_end_exclusive = .;
    }:segment_boot_core_stack

    /***********************************************************************************************
    * Code + RO Data + Global Offset Table
    ***********************************************************************************************/

    __binary_nonzero_start = .;
    .text : AT (ADDR(.text) - KERNEL_BASE)
    {
        KEEP(*(.text._start))
        *(.text._start_arguments)
        *(.text*)
    } :segment_code

    .rodata ALIGN(8): AT (ADDR(.rodata) - KERNEL_BASE)
     {
        *(.rodata*)
     } :segment_code

    /***********************************************************************************************
    * Data + BSS
    ***********************************************************************************************/
    .data : AT (ADDR(.data) - KERNEL_BASE)
     {
        *(.data*)
     }:segment_data

    . = ALIGN(16);
    __binary_nonzero_end_exclusive = .;

    .bss(NOLOAD): AT (ADDR(.bss) - KERNEL_BASE)
    {
        . = ALIGN(16);
        __bss_start = .;
        *(.bss*);
        . = ALIGN(16);
        __bss_end_exclusive = .;
    }: segment_bss

    /***********************************************************************************************
    * Misc
    ***********************************************************************************************/
    .got : { *(.got*) }
    ASSERT(SIZEOF(.got) == 0, "Relocation support not expected")

    /DISCARD/ : { *(.comment*) }
}
