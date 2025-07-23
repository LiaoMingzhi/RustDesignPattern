/*
 * 文件路径: /d%3A/workspace/RustLearn/RustDesignPattern/src/EnterpriseAppPattern/WebPresentationPatterns/front_controller.rs
 * 
 * Front Controller（前端控制器）模式
 * 
 * 定义：
 * Front Controller模式提供一个集中的入口点来处理所有的Web请求。
 * 它负责请求的路由、通用处理逻辑（如认证、日志、错误处理）以及将请求分发给相应的处理器。
 * 
 * 主要特点：
 * 1. 集中式请求处理
 * 2. 统一的安全控制
 * 3. 通用功能的复用
 * 4. 请求路由和分发
 * 5. 视图管理和渲染
 * 
 * 优势：
 * - 减少代码重复
 * - 统一的错误处理
 * - 集中的安全控制
 * - 更好的可维护性
 * 
 * 适用场景：
 * - Web应用的入口控制
 * - 需要统一认证和授权的系统
 * - 复杂的路由需求
 * - 微服务的API网关
 */

use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Display, Formatter};

/// 前端控制器错误类型
#[derive(Debug)]
pub enum FrontControllerError {
    RouteNotFound(String),
    AuthenticationRequired,
    AuthorizationFailed(String),
    ValidationError(String),
    InternalError(String),
    BadRequest(String),
}

impl Display for FrontControllerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            FrontControllerError::RouteNotFound(path) => write!(f, "路由未找到: {}", path),
            FrontControllerError::AuthenticationRequired => write!(f, "需要身份认证"),
            FrontControllerError::AuthorizationFailed(msg) => write!(f, "授权失败: {}", msg),
            FrontControllerError::ValidationError(msg) => write!(f, "验证错误: {}", msg),
            FrontControllerError::InternalError(msg) => write!(f, "内部错误: {}", msg),
            FrontControllerError::BadRequest(msg) => write!(f, "请求错误: {}", msg),
        }
    }
}

impl Error for FrontControllerError {}

/// HTTP方法枚举
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
    OPTIONS,
    HEAD,
}

impl From<&str> for HttpMethod {
    fn from(method: &str) -> Self {
        match method.to_uppercase().as_str() {
            "GET" => HttpMethod::GET,
            "POST" => HttpMethod::POST,
            "PUT" => HttpMethod::PUT,
            "DELETE" => HttpMethod::DELETE,
            "PATCH" => HttpMethod::PATCH,
            "OPTIONS" => HttpMethod::OPTIONS,
            "HEAD" => HttpMethod::HEAD,
            _ => HttpMethod::GET, // 默认为GET
        }
    }
}

/// 请求上下文
#[derive(Debug, Clone)]
pub struct RequestContext {
    pub method: HttpMethod,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub query_params: HashMap<String, String>,
    pub path_params: HashMap<String, String>,
    pub body: String,
    pub user_id: Option<String>,
    pub session_id: Option<String>,
}

impl RequestContext {
    pub fn new(method: HttpMethod, path: String) -> Self {
        Self {
            method,
            path,
            headers: HashMap::new(),
            query_params: HashMap::new(),
            path_params: HashMap::new(),
            body: String::new(),
            user_id: None,
            session_id: None,
        }
    }
    
    pub fn with_header(mut self, key: String, value: String) -> Self {
        self.headers.insert(key, value);
        self
    }
    
    pub fn with_query_param(mut self, key: String, value: String) -> Self {
        self.query_params.insert(key, value);
        self
    }
    
    pub fn with_body(mut self, body: String) -> Self {
        self.body = body;
        self
    }
    
    pub fn get_header(&self, key: &str) -> Option<&String> {
        self.headers.get(key)
    }
    
    pub fn get_query_param(&self, key: &str) -> Option<&String> {
        self.query_params.get(key)
    }
    
    pub fn is_authenticated(&self) -> bool {
        self.user_id.is_some()
    }
}

/// 响应对象
#[derive(Debug, Clone)]
pub struct Response {
    pub status_code: u16,
    pub headers: HashMap<String, String>,
    pub body: String,
}

impl Response {
    pub fn new(status_code: u16) -> Self {
        Self {
            status_code,
            headers: HashMap::new(),
            body: String::new(),
        }
    }
    
    pub fn ok() -> Self {
        Self::new(200)
    }
    
    pub fn created() -> Self {
        Self::new(201)
    }
    
