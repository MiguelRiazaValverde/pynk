#![deny(clippy::all)]

mod client;
mod client_builder;
mod config;
mod stream;
mod stream_prefs;
mod utils;

extern crate napi;
#[macro_use]
extern crate napi_derive;
