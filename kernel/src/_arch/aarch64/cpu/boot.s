 /*
 ttbr0_el1              0x0     +---- +---------------+
                                |     |               |
                                |     |               |
                                |     |               |
                                |     |               |
                                |     |               |                                lower l2
                                |     |               |                   +------ +----------------+
                                |     |   lower free  |                   |       |      0         |------------------+
                                |     |               |                   |       |--------------- |                  |
                                |     |               |                   |       |                |                  |
                                |     |               |                   |       |                |                  |
                                |     |               |                   |       |                |                  |
                                |     |               |                   |       |                |                  |
                                |     |---------------|                   |       |                |                  |
                                 -----|     511       |                   |       |                |                  |
                                      +---------------+                   |       |                |                  |
                                                                          |       |                |                  |
                                                                          |       |                |                  |           lower l3                     physical
                                                                          |       |                |                  |---+ +--------------+               +--------------+
                                                                          |       |                |                        |              |               |              |
                                                                          |       |                |                        |              |               |              |
                                                                          |       |                |                        |--------------| ------------- |--------------|
                                                                          |       +----------------+                        |     code     |               |    code      |
                                                                          |                                                 |--------------|               |	bss       |
                                          higher l1                       |                                                 |     bss      |               |              |
ttbr1_el1   0xffffff8000000000   ----++---------------+                   |                                                 |------------- | --------------|--------------|
                                 |    |      0        |-------------------+                                                 |  free start  |               |    ptable    |
                                 |    |---------------|                               higher l2                             |              |               |              |
                                 |    |               |                      ----++----------------+                        |              |          +----|--------------|
                                 |    |               |                      |    |                |                        |              |          |    |    stack     |
                                 |    |               |                      |    |                |                        |              |          |    |              |
                                 |    |               |                      |    |                |                        |              |          |    |--------------|
                                 |    |               |                      |    |                |                        +--------------+          |    |              |
                                 |    |               |                      |    |                |                                                  |    |              |
                                 |    |               |                      |    |                |                                                  |    |              |
                                 |    |               |                      |    |                |                                                  |    |              |
                                 |    | ------------- |                      |    |                |                                                  |    |              |
                                 |    |      510      -----------------------+    |----------------|                             higher l3            |    |              |
                                 |    |---------------|                           |      495       |-----------------------++--------------+          |    |              |
                                 +----|      511      |                           |----------------|                        |              |          |    |              |
                                      +---------------+                           |                |                        |              |          |    |              |
                                                                                  |MMIO: 496 - 511 |                        |              |          |    |    free      |
                                                                                  |                |                        |              |          |    |              |
                                                                                  |                |                        |  free end    |          |    |              |
                                                                                  +----------------+                        |------------- |          |    |              |
                                                                                                                            |  guard       |          |    |              |
                                                                                                                            |--------------|-----------    |              |
                                                                                                                            |  stack       |               |              |
                                                                                                                            |------------- |               |              |
                                                                                                                            | double stack |               |              |
                                                                                                                            |--------------|               |              |
                                                                                                                            |      511     |               |              |
                                                                                                                            +--------------+               |              |
                                                                                                                                                           |              |
                                                                                                                                                           |              |
                                                                                                                                                           +--------------+
                                                                                                                                                           */
.include "defines.s"
.include "macro.s"
.include "context_switch.s"




//.equ DEBUG_PAGE_TABLE, 1







.section .text._start


.global	_start
_start:
    mrs         x0, CurrentEL
    ldr         x1, =.L_CONST_EL2
    cmp         x0, x1
    b.ne        .L_parking_loop

    mrs         x1, MPIDR_EL1
    ldr         x2, =.L_CONST_CORE_ID_MASK
    and         x1, x1, x2
    ldr         x3, =.L_BOOT_CORE_ID
    cmp         x1, x3
    b.ne        .L_parking_loop



    b           .L_el2_to_el1      /*SP_EL1 will be set up open return*/

.L_in_el1:
    adr_load    x0, __bss_start
    adr_load    x1, __bss_end_exclusive
    bl          clear_memory_range

    adr_load    x0, __page_table_start
    adr_load    x1, __page_table_end_exclusive
    bl          clear_memory_range

    bl init_mini_uart

    bl .L_system_counter
    bl .L_map_lower_half
    bl .L_map_higher_half
    b  .L_enable_paging






.L_system_counter:
    adr_load           x0, SYSTEM_COUNTER_FREQUENCY
    mrs 	       x1, CNTFRQ_EL0
    str		       x1, [x0]
    ret