    pub fn bad_request() -> Self {
        Self::new(400)
    }
    
    pub fn unauthorized() -> Self {
        Self::new(401)
    }
    
    pub fn forbidden() -> Self {
        Self::new(403)
    }
    
    pub fn not_found() -> Self {
        Self::new(404)
    }
    
    pub fn internal_server_error() -> Self {
        Self::new(500)
    }
    
    pub fn with_header(mut self, key: String, value: String) -> Self {
        self.headers.insert(key, value);
        self
    }
    
    pub fn with_body(mut self, body: String) -> Self {
        self.body = body;
        self
    }
    
    pub fn json(mut self, json_body: String) -> Self {
        self.headers.insert("Content-Type".to_string(), "application/json".to_string());
        self.body = json_body;
        self
    }
    
    pub fn html(mut self, html_body: String) -> Self {
        self.headers.insert("Content-Type".to_string(), "text/html; charset=utf-8".to_string());
        self.body = html_body;
        self
    }
}

/// 命令接口
pub trait Command {
    fn execute(&self, context: &RequestContext) -> Result<Response, FrontControllerError>;
}

/// 中间件trait
pub trait Middleware {
    fn process(&self, context: &mut RequestContext) -> Result<(), FrontControllerError>;
}

/// 路由规则
#[derive(Debug, Clone)]
pub struct Route {
    pub method: HttpMethod,
    pub pattern: String,
    pub command_name: String,
    pub requires_auth: bool,
    pub roles: Vec<String>,
}

impl Route {
    pub fn new(method: HttpMethod, pattern: String, command_name: String) -> Self {
        Self {
            method,
            pattern,
            command_name,
            requires_auth: false,
            roles: Vec::new(),
        }
    }
    
    pub fn requires_auth(mut self) -> Self {
        self.requires_auth = true;
        self
    }
    
    pub fn with_roles(mut self, roles: Vec<String>) -> Self {
        self.roles = roles;
        self
    }
    
    /// 检查路径是否匹配
    pub fn matches(&self, method: &HttpMethod, path: &str) -> Option<HashMap<String, String>> {
        if self.method != *method {
            return None;
        }
        
        // 简单的路径匹配实现
        let pattern_parts: Vec<&str> = self.pattern.split('/').collect();
        let path_parts: Vec<&str> = path.split('/').collect();
        
        if pattern_parts.len() != path_parts.len() {
            return None;
        }
        
        let mut params = HashMap::new();
        
        for (pattern_part, path_part) in pattern_parts.iter().zip(path_parts.iter()) {
            if pattern_part.starts_with(':') {
                // 路径参数
                let param_name = &pattern_part[1..];
                params.insert(param_name.to_string(), path_part.to_string());
            } else if pattern_part != path_part {
                return None;
            }
        }
        
        Some(params)
    }
}

/// 认证中间件
pub struct AuthenticationMiddleware {
    pub secret_key: String,
}

impl AuthenticationMiddleware {
    pub fn new(secret_key: String) -> Self {
        Self { secret_key }
    }
}

impl Middleware for AuthenticationMiddleware {
    fn process(&self, context: &mut RequestContext) -> Result<(), FrontControllerError> {
        if let Some(auth_header) = context.get_header("Authorization") {
            if auth_header.starts_with("Bearer ") {
                let token = &auth_header[7..];
                // 简化的JWT验证（实际应该验证签名）
                if token.contains("valid_token") {
                    context.user_id = Some("user_123".to_string());
                    context.session_id = Some("session_456".to_string());
                    return Ok(());
                }
            }
        }
        
        // 没有有效的认证信息
        Ok(()) // 这里不抛出错误，让路由决定是否需要认证
    }
}

/// 日志中间件
pub struct LoggingMiddleware;

impl Middleware for LoggingMiddleware {
    fn process(&self, context: &mut RequestContext) -> Result<(), FrontControllerError> {
        println!("[LOG] {} {} - User: {:?}", 
                context.method.clone() as u32, 
                context.path, 
                context.user_id);
        Ok(())
    }
}

/// CORS中间件
pub struct CorsMiddleware {
    pub allowed_origins: Vec<String>,
}

impl CorsMiddleware {
    pub fn new(allowed_origins: Vec<String>) -> Self {
        Self { allowed_origins }
    }
}

