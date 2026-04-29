//! E2E integration tests for the Reverie Subsonic API.
//!
//! These tests start a full server process and exercise the API
//! through real HTTP requests.

// Shared test infrastructure lives in `tests/common/mod.rs`.
// The `#[path]` attribute lets us import it from the sibling directory.
#[path = "../common/mod.rs"]
mod common;

mod smoke_test;
mod subsonic_api_e2e;
