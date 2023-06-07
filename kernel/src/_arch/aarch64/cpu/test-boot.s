
.include "defines.s"
.include "macro.s"


.section .text._start


.global	_start
_start:
    mrs         x0, CurrentEL
    cmp         x0, .L_CONST_EL2
    b.ne        .L_parking_loop

    mrs         x1, MPIDR_EL1
    and         x1, x1, .L_CONST_CORE_ID_MASK
    ldr         x2, =.L_BOOT_CORE_ID
    cmp         x1, x2
    b.ne        .L_parking_loop

    b .L_el2_to_el1

.L_bss_init_loop:
    adr_load    x0, __bss_start
   adr_load    x1, __bss_end_exclusive
    bl     clear_memory_range

    //it takes too long to clear in qemu... let's assume it is all zeros
    //adr_load    x0, __page_table_start
    //adr_load    x0, __page_table_end_exclusive
    //bl     clear_memory_range

    bl     .L_map_lower_half
    b      .L_enable_paging




	// Infinitely wait for events (aka "park the core").
.L_parking_loop:
	wfe
	b	.L_parking_loop
.L_el2_to_el1:
    ldr                 x0, =.L_CNTHCTL_EL2_val
    msr                 CNTHCTL_EL2, x0

    ldr                 x0, =.L_CNTVOFF_EL2_val
    msr                 CNTVOFF_EL2, x0

    ldr                 x0, =.L_HCR_EL2_val
    msr                 HCR_EL2, x0

    ldr                 x0, =.L_SPSR_EL2_val //all interrupts are masked
    msr                 SPSR_EL2, x0

    adr_load            x0, .L_bss_init_loop
    msr                 ELR_EL2, x0

    adr_load            x0, __exception_vector_start
    msr                 VBAR_EL1, x0  //exception vector needs to be virtual

    adr_load            x0, __boot_core_stack_end_exclusive
    msr                 SP_EL1, x0

    eret


