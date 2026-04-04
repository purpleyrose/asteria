.section .text
.global _start

_start:
    mov $0x3F8, %dx
    mov $0x41, %al    # 'A'
    out %al, %dx
    lea stack_top(%rip), %rsp
    call kernel_main

.halt:
    hlt
    jmp .halt

.section .bss
.align 16
stack_bottom:
    .skip 16384
stack_top:
