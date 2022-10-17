pub trait ToAnnaValue: Sized {
    fn to_anna_value(&self) -> Vec<u8>;
}

impl ToAnnaValue for Vec<u8> {
    fn to_anna_value(&self) -> Vec<u8> {
        self.clone()
    }
}

impl ToAnnaValue for String {
    fn to_anna_value(&self) -> Vec<u8> {
        self.to_string().into_bytes()
    }
}

impl ToAnnaValue for &str {
    fn to_anna_value(&self) -> Vec<u8> {
        self.as_bytes().to_vec()
    }
}

impl ToAnnaValue for u8 {
    fn to_anna_value(&self) -> Vec<u8> {
        vec![*self]
    }
}

impl ToAnnaValue for i8 {
    fn to_anna_value(&self) -> Vec<u8> {
        vec![*self as u8]
    }
}

impl ToAnnaValue for u16 {
    fn to_anna_value(&self) -> Vec<u8> {
        self.to_be_bytes().to_vec()
    }
}

impl ToAnnaValue for i16 {
    fn to_anna_value(&self) -> Vec<u8> {
        self.to_be_bytes().to_vec()
    }
}

impl ToAnnaValue for u32 {
    fn to_anna_value(&self) -> Vec<u8> {
        self.to_be_bytes().to_vec()
    }
}

impl ToAnnaValue for i32 {
    fn to_anna_value(&self) -> Vec<u8> {
        self.to_be_bytes().to_vec()
    }
}

impl ToAnnaValue for u64 {
    fn to_anna_value(&self) -> Vec<u8> {
        self.to_be_bytes().to_vec()
    }
}

impl ToAnnaValue for i64 {
    fn to_anna_value(&self) -> Vec<u8> {
        self.to_be_bytes().to_vec()
    }
}

impl ToAnnaValue for usize {
    fn to_anna_value(&self) -> Vec<u8> {
        self.to_be_bytes().to_vec()
    }
}

impl ToAnnaValue for isize {
    fn to_anna_value(&self) -> Vec<u8> {
        self.to_be_bytes().to_vec()
    }
}