//assume the kenel is less than 2MB
.L_map_lower_half:

    mov                 x6, lr      // save the link register

    adr_load            x0, l1_lower_page_table

    //fill in the recursive entry
    //L1[511] = L1
    adr_load            x1, l1_lower_page_table
    ldr                 x2, =.L_TABLE_ATTR
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



    //update L3 according to memory type
    //stack and peripheral will be mapped in the higher half


    //code + rodata
    adr_load            x0, l3_lower_page_table
    adr_load            x1, __code_start
    adr_load            x2, __data_end_exclusive
    ldr                 x3, =.L_XNORMAL
    bl .L_fill_l3_table

    //bss
    adr_load            x0, l3_lower_page_table
    adr_load            x1, __bss_start
    adr_load            x2, __bss_end_exclusive
    ldr                 x3, =.L_RWNORMAL
    bl .L_fill_l3_table


    //page table, just for debug, so we can manually read the table using
    //lower addresses
.ifdef DEBUG_PAGE_TABLE
    adr_load            x0, l3_lower_page_table
    adr_load            x1, __page_table_start
    adr_load            x2, __page_table_end_exclusive
    ldr                 x3, =.L_RWNORMAL
    bl .L_fill_l3_table
.endif

    mov lr, x6
    ret



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

    adr_load            x0, .L_in_el1
    msr                 ELR_EL2, x0

    ldr                 x1, =.L_KERNEL_BASE
    adr_load            x0, __exception_vector_start
    add                 x0, x0, x1
    msr                 VBAR_EL1, x0  //exception vector needs to be virtual

    adr_load            x0, initial_stack_top
    msr                 SP_EL1, x0

    eret