impl Middleware for CorsMiddleware {
    fn process(&self, context: &mut RequestContext) -> Result<(), FrontControllerError> {
        if let Some(origin) = context.get_header("Origin") {
            if !self.allowed_origins.contains(origin) && !self.allowed_origins.contains(&"*".to_string()) {
                return Err(FrontControllerError::AuthorizationFailed(
                    "CORS origin not allowed".to_string()
                ));
            }
        }
        Ok(())
    }
}

/// 用户控制器命令
pub struct UserListCommand;

impl Command for UserListCommand {
    fn execute(&self, context: &RequestContext) -> Result<Response, FrontControllerError> {
        // 模拟获取用户列表
        let page = context.get_query_param("page").unwrap_or(&"1".to_string()).clone();
        let limit = context.get_query_param("limit").unwrap_or(&"10".to_string()).clone();
        
        let json_response = format!(
            r#"{{"users": [{{"id": "1", "name": "张三"}}, {{"id": "2", "name": "李四"}}], "page": {}, "limit": {}}}"#,
            page, limit
        );
        
        Ok(Response::ok().json(json_response))
    }
}

/// 用户详情命令
pub struct UserDetailCommand;

impl Command for UserDetailCommand {
    fn execute(&self, context: &RequestContext) -> Result<Response, FrontControllerError> {
        let user_id = context.path_params.get("id")
            .ok_or_else(|| FrontControllerError::BadRequest("缺少用户ID参数".to_string()))?;
        
        if user_id == "123" {
            let json_response = format!(
                r#"{{"id": "{}", "name": "张三", "email": "zhang@example.com"}}"#,
                user_id
            );
            Ok(Response::ok().json(json_response))
        } else {
            Err(FrontControllerError::RouteNotFound(format!("用户 {} 不存在", user_id)))
        }
    }
}

/// 用户创建命令
pub struct UserCreateCommand;

impl Command for UserCreateCommand {
    fn execute(&self, context: &RequestContext) -> Result<Response, FrontControllerError> {
        if !context.is_authenticated() {
            return Err(FrontControllerError::AuthenticationRequired);
        }
        
        if context.body.is_empty() {
            return Err(FrontControllerError::ValidationError("请求体不能为空".to_string()));
        }
        
        // 模拟创建用户
        let new_user_id = "new_user_456";
        let json_response = format!(
            r#"{{"id": "{}", "message": "用户创建成功"}}"#,
            new_user_id
        );
        
        Ok(Response::created().json(json_response))
    }
}

/// 首页命令
pub struct HomePageCommand;

impl Command for HomePageCommand {
    fn execute(&self, _context: &RequestContext) -> Result<Response, FrontControllerError> {
        let html_content = r#"
<!DOCTYPE html>
<html>
<head>
    <title>欢迎</title>
    <meta charset="utf-8">
</head>
<body>
    <h1>欢迎来到我们的网站</h1>
    <p>这是一个使用Front Controller模式的Web应用示例。</p>
    <ul>
        <li><a href="/api/users">用户列表</a></li>
        <li><a href="/api/users/123">用户详情</a></li>
    </ul>
</body>
</html>
        "#.trim();
        
        Ok(Response::ok().html(html_content.to_string()))
    }
}

/// 前端控制器
pub struct FrontController {
    routes: Vec<Route>,
    commands: HashMap<String, Box<dyn Command>>,
    middlewares: Vec<Box<dyn Middleware>>,
}

impl FrontController {
    pub fn new() -> Self {
        Self {
            routes: Vec::new(),
            commands: HashMap::new(),
            middlewares: Vec::new(),
        }
    }
    
    /// 注册路由
    pub fn register_route(&mut self, route: Route) {
        self.routes.push(route);
    }
    
    /// 注册命令
    pub fn register_command(&mut self, name: String, command: Box<dyn Command>) {
        self.commands.insert(name, command);
    }
    
    /// 添加中间件
    pub fn add_middleware(&mut self, middleware: Box<dyn Middleware>) {
        self.middlewares.push(middleware);
    }
    
