/*
 * 文件路径: /d%3A/workspace/RustLearn/RustDesignPattern/src/EnterpriseAppPattern/Layering/presentation_layer.rs
 * 
 * Presentation Layer（表现层）模式
 */

use serde::{Serialize, Deserialize};

/// 简化的错误类型
#[derive(Debug)]
pub struct PresentationError {
    pub message: String,
}

impl std::fmt::Display for PresentationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for PresentationError {}

/// 简化的 HTTP 请求
#[derive(Debug)]
pub struct HttpRequest {
    pub body: String,
}

impl HttpRequest {
    pub fn new(body: String) -> Self {
        Self { body }
    }
}

/// 简化的 HTTP 响应
#[derive(Debug)]
pub struct HttpResponse {
    pub status_code: u16,
    pub body: String,
}

impl HttpResponse {
    pub fn new(status_code: u16, body: String) -> Self {
        Self { status_code, body }
    }
}

/// 用户注册请求
#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
}

/// 用户控制器
pub struct UserController;

impl UserController {
    pub fn new() -> Self {
        Self
    }
    
    pub fn register(&self, _request: &HttpRequest) -> Result<HttpResponse, PresentationError> {
        Ok(HttpResponse::new(201, "注册成功".to_string()))
    }
}

/// 演示函数
pub fn demo() {
    println!("=== Presentation Layer 模式演示 ===");
    
    let controller = UserController::new();
    let request = HttpRequest::new("{}".to_string());
    
    match controller.register(&request) {
        Ok(response) => println!("响应: {} - {}", response.status_code, response.body),
        Err(e) => println!("错误: {}", e),
    }
    
    println!("=== 演示完成 ===");
} 
