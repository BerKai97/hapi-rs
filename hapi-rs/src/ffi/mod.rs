#![allow(dead_code)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#[allow(clippy::all)]
mod bindings;

pub(crate) mod raw {
    pub use super::bindings::*;
}

mod extensions;
mod functions;
mod structs;

pub use functions::*;
pub use structs::*;
