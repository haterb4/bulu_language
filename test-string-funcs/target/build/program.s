.section .data
__concat_length: .quad 0
__heap_start: .quad 0
__heap_current: .quad 0
__heap_size: .quad 1048576
str_6: .ascii "!"
str_6_len = . - str_6
str_3: .ascii "A"
str_3_len = . - str_3
str_2: .ascii " World"
str_2_len = . - str_2
str_5: .ascii "C"
str_5_len = . - str_5
str_1: .ascii "Hello"
str_1_len = . - str_1
str_0: .ascii "=== Test String Concatenation ==="
str_0_len = . - str_0
str_4: .ascii "B"
str_4_len = . - str_4
newline: .byte 10

.section .text
.global _start

__bulu_print_int:
    push %rbp
    mov %rsp, %rbp
    sub $32, %rsp
    mov %rdi, %rax
    lea -32(%rbp), %rsi
    mov $0, %rcx
    mov $10, %rbx
    test %rax, %rax
    jns .print_int_positive
    neg %rax
.print_int_positive:
.print_int_digit_loop:
    xor %rdx, %rdx
    div %rbx
    add $'0', %rdx
    push %rdx
    inc %rcx
    test %rax, %rax
    jnz .print_int_digit_loop
    mov %rcx, %rdx
.print_int_write_loop:
    pop %rax
    mov %al, (%rsi)
    inc %rsi
    loop .print_int_write_loop
    # Write to stdout
    mov $1, %rax
    mov $1, %rdi
    lea -32(%rbp), %rsi
    syscall
    # Write newline
    mov $1, %rax
    mov $1, %rdi
    lea newline(%rip), %rsi
    mov $1, %rdx
    syscall
    add $32, %rsp
    pop %rbp
    ret

__bulu_int_to_string:
    push %rbp
    mov %rsp, %rbp
    push %rbx
    push %r12
    push %r13
    mov %rdi, %rax
    mov %rsi, %r12      # Save buffer pointer
    mov $0, %r13        # Digit count
    mov $10, %rbx
    test %rax, %rax
    jns .i2s_positive
    neg %rax
.i2s_positive:
.i2s_digit_loop:
    xor %rdx, %rdx
    div %rbx
    add $'0', %rdx
    push %rdx
    inc %r13
    test %rax, %rax
    jnz .i2s_digit_loop
    mov %r13, %rcx      # Digit count for loop
    mov %r12, %rsi      # Restore buffer pointer
.i2s_write_loop:
    pop %rax
    mov %al, (%rsi)
    inc %rsi
    loop .i2s_write_loop
    mov %r13, %rax      # Return length
    pop %r13
    pop %r12
    pop %rbx
    pop %rbp
    ret

__init_heap:
    push %rbp
    mov %rsp, %rbp
    # Allocate heap using brk syscall
    mov $12, %rax       # sys_brk
    mov $0, %rdi        # Get current break
    syscall
    movq %rax, __heap_start(%rip)
    movq %rax, __heap_current(%rip)
    # Extend heap
    mov $12, %rax       # sys_brk
    movq __heap_start(%rip), %rdi
    addq __heap_size(%rip), %rdi
    syscall
    pop %rbp
    ret

__malloc:
    push %rbp
    mov %rsp, %rbp
    # Align size to 8 bytes
    add $7, %rdi
    and $-8, %rdi
    # Check if we have enough space
    movq __heap_current(%rip), %rax
    movq %rax, %rcx
    addq %rdi, %rcx
    movq __heap_start(%rip), %rdx
    addq __heap_size(%rip), %rdx
    cmpq %rdx, %rcx
    ja .malloc_fail
    # Update heap pointer
    movq %rcx, __heap_current(%rip)
    # Return old pointer
    pop %rbp
    ret
.malloc_fail:
    mov $0, %rax
    pop %rbp
    ret

