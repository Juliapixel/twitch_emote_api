#![warn(clippy::unwrap_used, reason = "unwrapping is bad, and makes things hard to debug, use expect() instead")]

pub mod cache;
pub mod cli;
pub mod emote;
pub mod platforms;
