//! Reverie Music Server Web UI
//!
//! A modern web interface for Reverie Music Server built with Dioxus.
//! Designed to match the functionality and layout of Navidrome's UI.

mod api;
mod components;
mod pages;
mod routes;
mod state;

pub use routes::App;

fn main() {
    // Initialize logging
    dioxus::logger::init(dioxus::logger::tracing::Level::INFO).expect("failed to init logger");

    // Launch the Dioxus app
    dioxus::launch(App);
}
