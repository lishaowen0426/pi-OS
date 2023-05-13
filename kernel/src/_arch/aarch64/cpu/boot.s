
.include "defines.s"
/*load-time address, i.e., pc-relative*/
.macro ADR_LOAD register, symbol
	adrp	\register, \symbol
	add	    \register, \register, #:lo12:\symbol
.endm


// Load a link-time address
.macro ADR_LINK register, symbol
	movz	\register, #:abs_g2:\symbol
	movk	\register, #:abs_g1_nc:\symbol
	movk	\register, #:abs_g0_nc:\symbol
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

    ADR_LOAD    x0, __bss_start
    ADR_LOAD    x1, __bss_end_exclusive


.L_bss_init_loop:
    cmp         x0, x1
    b.eq        .L_prepare_rust
    stp         xzr, xzr, [x0], #16
    b           .L_bss_init_loop


.L_prepare_rust:
    ADR_LOAD x0, __boot_core_stack_end_exclusive
    mov sp, x0


    ADR_LOAD    x0, l1_page_table
    add         x1, x0, #4096
.L_prepare_l1_page_table:
    stp         xzr, xzr, [x0], #16
    cmp         x0, x1
    b.ne        .L_prepare_l1_page_table


    b _start_rust


    
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


.section page_table, "aw", @nobits
.p2align 12 
.global l1_page_table
l1_page_table:
    .space 4096, 0
