ENTRY(_start)

PHDRS {
    image PT_LOAD FILEHDR PHDRS FLAGS(5);
}

SECTIONS {
    . = 0x400000 + SIZEOF_HEADERS;
    .text : { *(.text.start) *(.text*) } :image
    .wbg3 : ALIGN(4) { KEEP(*(.rodata.wbg3)) } :image
    .rodata : ALIGN(8) { *(.rodata*) } :image
    /DISCARD/ : { *(.comment*) *(.note*) *(.eh_frame*) }
}