.L_map_higher_half:
    mov                 x7, lr      // save the link register

    adr_load            x0, l1_higher_page_table

    //fill in the recursive entry
    //higher L1[511] = L1
    adr_load            x1, l1_higher_page_table
    ldr                 x2, =.L_TABLE_ATTR
    make_recursive_entry  x1, x1, x2
    ldr                 x2, =.L_RECURSIVE_INDEX
    str                 x1, [x0, x2, LSL #3]

    //higher L1[0] = lower L2, lower l2[0] = lower l3
    adr_load            x1, l2_lower_page_table
    ldr                 x2, =.L_TABLE_ATTR
    make_table_entry    x1, x1, x2
    str                 x1, [x0, xzr, LSL #3]



    //higher L1[STACK_MMIO_L1_INDEX(510)] = higher L2
    adr_load            x1, l2_higher_page_table
    ldr                 x2, =.L_TABLE_ATTR
    make_table_entry    x1, x1, x2
    ldr                 x3, =.L_STACK_PERIPHERAL_L1_INDEX
    str                 x1, [x0, x3, LSL #3]

    //fill higher l2 with MMIO
    adr_load            x0, l2_higher_page_table
    ldr                 x1, =.L_PERIPHERAL_PHYSICAL_START
    ldr                 x2, =.L_PERIPHERAL_PHYSICAL_END
    ldr                 x3, =.L_PERIPHERAL_PHYSICAL_START
    get_level2_index    x1, x1
    get_level2_index    x2, x2   //should be  496 - 511
    ldr                 x4, =.L_RWDEVICE
1:
    make_block_entry    x5, x3, x4
    str                 x5, [x0, x1, LSL #3]
    add                 x1,  x1, #1
    add                 x3, x3, #0x200000
    cmp                 x1, x2
    b.le                1b


    //higher l2[495] = higher l3
    adr_load            x0, l2_higher_page_table
    adr_load            x1, l3_higher_page_table
    ldr                 x2, =.L_STACK_L2_INDEX
    ldr                 x3, =.L_TABLE_ATTR
    make_table_entry    x4, x1, x3
    str                 x4, [x0, x2, LSL #3]



    //fill higher l3 with double stack + stack
    adr_load           x0, l3_higher_page_table
    adr_load           x1, initial_stack_bottom // start
    adr_load           x2, initial_double_stack_top //end

    ldr                x3, =.L_INITIAL_STACK_SIZE
    add		       x3, x3, #1  //add the double stack
    mov                x4, #512
    sub                x4, x4, x3
    ldr                x5, =.L_RWNORMAL

2:
    make_page_entry    x6, x1, x5
    str                x6, [x0, x4, LSL #3]
    add                x1,  x1, #0x1000
    add                x4,  x4, #1
    cmp                x1,   x2
    b.lt               2b



    mov lr, x7
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


    ldr                x0, =.L_STACK_TOP_VIRTUAL
    mov                sp, x0


    adr_link            x0, .L_higher_half
    br                  x0


.L_higher_half:
    //we are in the higher half
    //DONOT use adr_link anymore, as it is based on lower half + kernel_base

.ifnotdef DEBUG_PAGE_TABLE
    bl                 .L_unmapped_lower
.endif

    bl                  .L_prepare_boot_info

    b                  kernel_main


.L_unmapped_lower:
   ldr                  x0, =.L_LOWER_L1_VIRTUAL_ADDR
   make_invalid_entry   x1
   str                  x1, [x0, xzr, LSL #3]

   DSB SY
   TLBI VMALLE1
   DSB SY
   ISB sy

   ret


// put address of the boot info in x0
// Note that when you call this function
// you're LIKELY already in higher half
// this effects how you use the PC-relative address
.L_prepare_boot_info:
    ldr     x0, =.L_KERNEL_BASE
    sub     sp, sp, #176

    //code_and_ro
    adr_load    x1, __code_start
    adr_load    x2, __code_end_exclusive
    sub         x3,  x1,  x0      //the lower half where it's identity mapped
    sub         x4,  x2,  x0
    stp         x1, x2, [sp, #16 * 0]
    stp         x3, x4, [sp, #16 * 1]

    //bss
    adr_load    x1, __bss_start
    adr_load    x2, __bss_end_exclusive
    sub         x3,  x1,  x0
    sub         x4,  x2,  x0
    stp         x1, x2, [sp, #16 * 2]
    stp         x3, x4, [sp, #16 * 3]

    //stack
    ldr         x1, =.L_STACK_BOTTOM_VIRTUAL
    ldr         x2, =.L_STACK_BOTTOM_VIRTUAL
    ldr		x3, =.L_INITIAL_STACK_SIZE
1:
    add		x2, x2, #0x1000
    sub		x3, x3, #1
    cmp		x3, xzr
    b.gt	1b
    stp         x1, x2, [sp, #16 * 4]

    adr_load    x3, initial_stack_bottom
    adr_load    x4, initial_stack_top
    sub         x3,  x3,  x0
    sub         x4,  x4,  x0
    stp         x3, x4, [sp, #16 * 5]

    //peripheral

    ldr         x1,  =.L_PERIPHERAL_VIRTUAL_START
    ldr         x2,  =.L_PERIPHERAL_VIRTUAL_END
    ldr         x3,  =.L_PERIPHERAL_PHYSICAL_START
    ldr         x4,  =.L_PERIPHERAL_PHYSICAL_END
    stp         x1, x2, [sp, #16 * 6]
    stp         x3, x4, [sp, #16 * 7]


    //free frame
    ldr     x0, =.L_KERNEL_BASE
    adr_load    x1,  initial_double_stack_top
    sub         x1,  x1,  x0
    ldr         x2,  =.L_PERIPHERAL_PHYSICAL_START
    stp         x1, x2, [sp, #16 * 8]

    //lower free page
    ldr     x1, =.L_LOWER_VIRTUAL_START
    ldr     x2, =.L_LOWER_VIRTUAL_END_EXCLUSIVE
    stp     x1, x2, [sp, #16 * 9]


    //higher free page
.ifdef DEBUG_PAGE_TABLE
    adr_load     x1, __page_table_end_exclusive
.else
    adr_load     x1, __bss_end_exclusive
.endif
    ldr          x2, =.L_STACK_BOTTOM_VIRTUAL
    sub		 x2, x2, #0x1000 //minus the stack guard

    stp     x1, x2, [sp, #16 * 10]


    mov         x0, sp

    ret




.L_qemu_print:
    ldr         x0, =.L_QEMU_CONSOLE
    mov         x1, #0b1000001
    str         x1, [x0]
    ret

.L_mini_print:
    ldr   x0, =.L_PHYSICAL_MINI_UART
    mov   x1, #0b1000001
    str  x1, [x0]
    ret

.L_virtual_mini_print:
    ldr   x0, =.L_VIRTUAL_MINI_UART
    mov   x1, #0b1000010
    str  x1, [x0]
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
l2_higher_page_table:    /*peripheral*/
    .space 4096, 0
l3_lower_page_table:
    .space 4096, 0
l3_higher_page_table:
    .space 4096, 0


.section stack, "aw", @nobits
.p2align 12
.global initial_stack_guard_page
initial_stack_guard_page:
    .space 4096, 0
.global initial_stack_bottom
initial_stack_bottom:
    .space 4096 * .L_INITIAL_STACK_SIZE, 0
.global initial_stack_top
initial_stack_top:
    .space 4096, 0
.global initial_double_stack_top
initial_double_stack_top:
