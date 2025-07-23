/*
 * 文件路径: /d%3A/workspace/RustLearn/RustDesignPattern/src/DistributedSystemMode/CommunicationPatterns/api_gateway.rs
 * 
 * API Gateway模式 (API网关)
 * 
 * API Gateway是微服务架构中的一个重要组件，作为所有客户端请求的统一入口点。
 * 它负责请求路由、认证、限流、监控、缓存等功能，简化了客户端与微服务的交互。
 * 
 * 主要特点：
 * 1. 统一入口 - 所有外部请求通过网关进入系统
 * 2. 请求路由 - 根据路径和规则将请求转发到相应的微服务
 * 3. 认证授权 - 集中处理用户认证和权限验证
 * 4. 限流控制 - 防止系统过载，保护后端服务
 * 5. 监控日志 - 收集请求指标和日志信息
 * 6. 响应缓存 - 缓存常用数据以提高性能
 */

use std::collections::HashMap;
use std::sync::{Arc, RwLock, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use std::fmt;

// =================
// 基础数据结构
// =================

/// HTTP请求结构
#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub method: String,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub body: String,
    pub query_params: HashMap<String, String>,
    pub client_ip: String,
    pub timestamp: u64,
}

/// HTTP响应结构
#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub status_code: u16,
    pub headers: HashMap<String, String>,
    pub body: String,
    pub processing_time: Duration,
}

/// API网关错误类型
#[derive(Debug, Clone)]
pub enum GatewayError {
    ServiceUnavailable,
    Unauthorized,
    RateLimitExceeded,
    BadRequest(String),
    ServiceTimeout,
    RouteNotFound,
    InternalError(String),
}

impl fmt::Display for GatewayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GatewayError::ServiceUnavailable => write!(f, "服务不可用"),
            GatewayError::Unauthorized => write!(f, "未授权访问"),
            GatewayError::RateLimitExceeded => write!(f, "请求频率超限"),
            GatewayError::BadRequest(msg) => write!(f, "错误请求: {}", msg),
            GatewayError::ServiceTimeout => write!(f, "服务超时"),
            GatewayError::RouteNotFound => write!(f, "路由未找到"),
            GatewayError::InternalError(msg) => write!(f, "内部错误: {}", msg),
        }
    }
}

pub type GatewayResult<T> = Result<T, GatewayError>;

// =================
// 路由配置
// =================

/// 路由规则
#[derive(Debug, Clone)]
pub struct Route {
    pub path_pattern: String,
    pub target_service: String,
    pub target_path: String,
    pub methods: Vec<String>,
    pub require_auth: bool,
    pub rate_limit: Option<RateLimit>,
    pub timeout: Duration,
    pub cache_ttl: Option<Duration>,
}

/// 限流配置
#[derive(Debug, Clone)]
pub struct RateLimit {
    pub requests_per_minute: u32,
    pub requests_per_hour: u32,
}

/// 路由管理器
pub struct RouteManager {
    routes: Arc<RwLock<Vec<Route>>>,
}

impl RouteManager {
    pub fn new() -> Self {
        Self {
            routes: Arc::new(RwLock::new(Vec::new())),
        }
    }
    
    pub fn add_route(&self, route: Route) {
        let mut routes = self.routes.write().unwrap();
        routes.push(route);
    }
    
    pub fn find_route(&self, path: &str, method: &str) -> Option<Route> {
        let routes = self.routes.read().unwrap();
        for route in routes.iter() {
            if self.matches_pattern(&route.path_pattern, path) && 
               route.methods.contains(&method.to_string()) {
                return Some(route.clone());
            }
        }
        None
    }
    
    fn matches_pattern(&self, pattern: &str, path: &str) -> bool {
        // 简单的路径匹配实现
        if pattern.contains("*") {
            let prefix = pattern.trim_end_matches("*");
            path.starts_with(prefix)
        } else {
            pattern == path
        }
    }
}

// =================
// 认证管理
// =================

/// 用户认证信息
#[derive(Debug, Clone)]
pub struct AuthContext {
    pub user_id: String,
    pub username: String,
    pub roles: Vec<String>,
    pub permissions: Vec<String>,
    pub expires_at: u64,
}

/// 认证管理器
pub struct AuthManager {
    tokens: Arc<RwLock<HashMap<String, AuthContext>>>,
    api_keys: Arc<RwLock<HashMap<String, AuthContext>>>,
}

