.equ .L_KERNEL_BASE, 0x2000000
.equ  .L_CONST_EL2, 0b1000
.equ .L_CONST_CORE_ID_MASK , 0b11
.equ .L_BOOT_CORE_ID,  0
.equ .L_QEMU_CONSOLE , 0x3F201000


// Load a load-time address, i.e., pc-relative
.macro adr_load register, symbol
	adrp	\register, \symbol
	add	    \register, \register, #:lo12:\symbol
.endm


// Load a link-time address
.macro adr_link dest, symbol
    ldr                 x0, =.L_KERNEL_BASE
    adr_load            x1, \symbol
    add                 \dest, x0, x1
.endm

// Public Code
//--------------------------------------------------------------------------------------------------
.section .text._start

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


    adr_link    x2, __bss_start
    adr_link    x3, __bss_end_exclusive


.L_bss_init_loop:
    cmp         x2, x3
    b.eq        .L_relocate_binary
    stp         xzr, xzr, [x2], #16
    b           .L_bss_init_loop

.L_relocate_binary:
    adr_load    x2, __binary_nonzero_start  // the load-time binary address
    adr_link    x3, __binary_nonzero_start  // the link-time binary address
    adr_link    x4, __binary_nonzero_end_exclusive

.L_copy_loop:      //copy from the load-time address to the link-time address
    ldp         x5, x6, [x2], #16
    stp         x5, x6, [x3], #16
    cmp         x3, x4
    b.lt        .L_copy_loop

.L_chainloader_main:
    adr_link    x0, __boot_core_stack_end_exclusive
    mov         sp, x0

    adr_link    x2, chainloader_main
    ldr         x0, =.L_KERNEL_BASE
    br          x2


	// Infinitely wait for events (aka "park the core").
.L_parking_loop:
	wfe
	b	.L_parking_loop



//x0 start address
//x1 end exclusive
.global clear_memory_range
clear_memory_range:
    stp         xzr, xzr, [x0], #16
    cmp         x0, x1
    b.lt        clear_memory_range
    ret


.L_qemu_print:
    ldr         x0, =.L_QEMU_CONSOLE
    mov         x1, #0b1000001
    str         x1, [x0]
    ret

.size	_start, . - _start
.type	_start, function
.global	_start
