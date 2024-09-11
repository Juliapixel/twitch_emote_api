#![warn(clippy::unwrap_used, reason = "unwrapping is bad, and makes things hard to debug, use expect() instead")]
#![allow(async_fn_in_trait, reason = "this is fine, we're not a library")]

pub mod cache;
pub mod cli;
pub mod emote;
pub mod platforms;
