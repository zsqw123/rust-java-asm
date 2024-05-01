pub struct Frame {}

impl Frame {
    pub const ITEM_Top: u8 = 0;
    pub const ITEM_Integer: u8 = 1;
    pub const ITEM_Double: u8 = 3;
    pub const ITEM_Long: u8 = 4;
    pub const ITEM_Float: u8 = 2;
    pub const ITEM_Null: u8 = 5;
    pub const ITEM_UninitializedThis: u8 = 6;
    pub const ITEM_Object: u8 = 7;
    pub const ITEM_Uninitialized: u8 = 8;
}

