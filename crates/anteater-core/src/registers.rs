// 1. The enum
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Value {
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    Float(f32),
    Double(f64),
    Byte64([u8; 8]),
    Byte128([u8; 16]),
}

// 2. The macro (private to this file)
macro_rules! impl_value_from {
    (
    $($type:ty => $enum:ident ),* $(,)?
    ) => { $(
        impl From<$type> for Value {
            fn from(v: $type) -> Self {
                Value::$enum(v)
            }
        }
    )*
    }
}
impl_value_from!(u8 => U8, u16 => U16, u32 => U32, u64 => U64, i8 => I8, i16 => I16, i32 => I32, i64 => I64, f32 => Float, f64 => Double, [u8; 8] => Byte64, [u8; 16] => Byte128);

// 3. Later: read_register / write_register helpers
