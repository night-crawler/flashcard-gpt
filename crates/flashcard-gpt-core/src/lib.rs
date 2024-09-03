#![feature(let_chains)]
#![feature(if_let_guard)]
#![feature(array_chunks)]
#![feature(iter_array_chunks)]

pub mod dto;
pub mod repo;
pub mod error;
pub mod ext;
#[cfg(test)]
pub mod tests;
pub mod logging;
pub mod reexports;
