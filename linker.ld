/* Linker script for m5rOS kernel
 * Target: x86_64
 * Format: ELF64
 */

ENTRY(kernel_main)

SECTIONS
{
    /* Kernel is loaded at higher half (virtual address) */
    . = 0xFFFFFFFF80000000;

    kernel_start = .;

    /* Read-only code section */
    .text ALIGN(4K) : {
        *(.text .text.*)
    }

    /* Read-only data section */
    .rodata ALIGN(4K) : {
        *(.rodata .rodata.*)
    }

    /* Read-write data section */
    .data ALIGN(4K) : {
        *(.data .data.*)
    }

    /* BSS section (uninitialized data) */
    .bss ALIGN(4K) : {
        *(.bss .bss.*)
        *(COMMON)
    }

    kernel_end = .;

    /* Discard debug information and comments */
    /DISCARD/ : {
        *(.comment)
        *(.eh_frame)
    }
}
