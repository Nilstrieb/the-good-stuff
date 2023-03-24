#![feature(ptr_metadata)]
#![feature(trace_macros)]
#![feature(auto_traits)]
#![feature(negative_impls)]
#![feature(strict_provenance)]

pub mod cfg_match;
pub mod innocent_linked_list;
pub mod scratch;
pub mod sendsync;
pub mod thin_u128;
pub mod unroll_int;
pub mod unsized_clone;

pub mod safe_extern {
    pub use pm::safe_extern;
}
