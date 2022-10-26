use eyre::ContextCompat;

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

pub trait FromAnnaValue: Sized {
    fn from_anna_value(value: &[u8]) -> eyre::Result<Self>;
}

impl FromAnnaValue for Vec<u8> {
    fn from_anna_value(value: &[u8]) -> eyre::Result<Self> {
        Ok(value.to_vec())
    }
}

impl FromAnnaValue for String {
    fn from_anna_value(value: &[u8]) -> eyre::Result<Self> {
        Ok(String::from_utf8(value.to_vec())?)
    }
}

impl FromAnnaValue for u8 {
    fn from_anna_value(value: &[u8]) -> eyre::Result<Self> {
        Ok(*value.get(0).context("cannot convert empty value to u8")?)
    }
}

impl FromAnnaValue for i8 {
    fn from_anna_value(value: &[u8]) -> eyre::Result<Self> {
        Ok(*value.get(0).context("cannot convert empty value to i8")? as i8)
    }
}

impl FromAnnaValue for u16 {
    fn from_anna_value(value: &[u8]) -> eyre::Result<Self> {
        Ok(u16::from_be_bytes(value.try_into()?))
    }
}

impl FromAnnaValue for i16 {
    fn from_anna_value(value: &[u8]) -> eyre::Result<Self> {
        Ok(i16::from_be_bytes(value.try_into()?))
    }
}

impl FromAnnaValue for u32 {
    fn from_anna_value(value: &[u8]) -> eyre::Result<Self> {
        Ok(u32::from_be_bytes(value.try_into()?))
    }
}

impl FromAnnaValue for i32 {
    fn from_anna_value(value: &[u8]) -> eyre::Result<Self> {
        Ok(i32::from_be_bytes(value.try_into()?))
    }
}

impl FromAnnaValue for u64 {
    fn from_anna_value(value: &[u8]) -> eyre::Result<Self> {
        Ok(u64::from_be_bytes(value.try_into()?))
    }
}

impl FromAnnaValue for i64 {
    fn from_anna_value(value: &[u8]) -> eyre::Result<Self> {
        Ok(i64::from_be_bytes(value.try_into()?))
    }
}

impl FromAnnaValue for usize {
    fn from_anna_value(value: &[u8]) -> eyre::Result<Self> {
        Ok(usize::from_be_bytes(value.try_into()?))
    }
}

impl FromAnnaValue for isize {
    fn from_anna_value(value: &[u8]) -> eyre::Result<Self> {
        Ok(isize::from_be_bytes(value.try_into()?))
    }
}
