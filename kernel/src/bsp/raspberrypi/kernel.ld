/* SPDX-License-Identifier: MIT OR Apache-2.0
 *
 * Copyright (c) 2018-2022 Andre Richter <andre.o.richter@gmail.com>
 */


KERNEL_BASE = 0xFFFFFF8000000000;
/*KERNEL_BASE = 0x000000000000;*/
/* The physical address at which the the kernel binary will be loaded by the Raspberry's firmware */
LOADER_OFFSET = 0x80000;



/* Flags:
 *     4 == R
 *     5 == RX
 *     6 == RW
 */
PHDRS
{
    segment_code PT_LOAD FLAGS(5);
    segment_bss PT_LOAD FLAGS(6);
    segment_data PT_LOAD FLAGS(6);
    segment_table PT_LOAD FLAGS(6);
    segment_stack PT_LOAD FLAGS(6);
    segment_debug PT_LOAD FLAGS(6);
}

SECTIONS
{
    . = KERNEL_BASE;
    . += LOADER_OFFSET;


    /***********************************************************************************************
    * Code + RO Data 
    ***********************************************************************************************/
    __code_start = .;
    .text ALIGN(4K) : AT (ADDR(.text) - KERNEL_BASE)
    {
        KEEP(*(.text._start))
        *(.text._start_arguments)
        __kernel_main = .;
        *(.text.kernel_main)
        *(.text._exception)
        *(.text*)
    } :segment_code

    .rodata : AT (ADDR(.rodata) - KERNEL_BASE)
    { 
        *(.rodata*) 
        . = ALIGN(4K);
        __code_end_exclusive = .;
    } :segment_code


    /***********************************************************************************************
    * Data + BSS
    ***********************************************************************************************/
    .data : AT (ADDR(.data) - KERNEL_BASE)
    {
        __data_start = .;
        *(.data*);
        . = ALIGN(4K);
        __data_end_exclusive = .;
    }:segment_data


    .bss ALIGN(4K) (NOLOAD) : AT (ADDR(.bss) - KERNEL_BASE) 
    {
        __bss_start = .;
        *(.bss*);
        /*
        *(page_table);
        */
        . = ALIGN(4K);
        __bss_end_exclusive = .;
    }: segment_bss


    .page_table ALIGN(4K) (NOLOAD) : AT (ADDR(.page_table) - KERNEL_BASE)
    {
        __page_table_start = .;
        *(page_table);
        . = ALIGN(4K);
        __page_table_end_exclusive = .;
    }: segment_table


    .stack ALIGN(4K) (NOLOAD) : AT (ADDR(.stack) - KERNEL_BASE) 
    {
        __stack_bottom = . + 4K; /*one page guard*/
        *(stack);
        __stack_top = .;
    }: segment_stack
    


    /***********************************************************************************************
    * Misc
    ***********************************************************************************************/
    .got : { *(.got*) }
    ASSERT(SIZEOF(.got) == 0, "Relocation support not expected")

    /DISCARD/ : { *(.comment*) }
}
