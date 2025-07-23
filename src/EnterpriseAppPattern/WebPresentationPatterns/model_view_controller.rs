//! 模型视图控制器模式（Model View Controller）
//! 
//! MVC是最经典的Web表示层架构模式，将应用程序分为三个主要组件：
//! - Model：处理数据和业务逻辑
//! - View：负责用户界面展示
//! - Controller：处理用户输入，协调Model和View
//! 
//! 文件位置：/d%3A/workspace/RustLearn/RustDesignPattern/src/EnterpriseAppPattern/WebPresentationPatterns/model_view_controller.rs

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::fmt;

/// HTTP请求结构
#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub method: String,
    pub path: String,
    pub params: HashMap<String, String>,
    pub body: Option<String>,
    pub headers: HashMap<String, String>,
}

/// HTTP响应结构
#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub status_code: u16,
    pub headers: HashMap<String, String>,
    pub body: String,
}

impl HttpResponse {
    pub fn new(status_code: u16, body: String) -> Self {
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "text/html; charset=utf-8".to_string());
        
        Self {
            status_code,
            headers,
            body,
        }
    }
    
    pub fn json(status_code: u16, body: String) -> Self {
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        
        Self {
            status_code,
            headers,
            body,
        }
    }
}

// =================
// Model 层
// =================

/// 用户模型
#[derive(Debug, Clone)]
pub struct User {
    pub id: u32,
    pub username: String,
    pub email: String,
    pub created_at: String,
    pub is_active: bool,
}

impl User {
    pub fn new(id: u32, username: String, email: String) -> Self {
        Self {
            id,
            username,
            email,
            created_at: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            is_active: true,
        }
    }
    
    /// 用户验证
    pub fn validate(&self) -> Result<(), String> {
        if self.username.is_empty() {
            return Err("用户名不能为空".to_string());
        }
        if !self.email.contains('@') {
            return Err("邮箱格式不正确".to_string());
        }
        Ok(())
    }
    
    /// 激活用户
    pub fn activate(&mut self) {
        self.is_active = true;
    }
    
    /// 停用用户
    pub fn deactivate(&mut self) {
        self.is_active = false;
    }
}

/// 用户仓储接口
pub trait UserRepository {
    fn find_by_id(&self, id: u32) -> Option<User>;
    fn find_all(&self) -> Vec<User>;
    fn save(&mut self, user: User) -> Result<u32, String>;
    fn update(&mut self, user: User) -> Result<(), String>;
    fn delete(&mut self, id: u32) -> Result<(), String>;
    fn find_by_username(&self, username: &str) -> Option<User>;
}

/// 内存用户仓储实现
pub struct InMemoryUserRepository {
    users: Arc<Mutex<HashMap<u32, User>>>,
    next_id: Arc<Mutex<u32>>,
}

impl InMemoryUserRepository {
    pub fn new() -> Self {
        let mut users = HashMap::new();
        
        // 初始化一些测试数据
        users.insert(1, User::new(1, "admin".to_string(), "admin@example.com".to_string()));
        users.insert(2, User::new(2, "user1".to_string(), "user1@example.com".to_string()));
        
        Self {
            users: Arc::new(Mutex::new(users)),
            next_id: Arc::new(Mutex::new(3)),
        }
    }
}

impl UserRepository for InMemoryUserRepository {
    fn find_by_id(&self, id: u32) -> Option<User> {
        let users = self.users.lock().unwrap();
        users.get(&id).cloned()
    }
    
    fn find_all(&self) -> Vec<User> {
        let users = self.users.lock().unwrap();
        users.values().cloned().collect()
    }
    
    fn save(&mut self, mut user: User) -> Result<u32, String> {
        user.validate()?;
        
        let mut users = self.users.lock().unwrap();
        let mut next_id = self.next_id.lock().unwrap();
        
        user.id = *next_id;
        users.insert(user.id, user.clone());
        *next_id += 1;
        
        Ok(user.id)
    }
    
    fn update(&mut self, user: User) -> Result<(), String> {
        user.validate()?;
        
        let mut users = self.users.lock().unwrap();
        if users.contains_key(&user.id) {
            users.insert(user.id, user);
            Ok(())
        } else {
            Err("用户不存在".to_string())
        }
    }
    
    fn delete(&mut self, id: u32) -> Result<(), String> {
        let mut users = self.users.lock().unwrap();
        if users.remove(&id).is_some() {
            Ok(())
        } else {
            Err("用户不存在".to_string())
        }
    }
    
    fn find_by_username(&self, username: &str) -> Option<User> {
        let users = self.users.lock().unwrap();
        users.values().find(|u| u.username == username).cloned()
    }
}

/// 用户服务（Model层的业务逻辑）
pub struct UserService {
    repository: Box<dyn UserRepository + Send + Sync>,
}

impl UserService {
    pub fn new(repository: Box<dyn UserRepository + Send + Sync>) -> Self {
        Self { repository }
    }
    
    /// 创建用户
    pub fn create_user(&mut self, username: String, email: String) -> Result<u32, String> {
        // 检查用户名是否已存在
        if self.repository.find_by_username(&username).is_some() {
            return Err("用户名已存在".to_string());
        }
        
        let user = User::new(0, username, email);
        self.repository.save(user)
    }
    
    /// 获取用户
    pub fn get_user(&self, id: u32) -> Option<User> {
        self.repository.find_by_id(id)
    }
    
    /// 获取所有用户
    pub fn get_all_users(&self) -> Vec<User> {
        self.repository.find_all()
    }
    
    /// 更新用户
    pub fn update_user(&mut self, user: User) -> Result<(), String> {
        self.repository.update(user)
    }
    
    /// 删除用户
    pub fn delete_user(&mut self, id: u32) -> Result<(), String> {
        self.repository.delete(id)
    }
    
    /// 激活用户
    pub fn activate_user(&mut self, id: u32) -> Result<(), String> {
        if let Some(mut user) = self.repository.find_by_id(id) {
            user.activate();
            self.repository.update(user)
        } else {
            Err("用户不存在".to_string())
        }
    }
    
    /// 停用用户
    pub fn deactivate_user(&mut self, id: u32) -> Result<(), String> {
        if let Some(mut user) = self.repository.find_by_id(id) {
            user.deactivate();
            self.repository.update(user)
        } else {
            Err("用户不存在".to_string())
        }
    }
}

// =================
// View 层
// =================

/// 视图接口
pub trait View {
    fn render(&self, data: &ViewData) -> String;
}

/// 视图数据
#[derive(Debug, Clone)]
pub enum ViewData {
    UserList(Vec<User>),
    UserDetail(User),
    UserForm(Option<User>),
    Message(String),
    Error(String),
    Empty,
}

/// HTML视图实现
pub struct HtmlView;

impl View for HtmlView {
    fn render(&self, data: &ViewData) -> String {
        match data {
            ViewData::UserList(users) => self.render_user_list(users),
            ViewData::UserDetail(user) => self.render_user_detail(user),
            ViewData::UserForm(user) => self.render_user_form(user),
            ViewData::Message(msg) => self.render_message(msg),
            ViewData::Error(err) => self.render_error(err),
            ViewData::Empty => self.render_empty(),
        }
    }
}

impl HtmlView {
    fn render_user_list(&self, users: &[User]) -> String {
        let mut html = String::from(r#"
<!DOCTYPE html>
<html>
<head>
    <title>用户列表</title>
    <meta charset="utf-8">
    <style>
        body { font-family: Arial, sans-serif; margin: 20px; }
        table { border-collapse: collapse; width: 100%; }
        th, td { border: 1px solid #ddd; padding: 8px; text-align: left; }
        th { background-color: #f2f2f2; }
        .btn { padding: 5px 10px; margin: 2px; text-decoration: none; }
        .btn-primary { background-color: #007bff; color: white; }
        .btn-success { background-color: #28a745; color: white; }
        .btn-danger { background-color: #dc3545; color: white; }
    </style>
</head>
<body>
    <h1>用户管理系统</h1>
    <a href="/users/new" class="btn btn-success">添加用户</a>
    <table>
        <tr>
            <th>ID</th>
            <th>用户名</th>
            <th>邮箱</th>
            <th>创建时间</th>
            <th>状态</th>
            <th>操作</th>
        </tr>"#);
        
        for user in users {
            let status = if user.is_active { "激活" } else { "停用" };
            let status_color = if user.is_active { "green" } else { "red" };
            
            html.push_str(&format!(r#"
        <tr>
            <td>{}</td>
            <td>{}</td>
            <td>{}</td>
            <td>{}</td>
            <td style="color: {}">{}</td>
            <td>
                <a href="/users/{}" class="btn btn-primary">查看</a>
                <a href="/users/{}/edit" class="btn btn-primary">编辑</a>
                <a href="/users/{}/delete" class="btn btn-danger">删除</a>
            </td>
        </tr>"#, 
                user.id, user.username, user.email, user.created_at, 
                status_color, status, user.id, user.id, user.id));
        }
        
        html.push_str(r#"
    </table>
</body>
</html>"#);
        html
    }
    
    fn render_user_detail(&self, user: &User) -> String {
        let status = if user.is_active { "激活" } else { "停用" };
        format!(r#"
<!DOCTYPE html>
<html>
<head>
    <title>用户详情</title>
    <meta charset="utf-8">
    <style>
        body {{ font-family: Arial, sans-serif; margin: 20px; }}
        .user-info {{ border: 1px solid #ddd; padding: 20px; border-radius: 5px; }}
        .btn {{ padding: 10px 15px; margin: 5px; text-decoration: none; color: white; }}
        .btn-primary {{ background-color: #007bff; }}
        .btn-secondary {{ background-color: #6c757d; }}
    </style>
</head>
<body>
    <h1>用户详情</h1>
    <div class="user-info">
        <p><strong>ID:</strong> {}</p>
        <p><strong>用户名:</strong> {}</p>
        <p><strong>邮箱:</strong> {}</p>
        <p><strong>创建时间:</strong> {}</p>
        <p><strong>状态:</strong> {}</p>
    </div>
    <a href="/users" class="btn btn-secondary">返回列表</a>
    <a href="/users/{}/edit" class="btn btn-primary">编辑用户</a>
</body>
</html>"#, user.id, user.username, user.email, user.created_at, status, user.id)
    }
    
    fn render_user_form(&self, user: &Option<User>) -> String {
        let (id, username, email, title, action) = match user {
            Some(u) => (u.id.to_string(), u.username.clone(), u.email.clone(), "编辑用户", format!("/users/{}", u.id)),
            None => ("".to_string(), "".to_string(), "".to_string(), "添加用户", "/users".to_string()),
        };
        
        format!(r#"
<!DOCTYPE html>
<html>
<head>
    <title>{}</title>
    <meta charset="utf-8">
    <style>
        body {{ font-family: Arial, sans-serif; margin: 20px; }}
        .form-group {{ margin-bottom: 15px; }}
        label {{ display: block; margin-bottom: 5px; }}
        input {{ width: 300px; padding: 8px; border: 1px solid #ddd; }}
        .btn {{ padding: 10px 15px; margin: 5px; border: none; cursor: pointer; }}
        .btn-primary {{ background-color: #007bff; color: white; }}
        .btn-secondary {{ background-color: #6c757d; color: white; }}
    </style>
</head>
<body>
    <h1>{}</h1>
    <form method="post" action="{}">
        <div class="form-group">
            <label for="username">用户名:</label>
            <input type="text" id="username" name="username" value="{}" required>
        </div>
        <div class="form-group">
            <label for="email">邮箱:</label>
            <input type="email" id="email" name="email" value="{}" required>
        </div>
        <button type="submit" class="btn btn-primary">保存</button>
        <a href="/users" class="btn btn-secondary">取消</a>
    </form>
</body>
</html>"#, title, title, action, username, email)
    }
    
    fn render_message(&self, msg: &str) -> String {
        format!(r#"
<!DOCTYPE html>
<html>
<head>
    <title>操作成功</title>
    <meta charset="utf-8">
    <style>
        body {{ font-family: Arial, sans-serif; margin: 20px; }}
        .message {{ padding: 15px; background-color: #d4edda; border: 1px solid #c3e6cb; color: #155724; }}
        .btn {{ padding: 10px 15px; margin: 10px 0; text-decoration: none; background-color: #007bff; color: white; }}
    </style>
</head>
<body>
    <div class="message">{}</div>
    <a href="/users" class="btn">返回用户列表</a>
</body>
</html>"#, msg)
    }
    
    fn render_error(&self, err: &str) -> String {
        format!(r#"
<!DOCTYPE html>
<html>
<head>
    <title>错误</title>
    <meta charset="utf-8">
    <style>
        body {{ font-family: Arial, sans-serif; margin: 20px; }}
        .error {{ padding: 15px; background-color: #f8d7da; border: 1px solid #f5c6cb; color: #721c24; }}
        .btn {{ padding: 10px 15px; margin: 10px 0; text-decoration: none; background-color: #6c757d; color: white; }}
    </style>
</head>
<body>
    <div class="error">错误: {}</div>
    <a href="/users" class="btn">返回用户列表</a>
</body>
</html>"#, err)
    }
    
    fn render_empty(&self) -> String {
        r#"
<!DOCTYPE html>
<html>
<head>
    <title>页面不存在</title>
    <meta charset="utf-8">
</head>
<body>
    <h1>404 - 页面不存在</h1>
    <a href="/users">返回首页</a>
</body>
</html>"#.to_string()
    }
}

/// JSON视图实现
pub struct JsonView;

impl View for JsonView {
    fn render(&self, data: &ViewData) -> String {
        match data {
            ViewData::UserList(users) => {
                let users_json: Vec<_> = users.iter().map(|u| format!(
                    r#"{{"id":{},"username":"{}","email":"{}","created_at":"{}","is_active":{}}}"#,
                    u.id, u.username, u.email, u.created_at, u.is_active
                )).collect();
                format!(r#"{{"users":[{}]}}"#, users_json.join(","))
            },
            ViewData::UserDetail(user) => {
                format!(
                    r#"{{"id":{},"username":"{}","email":"{}","created_at":"{}","is_active":{}}}"#,
                    user.id, user.username, user.email, user.created_at, user.is_active
                )
            },
            ViewData::Message(msg) => format!(r#"{{"message":"{}"}}"#, msg),
            ViewData::Error(err) => format!(r#"{{"error":"{}"}}"#, err),
            _ => r#"{"error":"Invalid request"}"#.to_string(),
        }
    }
}

// =================
// Controller 层
// =================

/// 用户控制器
pub struct UserController {
    user_service: UserService,
    view: Box<dyn View>,
}

impl UserController {
    pub fn new(user_service: UserService, view: Box<dyn View>) -> Self {
        Self {
            user_service,
            view,
        }
    }
    
    /// 处理HTTP请求的主入口
    pub fn handle_request(&mut self, request: &HttpRequest) -> HttpResponse {
        match (request.method.as_str(), request.path.as_str()) {
            ("GET", "/users") => self.index(),
            ("GET", path) if path.starts_with("/users/") => {
                if path.ends_with("/edit") {
                    self.edit(path)
                } else if path.ends_with("/delete") {
                    self.delete(path)
                } else {
                    self.show(path)
                }
            },
            ("GET", "/users/new") => self.new_form(),
            ("POST", "/users") => self.create(request),
            ("POST", path) if path.starts_with("/users/") => self.update(request, path),
            _ => self.not_found(),
        }
    }
    
    /// 显示用户列表
    fn index(&self) -> HttpResponse {
        let users = self.user_service.get_all_users();
        let content = self.view.render(&ViewData::UserList(users));
        HttpResponse::new(200, content)
    }
    
    /// 显示单个用户
    fn show(&self, path: &str) -> HttpResponse {
        if let Some(id) = self.extract_id_from_path(path) {
            if let Some(user) = self.user_service.get_user(id) {
                let content = self.view.render(&ViewData::UserDetail(user));
                HttpResponse::new(200, content)
            } else {
                let content = self.view.render(&ViewData::Error("用户不存在".to_string()));
                HttpResponse::new(404, content)
            }
        } else {
            self.bad_request()
        }
    }
    
    /// 显示新建用户表单
    fn new_form(&self) -> HttpResponse {
        let content = self.view.render(&ViewData::UserForm(None));
        HttpResponse::new(200, content)
    }
    
    /// 创建用户
    fn create(&mut self, request: &HttpRequest) -> HttpResponse {
        if let (Some(username), Some(email)) = (
            request.params.get("username"),
            request.params.get("email")
        ) {
            match self.user_service.create_user(username.clone(), email.clone()) {
                Ok(_) => {
                    let content = self.view.render(&ViewData::Message("用户创建成功".to_string()));
                    HttpResponse::new(200, content)
                },
                Err(err) => {
                    let content = self.view.render(&ViewData::Error(err));
                    HttpResponse::new(400, content)
                }
            }
        } else {
            self.bad_request()
        }
    }
    
    /// 显示编辑用户表单
    fn edit(&self, path: &str) -> HttpResponse {
        if let Some(id) = self.extract_id_from_path(path) {
            if let Some(user) = self.user_service.get_user(id) {
                let content = self.view.render(&ViewData::UserForm(Some(user)));
                HttpResponse::new(200, content)
            } else {
                let content = self.view.render(&ViewData::Error("用户不存在".to_string()));
                HttpResponse::new(404, content)
            }
        } else {
            self.bad_request()
        }
    }
    
    /// 更新用户
    fn update(&mut self, request: &HttpRequest, path: &str) -> HttpResponse {
        if let Some(id) = self.extract_id_from_path(path) {
            if let (Some(username), Some(email)) = (
                request.params.get("username"),
                request.params.get("email")
            ) {
                let user = User::new(id, username.clone(), email.clone());
                match self.user_service.update_user(user) {
                    Ok(_) => {
                        let content = self.view.render(&ViewData::Message("用户更新成功".to_string()));
                        HttpResponse::new(200, content)
                    },
                    Err(err) => {
                        let content = self.view.render(&ViewData::Error(err));
                        HttpResponse::new(400, content)
                    }
                }
            } else {
                self.bad_request()
            }
        } else {
            self.bad_request()
        }
    }
    
    /// 删除用户
    fn delete(&mut self, path: &str) -> HttpResponse {
        if let Some(id) = self.extract_id_from_path(path) {
            match self.user_service.delete_user(id) {
                Ok(_) => {
                    let content = self.view.render(&ViewData::Message("用户删除成功".to_string()));
                    HttpResponse::new(200, content)
                },
                Err(err) => {
                    let content = self.view.render(&ViewData::Error(err));
                    HttpResponse::new(400, content)
                }
            }
        } else {
            self.bad_request()
        }
    }
    
    /// 404页面
    fn not_found(&self) -> HttpResponse {
        let content = self.view.render(&ViewData::Empty);
        HttpResponse::new(404, content)
    }
    
    /// 400错误
    fn bad_request(&self) -> HttpResponse {
        let content = self.view.render(&ViewData::Error("请求参数错误".to_string()));
        HttpResponse::new(400, content)
    }
    
    /// 从路径中提取ID
    fn extract_id_from_path(&self, path: &str) -> Option<u32> {
        let parts: Vec<&str> = path.split('/').collect();
        if parts.len() >= 3 {
            parts[2].parse().ok()
        } else {
            None
        }
    }
}

// =================
// MVC应用程序
// =================

/// MVC应用程序主类
pub struct MVCApplication {
    controller: UserController,
}

impl MVCApplication {
    pub fn new() -> Self {
        // 初始化Model层
        let repository = Box::new(InMemoryUserRepository::new());
        let user_service = UserService::new(repository);
        
        // 初始化View层
        let view = Box::new(HtmlView);
        
        // 初始化Controller层
        let controller = UserController::new(user_service, view);
        
        Self { controller }
    }
    
    pub fn new_with_json_view() -> Self {
        // 使用JSON视图的版本
        let repository = Box::new(InMemoryUserRepository::new());
        let user_service = UserService::new(repository);
        let view = Box::new(JsonView);
        let controller = UserController::new(user_service, view);
        
        Self { controller }
    }
    
    /// 处理HTTP请求
    pub fn handle_request(&mut self, request: HttpRequest) -> HttpResponse {
        println!("处理请求: {} {}", request.method, request.path);
        let response = self.controller.handle_request(&request);
        println!("响应状态: {}", response.status_code);
        response
    }
}

/// MVC模式演示
pub fn demo_mvc_pattern() {
    println!("=== 模型视图控制器（MVC）模式演示 ===\n");
    
    // 创建MVC应用程序
    let mut app = MVCApplication::new();
    
    println!("1. 获取用户列表:");
    let request = HttpRequest {
        method: "GET".to_string(),
        path: "/users".to_string(),
        params: HashMap::new(),
        body: None,
        headers: HashMap::new(),
    };
    let response = app.handle_request(request);
    println!("状态码: {}", response.status_code);
    println!("响应体长度: {} bytes\n", response.body.len());
    
    println!("2. 创建新用户:");
    let mut params = HashMap::new();
    params.insert("username".to_string(), "newuser".to_string());
    params.insert("email".to_string(), "newuser@example.com".to_string());
    
    let request = HttpRequest {
        method: "POST".to_string(),
        path: "/users".to_string(),
        params,
        body: None,
        headers: HashMap::new(),
    };
    let response = app.handle_request(request);
    println!("状态码: {}", response.status_code);
    println!("响应体长度: {} bytes\n", response.body.len());
    
    println!("3. 查看特定用户:");
    let request = HttpRequest {
        method: "GET".to_string(),
        path: "/users/1".to_string(),
        params: HashMap::new(),
        body: None,
        headers: HashMap::new(),
    };
    let response = app.handle_request(request);
    println!("状态码: {}", response.status_code);
    println!("响应体长度: {} bytes\n", response.body.len());
    
    println!("4. 编辑用户表单:");
    let request = HttpRequest {
        method: "GET".to_string(),
        path: "/users/1/edit".to_string(),
        params: HashMap::new(),
        body: None,
        headers: HashMap::new(),
    };
    let response = app.handle_request(request);
    println!("状态码: {}", response.status_code);
    println!("响应体长度: {} bytes\n", response.body.len());
    
    println!("5. JSON API演示:");
    let mut json_app = MVCApplication::new_with_json_view();
    let response = json_app.handle_request(HttpRequest {
        method: "GET".to_string(),
        path: "/users".to_string(),
        params: HashMap::new(),
        body: None,
        headers: HashMap::new(),
    });
    println!("JSON响应: {}\n", response.body);
    
    println!("=== MVC模式特点 ===");
    println!("✓ 分离关注点 - Model处理数据，View处理展示，Controller处理逻辑");
    println!("✓ 松耦合 - 三层之间通过接口交互，便于测试和维护");
    println!("✓ 可重用性 - 同一个Model可以对应多个View");
    println!("✓ 并行开发 - 三层可以独立开发和测试");
    println!("✓ 多种视图 - 支持HTML、JSON等多种响应格式");
}

/// MVC模式演示（包装函数）
pub fn demo() {
    demo_mvc_pattern();
}