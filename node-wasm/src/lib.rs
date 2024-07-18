#![doc = include_str!("../README.md")]
#![cfg(target_arch = "wasm32")]

pub mod error;
pub mod node;
mod oneshot_channel;
pub mod utils;
mod worker;
mod wrapper;

#[cfg(test)]
wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);