__string_create:
    push %rbp
    mov %rsp, %rbp
    push %rdi
    push %rsi
    # Allocate memory for length + data
    mov %rsi, %rdi
    add $8, %rdi        # 8 bytes for length
    call __malloc
    test %rax, %rax
    jz .string_create_fail
    pop %rsi            # length
    pop %rdi            # source
    # Store length
    movq %rsi, (%rax)
    # Copy data
    lea 8(%rax), %rdx   # destination
    mov %rsi, %rcx      # count
    push %rax
    mov %rdi, %rsi      # source
    mov %rdx, %rdi      # destination
    rep movsb
    pop %rax
    pop %rbp
    ret
.string_create_fail:
    pop %rsi
    pop %rdi
    mov $0, %rax
    pop %rbp
    ret

__string_concat:
    push %rbp
    mov %rsp, %rbp
    push %r12
    push %r13
    mov %rdi, %r12      # Save string1
    mov %rsi, %r13      # Save string2
    # Get lengths
    movq (%r12), %rcx   # len1
    movq (%r13), %rdx   # len2
    # Calculate total length
    mov %rcx, %rdi
    add %rdx, %rdi
    add $8, %rdi        # + 8 for length field
    push %rcx
    push %rdx
    call __malloc
    pop %rdx
    pop %rcx
    test %rax, %rax
    jz .concat_fail
    # Store total length
    mov %rcx, %r8
    add %rdx, %r8
    movq %r8, (%rax)
    # Copy first string
    push %rax           # Save result
    lea 8(%r12), %rsi   # source1
    lea 8(%rax), %rdi   # dest
    rep movsb
    # Copy second string
    lea 8(%r13), %rsi   # source2
    mov %rdx, %rcx      # len2
    rep movsb
    pop %rax            # Restore result
    pop %r13
    pop %r12
    pop %rbp
    ret
.concat_fail:
    mov $0, %rax
    pop %r13
    pop %r12
    pop %rbp
    ret

__string_print:
    push %rbp
    mov %rsp, %rbp
    test %rdi, %rdi
    jz .string_print_null
    # Get length and data
    movq (%rdi), %rdx   # length
    lea 8(%rdi), %rsi   # data
    # Write syscall
    mov $1, %rax
    mov $1, %rdi
    syscall
    # Print newline
    mov $1, %rax
    mov $1, %rdi
    lea newline(%rip), %rsi
    mov $1, %rdx
    syscall
    pop %rbp
    ret
.string_print_null:
    # Print "(null)"
    mov $1, %rax
    mov $1, %rdi
    lea .null_str(%rip), %rsi
    mov $7, %rdx
    syscall
    pop %rbp
    ret

__string_uppercase:
    push %rbp
    mov %rsp, %rbp
    test %rdi, %rdi
    jz .uppercase_null
    push %rdi
    # Get length
    movq (%rdi), %rsi   # length
    lea 8(%rdi), %rdi   # source data
    call __string_create
    test %rax, %rax
    jz .uppercase_fail
    pop %rdi            # original string
    # Convert to uppercase
    movq (%rdi), %rcx   # length
    lea 8(%rdi), %rsi   # source
    lea 8(%rax), %rdi   # dest
    push %rax
.uppercase_loop:
    test %rcx, %rcx
    jz .uppercase_done
    movb (%rsi), %al
    # Check if lowercase letter
    cmpb $'a', %al
    jb .uppercase_copy
    cmpb $'z', %al
    ja .uppercase_copy
    # Convert to uppercase
    subb $32, %al
.uppercase_copy:
    movb %al, (%rdi)
    inc %rsi
    inc %rdi
    dec %rcx
    jmp .uppercase_loop
.uppercase_done:
    pop %rax
    pop %rbp
    ret
.uppercase_null:
.uppercase_fail:
    mov $0, %rax
    pop %rbp
    ret

__string_repeat:
    push %rbp
    mov %rsp, %rbp
    push %rdi
    push %rsi
    push %rdx
    # Calculate total length
    mov %rsi, %rax
    mul %rdx
    mov %rax, %rdi
    add $8, %rdi        # + 8 for length field
    call __malloc
    test %rax, %rax
    jz .repeat_fail
    pop %rdx            # count
    pop %rsi            # length
    pop %rdi            # source
    # Store total length
    push %rax           # save result pointer
    mov %rsi, %rax
    mul %rdx
    mov %rax, %r8       # total length
    pop %rax            # restore result pointer
    movq %r8, (%rax)
    # Copy string multiple times
    lea 8(%rax), %r9    # dest pointer
    push %rax
.repeat_loop:
    test %rdx, %rdx
    jz .repeat_done
    # Copy one instance
    mov %rdi, %rsi      # source
    mov %r9, %rdi       # dest
    mov -16(%rbp), %rcx # original length
    rep movsb
    mov %rdi, %r9       # update dest
    mov -24(%rbp), %rdi # restore source
    dec %rdx
    jmp .repeat_loop
.repeat_done:
    pop %rax
    pop %rbp
    ret
.repeat_fail:
    pop %rdx
    pop %rsi
    pop %rdi
    mov $0, %rax
    pop %rbp
    ret

.section .rodata
.null_str: .ascii "(null)\n"
.section .text

main:
    push %rbp
    mov %rsp, %rbp
    sub $136, %rsp
    # write syscall for string
    mov $1, %rax
    mov $1, %rdi
    lea str_0(%rip), %rsi
    mov $str_0_len, %rdx
    syscall
    mov $1, %rax
    mov $1, %rdi
    lea newline(%rip), %rsi
    mov $1, %rdx
    syscall
    lea str_1(%rip), %rdi
    mov $str_1_len, %rsi
    call __string_create
    movq %rax, -16(%rbp)
    lea str_2(%rip), %rdi
    mov $str_2_len, %rsi
    call __string_create
    movq %rax, -24(%rbp)
    movq -16(%rbp), %rdi
    # Check if value looks like a pointer (> 0x1000)
    cmp $0x1000, %rdi
    jb .main_add_as_int_0
    # Check if length field is reasonable (< 1MB)
    movq (%rdi), %rax
    cmp $1048576, %rax
    ja .main_add_as_int_0
    movq -24(%rbp), %rsi
    call __string_concat
    movq %rax, -40(%rbp)
    jmp .main_add_done_0
.main_add_as_int_0:
    movq -16(%rbp), %rax
    addq -24(%rbp), %rax
    movq %rax, -40(%rbp)
.main_add_done_0:
    movq -40(%rbp), %rax
    movq %rax, -32(%rbp)
    movq -32(%rbp), %rdi
    movq __concat_length(%rip), %rdx
    test %rdx, %rdx
    jz .println_print_as_int_1
    mov $1, %rax
    mov $1, %rdi
    movq -32(%rbp), %rsi
    syscall
    mov $1, %rax
    mov $1, %rdi
    lea newline(%rip), %rsi
    mov $1, %rdx
    syscall
    movq $0, __concat_length(%rip)
    jmp .println_print_done_1
.println_print_as_int_1:
    movq -32(%rbp), %rdi
    cmp $0x1000, %rdi
    jb .println_print_as_int_really_1
    # Check if it looks like a string (length < 1MB)
    movq (%rdi), %rax
    cmp $1048576, %rax
    ja .println_print_as_int_really_1
    # Print as string
    call __string_print
    jmp .println_print_done_1
.println_print_as_int_really_1:
    movq -32(%rbp), %rdi
    call __bulu_print_int
