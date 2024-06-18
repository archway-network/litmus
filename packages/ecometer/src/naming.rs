use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Eq, PartialEq, Copy, Clone)]
pub enum NameType {
    Numbered,
    Named,
}

pub trait Naming {
    fn name(&self) -> String;
    fn name_type() -> NameType;
}

impl Naming for String {
    fn name(&self) -> String {
        self.to_string()
    }

    fn name_type() -> NameType {
        NameType::Named
    }
}

impl Naming for &str {
    fn name(&self) -> String {
        self.to_string()
    }

    fn name_type() -> NameType {
        NameType::Named
    }
}

macro_rules! number {
    ($ty:ident) => {
        impl Naming for $ty {
            fn name(&self) -> String {
                self.to_string()
            }

            fn name_type() -> NameType {
                NameType::Numbered
            }
        }
    };
}

number!(u128);
number!(i128);
number!(usize);
number!(i8);
number!(u8);
number!(i16);
number!(u16);
number!(i32);
number!(u32);
number!(i64);
number!(u64);
number!(f32);
number!(f64);
