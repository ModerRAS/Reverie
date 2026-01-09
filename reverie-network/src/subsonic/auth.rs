//! Subsonic API authentication middleware
//!
//! Supports:
//! - Username/password authentication (u=, p= params)
//! - Token-based authentication (u=, t=, s= params)
//!
//! NOTE: Current implementation is simplified and does NOT perform real password/token validation.
//! This is a placeholder for development purposes.

use axum::{
    body::Body,
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use reverie_storage::SubsonicStorage;
use std::sync::Arc;

pub const SUBSONIC_API_VERSION: &str = "1.16.1";

/// Authentication context extracted from request
#[derive(Debug, Clone)]
pub struct AuthContext {
    pub username: String,
    pub is_admin: bool,
}

/// Authentication middleware for Subsonic API requests
pub async fn auth_middleware<S: SubsonicStorage>(
    State(storage): State<Arc<S>>,
    mut req: Request<Body>,
    next: Next,
) -> Result<Response, (i32, String)> {
    let auth_context = extract_auth(&storage, &req).await?;
    req.extensions_mut().insert(auth_context);
    Ok(next.run(req).await)
}

/// Extract authentication information from request query parameters
async fn extract_auth<S: SubsonicStorage>(
    storage: &Arc<S>,
    req: &Request<Body>,
) -> Result<AuthContext, (i32, String)> {
    let query = req.uri().query().unwrap_or("");

    // Parse query parameters
    let mut username = None;
    let mut _password = None;
    let mut _token = None;
    let mut _salt = None;

    for pair in query.split('&') {
        let parts: Vec<&str> = pair.splitn(2, '=').collect();
        if parts.len() == 2 {
            let key = parts[0];
            let value = percent_decode(parts[1]);

            match key {
                "u" => username = Some(value),
                "p" => _password = Some(value),
                "t" => _token = Some(value),
                "s" => _salt = Some(value),
                _ => {}
            }
        }
    }

    // Check if username is provided
    let username = username.ok_or((10, "Missing username parameter".to_string()))?;

    // Look up user in storage
    match storage.get_user(&username).await {
        Ok(Some(user)) => {
            // TODO: Implement proper password/token validation
            // For now, just check that user exists
            Ok(AuthContext {
                username: user.username,
                is_admin: user.admin_role,
            })
        }
        Ok(None) => Err((40, "User not found".to_string())),
        Err(e) => Err((0, format!("Database error: {}", e))),
    }
}

/// Simple percent-decoding for URL parameters
fn percent_decode(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '%' {
            let hex: String = chars.by_ref().take(2).collect();
            if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                result.push(byte as char);
            } else {
                result.push('%');
                result.push_str(&hex);
            }
        } else if c == '+' {
            result.push(' ');
        } else {
            result.push(c);
        }
    }
    result
}

/// Helper to get auth context from request extensions
#[allow(dead_code)]
pub fn get_auth_context(req: &Request<Body>) -> Option<&AuthContext> {
    req.extensions().get()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_percent_decode() {
        assert_eq!(percent_decode("hello%20world"), "hello world");
        assert_eq!(percent_decode("test+space"), "test space");
        assert_eq!(percent_decode("normal"), "normal");
    }

    #[test]
    fn test_api_version() {
        assert_eq!(SUBSONIC_API_VERSION, "1.16.1");
    }
}