impl AuthManager {
    pub fn new() -> Self {
        Self {
            tokens: Arc::new(RwLock::new(HashMap::new())),
            api_keys: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    pub fn create_token(&self, user_id: String, username: String, roles: Vec<String>) -> String {
        let token = format!("token_{}", user_id);
        let auth_context = AuthContext {
            user_id,
            username,
            roles,
            permissions: vec!["read".to_string(), "write".to_string()],
            expires_at: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() + 3600,
        };
        
        let mut tokens = self.tokens.write().unwrap();
        tokens.insert(token.clone(), auth_context);
        token
    }
    
    pub fn create_api_key(&self, user_id: String, username: String) -> String {
        let api_key = format!("api_key_{}", user_id);
        let auth_context = AuthContext {
            user_id,
            username,
            roles: vec!["api_client".to_string()],
            permissions: vec!["api_access".to_string()],
            expires_at: u64::MAX, // API密钥永不过期
        };
        
        let mut api_keys = self.api_keys.write().unwrap();
        api_keys.insert(api_key.clone(), auth_context);
        api_key
    }
    
    pub fn authenticate(&self, request: &HttpRequest) -> GatewayResult<Option<AuthContext>> {
        // 检查Authorization头
        if let Some(auth_header) = request.headers.get("Authorization") {
            if auth_header.starts_with("Bearer ") {
                let token = &auth_header[7..];
                return self.validate_token(token);
            }
        }
        
        // 检查API密钥
        if let Some(api_key) = request.headers.get("X-API-Key") {
            return self.validate_api_key(api_key);
        }
        
        Ok(None)
    }
    
    fn validate_token(&self, token: &str) -> GatewayResult<Option<AuthContext>> {
        let tokens = self.tokens.read().unwrap();
        if let Some(auth_context) = tokens.get(token) {
            let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
            if auth_context.expires_at > now {
                Ok(Some(auth_context.clone()))
            } else {
                Err(GatewayError::Unauthorized)
            }
        } else {
            Err(GatewayError::Unauthorized)
        }
    }
    
    fn validate_api_key(&self, api_key: &str) -> GatewayResult<Option<AuthContext>> {
        let api_keys = self.api_keys.read().unwrap();
        if let Some(auth_context) = api_keys.get(api_key) {
            Ok(Some(auth_context.clone()))
        } else {
            Err(GatewayError::Unauthorized)
        }
    }
}

// =================
// 限流管理
// =================

/// 限流记录
#[derive(Debug, Clone)]
pub struct RateLimitRecord {
    pub requests_per_minute: u32,
    pub requests_per_hour: u32,
    pub last_minute_reset: u64,
    pub last_hour_reset: u64,
}

/// 限流管理器
pub struct RateLimiter {
    records: Arc<Mutex<HashMap<String, RateLimitRecord>>>,
}

impl RateLimiter {
    pub fn new() -> Self {
        Self {
            records: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    pub fn check_rate_limit(&self, client_id: &str, limit: &RateLimit) -> GatewayResult<()> {
        let mut records = self.records.lock().unwrap();
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let current_minute = now / 60;
        let current_hour = now / 3600;
        
        let record = records.entry(client_id.to_string()).or_insert(RateLimitRecord {
            requests_per_minute: 0,
            requests_per_hour: 0,
            last_minute_reset: current_minute,
            last_hour_reset: current_hour,
        });
        
        // 重置分钟计数器
        if record.last_minute_reset < current_minute {
            record.requests_per_minute = 0;
            record.last_minute_reset = current_minute;
        }
        
        // 重置小时计数器
        if record.last_hour_reset < current_hour {
            record.requests_per_hour = 0;
            record.last_hour_reset = current_hour;
        }
        
        // 检查限流
        if record.requests_per_minute >= limit.requests_per_minute {
            return Err(GatewayError::RateLimitExceeded);
        }
        
        if record.requests_per_hour >= limit.requests_per_hour {
            return Err(GatewayError::RateLimitExceeded);
        }
        
        // 增加计数器
        record.requests_per_minute += 1;
        record.requests_per_hour += 1;
        
        Ok(())
    }
}

// =================
// 缓存管理
// =================

/// 缓存条目
#[derive(Debug, Clone)]
pub struct CacheEntry {
    pub response: HttpResponse,
    pub expires_at: u64,
}

/// 响应缓存管理器
pub struct ResponseCache {
    cache: Arc<RwLock<HashMap<String, CacheEntry>>>,
}

impl ResponseCache {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    pub fn get(&self, key: &str) -> Option<HttpResponse> {
        let cache = self.cache.read().unwrap();
        if let Some(entry) = cache.get(key) {
            let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
            if entry.expires_at > now {
                return Some(entry.response.clone());
            }
        }
        None
    }
    
    pub fn put(&self, key: String, response: HttpResponse, ttl: Duration) {
        let expires_at = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() + ttl.as_secs();
        let entry = CacheEntry {
            response,
            expires_at,
        };
        
        let mut cache = self.cache.write().unwrap();
        cache.insert(key, entry);
    }
    
    pub fn generate_cache_key(&self, request: &HttpRequest) -> String {
        format!("{}:{}:{}", request.method, request.path, 
                request.query_params.iter()
                    .map(|(k, v)| format!("{}={}", k, v))
                    .collect::<Vec<_>>()
                    .join("&"))
    }
}

// =================
// 监控和指标
// =================

/// 请求指标
#[derive(Debug, Clone)]
pub struct RequestMetrics {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub average_response_time: Duration,
    pub requests_by_status: HashMap<u16, u64>,
    pub requests_by_path: HashMap<String, u64>,
}

/// 监控管理器
pub struct MonitoringManager {
    metrics: Arc<RwLock<RequestMetrics>>,
    request_logs: Arc<Mutex<Vec<String>>>,
}

impl MonitoringManager {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(RequestMetrics {
                total_requests: 0,
                successful_requests: 0,
                failed_requests: 0,
                average_response_time: Duration::new(0, 0),
                requests_by_status: HashMap::new(),
                requests_by_path: HashMap::new(),
            })),
            request_logs: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    pub fn record_request(&self, request: &HttpRequest, response: &HttpResponse) {
        let mut metrics = self.metrics.write().unwrap();
        metrics.total_requests += 1;
        
        if response.status_code < 400 {
            metrics.successful_requests += 1;
        } else {
            metrics.failed_requests += 1;
        }
        
        // 更新状态码统计
        *metrics.requests_by_status.entry(response.status_code).or_insert(0) += 1;
        
        // 更新路径统计
        *metrics.requests_by_path.entry(request.path.clone()).or_insert(0) += 1;
        
        // 更新平均响应时间
        let total_time = metrics.average_response_time.as_millis() as u64 * (metrics.total_requests - 1) + 
                        response.processing_time.as_millis() as u64;
        metrics.average_response_time = Duration::from_millis(total_time / metrics.total_requests);
        
        // 记录日志
        let log_entry = format!("[{}] {} {} {} {}ms", 
                               SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                               request.method, request.path, response.status_code, 
                               response.processing_time.as_millis());
        
        let mut logs = self.request_logs.lock().unwrap();
        logs.push(log_entry);
        
        // 保持最新的1000条日志
        if logs.len() > 1000 {
            logs.remove(0);
        }
    }
    
    pub fn get_metrics(&self) -> RequestMetrics {
        self.metrics.read().unwrap().clone()
    }
    
    pub fn get_recent_logs(&self, count: usize) -> Vec<String> {
        let logs = self.request_logs.lock().unwrap();
        logs.iter().rev().take(count).cloned().collect()
    }
}

// =================
// 微服务模拟
// =================

/// 模拟的微服务
pub struct MockService {
    name: String,
    response_time: Duration,
    success_rate: f32,
}

impl MockService {
    pub fn new(name: String, response_time: Duration, success_rate: f32) -> Self {
        Self {
            name,
            response_time,
            success_rate,
        }
    }
    
    pub fn handle_request(&self, request: &HttpRequest) -> GatewayResult<HttpResponse> {
        // 模拟处理时间
        std::thread::sleep(self.response_time);
        
        // 模拟成功率
        let random_value = (SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos() % 100) as f32 / 100.0;
        
        if random_value > self.success_rate {
            return Err(GatewayError::ServiceUnavailable);
        }
        
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        headers.insert("X-Service".to_string(), self.name.clone());
        
        let response_body = format!(r#"{{"service": "{}", "path": "{}", "method": "{}"}}"#, 
                                   self.name, request.path, request.method);
        
        Ok(HttpResponse {
            status_code: 200,
            headers,
            body: response_body,
            processing_time: self.response_time,
        })
    }
}

// =================
// API网关主体
// =================

/// API网关
pub struct ApiGateway {
    route_manager: RouteManager,
    auth_manager: AuthManager,
    rate_limiter: RateLimiter,
    response_cache: ResponseCache,
    monitoring: MonitoringManager,
    services: HashMap<String, MockService>,
}

impl ApiGateway {
    pub fn new() -> Self {
        Self {
            route_manager: RouteManager::new(),
            auth_manager: AuthManager::new(),
            rate_limiter: RateLimiter::new(),
            response_cache: ResponseCache::new(),
            monitoring: MonitoringManager::new(),
            services: HashMap::new(),
        }
    }
    
    pub fn add_route(&mut self, route: Route) {
        self.route_manager.add_route(route);
    }
    
    pub fn add_service(&mut self, name: String, service: MockService) {
        self.services.insert(name, service);
    }
    
    pub fn create_user_token(&self, user_id: String, username: String, roles: Vec<String>) -> String {
        self.auth_manager.create_token(user_id, username, roles)
    }
    
    pub fn create_api_key(&self, user_id: String, username: String) -> String {
        self.auth_manager.create_api_key(user_id, username)
    }
    
    pub fn handle_request(&self, request: HttpRequest) -> HttpResponse {
        let start_time = Instant::now();
        
        let result = self.process_request(&request);
        
        let response = match result {
            Ok(response) => response,
            Err(error) => self.create_error_response(error),
        };
        
        let final_response = HttpResponse {
            processing_time: start_time.elapsed(),
            ..response
        };
        
        // 记录请求指标
        self.monitoring.record_request(&request, &final_response);
        
        final_response
    }
    
    fn process_request(&self, request: &HttpRequest) -> GatewayResult<HttpResponse> {
        // 1. 路由匹配
        let route = self.route_manager.find_route(&request.path, &request.method)
            .ok_or(GatewayError::RouteNotFound)?;
        
        // 2. 认证检查
        if route.require_auth {
            let auth_context = self.auth_manager.authenticate(request)?;
            if auth_context.is_none() {
                return Err(GatewayError::Unauthorized);
            }
        }
        
        // 3. 限流检查
        if let Some(rate_limit) = &route.rate_limit {
            let client_id = request.headers.get("X-Client-ID")
                .unwrap_or(&request.client_ip);
            self.rate_limiter.check_rate_limit(client_id, rate_limit)?;
        }
        
        // 4. 缓存检查
        if let Some(_cache_ttl) = route.cache_ttl {
            let cache_key = self.response_cache.generate_cache_key(request);
            if let Some(cached_response) = self.response_cache.get(&cache_key) {
                return Ok(cached_response);
            }
        }
        
        // 5. 转发请求到目标服务
        let service = self.services.get(&route.target_service)
            .ok_or(GatewayError::ServiceUnavailable)?;
        
        let response = service.handle_request(request)?;
        
        // 6. 缓存响应
        if let Some(cache_ttl) = route.cache_ttl {
            let cache_key = self.response_cache.generate_cache_key(request);
            self.response_cache.put(cache_key, response.clone(), cache_ttl);
        }
        
        Ok(response)
    }
    
    fn create_error_response(&self, error: GatewayError) -> HttpResponse {
        let (status_code, message) = match error {
            GatewayError::RouteNotFound => (404, "路由未找到"),
            GatewayError::Unauthorized => (401, "未授权访问"),
            GatewayError::RateLimitExceeded => (429, "请求频率超限"),
            GatewayError::ServiceUnavailable => (503, "服务不可用"),
            GatewayError::ServiceTimeout => (504, "服务超时"),
            GatewayError::BadRequest(_) => (400, "错误请求"),
            GatewayError::InternalError(_) => (500, "内部错误"),
        };
        
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        
        HttpResponse {
            status_code,
            headers,
            body: format!(r#"{{"error": "{}", "message": "{}"}}"#, error, message),
            processing_time: Duration::new(0, 0),
        }
    }
    
    pub fn get_metrics(&self) -> RequestMetrics {
        self.monitoring.get_metrics()
    }
    
    pub fn get_recent_logs(&self, count: usize) -> Vec<String> {
        self.monitoring.get_recent_logs(count)
    }
}

// =================
// 演示函数
// =================

/// API Gateway模式演示
pub fn demo_api_gateway() {
    println!("=== API Gateway模式演示 ===\n");
    
    // 创建API网关
    let mut gateway = ApiGateway::new();
    
    // 添加模拟服务
    gateway.add_service("user-service".to_string(), 
                       MockService::new("user-service".to_string(), Duration::from_millis(100), 0.95));
    gateway.add_service("order-service".to_string(), 
                       MockService::new("order-service".to_string(), Duration::from_millis(150), 0.90));
    gateway.add_service("product-service".to_string(), 
                       MockService::new("product-service".to_string(), Duration::from_millis(80), 0.98));
    
    // 配置路由
    gateway.add_route(Route {
        path_pattern: "/api/users/*".to_string(),
        target_service: "user-service".to_string(),
        target_path: "/users/*".to_string(),
        methods: vec!["GET".to_string(), "POST".to_string()],
        require_auth: true,
        rate_limit: Some(RateLimit {
            requests_per_minute: 100,
            requests_per_hour: 1000,
        }),
        timeout: Duration::from_secs(5),
        cache_ttl: Some(Duration::from_secs(300)),
    });
    
    gateway.add_route(Route {
        path_pattern: "/api/orders/*".to_string(),
        target_service: "order-service".to_string(),
        target_path: "/orders/*".to_string(),
        methods: vec!["GET".to_string(), "POST".to_string(), "PUT".to_string()],
        require_auth: true,
        rate_limit: Some(RateLimit {
            requests_per_minute: 50,
            requests_per_hour: 500,
        }),
        timeout: Duration::from_secs(10),
        cache_ttl: None,
    });
    
    gateway.add_route(Route {
        path_pattern: "/api/products/*".to_string(),
        target_service: "product-service".to_string(),
        target_path: "/products/*".to_string(),
        methods: vec!["GET".to_string()],
        require_auth: false,
        rate_limit: Some(RateLimit {
            requests_per_minute: 200,
            requests_per_hour: 2000,
        }),
        timeout: Duration::from_secs(3),
        cache_ttl: Some(Duration::from_secs(600)),
    });
    
    // 1. 认证演示
    println!("1. 认证演示:");
    let token = gateway.create_user_token("user123".to_string(), "张三".to_string(), vec!["user".to_string()]);
    let api_key = gateway.create_api_key("client123".to_string(), "移动应用".to_string());
    println!("创建用户令牌: {}", token);
    println!("创建API密钥: {}", api_key);
    
    // 2. 请求处理演示
    println!("\n2. 请求处理演示:");
    
    // 未授权访问
    let request1 = HttpRequest {
        method: "GET".to_string(),
        path: "/api/users/123".to_string(),
        headers: HashMap::new(),
        body: String::new(),
        query_params: HashMap::new(),
        client_ip: "192.168.1.100".to_string(),
        timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
    };
    
    let response1 = gateway.handle_request(request1);
    println!("未授权请求: {} - {}", response1.status_code, response1.body);
    
    // 授权访问
    let mut headers = HashMap::new();
    headers.insert("Authorization".to_string(), format!("Bearer {}", token));
    
    let request2 = HttpRequest {
        method: "GET".to_string(),
        path: "/api/users/123".to_string(),
        headers,
        body: String::new(),
        query_params: HashMap::new(),
        client_ip: "192.168.1.100".to_string(),
        timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
    };
    
    let response2 = gateway.handle_request(request2);
    println!("授权请求: {} - {}", response2.status_code, response2.body);
    
    // 公开API访问
    let request3 = HttpRequest {
        method: "GET".to_string(),
        path: "/api/products/456".to_string(),
        headers: HashMap::new(),
        body: String::new(),
        query_params: HashMap::new(),
        client_ip: "192.168.1.101".to_string(),
        timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
    };
    
    let response3 = gateway.handle_request(request3);
    println!("公开API请求: {} - {}", response3.status_code, response3.body);
    
    // 路由不存在
    let request4 = HttpRequest {
        method: "GET".to_string(),
        path: "/api/unknown".to_string(),
        headers: HashMap::new(),
        body: String::new(),
        query_params: HashMap::new(),
        client_ip: "192.168.1.102".to_string(),
        timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
    };
    
    let response4 = gateway.handle_request(request4);
    println!("未知路由请求: {} - {}", response4.status_code, response4.body);
    
    // 3. 监控统计
    println!("\n3. 监控统计:");
    let metrics = gateway.get_metrics();
    println!("总请求数: {}", metrics.total_requests);
    println!("成功请求数: {}", metrics.successful_requests);
    println!("失败请求数: {}", metrics.failed_requests);
    println!("平均响应时间: {}ms", metrics.average_response_time.as_millis());
    
    println!("状态码分布:");
    for (status, count) in metrics.requests_by_status {
        println!("  {}: {} 次", status, count);
    }
    
    println!("路径访问统计:");
    for (path, count) in metrics.requests_by_path {
        println!("  {}: {} 次", path, count);
    }
    
    // 4. 最近日志
    println!("\n4. 最近请求日志:");
    let logs = gateway.get_recent_logs(5);
    for log in logs {
        println!("  {}", log);
    }
    
    println!("\n【API Gateway模式特点】");
    println!("✓ 统一入口 - 所有外部请求通过网关进入系统");
    println!("✓ 请求路由 - 根据路径和规则将请求转发到相应的微服务");
    println!("✓ 认证授权 - 集中处理用户认证和权限验证");
    println!("✓ 限流控制 - 防止系统过载，保护后端服务");
    println!("✓ 监控日志 - 收集请求指标和日志信息");
    println!("✓ 响应缓存 - 缓存常用数据以提高性能");
} 