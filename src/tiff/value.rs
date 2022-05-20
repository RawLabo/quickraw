use super::{utility::GetBytesFromInt, *};

macro_rules! to_type_value {
    ($t:tt) => {
        pub fn $t(&self) -> Result<$t, ValueError> {
            match self {
                &Value::U16(x) => Ok(x as $t),
                &Value::U32(x) => Ok(x as $t),
                &Value::R64(x) => Ok(x as $t),
                _ => Err(ValueError::ValueTypeIsNotDesired(stringify!($t))),
            }
        }
    };
}

impl Value {
    to_type_value!(u16);
    to_type_value!(u32);
    to_type_value!(i32);
    to_type_value!(f64);
    to_type_value!(usize);

    pub fn str<'a>(&'a self) -> Result<&'a str, ValueError> {
        match self {
            Value::Str(x) => Ok(x.as_str()),
            _ => Err(ValueError::ValueTypeIsNotDesired("String")),
        }
    }

    pub fn u8a4(&self, is_le: bool) -> Result<[u8; 4], ValueError> {
        match self {
            &Value::U32(x) => Ok(x.to_bytes(is_le)),
            _ => Err(ValueError::ValueTypeIsNotDesired("U32")),
        }
    }
}
