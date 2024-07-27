#![allow(dead_code)]

pub(crate) mod private {
    pub trait Sealed { }

    pub struct Internal;
}
