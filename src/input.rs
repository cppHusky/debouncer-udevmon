#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(unused_imports)]
#![allow(dead_code)]
include!(concat!(env!("OUT_DIR"),"/bindings.rs"));
pub type InputEvent=input_event;
pub const SIZEOF_EVENT:usize=std::mem::size_of::<InputEvent>();
