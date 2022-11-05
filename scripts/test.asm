start.commands:
    CLR  // clear screen

    //print b
    LD V6, 0x25
    LD V7, 0x14
    LD V8, 0xB
    LD F, V8
    DRW V7, V6, 0b101

    LD V7, 0x19
    LD V8, 0xa
    LD F, V8
    DRW V7, V6, 0b101

    LD V7, 0x1E
    LD V8, 0xb
    LD F, V8
    DRW V7, V6, 0b101

    LD V7, 0x23
    LD V8, 0xe
    LD F, V8
    DRW V7, V6, 0b101

nobel.sprite: 
    0x88 0xc8 0xa8 0x98 0x70