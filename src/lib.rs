#![warn(clippy::all, rust_2018_idioms)]

mod app;
pub use app::EncodecExplorer;
mod compute;
mod audio;
mod synth;
mod code_ui;