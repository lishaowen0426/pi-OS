/* SPDX-License-Identifier: MIT OR Apache-2.0
 *
 * Copyright (c) 2018-2022 Andre Richter <andre.o.richter@gmail.com>
 */

__rpi_phys_dram_start_addr = 0;
/* The physical address at which the the kernel binary will be loaded by the Raspberry's firmware */
__rpi_phys_binary_load_addr = 0x80000;

/**/

PAGE_SIZE = 4K;
PAGE_MASK = PAGE_SIZE - 1;



ENTRY(__rpi_phys_binary_load_addr)

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
    segment_table PT_LOAD FLAGS(6);
    segment_debug PT_LOAD FLAGS(6);
}

SECTIONS
{
    . = __rpi_phys_dram_start_addr;
    /***********************************************************************************************
    * Boot Core Stack
    ***********************************************************************************************/
    .boot_core_stack(NOLOAD):
    {
        . += __rpi_phys_binary_load_addr;
        __boot_core_stack_end_exclusive = .;
    }:segment_boot_core_stack

    ASSERT((. & PAGE_MASK) == 0, "End of boot core stack is not page aligned")

    /***********************************************************************************************
    * Code + RO Data + Global Offset Table
    ***********************************************************************************************/
    __code_start = .;
    .text :
    {
        KEEP(*(.text._start))
        *(.text._start_arguments)
        *(.text._start_rust)
        *(.text._exception)
        *(.text*)
    } :segment_code

    .rodata : ALIGN(8) { *(.rodata*) } :segment_code

    . = ALIGN(PAGE_SIZE);
    __code_end_exclusive = .;

    /***********************************************************************************************
    * Data + BSS
    ***********************************************************************************************/
    .data : {
        __data_start = .;
        *(.data*);
        . = ALIGN(PAGE_SIZE);
        __data_end_exclusive = .;
    }:segment_data


    .bss(NOLOAD) : ALIGN(PAGE_SIZE) 
    {
        __bss_start = .;
        *(.bss*);
        /*
        *(page_table);
        */
        . = ALIGN(PAGE_SIZE);
        __bss_end_exclusive = .;
    }: segment_bss


    .page_table(NOLOAD) : ALIGN(PAGE_SIZE)
    {
        __l1_page_table_start = .;
        *(page_table);
    }: segment_table




    /***********************************************************************************************
    * Misc
    ***********************************************************************************************/
    .got : { *(.got*) }
    ASSERT(SIZEOF(.got) == 0, "Relocation support not expected")

    /DISCARD/ : { *(.comment*) }
}