    /// 处理请求
    pub fn handle_request(&self, mut context: RequestContext) -> Result<Response, FrontControllerError> {
        // 1. 执行中间件
        for middleware in &self.middlewares {
            middleware.process(&mut context)?;
        }
        
        // 2. 路由匹配
        let mut matched_route = None;
        for route in &self.routes {
            if let Some(params) = route.matches(&context.method, &context.path) {
                context.path_params = params;
                matched_route = Some(route);
                break;
            }
        }
        
        let route = matched_route.ok_or_else(|| {
            FrontControllerError::RouteNotFound(format!("{} {}", context.method as u32, context.path))
        })?;
        
        // 3. 认证检查
        if route.requires_auth && !context.is_authenticated() {
            return Err(FrontControllerError::AuthenticationRequired);
        }
        
        // 4. 授权检查
        if !route.roles.is_empty() {
            // 简化的角色检查
            let user_role = "user"; // 实际应该从用户信息中获取
            if !route.roles.contains(&user_role.to_string()) {
                return Err(FrontControllerError::AuthorizationFailed(
                    "权限不足".to_string()
                ));
            }
        }
        
        // 5. 执行命令
        let command = self.commands.get(&route.command_name)
            .ok_or_else(|| FrontControllerError::InternalError(
                format!("命令未找到: {}", route.command_name)
            ))?;
        
        let mut response = command.execute(&context)?;
        
        // 6. 添加通用响应头
        response.headers.insert("X-Request-ID".to_string(), "req_12345".to_string());
        response.headers.insert("X-Response-Time".to_string(), "10ms".to_string());
        
        Ok(response)
    }
    
