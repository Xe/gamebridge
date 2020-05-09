bitflags! {
    // 0x0100 Digital Pad Right
    // 0x0200 Digital Pad Left
    // 0x0400 Digital Pad Down
    // 0x0800 Digital Pad Up
    // 0x1000 Start
    // 0x2000 Z
    // 0x4000 B
    // 0x8000 A
    pub(crate) struct HiButtons: u8 {
        const NONE = 0x00;
        const DPAD_RIGHT = 0x01;
        const DPAD_LEFT = 0x02;
        const DPAD_DOWN = 0x04;
        const DPAD_UP = 0x08;
        const START = 0x10;
        const Z_BUTTON = 0x20;
        const B_BUTTON = 0x40;
        const A_BUTTON = 0x80;
    }
}

bitflags! {
    // 0x0001 C-Right
    // 0x0002 C-Left
    // 0x0004 C-Down
    // 0x0008 C-Up
    // 0x0010 R
    // 0x0020 L
    // 0x0040 (reserved)
    // 0x0080 (reserved)
    pub(crate) struct LoButtons: u8 {
        const NONE = 0x00;
        const C_RIGHT = 0x01;
        const C_LEFT = 0x02;
        const C_DOWN = 0x04;
        const C_UP = 0x08;
        const R_BUTTON = 0x10;
        const L_BUTTON = 0x20;
    }
}
