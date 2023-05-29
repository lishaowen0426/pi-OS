
.include "defines.s"
/*load-time address, i.e., pc-relative*/
.macro adr_load register, symbol
	adrp	\register, \symbol
	add	    \register, \register, #:lo12:\symbol
.endm

.macro adr_absolute dest, symbol
    movz x0, #:abs_g2:\symbol		// bits 32-47, overflow check
    movk x0, #:abs_g1_nc:\symbol	// bits 16-31, no overflow check
    movk x0, #:abs_g0_nc:\symbol	// bits  0-15, no overflow check
.endm


.macro adr_link dest, symbol
    ldr                 x0, =.L_KERNEL_BASE
    adr_load            x1, \symbol
    add                 \dest, x0, x1

.endm

.macro make_invalid_entry dest
    mov \dest, xzr
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




    b           .L_el2_to_el1      /*SP_EL1 will be set up open return*/
    
.L_in_el1:
    adr_load    x0, __bss_start
    adr_load    x1, __bss_end_exclusive
    bl          clear_memory_range

    adr_load    x0, l1_lower_page_table
    add         x1, x0, #4096 
    bl          clear_memory_range

    adr_load    x0, l1_higher_page_table
    add         x1, x0, #4096 
    bl          clear_memory_range

    adr_load    x0, l2_lower_page_table
    add         x1, x0, #4096
    bl          clear_memory_range

    adr_load    x0, l2_higher_page_table
    add         x1, x0, #4096
    bl          clear_memory_range

    adr_load    x0, l3_lower_page_table
    add         x1, x0, #4096
    bl          clear_memory_range


    adr_load    x0, l3_higher_page_table
    add         x1, x0, #4096
    bl          clear_memory_range

.L_prepare_rust:


    bl init_mini_uart

    bl .L_map_lower_half
    bl .L_map_higher_half
    b  .L_enable_paging









