//! 应用控制器模式 (Application Controller)
//! 
//! 集中处理应用程序的屏幕导航和应用程序流程控制
//! 文件路径：/d%3A/workspace/RustLearn/RustDesignPattern/src/EnterpriseAppPattern/WebPresentationPatterns/application_controller.rs

use std::collections::HashMap;
use std::fmt;

/// 应用控制器错误类型
#[derive(Debug)]
pub enum ApplicationControllerError {
    InvalidCommand(String),
    StateTransitionError(String),
    PermissionDenied(String),
    NavigationError(String),
    ConfigurationError(String),
}

impl fmt::Display for ApplicationControllerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApplicationControllerError::InvalidCommand(msg) => write!(f, "无效命令: {}", msg),
            ApplicationControllerError::StateTransitionError(msg) => write!(f, "状态转换错误: {}", msg),
            ApplicationControllerError::PermissionDenied(msg) => write!(f, "权限拒绝: {}", msg),
            ApplicationControllerError::NavigationError(msg) => write!(f, "导航错误: {}", msg),
            ApplicationControllerError::ConfigurationError(msg) => write!(f, "配置错误: {}", msg),
        }
    }
}

/// HTTP请求对象
#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub method: String,
    pub path: String,
    pub params: HashMap<String, String>,
    pub headers: HashMap<String, String>,
    pub session_id: Option<String>,
    pub user_id: Option<u32>,
}

impl HttpRequest {
    pub fn new(method: &str, path: &str) -> Self {
        Self {
            method: method.to_string(),
            path: path.to_string(),
            params: HashMap::new(),
            headers: HashMap::new(),
            session_id: None,
            user_id: None,
        }
    }

    pub fn with_param(mut self, key: &str, value: &str) -> Self {
        self.params.insert(key.to_string(), value.to_string());
        self
    }

    pub fn with_user(mut self, user_id: u32) -> Self {
        self.user_id = Some(user_id);
        self
    }

    pub fn with_session(mut self, session_id: &str) -> Self {
        self.session_id = Some(session_id.to_string());
        self
    }
}

/// HTTP响应对象
#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub status_code: u16,
    pub headers: HashMap<String, String>,
    pub body: String,
    pub redirect_url: Option<String>,
}

impl HttpResponse {
    pub fn ok(body: String) -> Self {
        Self {
            status_code: 200,
            headers: HashMap::new(),
            body,
            redirect_url: None,
        }
    }

    pub fn redirect(url: String) -> Self {
        let mut headers = HashMap::new();
        headers.insert("Location".to_string(), url.clone());
        
        Self {
            status_code: 302,
            headers,
            body: String::new(),
            redirect_url: Some(url),
        }
    }

    pub fn error(status_code: u16, message: String) -> Self {
        Self {
            status_code,
            headers: HashMap::new(),
            body: message,
            redirect_url: None,
        }
    }
}

/// 应用状态
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ApplicationState {
    NotLoggedIn,
    LoggedIn,
    AdminMode,
    CheckingOut,
    PaymentPending,
    OrderCompleted,
}

/// 命令类型
#[derive(Debug, Clone)]
pub enum Command {
    Login { username: String, password: String },
    Logout,
    ViewProfile,
    EditProfile,
    ViewProducts,
    AddToCart { product_id: u32 },
    Checkout,
    ConfirmPayment,
    ViewOrders,
    AdminDashboard,
    ManageUsers,
    ManageProducts,
}

/// 应用流程状态机
pub struct ApplicationStateMachine {
    current_state: ApplicationState,
    transitions: HashMap<(ApplicationState, String), ApplicationState>,
}

impl ApplicationStateMachine {
    pub fn new() -> Self {
        let mut transitions = HashMap::new();
        
        // 定义状态转换规则
        // 未登录状态
        transitions.insert((ApplicationState::NotLoggedIn, "login".to_string()), ApplicationState::LoggedIn);
        
        // 已登录状态
        transitions.insert((ApplicationState::LoggedIn, "logout".to_string()), ApplicationState::NotLoggedIn);
        transitions.insert((ApplicationState::LoggedIn, "admin".to_string()), ApplicationState::AdminMode);
        transitions.insert((ApplicationState::LoggedIn, "checkout".to_string()), ApplicationState::CheckingOut);
        
        // 管理员模式
        transitions.insert((ApplicationState::AdminMode, "logout".to_string()), ApplicationState::NotLoggedIn);
        transitions.insert((ApplicationState::AdminMode, "user_mode".to_string()), ApplicationState::LoggedIn);
        
        // 结账流程
        transitions.insert((ApplicationState::CheckingOut, "payment".to_string()), ApplicationState::PaymentPending);
        transitions.insert((ApplicationState::CheckingOut, "cancel".to_string()), ApplicationState::LoggedIn);
        
        // 支付流程
        transitions.insert((ApplicationState::PaymentPending, "confirm".to_string()), ApplicationState::OrderCompleted);
        transitions.insert((ApplicationState::PaymentPending, "cancel".to_string()), ApplicationState::LoggedIn);
        
        // 订单完成
        transitions.insert((ApplicationState::OrderCompleted, "continue".to_string()), ApplicationState::LoggedIn);
        
        Self {
            current_state: ApplicationState::NotLoggedIn,
            transitions,
        }
    }