.println_print_done_1:
    lea str_3(%rip), %rdi
    mov $str_3_len, %rsi
    call __string_create
    movq %rax, -56(%rbp)
    lea str_4(%rip), %rdi
    mov $str_4_len, %rsi
    call __string_create
    movq %rax, -64(%rbp)
    lea str_5(%rip), %rdi
    mov $str_5_len, %rsi
    call __string_create
    movq %rax, -72(%rbp)
    movq -56(%rbp), %rdi
    # Check if value looks like a pointer (> 0x1000)
    cmp $0x1000, %rdi
    jb .main_add_as_int_2
    # Check if length field is reasonable (< 1MB)
    movq (%rdi), %rax
    cmp $1048576, %rax
    ja .main_add_as_int_2
    movq -64(%rbp), %rsi
    call __string_concat
    movq %rax, -88(%rbp)
    jmp .main_add_done_2
.main_add_as_int_2:
    movq -56(%rbp), %rax
    addq -64(%rbp), %rax
    movq %rax, -88(%rbp)
.main_add_done_2:
    movq -88(%rbp), %rdi
    # Check if value looks like a pointer (> 0x1000)
    cmp $0x1000, %rdi
    jb .main_add_as_int_3
    # Check if length field is reasonable (< 1MB)
    movq (%rdi), %rax
    cmp $1048576, %rax
    ja .main_add_as_int_3
    movq -72(%rbp), %rsi
    call __string_concat
    movq %rax, -96(%rbp)
    jmp .main_add_done_3
.main_add_as_int_3:
    movq -88(%rbp), %rax
    addq -72(%rbp), %rax
    movq %rax, -96(%rbp)
.main_add_done_3:
    movq -96(%rbp), %rax
    movq %rax, -80(%rbp)
    movq -80(%rbp), %rdi
    movq __concat_length(%rip), %rdx
    test %rdx, %rdx
    jz .println_print_as_int_4
    mov $1, %rax
    mov $1, %rdi
    movq -80(%rbp), %rsi
    syscall
    mov $1, %rax
    mov $1, %rdi
    lea newline(%rip), %rsi
    mov $1, %rdx
    syscall
    movq $0, __concat_length(%rip)
    jmp .println_print_done_4
.println_print_as_int_4:
    movq -80(%rbp), %rdi
    cmp $0x1000, %rdi
    jb .println_print_as_int_really_4
    # Check if it looks like a string (length < 1MB)
    movq (%rdi), %rax
    cmp $1048576, %rax
    ja .println_print_as_int_really_4
    # Print as string
    call __string_print
    jmp .println_print_done_4
.println_print_as_int_really_4:
    movq -80(%rbp), %rdi
    call __bulu_print_int
.println_print_done_4:
    lea str_1(%rip), %rdi
    mov $str_1_len, %rsi
    call __string_create
    movq %rax, -112(%rbp)
    movq -112(%rbp), %rdi
    push %rdi
    lea str_6(%rip), %rdi
    mov $str_6_len, %rsi
    call __string_create
    mov %rax, %rsi
    pop %rdi
    call __string_concat
    movq %rax, -128(%rbp)
    movq -128(%rbp), %rax
    movq %rax, -120(%rbp)
    movq -120(%rbp), %rdi
    movq __concat_length(%rip), %rdx
    test %rdx, %rdx
    jz .println_print_as_int_5
    mov $1, %rax
    mov $1, %rdi
    movq -120(%rbp), %rsi
    syscall
    mov $1, %rax
    mov $1, %rdi
    lea newline(%rip), %rsi
    mov $1, %rdx
    syscall
    movq $0, __concat_length(%rip)
    jmp .println_print_done_5
.println_print_as_int_5:
    movq -120(%rbp), %rdi
    cmp $0x1000, %rdi
    jb .println_print_as_int_really_5
    # Check if it looks like a string (length < 1MB)
    movq (%rdi), %rax
    cmp $1048576, %rax
    ja .println_print_as_int_really_5
    # Print as string
    call __string_print
    jmp .println_print_done_5
.println_print_as_int_really_5:
    movq -120(%rbp), %rdi
    call __bulu_print_int
.println_print_done_5:
    mov %rbp, %rsp
    pop %rbp
    ret

_start:
    call __init_heap
    call main
    mov $60, %rax    # sys_exit
    xor %rdi, %rdi   # exit code 0
    syscall
