


.equ    .L_CONST_EL2, 0b1000
.equ .L_CONST_CORE_ID_MASK , 0b11
.equ .L_QEMU_CONSOLE , 0x3F201000
.equ .L_BOOT_CORE_ID,  0
.equ .L_INITIAL_STACK_SIZE , 2
.equ .L_RWNORMAL , 0b0000000001100000000000000000000000000000000000000000011100000100
.equ .L_RONORMAL , 0b0000000001100000000000000000000000000000000000000000011110000100
.equ .L_XNORMAL , 0b0000000001000000000000000000000000000000000000000000011110000100
.equ .L_RWXNORMAL , 0b0000000000000000000000000000000000000000000000000000011100000100
.equ .L_RWDEVICE , 0b0000000001100000000000000000000000000000000000000000010000000000
.equ .L_RODEVICE , 0b0000000001100000000000000000000000000000000000000000010010000000
.equ .L_TABLE_ATTR , 0b0010000000000000000000000000000000000000000000000000010000000000
.equ .L_INDEX_MASK , 0b111111111
.equ .L_L1_SHIFT , 9 + 9 + 12
.equ .L_L2_SHIFT , 9 + 12
.equ .L_L3_SHIFT , 12
.equ .L_TCR_EL1_val , 0b0000000000000000000000000000000010110101000110010011010100011001
.equ .L_MAIR_EL1_val , 0b0000000000000000000000000000000000000000000000001111111100000100
.equ .L_SCTLR_EL1_val , 0b0000000000000000000000000000000000000000110001010001100000111101
.equ .L_HCR_EL2_val , 0b0000000000000000000000000000000010000000000000000000000000000000
.equ .L_SPSR_EL2_val , 0b0000000000000000000000000000000000000000000000000000001111000101
.equ .L_CNTHCTL_EL2_val , 0b0000000000000000000000000000000000000000000000000000000000000011
.equ .L_CNTVOFF_EL2_val , 0b0000000000000000000000000000000000000000000000000000000000000000
.equ .L_KERNAL_BASE , 0xFFFFFF8000000000
.equ .L_RECURSIVE_INDEX , 511
.equ .L_STACK_PERIPHERAL_L1_INDEX , 510
.equ .L_STACK_L2_INDEX , 495
.equ .L_STACK_TOP_L3_INDEX , 510
.equ .L_DOUBLE_STACK_TOP_L3_INDEX , 511
.equ .L_PERIPHERAL_L2_INDEX , 496
.equ .L_PERIPHERAL_PHYSICAL_START , 0xFE000000
.equ .L_PERIPHERAL_PHYSICAL_END , 0xFFFFFFFF
.equ .L_PERIPHERAL_VIRTUAL_START , .L_KERNAL_BASE | (.L_STACK_PERIPHERAL_L1_INDEX << .L_L1_SHIFT) | (.L_PERIPHERAL_L2_INDEX << .L_L2_SHIFT)
.equ .L_PERIPHERAL_VIRTUAL_END , .L_PERIPHERAL_VIRTUAL_START + (.L_PERIPHERAL_PHYSICAL_END - .L_PERIPHERAL_PHYSICAL_START)
.equ .L_DOUBLE_STACK_TOP_VIRTUAL , .L_KERNAL_BASE | (.L_STACK_PERIPHERAL_L1_INDEX << .L_L1_SHIFT) | (.L_STACK_L2_INDEX << .L_L2_SHIFT) | (.L_DOUBLE_STACK_TOP_L3_INDEX << .L_L3_SHIFT)
.equ .L_STACK_TOP_VIRTUAL , .L_KERNAL_BASE | (.L_STACK_PERIPHERAL_L1_INDEX << .L_L1_SHIFT) | (.L_STACK_L2_INDEX << .L_L2_SHIFT) | (.L_STACK_TOP_L3_INDEX << .L_L3_SHIFT)
.equ .L_LOWER_VIRTUAL_START, 0x0
.equ .L_LOWER_VIRTUAL_END_EXCLUSIVE,  .L_RECURSIVE_INDEX << .L_L1_SHIFT
.equ .L_HIGHER_VIRTUAL_START, .L_KERNAL_BASE | (1 << .L_L1_SHIFT)
.equ .L_HIGHER_VIRTUAL_END_EXCLUSIVE, .L_KERNAL_BASE | (.L_STACK_PERIPHERAL_L1_INDEX << .L_L1_SHIFT)

.equ .L_LOWER_L1_VIRTUAL_ADDR, (.L_RECURSIVE_INDEX << .L_L1_SHIFT) | (.L_RECURSIVE_INDEX << .L_L2_SHIFT) | (.L_RECURSIVE_INDEX << .L_L3_SHIFT)
.equ .L_HIGHER_L1_VIRTUAL_ADDR, .L_KERNAL_BASE|(.L_RECURSIVE_INDEX << .L_L1_SHIFT) | (.L_RECURSIVE_INDEX << .L_L2_SHIFT) | (.L_RECURSIVE_INDEX << .L_L3_SHIFT)

.equ .L_PHYSICAL_MINI_UART, 0xFE215040
.equ .L_VIRTUAL_MINI_UART, .L_PERIPHERAL_VIRTUAL_START + 0x215040


