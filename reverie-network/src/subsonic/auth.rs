//! Subsonic API 身份验证中间件
//!
//! 支持：
//! - 用户名/密码身份验证 (u=, p= 参数)
//! - 基于令牌的身份验证 (u=, t=, s= 参数)
//!
//! 注意：当前实现是简化版本，不执行真实的密码/令牌验证。
//! 这是用于开发目的的占位符。

use axum::{
    body::Body,
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use reverie_storage::SubsonicStorage;
use std::sync::Arc;

pub const SUBSONIC_API_VERSION: &str = "1.16.1";

/// 从请求中提取的身份验证上下文
#[derive(Debug, Clone)]
pub struct AuthContext {
    pub username: String,
    pub is_admin: bool,
}

/// Subsonic API 请求的身份验证中间件
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

    // 解析查询参数
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

    // 检查是否提供了用户名
    let username = username.ok_or((10, "Missing username parameter".to_string()))?;

    // 在存储中查找用户
    match storage.get_user(&username).await {
        Ok(Some(user)) => {
            // TODO: 实现正确的密码/令牌验证
            // 目前只检查用户是否存在
            Ok(AuthContext {
                username: user.username,
                is_admin: user.admin_role,
            })
        }
        Ok(None) => Err((40, "User not found".to_string())),
        Err(e) => Err((0, format!("Database error: {}", e))),
    }
}

/// 简单的 URL 参数百分号解码
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

/// 从请求扩展中获取身份验证上下文的辅助函数
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