.L_map_lower_half:

    mov                 x6, lr      // save the link register

    adr_load            x0, l1_lower_page_table

    //fill in the recursive entry
    //L1[511] = L1
    adr_load            x1, l1_lower_page_table
    ldr                 x2, =.L_RECURSIVE_ATTR
    make_recursive_entry    x1, x1, x2
    ldr                 x2, =.L_RECURSIVE_INDEX
    adr_load            x0, l1_lower_page_table
    str                 x1, [x0, x2, LSL #3]

    //L1[0] = lower L2
    adr_load            x1, l2_lower_page_table
    ldr                 x2, =.L_TABLE_ATTR
    make_table_entry    x1, x1, x2
    str                 x1, [x0, xzr, LSL #3]

    //lower L2[0] = L3
    adr_load            x1, l3_lower_page_table
    ldr                 x2, =.L_TABLE_ATTR
    make_table_entry    x1, x1, x2
    adr_load            x3, l2_lower_page_table
    str                 x1, [x3, xzr, LSL #3]



//stack
    adr_load            x0, l3_lower_page_table
    adr_load            x1, __rpi_phys_dram_start_addr
    adr_load            x2, __boot_core_stack_end_exclusive
    ldr                 x3, =.L_RWNORMAL
    bl .L_fill_l3_table



    //code
    adr_load            x0, l3_lower_page_table
    adr_load            x1, __code_start
    adr_load            x2, __code_end_exclusive
    ldr                 x3, =.L_XNORMAL
    bl .L_fill_l3_table

    //rodata
    adr_load            x0, l3_lower_page_table
    adr_load            x1, __data_start
    adr_load            x2, __data_end_exclusive
    ldr                 x3, =.L_RONORMAL
    bl .L_fill_l3_table

    //bss
    adr_load            x0, l3_lower_page_table
    adr_load            x1, __bss_start
    adr_load            x2, __bss_end_exclusive
    ldr                 x3, =.L_RWNORMAL
    bl .L_fill_l3_table

    //pi3 peripheral
    adr_load            x0, l2_lower_page_table
    ldr            	x1, =.L_PERIPHERAL_PHYSICAL_START
    ldr           	x2, =.L_PERIPHERAL_PHYSICAL_END
    ldr                 x3, =.L_RWDEVICE
    bl  .L_fill_l2_table



    mov lr, x6
    ret

.L_enable_paging:
    adr_load           x0, l1_lower_page_table
    msr                ttbr0_el1, x0


    adr_load           x0, l1_higher_page_table
    msr                ttbr1_el1, x0

    ldr                x0, =.L_TCR_EL1_val
    msr                TCR_EL1, x0


    ldr                x0, =.L_MAIR_EL1_val
    msr                MAIR_EL1, x0


    ldr                x0, =.L_SCTLR_EL1_val
    msr                SCTLR_EL1, x0

    isb                sy


    adr_load                x0, __boot_core_stack_end_exclusive
    mov                sp, x0
    bl 		.L_prepare_boot_info
    b kernel_main

//x0: l3 base address
//x1: start address
//x2: end address exclusive
//x3: memory type
.L_fill_l3_table:
    and                 x1, x1, #~(0x1000-1)
    get_level3_index    x4, x1
    make_page_entry     x5, x1, x3
    str                 x5, [x0, x4, LSL #3]

    add                 x1, x1, #0x1000
    cmp                 x1, x2
    b.lt                .L_fill_l3_table
    ret


//x0: l2 base address
//x1: start address
//x2: end address exclusive
//x3: memory type
.L_fill_l2_table:
    and                 x1, x1, #~(0x200000-1)
    get_level2_index    x4, x1
    make_block_entry    x5, x1, x3
    str                 x5, [x0, x4, LSL #3]

    add                 x1, x1, #0x200000
    cmp                 x1, x2
    b.lt                .L_fill_l2_table
    ret

.L_qemu_print:
    ldr         x0, =.L_QEMU_CONSOLE
    mov         x1, #0b1000001
    str         x1, [x0]
    ret

.L_prepare_boot_info:
    sub     sp, sp, #176
    //code_and_ro
    adr_load    x1, __code_start
    adr_load    x2, __data_end_exclusive
    stp         x1, x2, [sp, #16 * 0]
    stp         x1, x2, [sp, #16 * 1]

    //bss
    adr_load    x1, __bss_start
    adr_load    x2, __bss_end_exclusive
    stp         x1, x2, [sp, #16 * 2]
    stp         x1, x2, [sp, #16 * 3]


    //stack
    adr_load    x1, __rpi_phys_dram_start_addr
    adr_load    x2, __boot_core_stack_end_exclusive
    stp         x1, x2, [sp, #16 * 4]
    stp         x1, x2, [sp, #16 * 5]

    //peripheral
    ldr         x1,  =.L_PERIPHERAL_PHYSICAL_START
    ldr         x2,  =.L_PERIPHERAL_PHYSICAL_END
    stp         x1, x2, [sp, #16 * 6]
    stp         x1, x2, [sp, #16 * 7]

    //free frame
    adr_load    x1,  __bss_end_exclusive
    ldr         x2,  =.L_PERIPHERAL_PHYSICAL_START
    stp         x1, x2, [sp, #16 * 8]

    //lower free page, empty
    stp     	x2, x2, [sp, #16 * 9]

    //higher free page
    stp     	x1, x2, [sp, #16 * 10]

   mov        x0, sp

    ret

//x0 start address
//x1 end exclusive
.global clear_memory_range
clear_memory_range:
    stp         xzr, xzr, [x0], #16
    cmp         x0, x1
    b.ne        clear_memory_range
    ret


.size	_start, . - _start
.type	_start, function


.include "exception.s"






.section page_table, "aw", @nobits
.p2align 12
.global l1_lower_page_table
l1_lower_page_table:
    .space 4096, 0
.global l1_higher_page_table
l1_higher_page_table:
    .space 4096, 0
l2_lower_page_table:
    .space 4096, 0
l2_higher_page_table:
    .space 4096, 0
l3_lower_page_table:
    .space 4096, 0
l3_higher_page_table:
    .space 4096, 0
