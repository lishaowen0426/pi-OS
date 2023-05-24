
.include "defines.s"
/*load-time address, i.e., pc-relative*/
.macro adr_load register, symbol
	adrp	\register, \symbol
	add	    \register, \register, #:lo12:\symbol
.endm


// Load a link-time address
.macro adr_link register, symbol
	movz	\register, #:abs_g2:\symbol
	movk	\register, #:abs_g1_nc:\symbol
	movk	\register, #:abs_g0_nc:\symbol
.endm


.macro make_table_entry  dest, src,  memory_type
    orr \dest, \src,  \memory_type
    orr \dest, \dest, #0b11
.endm

.macro make_block_entry  dest, src,  memory_type
    orr \dest, \src,  \memory_type
    orr \dest, \dest, #0b01
.endm

.macro make_page_entry  dest, src,  memory_type
    orr \dest, \src,  \memory_type
    orr \dest, \dest, #0b11
.endm

.macro get_level1_index dest, src
    mov \dest, \src, LSR .L1_SHIFT
    and \dest, \dest, .INDEX_MASK
.endm

.macro get_level2_index dest, src
    mov \dest, \src, LSR .L2_SHIFT
    and \dest, \dest, .INDEX_MASK
.endm

.macro get_level3_index dest, src
    mov \dest, \src, LSR .L3_SHIFT
    and \dest, \dest, .INDEX_MASK
.endm







.section .text._start


.global	_start
_start:
    mrs         x0, CurrentEL
    cmp         x0, .L_CONST_EL2
    b.ne        .L_parking_loop

    mrs         x1, MPIDR_EL1
    and         x1, x1, .L_CONST_CORE_ID_MASK
    ldr         x2, BOOT_CORE_ID
    cmp         x1, x2
    b.ne        .L_parking_loop

    adr_load    x0, __bss_start
    adr_load    x1, __bss_end_exclusive


.L_bss_init_loop:
    cmp         x0, x1
    b.eq        .L_prepare_rust
    stp         xzr, xzr, [x0], #16
    b           .L_bss_init_loop


.L_prepare_rust:
    adr_load x0, __boot_core_stack_end_exclusive
    mov sp, x0


    adr_load    x0, l1_page_table
    add         x1, x0, #4096 
    bl .L_clear_page_table

    adr_load    x0, l2_page_table
    add         x1, x0, #4096
    bl .L_clear_page_table

    adr_load    x0, l3_page_table
    add         x1, x0, #4096
    bl .L_clear_page_table



    bl init_mini_uart

    bl .L_identity_map_lower

    b _start_rust

// x0: start address
// x1: end address exclusive
.L_clear_page_table:
    stp         xzr, xzr, [x0], #16
    cmp         x0, x1
    b.ne        .L_clear_page_table
    ret 

//assume the kenel is less than 2MB
.L_identity_map_lower:

    adr_load            x0, l1_page_table
    mov                 x6, lr      // save the link register

    //fill in the recursive entry
    //L1[511] = L1
    adr_load            x1, l1_page_table
    ldr                 x2, =.TABLE_ATTR
    make_table_entry    x1, x1, x2
    ldr                 x2, =.RECURSIVE_INDEX
    str                 x1, [x0, x2, LSL #3]      

    //L1[0] = L2
    adr_load            x1, l2_page_table
    ldr                 x2, =.TABLE_ATTR
    make_table_entry    x1, x1, x2
    str                 x1, [x0, xzr, LSL #3]      

    //L2[0] = L3
    adr_load            x1, l3_page_table
    ldr                 x2, =.TABLE_ATTR
    make_table_entry    x1, x1, x2
    adr_load            x3, l2_page_table
    str                 x1, [x3, xzr, LSL #3]      


    //update L3 according to memory type
    
    //boot stack
    adr_load            x0, l3_page_table
    adr_load            x1, __kernel_offset
    adr_load            x2, __boot_core_stack_end_exclusive
    ldr                 x3, =.RWNORMAL

    bl .L_fill_l3_table


    //code + rodata
    adr_load            x0, l3_page_table
    adr_load            x1, __code_start
    adr_load            x2, __data_end_exclusive
    ldr                 x3, =.XNORMAL

    bl .L_fill_l3_table
    
    //bss    
    adr_load            x0, l3_page_table
    adr_load            x1, __bss_start
    adr_load            x2, __bss_end_exclusive
    ldr                 x3, =.RWNORMAL

    bl .L_fill_l3_table

    



    

    mov lr, x6
    ret
    

//x0: l3 base address
//x1: start address
//x2: end address exclusive
//x3: memory type
.L_fill_l3_table:
    get_level3_index    x4, x1
    make_page_entry     x5, x1, x3     
    str                 x5, [x0, x4, LSL #3]      

    add                 x1, x1, #4096
    cmp                 x1, x2
    b.lt                .L_fill_l3_table
    ret
    

    
	// Infinitely wait for events (aka "park the core").
.L_parking_loop:
	wfe
	b	.L_parking_loop


//x0 start address
.global clear_frame
clear_frame:
    add x1, x0, #4096
1:
    stp         xzr, xzr, [x0], #16
    cmp         x0, x1
    b.ne        1b
    ret




.size	_start, . - _start
.type	_start, function


.include "exception.s"

.section page_table, "aw", @nobits
.p2align 12 
.global l1_page_table
l1_page_table:
    .space 4096, 0
l2_page_table:
    .space 4096, 0
l3_page_table:
    .space 4096, 0



