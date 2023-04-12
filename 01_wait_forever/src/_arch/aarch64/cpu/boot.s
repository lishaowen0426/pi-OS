// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2021-2022 Andre Richter <andre.o.richter@gmail.com>

//--------------------------------------------------------------------------------------------------

// Load the address of a symbol into a register, PC-relative
.macro ADR_REL register, symbol
	adrp	\register, \symbol
	add	    \register, \register, #:lo12:\symbol
.endm

// Public Code
//--------------------------------------------------------------------------------------------------
.section .text._start

//------------------------------------------------------------------------------
// fn _start()
//------------------------------------------------------------------------------
_start:
    mrs     x0, MPIDR_EL1
    and     x0, x0, {CONST_CORE_ID_MASK}
    ldr     x1, BOOT_CORE_ID
    cmp     x0, x1
    b.ne    .L_parking_loop

    ADR_REL x0, __bss_start
    ADR_REL x1, __bss_end_exclusive


.L_bss_init_loop:
    cmp     x0, x1
    b.eq    .L_prepare_rust
    stp     xzr, xzr, [x0], #16
    b       .L_bss_init_loop


.L_prepare_rust:
    ADR_REL x0, __boot_core_stack_end_exclusive
    mov     sp, x0 
    b       _start_rust
    
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
