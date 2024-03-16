#![feature(ptr_metadata)]
#![feature(strict_provenance)]

pub mod cfg_match;
pub mod hashmaps;
pub mod innocent_linked_list;
pub mod scratch;
#[cfg(FALSE)]
pub mod sendsync;
pub mod thin_u128;
pub mod unroll_int;
pub mod unsized_clone;

pub mod safe_extern {
    pub use pm::safe_extern;
}
