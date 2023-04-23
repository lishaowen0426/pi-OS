// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2021-2022 Andre Richter <andre.o.richter@gmail.com>

//--------------------------------------------------------------------------------------------------

// Load a load-time address, i.e., pc-relative
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

// Public Code
//--------------------------------------------------------------------------------------------------
.section .text._start

//------------------------------------------------------------------------------
// fn _start()
//------------------------------------------------------------------------------
_start:
    mrs         x0, CurrentEL
    cmp         x0, {CONST_EL2}
    b.ne        .L_parking_loop

    mrs         x1, MPIDR_EL1
    and         x1, x1, {CONST_CORE_ID_MASK}
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


    // x0 holds the function argument to _start_rust
    b _start_rust


    
	// Infinitely wait for events (aka "park the core").
.L_parking_loop:
	wfe
	b	.L_parking_loop


get_exception_level:
    mrs  x0, CurrentEL
    lsr  x0, x0, #2
    ret




.size	_start, . - _start
.type	_start, function
.global	_start
.global get_exception_level
