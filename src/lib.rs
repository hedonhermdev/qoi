#![allow(dead_code)]
#![allow(clippy::many_single_char_names)]

pub mod decode;

pub type BoxError = std::boxed::Box<
    dyn std::error::Error // must implement Error to satisfy ?
        + std::marker::Send // needed for threads
        + std::marker::Sync, // needed for threads
>;
