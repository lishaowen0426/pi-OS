/*load-time address, i.e., pc-relative*/
.macro adr_load register, symbol
	adrp	\register, \symbol
	add	    \register, \register, #:lo12:\symbol
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

.macro make_recursive_entry  dest, src,  memory_type
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
    mov \dest, \src, LSR .L_L1_SHIFT
    and \dest, \dest, .L_INDEX_MASK
.endm

.macro get_level2_index dest, src
    mov \dest, \src, LSR .L_L2_SHIFT
    and \dest, \dest, .L_INDEX_MASK
.endm

.macro get_level3_index dest, src
    mov \dest, \src, LSR .L_L3_SHIFT
    and \dest, \dest, .L_INDEX_MASK
.endm




.macro get_rust_const_load dest symbol
    adr_load    \dest   \symbol
    ldr    \dest, [\dest]
.endm
