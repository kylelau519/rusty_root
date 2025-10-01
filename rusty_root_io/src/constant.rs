
pub const K_HAS_BYTECOUNT: u32 = 0x4000_0000;   // bit30
pub const K_NEW_CLASSBIT:  u32 = 0x8000_0000;   // bit31 (class/ref tag bit)
pub const K_BYTECOUNTMASK: u32 = 0x3FFF_FFFF;   // low 30 bits
pub const K_NEWCLASSTAG:   u32 = 0xFFFF_FFFF;   // -1 new-class tag
pub const K_NULLTAG:       u32 = 0x0000_0000;   // null
pub const K_MAP_OFFSET:  u32 = 2;