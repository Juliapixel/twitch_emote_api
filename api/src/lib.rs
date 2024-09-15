#![warn(
    clippy::unwrap_used,
    reason = "unwrapping is bad, and makes things hard to debug, use expect() instead"
)]
#![allow(async_fn_in_trait, reason = "this is fine, we're not a library")]
#![feature(impl_trait_in_assoc_type)]
#![feature(iterator_try_collect)]

pub mod cache;
pub mod cli;
pub mod emote;
pub mod platforms;
