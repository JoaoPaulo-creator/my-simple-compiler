global _start
section .text
_start:
    push 5
    push 3
    pop rbx
    pop rax
    add rax, rbx
    push rax
    push 2
    pop rbx
    pop rax
    imul rbx
    push rax
    pop rdi
    call print_int
    mov rax, 60
    xor rdi, rdi
    syscall
print_int:
    mov rax, rdi
    mov rcx, 0
    mov rbx, 10
    push 0
divide_loop:
    xor rdx, rdx
    div rbx
    add dl, '0'
    dec rsp
    mov [rsp], dl
    inc rcx
    test rax, rax
    jnz divide_loop
    mov rax, 1
    mov rdi, 1
    mov rsi, rsp
    mov rdx, rcx
    syscall
    add rsp, rcx
    inc rsp
    ret