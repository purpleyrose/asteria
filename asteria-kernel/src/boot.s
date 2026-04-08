.section .text
.global _start

_start:
    lea stack_top(%rip), %rsp
    call kernel_main

.halt:
    hlt
    jmp .halt

.section .bss
.align 16
stack_bottom:
    .skip 65536
stack_top:
