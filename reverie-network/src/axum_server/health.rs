//! 健康检查处理器
use axum::{
    response::Json,
    routing::get,
    Router,
};
use crate::dto::HealthResponse;

/// 健康检查处理程序
pub async fn health_handler() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

/// 创建健康检查路由
pub fn create_router() -> Router {
    Router::new().route("/health", get(health_handler))
}