    pub fn get_current_state(&self) -> &ApplicationState {
        &self.current_state
    }

    pub fn transition(&mut self, event: &str) -> Result<&ApplicationState, ApplicationControllerError> {
        let key = (self.current_state.clone(), event.to_string());
        
        match self.transitions.get(&key) {
            Some(new_state) => {
                println!("状态转换: {:?} --[{}]--> {:?}", self.current_state, event, new_state);
                self.current_state = new_state.clone();
                Ok(&self.current_state)
            }
            None => Err(ApplicationControllerError::StateTransitionError(
                format!("无法从状态 {:?} 通过事件 '{}' 进行转换", self.current_state, event)
            ))
        }
    }

    pub fn can_transition(&self, event: &str) -> bool {
        let key = (self.current_state.clone(), event.to_string());
        self.transitions.contains_key(&key)
    }

    pub fn get_available_events(&self) -> Vec<String> {
        self.transitions.keys()
            .filter(|(state, _)| *state == self.current_state)
            .map(|(_, event)| event.clone())
            .collect()
    }
}

/// 导航配置
#[derive(Debug, Clone)]
pub struct NavigationConfig {
    pub view_name: String,
    pub requires_auth: bool,
    pub required_role: Option<String>,
    pub state_requirements: Vec<ApplicationState>,
}

impl NavigationConfig {
    pub fn new(view_name: &str) -> Self {
        Self {
            view_name: view_name.to_string(),
            requires_auth: false,
            required_role: None,
            state_requirements: vec![],
        }
    }

    pub fn requires_auth(mut self) -> Self {
        self.requires_auth = true;
        self
    }

    pub fn requires_role(mut self, role: &str) -> Self {
        self.required_role = Some(role.to_string());
        self
    }

    pub fn requires_state(mut self, state: ApplicationState) -> Self {
        self.state_requirements.push(state);
        self
    }
}

/// 应用控制器
pub struct ApplicationController {
    state_machine: ApplicationStateMachine,
    navigation_config: HashMap<String, NavigationConfig>,
    user_sessions: HashMap<String, UserSession>,
}

#[derive(Debug, Clone)]
pub struct UserSession {
    pub user_id: u32,
    pub username: String,
    pub role: String,
    pub login_time: String,
}

impl ApplicationController {
    pub fn new() -> Self {
        let mut controller = Self {
            state_machine: ApplicationStateMachine::new(),
            navigation_config: HashMap::new(),
            user_sessions: HashMap::new(),
        };
        
        controller.setup_navigation_config();
        controller
    }

    fn setup_navigation_config(&mut self) {
        // 配置各个页面的导航规则
        self.navigation_config.insert("login".to_string(), 
            NavigationConfig::new("login_page"));
        
        self.navigation_config.insert("home".to_string(), 
            NavigationConfig::new("home_page").requires_auth());
        
        self.navigation_config.insert("profile".to_string(), 
            NavigationConfig::new("profile_page").requires_auth());
        
        self.navigation_config.insert("products".to_string(), 
            NavigationConfig::new("products_page").requires_auth());
        
        self.navigation_config.insert("cart".to_string(), 
            NavigationConfig::new("cart_page").requires_auth());
        
        self.navigation_config.insert("checkout".to_string(), 
            NavigationConfig::new("checkout_page")
                .requires_auth()
                .requires_state(ApplicationState::CheckingOut));
        
        self.navigation_config.insert("payment".to_string(), 
            NavigationConfig::new("payment_page")
                .requires_auth()
                .requires_state(ApplicationState::PaymentPending));
        
        self.navigation_config.insert("admin".to_string(), 
            NavigationConfig::new("admin_dashboard")
                .requires_auth()
                .requires_role("admin")
                .requires_state(ApplicationState::AdminMode));
        
        self.navigation_config.insert("manage_users".to_string(), 
            NavigationConfig::new("manage_users_page")
                .requires_auth()
                .requires_role("admin"));
    }

    /// 处理用户登录
    pub fn handle_login(&mut self, username: &str, password: &str, session_id: &str) 
        -> Result<HttpResponse, ApplicationControllerError> {
        
        // 简化的认证逻辑
        if self.authenticate_user(username, password) {
            // 创建用户会话
            let role = if username == "admin" { "admin" } else { "user" };
            let session = UserSession {
                user_id: 1, // 简化
                username: username.to_string(),
                role: role.to_string(),
                login_time: "2024-01-01 10:00:00".to_string(), // 简化
            };
            
            self.user_sessions.insert(session_id.to_string(), session);
            
            // 状态转换
            self.state_machine.transition("login")?;
            
            println!("用户 {} 登录成功", username);
            
            // 根据角色决定重定向
            let redirect_url = if role == "admin" {
                "/admin"
            } else {
                "/home"
            };
            
            Ok(HttpResponse::redirect(redirect_url.to_string()))
        } else {
            Err(ApplicationControllerError::PermissionDenied("用户名或密码错误".to_string()))
        }
    }

    /// 处理用户登出
    pub fn handle_logout(&mut self, session_id: &str) -> Result<HttpResponse, ApplicationControllerError> {
        if let Some(session) = self.user_sessions.remove(session_id) {
            self.state_machine.transition("logout")?;
            println!("用户 {} 登出成功", session.username);
            Ok(HttpResponse::redirect("/login".to_string()))
        } else {
            Err(ApplicationControllerError::PermissionDenied("无效的会话".to_string()))
        }
    }

    /// 处理导航请求
    pub fn handle_navigation(&mut self, path: &str, request: &HttpRequest) 
        -> Result<HttpResponse, ApplicationControllerError> {
        
        // 移除路径前的斜杠
        let clean_path = path.trim_start_matches('/');
        
        // 获取导航配置
        let config = self.navigation_config.get(clean_path)
            .ok_or_else(|| ApplicationControllerError::NavigationError(
                format!("未知的路径: {}", path)
            ))?;
        
        // 检查认证要求
        if config.requires_auth {
            let session = self.get_user_session(&request)?;
            
            // 检查角色要求
            if let Some(ref required_role) = config.required_role {
                if session.role != *required_role {
                    return Err(ApplicationControllerError::PermissionDenied(
                        format!("需要 {} 角色权限", required_role)
                    ));
                }
            }
        }
        
        // 检查状态要求
        if !config.state_requirements.is_empty() {
            let current_state = self.state_machine.get_current_state();
            if !config.state_requirements.contains(current_state) {
                return Err(ApplicationControllerError::StateTransitionError(
                    format!("当前状态 {:?} 不满足访问要求", current_state)
                ));
            }
        }
        
        // 执行页面特定的逻辑
        self.execute_page_logic(clean_path, request)
    }

    /// 处理命令
    pub fn handle_command(&mut self, command: Command, request: &HttpRequest) 
        -> Result<HttpResponse, ApplicationControllerError> {
        
        println!("处理命令: {:?}", command);
        
        match command {
            Command::Login { username, password } => {
                let session_id = request.session_id.as_ref()
                    .ok_or_else(|| ApplicationControllerError::InvalidCommand("缺少会话ID".to_string()))?;
                self.handle_login(&username, &password, session_id)
            }
            
            Command::Logout => {
                let session_id = request.session_id.as_ref()
                    .ok_or_else(|| ApplicationControllerError::InvalidCommand("缺少会话ID".to_string()))?;
                self.handle_logout(session_id)
            }
            
            Command::ViewProfile => {
                let _session = self.get_user_session(request)?;
                Ok(HttpResponse::ok("用户个人资料页面".to_string()))
            }
            
            Command::AddToCart { product_id } => {
                let session = self.get_user_session(request)?;
                println!("用户 {} 添加产品 {} 到购物车", session.username, product_id);
                Ok(HttpResponse::ok("产品已添加到购物车".to_string()))
            }
            
            Command::Checkout => {
                let _session = self.get_user_session(request)?;
                self.state_machine.transition("checkout")?;
                Ok(HttpResponse::redirect("/checkout".to_string()))
            }
            
            Command::ConfirmPayment => {
                let _session = self.get_user_session(request)?;
                self.state_machine.transition("confirm")?;
                Ok(HttpResponse::redirect("/order_success".to_string()))
            }
            
            Command::AdminDashboard => {
                let session = self.get_user_session(request)?;
                if session.role != "admin" {
                    return Err(ApplicationControllerError::PermissionDenied("需要管理员权限".to_string()));
                }
                self.state_machine.transition("admin")?;
                Ok(HttpResponse::redirect("/admin".to_string()))
            }
            
            _ => Ok(HttpResponse::ok("命令执行成功".to_string()))
        }
    }

    /// 获取用户会话
    fn get_user_session(&self, request: &HttpRequest) -> Result<&UserSession, ApplicationControllerError> {
        let session_id = request.session_id.as_ref()
            .ok_or_else(|| ApplicationControllerError::PermissionDenied("未登录".to_string()))?;
        
        self.user_sessions.get(session_id)
            .ok_or_else(|| ApplicationControllerError::PermissionDenied("会话已过期".to_string()))
    }

    /// 执行页面特定的逻辑
    fn execute_page_logic(&self, path: &str, _request: &HttpRequest) 
        -> Result<HttpResponse, ApplicationControllerError> {
        
        let page_content = match path {
            "login" => self.render_login_page(),
            "home" => self.render_home_page(),
            "profile" => self.render_profile_page(),
            "products" => self.render_products_page(),
            "cart" => self.render_cart_page(),
            "checkout" => self.render_checkout_page(),
            "payment" => self.render_payment_page(),
            "admin" => self.render_admin_page(),
            "manage_users" => self.render_manage_users_page(),
            _ => format!("页面: {}", path)
        };
        
        Ok(HttpResponse::ok(page_content))
    }

    /// 页面渲染方法
    fn render_login_page(&self) -> String {
        "登录页面 - 请输入用户名和密码".to_string()
    }

    fn render_home_page(&self) -> String {
        format!("首页 - 当前状态: {:?}, 可用操作: {:?}", 
                self.state_machine.get_current_state(),
                self.state_machine.get_available_events())
    }

    fn render_profile_page(&self) -> String {
        "个人资料页面 - 用户信息管理".to_string()
    }

    fn render_products_page(&self) -> String {
        "产品页面 - 浏览和购买产品".to_string()
    }

    fn render_cart_page(&self) -> String {
        "购物车页面 - 管理购物车商品".to_string()
    }

    fn render_checkout_page(&self) -> String {
        "结账页面 - 确认订单信息".to_string()
    }

    fn render_payment_page(&self) -> String {
        "支付页面 - 选择支付方式".to_string()
    }

    fn render_admin_page(&self) -> String {
        "管理员仪表板 - 系统管理功能".to_string()
    }

    fn render_manage_users_page(&self) -> String {
        "用户管理页面 - 管理系统用户".to_string()
    }

    /// 简化的用户认证
    fn authenticate_user(&self, username: &str, password: &str) -> bool {
        // 简化的认证逻辑
        (username == "admin" && password == "admin123") ||
        (username == "user" && password == "user123") ||
        (username == "alice" && password == "alice123")
    }

    /// 获取当前状态信息
    pub fn get_state_info(&self) -> String {
        format!("当前状态: {:?}, 可用事件: {:?}", 
                self.state_machine.get_current_state(),
                self.state_machine.get_available_events())
    }

    /// 获取所有活跃会话
    pub fn get_active_sessions(&self) -> Vec<&UserSession> {
        self.user_sessions.values().collect()
    }
}

/// 演示应用控制器模式
pub fn demo() {
    println!("=== 应用控制器模式演示 ===\n");
    
    let mut app_controller = ApplicationController::new();
    
    println!("1. 初始化应用控制器:");
    println!("   {}", app_controller.get_state_info());
    
    println!("\n{}", "=".repeat(50));
    
    // 用户登录流程
    println!("2. 用户登录流程:");
    
    let login_request = HttpRequest::new("POST", "/login")
        .with_session("session_123");
    
    match app_controller.handle_login("alice", "alice123", "session_123") {
        Ok(response) => {
            println!("✅ 登录成功");
            println!("   状态码: {}", response.status_code);
            if let Some(ref redirect) = response.redirect_url {
                println!("   重定向到: {}", redirect);
            }
            println!("   {}", app_controller.get_state_info());
        }
        Err(e) => println!("❌ 登录失败: {}", e),
    }
    
    println!("\n{}", "=".repeat(50));
    
    // 页面导航
    println!("3. 页面导航:");
    
    let nav_request = HttpRequest::new("GET", "/home")
        .with_session("session_123");
    
    match app_controller.handle_navigation("/home", &nav_request) {
        Ok(response) => {
            println!("✅ 访问首页成功");
            println!("   内容: {}", response.body);
        }
        Err(e) => println!("❌ 访问失败: {}", e),
    }
    
    // 尝试访问需要特殊权限的页面
    match app_controller.handle_navigation("/admin", &nav_request) {
        Ok(response) => {
            println!("✅ 访问管理页面成功");
            println!("   内容: {}", response.body);
        }
        Err(e) => println!("❌ 访问管理页面失败: {}", e),
    }
    
    println!("\n{}", "=".repeat(50));
    
    // 命令处理
    println!("4. 命令处理:");
    
    let cmd_request = HttpRequest::new("POST", "/add_to_cart")
        .with_session("session_123")
        .with_param("product_id", "101");
    
    let add_to_cart_cmd = Command::AddToCart { product_id: 101 };
    match app_controller.handle_command(add_to_cart_cmd, &cmd_request) {
        Ok(response) => {
            println!("✅ 添加到购物车成功");
            println!("   响应: {}", response.body);
        }
        Err(e) => println!("❌ 添加到购物车失败: {}", e),
    }
    
    // 结账流程
    let checkout_cmd = Command::Checkout;
    match app_controller.handle_command(checkout_cmd, &cmd_request) {
        Ok(response) => {
            println!("✅ 开始结账流程");
            if let Some(ref redirect) = response.redirect_url {
                println!("   重定向到: {}", redirect);
            }
            println!("   {}", app_controller.get_state_info());
        }
        Err(e) => println!("❌ 结账失败: {}", e),
    }
    
    // 访问结账页面
    match app_controller.handle_navigation("/checkout", &cmd_request) {
        Ok(response) => {
            println!("✅ 访问结账页面成功");
            println!("   内容: {}", response.body);
        }
        Err(e) => println!("❌ 访问结账页面失败: {}", e),
    }
    
    println!("\n{}", "=".repeat(50));
    
    // 状态转换演示
    println!("5. 状态转换演示:");
    
    // 进入支付流程
    if app_controller.state_machine.can_transition("payment") {
        match app_controller.state_machine.transition("payment") {
            Ok(_) => {
                println!("✅ 进入支付流程");
                println!("   {}", app_controller.get_state_info());
            }
            Err(e) => println!("❌ 状态转换失败: {}", e),
        }
    }
    
    // 确认支付
    let confirm_payment_cmd = Command::ConfirmPayment;
    match app_controller.handle_command(confirm_payment_cmd, &cmd_request) {
        Ok(response) => {
            println!("✅ 支付确认成功");
            if let Some(ref redirect) = response.redirect_url {
                println!("   重定向到: {}", redirect);
            }
            println!("   {}", app_controller.get_state_info());
        }
        Err(e) => println!("❌ 支付确认失败: {}", e),
    }
    
    println!("\n{}", "=".repeat(50));
    
    // 管理员登录演示
    println!("6. 管理员功能演示:");
    
    // 管理员登录
    match app_controller.handle_login("admin", "admin123", "admin_session") {
        Ok(response) => {
            println!("✅ 管理员登录成功");
            if let Some(ref redirect) = response.redirect_url {
                println!("   重定向到: {}", redirect);
            }
        }
        Err(e) => println!("❌ 管理员登录失败: {}", e),
    }
    
    let admin_request = HttpRequest::new("GET", "/admin")
        .with_session("admin_session");
    
    // 访问管理员页面
    match app_controller.handle_navigation("/admin", &admin_request) {
        Ok(response) => {
            println!("✅ 访问管理员页面成功");
            println!("   内容: {}", response.body);
        }
        Err(e) => println!("❌ 访问管理员页面失败: {}", e),
    }
    
    println!("\n{}", "=".repeat(50));
    
    // 会话管理
    println!("7. 会话管理:");
    
    let active_sessions = app_controller.get_active_sessions();
    println!("活跃会话数: {}", active_sessions.len());
    for session in active_sessions {
        println!("  - 用户: {} (角色: {}, 登录时间: {})", 
                session.username, session.role, session.login_time);
    }
    
    println!("\n{}", "=".repeat(50));
    
    println!("应用控制器模式的特点:");
    println!("✅ 集中管理应用程序流程控制");
    println!("✅ 处理复杂的状态转换逻辑");
    println!("✅ 统一的权限和认证管理");
    println!("✅ 灵活的导航配置");
    println!("✅ 支持多用户会话管理");
    
    println!("\n适用场景:");
    println!("• 复杂的业务流程应用");
    println!("• 需要状态管理的交互式应用");
    println!("• 多角色权限系统");
    println!("• 向导式操作流程");
    println!("• 需要集中控制导航的应用");
    
    println!("\n设计要点:");
    println!("• 明确定义应用状态和转换规则");
    println!("• 集中配置导航和权限规则");
    println!("• 分离业务逻辑和流程控制");
    println!("• 提供灵活的扩展机制");
    println!("• 考虑并发和会话管理");
} 