//assume the kenel is less than 2MB
.L_map_lower_half:

    mov                 x6, lr      // save the link register

    adr_load            x0, l1_lower_page_table

    //fill in the recursive entry
    //L1[511] = L1
    adr_load            x1, l1_lower_page_table
    ldr                 x2, =.TABLE_ATTR
    make_table_entry    x1, x1, x2
    ldr                 x2, =.RECURSIVE_INDEX
    str                 x1, [x0, x2, LSL #3]      

    //L1[0] = lower L2
    adr_load            x1, l2_lower_page_table
    ldr                 x2, =.TABLE_ATTR
    make_table_entry    x1, x1, x2
    str                 x1, [x0, xzr, LSL #3]      

    //lower L2[0] = L3
    adr_load            x1, l3_lower_page_table
    ldr                 x2, =.TABLE_ATTR
    make_table_entry    x1, x1, x2
    adr_load            x3, l2_lower_page_table
    str                 x1, [x3, xzr, LSL #3]      

    //L1[3] = higher L2 /*PI4 peripheral starts at 0xFE00_0000*/
    adr_load            x1, l2_higher_page_table
    ldr                 x2, =.TABLE_ATTR
    make_table_entry    x1, x1, x2
    ldr                 x3, =.PERIPHERAL_START
    get_level1_index    x4, x3
    str                 x1, [x0, x4, LSL #3]      

    //update L3 according to memory type
    
    //boot stack
    adr_load            x0, l3_lower_page_table
    adr_load            x1, initial_stack_bottom
    adr_load            x2, initial_double_stack_top
    ldr                 x3, =.RWNORMAL

    bl .L_fill_l3_table


    //code + rodata
    adr_load            x0, l3_lower_page_table
    adr_load            x1, __code_start
    adr_load            x2, __data_end_exclusive
    ldr                 x3, =.XNORMAL

    bl .L_fill_l3_table
    
    //bss    
    adr_load            x0, l3_lower_page_table
    adr_load            x1, __bss_start
    adr_load            x2, __bss_end_exclusive
    ldr                 x3, =.RWNORMAL

    bl .L_fill_l3_table



    //peripheral
    adr_load            x0, l2_higher_page_table
    ldr                 x1, =.PERIPHERAL_START
    ldr                 x2, =.PERIPHERAL_END
    ldr                 x3, =.RWDEVICE

    bl .L_fill_l2_table


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
    ldr                 x0, =.CNTHCTL_EL2_val
    msr                 CNTHCTL_EL2, x0

    ldr                 x0, =.CNTVOFF_EL2_val
    msr                 CNTVOFF_EL2, x0

    ldr                 x0, =.HCR_EL2_val 
    msr                 HCR_EL2, x0

    ldr                 x0, =.SPSR_EL2_val //all interrupts are masked
    msr                 SPSR_EL2, x0

    adr_load            x0, .L_in_el1 
    msr                 ELR_EL2, x0

    adr_load            x0, __exception_vector_start
    msr                 VBAR_EL1, x0

    adr_load            x0, initial_stack_top 
    msr                 SP_EL1, x0

    eret
    


.L_map_higher_half:
    mov                 x6, lr      // save the link register

    adr_load            x0, l1_higher_page_table

    //fill in the recursive entry
    //higher L1[511] = L1
    adr_load            x1, l1_higher_page_table
    ldr                 x2, =.TABLE_ATTR
    make_table_entry    x1, x1, x2
    ldr                 x2, =.RECURSIVE_INDEX
    str                 x1, [x0, x2, LSL #3]      

    //higher L1[0] = lower L2, lower l2[0] = lower l3
    adr_load            x1, l2_lower_page_table
    ldr                 x2, =.TABLE_ATTR
    make_table_entry    x1, x1, x2
    str                 x1, [x0, xzr, LSL #3]      



    //higher L1[STACK_MMIO_L1_INDEX(510)] = higher L2 /*PI4 peripheral at higher L2[496 - 511]*/
    adr_load            x1, l2_higher_page_table
    ldr                 x2, =.TABLE_ATTR
    make_table_entry    x1, x1, x2
    ldr                 x3, =.L_STACK_MMIO_L1_INDEX
    str                 x1, [x0, x3, LSL #3]      

    //fill higher l2 with MMIO
    adr_load            x0, l2_higher_page_table
    ldr                 x1, =.PERIPHERAL_START
    ldr                 x2, =.PERIPHERAL_END
    ldr                 x3, =.PERIPHERAL_START
    get_level2_index    x1, x1
    get_level2_index    x2, x2   //should be  496 - 511
    ldr                 x4, =.RWDEVICE
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
    ldr                 x2, =.PERIPHERAL_START
    get_level2_index    x2, x2
    sub                 x2, x2, #1
    ldr                 x3, =.TABLE_ATTR
    make_table_entry    x4, x1, x3
    str                 x4, [x0, x2, LSL #3]


    //fill higher l3 with stack, the double stack top starts from higher L3[511]
    adr_load           x0, l3_higher_page_table
    adr_load           x1, initial_double_stack_top
    adr_load           x2, initial_stack_bottom
    mov                x3, #511
    ldr                x4, =.RWNORMAL

2:
    make_page_entry    x5, x1, x4
    str                x5, [x0, x3, LSL #3]
    sub                x1,  x1, #0x1000
    sub                x3,  x3, #1
    cmp                x1,   x2
    b.gt               2b 
    




    mov lr, x6
    ret

    
.L_enable_paging:
    adr_load           x0, l1_lower_page_table 
    msr                ttbr0_el1, x0
    

    adr_load           x0, l1_higher_page_table 
    msr                ttbr1_el1, x0

    ldr                x0, =.TCR_EL1_val
    msr                TCR_EL1, x0


    ldr                x0, =.MAIR_EL1_val
    msr                MAIR_EL1, x0


    ldr                x0, =.SCTLR_EL1_val
    msr                SCTLR_EL1, x0

    isb                sy

    //update stack pointer
    ldr                x0, =.L_KERNEL_BASE
    ldr                x1, =.L_STACK_MMIO_L1_INDEX
    lsl                x1, x1, #(9+9+12)

    ldr                x2, =.PERIPHERAL_START
    get_level2_index   x2, x2
    sub                x2, x2, #1
    lsl                x2, x2, #(9+12)

    mov                x3, #510        //the kernel stack from higher L3[510], the double stack starts from higher L3[511], we actually have the last entry empty, since the stack grows downward. 
    lsl                x3, x3, #12

    orr                 x0, x0, x1
    orr                 x0, x0, x2
    orr                 x0, x0, x3

    mov                sp, x0



    adr_link            x0, .L_higher_half   
    br                  x0


.L_higher_half:
    //we are in the higher half

    adr_load            x2, .L_prepare_boot_info
    blr                 x2
    mov                 x3, x0  //x0 is used in adr_link, save it in other register

    adr_load            x2, kernel_main   
    mov                 x0, x3
    ldr                 x0, KERNAL_BASE
    br                  x2





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
    ldr                x0, =.L_KERNEL_BASE
    ldr                x1, =.L_STACK_MMIO_L1_INDEX
    lsl                x1, x1, #(9+9+12)

    ldr                x2, =.PERIPHERAL_START
    get_level2_index   x2, x2
    sub                x2, x2, #1
    lsl                x2, x2, #(9+12)

    mov                x3, #510        
    lsl                x3, x3, #12
    orr                x1, x0, x1
    orr                x1, x1, x2
    orr                x1, x1, x3

    adr_load    x3, initial_stack_bottom
    adr_load    x4, initial_stack_top
    sub         x5, x4, x3
    sub         x2, x1, x5
    sub         x3,  x3,  x0      
    sub         x4,  x4,  x0
    stp         x1, x2, [sp, #16 * 4]
    stp         x3, x4, [sp, #16 * 5]

    //peripheral
    
    ldr         x1,  =.PERIPHERAL_START
    ldr         x2,  =.PERIPHERAL_END
    stp         x1, x2, [sp, #16 * 6]
    stp         x1, x2, [sp, #16 * 7]

/*
    //free frame
    adr_load    x1, initial_stack_top
    sub         x1,  x1,  x0      
    adr_load    x2, .PERIPHERAL_START
    sub         x2,  x2,  x0      
    stp         x1, x2, [sp, #16 * 8]
    */
    



    mov         x0, sp

    ret

    
    

.L_qemu_print:
    ldr         x0, =.L_QEMU_CONSOLE
    mov         x1, #0b1000111
    str         x1, [x0]
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
    .space 4096 * .INITIAL_STACK_SIZE, 0
.global initial_stack_top
initial_stack_top:
    .space 4096, 0
.global initial_double_stack_top
initial_double_stack_top:
