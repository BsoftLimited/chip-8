start:
    CLR             // clear screen
    LD V6, 0x25     // set y coordinate

    // print n
    LD V7, 0x14
    LD I, character_n
    DRW V7, V6, 0b101

    //print o
    LD V7, 0x1a
    LD I, character_o
    DRW V7, V6, 0b101

    LD V0, K        // wait for key press

character_n.sprite: 0x88 0xc8 0xa8 0x98 0x88;
character_o.sprite: 0x70 0x88 0x88 0x88 0x70;