    /// 处理错误
    pub fn handle_error(&self, error: &FrontControllerError) -> Response {
        match error {
            FrontControllerError::RouteNotFound(path) => {
                Response::not_found()
                    .json(format!(r#"{{"error": "路由未找到", "path": "{}"}}"#, path))
            }
            FrontControllerError::AuthenticationRequired => {
                Response::unauthorized()
                    .json(r#"{"error": "需要身份认证"}"#.to_string())
            }
            FrontControllerError::AuthorizationFailed(msg) => {
                Response::forbidden()
                    .json(format!(r#"{{"error": "授权失败", "message": "{}"}}"#, msg))
            }
            FrontControllerError::ValidationError(msg) => {
                Response::bad_request()
                    .json(format!(r#"{{"error": "验证错误", "message": "{}"}}"#, msg))
            }
            FrontControllerError::BadRequest(msg) => {
                Response::bad_request()
                    .json(format!(r#"{{"error": "请求错误", "message": "{}"}}"#, msg))
            }
            FrontControllerError::InternalError(msg) => {
                Response::internal_server_error()
                    .json(format!(r#"{{"error": "内部错误", "message": "{}"}}"#, msg))
            }
        }
    }
}

impl Default for FrontController {
    fn default() -> Self {
        Self::new()
    }
}

/// 前端控制器模式演示
pub fn demo() {
    println!("=== Front Controller（前端控制器）模式演示 ===\n");
    
    // 1. 创建前端控制器
    let mut controller = FrontController::new();
    
    // 2. 注册中间件
    controller.add_middleware(Box::new(LoggingMiddleware));
    controller.add_middleware(Box::new(AuthenticationMiddleware::new("secret_key".to_string())));
    controller.add_middleware(Box::new(CorsMiddleware::new(vec!["*".to_string()])));
    
    // 3. 注册命令
    controller.register_command("home".to_string(), Box::new(HomePageCommand));
    controller.register_command("user_list".to_string(), Box::new(UserListCommand));
    controller.register_command("user_detail".to_string(), Box::new(UserDetailCommand));
    controller.register_command("user_create".to_string(), Box::new(UserCreateCommand));
    
    // 4. 注册路由
    controller.register_route(Route::new(
        HttpMethod::GET,
        "/".to_string(),
        "home".to_string(),
    ));
    
    controller.register_route(Route::new(
        HttpMethod::GET,
        "/api/users".to_string(),
        "user_list".to_string(),
    ));
    
    controller.register_route(Route::new(
        HttpMethod::GET,
        "/api/users/:id".to_string(),
        "user_detail".to_string(),
    ));
    
    controller.register_route(Route::new(
        HttpMethod::POST,
        "/api/users".to_string(),
        "user_create".to_string(),
    ).requires_auth());
    
    println!("前端控制器初始化完成，已注册 {} 个路由", controller.routes.len());
    
    println!("{}", "=".repeat(50));
    
    // 5. 模拟请求处理
    println!("1. 处理首页请求:");
    let home_request = RequestContext::new(HttpMethod::GET, "/".to_string());
    
    match controller.handle_request(home_request) {
        Ok(response) => {
            println!("响应状态: {}", response.status_code);
            println!("响应头: {:?}", response.headers);
            println!("响应体预览: {}...", &response.body[..100.min(response.body.len())]);
        }
        Err(e) => {
            println!("请求处理失败: {}", e);
            let error_response = controller.handle_error(&e);
            println!("错误响应: {}", error_response.body);
        }
    }
    
    println!("{}", "=".repeat(50));
    
    // 6. 处理API请求
    println!("2. 处理用户列表请求:");
    let users_request = RequestContext::new(HttpMethod::GET, "/api/users".to_string())
        .with_query_param("page".to_string(), "1".to_string())
        .with_query_param("limit".to_string(), "5".to_string());
    
    match controller.handle_request(users_request) {
        Ok(response) => {
            println!("响应状态: {}", response.status_code);
            println!("响应体: {}", response.body);
        }
        Err(e) => {
            println!("请求处理失败: {}", e);
        }
    }
    
    println!("{}", "=".repeat(50));
    
    // 7. 处理带路径参数的请求
    println!("3. 处理用户详情请求:");
    let user_detail_request = RequestContext::new(HttpMethod::GET, "/api/users/123".to_string());
    
    match controller.handle_request(user_detail_request) {
        Ok(response) => {
            println!("响应状态: {}", response.status_code);
            println!("响应体: {}", response.body);
        }
        Err(e) => {
            println!("请求处理失败: {}", e);
        }
    }
    
    println!("{}", "=".repeat(50));
    
    // 8. 处理需要认证的请求（无认证）
    println!("4. 处理需要认证的请求（无认证）:");
    let create_request_no_auth = RequestContext::new(HttpMethod::POST, "/api/users".to_string())
        .with_body(r#"{"name": "新用户", "email": "new@example.com"}"#.to_string());
    
    match controller.handle_request(create_request_no_auth) {
        Ok(response) => {
            println!("响应状态: {}", response.status_code);
            println!("响应体: {}", response.body);
        }
        Err(e) => {
            println!("请求处理失败: {}", e);
            let error_response = controller.handle_error(&e);
            println!("错误响应: {}", error_response.body);
        }
    }
    
    println!("{}", "=".repeat(50));
    
    // 9. 处理需要认证的请求（有认证）
    println!("5. 处理需要认证的请求（有认证）:");
    let create_request_with_auth = RequestContext::new(HttpMethod::POST, "/api/users".to_string())
        .with_header("Authorization".to_string(), "Bearer valid_token_12345".to_string())
        .with_body(r#"{"name": "新用户", "email": "new@example.com"}"#.to_string());
    
    match controller.handle_request(create_request_with_auth) {
        Ok(response) => {
            println!("响应状态: {}", response.status_code);
            println!("响应体: {}", response.body);
        }
        Err(e) => {
            println!("请求处理失败: {}", e);
        }
    }
    
    println!("{}", "=".repeat(50));
    
    // 10. 处理不存在的路由
    println!("6. 处理不存在的路由:");
    let not_found_request = RequestContext::new(HttpMethod::GET, "/api/nonexistent".to_string());
    
    match controller.handle_request(not_found_request) {
        Ok(response) => {
            println!("响应状态: {}", response.status_code);
            println!("响应体: {}", response.body);
        }
        Err(e) => {
            println!("请求处理失败: {}", e);
            let error_response = controller.handle_error(&e);
            println!("错误响应状态: {}", error_response.status_code);
            println!("错误响应体: {}", error_response.body);
        }
    }
    
    println!("\n=== Front Controller模式演示完成 ===");
    
    // 输出模式总结
    println!("\n【Front Controller模式总结】");
    println!("核心组件:");
    println!("1. FrontController：核心控制器，处理所有请求");
    println!("2. Command：具体的业务处理逻辑");
    println!("3. Router：路由匹配和分发");
    println!("4. Middleware：中间件处理通用逻辑");
    println!("5. Context：请求上下文对象");
    
    println!("\n优势:");
    println!("1. 集中控制：所有请求都通过统一入口");
    println!("2. 代码复用：通用逻辑在中间件中实现");
    println!("3. 安全统一：认证和授权集中处理");
    println!("4. 易于维护：路由和处理逻辑分离");
    println!("5. 可扩展性：易于添加新的功能和中间件");
    
    println!("\n适用场景:");
    println!("1. Web应用的请求处理");
    println!("2. API网关实现");
    println!("3. 微服务的统一入口");
    println!("4. 需要统一认证授权的系统");
    println!("5. 复杂的路由需求");
} 