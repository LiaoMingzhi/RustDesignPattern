// 文件路径: /d%3A/workspace/RustLearn/RustDesignPattern/src/EnterpriseAppPattern/WebPresentationPatterns/page_controller.rs

//! # 页面控制器模式 (Page Controller)
//!
//! ## 概述
//! 页面控制器模式为每个页面或页面组创建一个单独的控制器，
//! 每个控制器负责处理特定页面的请求和响应。
//!
//! ## 优点
//! - 简单直观，每个页面有自己的控制器
//! - 便于理解和维护
//! - 适合简单到中等复杂度的Web应用
//! - 易于测试单个页面的逻辑
//! - 支持页面级的定制化处理
//!
//! ## 适用场景
//! - 简单到中等复杂度的Web应用
//! - 页面间逻辑差异较大的应用
//! - 需要页面级定制的系统
//! - 团队成员技能水平参差不齐的项目

use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, Mutex};

/// 页面控制器错误
#[derive(Debug)]
pub enum PageControllerError {
    ValidationError(String),
    NotFound(String),
    ServiceError(String),
    TemplateError(String),
}

impl fmt::Display for PageControllerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PageControllerError::ValidationError(msg) => write!(f, "验证错误: {}", msg),
            PageControllerError::NotFound(msg) => write!(f, "未找到: {}", msg),
            PageControllerError::ServiceError(msg) => write!(f, "服务错误: {}", msg),
            PageControllerError::TemplateError(msg) => write!(f, "模板错误: {}", msg),
        }
    }
}

impl std::error::Error for PageControllerError {}

/// HTTP请求
#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub method: String,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub query_params: HashMap<String, String>,
    pub form_data: HashMap<String, String>,
    pub session: HashMap<String, String>,
}

impl HttpRequest {
    pub fn new(method: String, path: String) -> Self {
        Self {
            method,
            path,
            headers: HashMap::new(),
            query_params: HashMap::new(),
            form_data: HashMap::new(),
            session: HashMap::new(),
        }
    }

    pub fn with_query_param(mut self, key: String, value: String) -> Self {
        self.query_params.insert(key, value);
        self
    }

    pub fn with_form_data(mut self, data: HashMap<String, String>) -> Self {
        self.form_data = data;
        self
    }

    pub fn with_session(mut self, session: HashMap<String, String>) -> Self {
        self.session = session;
        self
    }
}

/// HTTP响应
#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: String,
}

impl HttpResponse {
    pub fn ok(body: String) -> Self {
        Self {
            status: 200,
            headers: HashMap::new(),
            body,
        }
    }

    pub fn redirect(location: String) -> Self {
        let mut headers = HashMap::new();
        headers.insert("Location".to_string(), location);
        Self {
            status: 302,
            headers,
            body: "".to_string(),
        }
    }

    pub fn not_found(body: String) -> Self {
        Self {
            status: 404,
            headers: HashMap::new(),
            body,
        }
    }

    pub fn bad_request(body: String) -> Self {
        Self {
            status: 400,
            headers: HashMap::new(),
            body,
        }
    }
}

/// 用户实体
#[derive(Debug, Clone)]
pub struct User {
    pub id: u32,
    pub username: String,
    pub email: String,
    pub full_name: String,
    pub created_at: String,
}

impl User {
    pub fn new(id: u32, username: String, email: String, full_name: String) -> Self {
        Self {
            id,
            username,
            email,
            full_name,
            created_at: "2024-01-01T00:00:00Z".to_string(),
        }
    }
}

/// 文章实体
#[derive(Debug, Clone)]
pub struct Article {
    pub id: u32,
    pub title: String,
    pub content: String,
    pub author_id: u32,
    pub published: bool,
    pub created_at: String,
}

impl Article {
    pub fn new(id: u32, title: String, content: String, author_id: u32) -> Self {
        Self {
            id,
            title,
            content,
            author_id,
            published: false,
            created_at: "2024-01-01T00:00:00Z".to_string(),
        }
    }
}

/// 模拟数据服务
pub struct DataService {
    users: Arc<Mutex<HashMap<u32, User>>>,
    articles: Arc<Mutex<HashMap<u32, Article>>>,
    next_user_id: Arc<Mutex<u32>>,
    next_article_id: Arc<Mutex<u32>>,
}

impl DataService {
    pub fn new() -> Self {
        let service = Self {
            users: Arc::new(Mutex::new(HashMap::new())),
            articles: Arc::new(Mutex::new(HashMap::new())),
            next_user_id: Arc::new(Mutex::new(1)),
            next_article_id: Arc::new(Mutex::new(1)),
        };
        
        service.init_test_data();
        service
    }

    fn init_test_data(&self) {
        // 初始化用户
        let mut users = self.users.lock().unwrap();
        let mut user_id = self.next_user_id.lock().unwrap();
        
        users.insert(1, User::new(1, "alice".to_string(), "alice@example.com".to_string(), "Alice Johnson".to_string()));
        users.insert(2, User::new(2, "bob".to_string(), "bob@example.com".to_string(), "Bob Smith".to_string()));
        *user_id = 3;
        
        // 初始化文章
        let mut articles = self.articles.lock().unwrap();
        let mut article_id = self.next_article_id.lock().unwrap();
        
        articles.insert(1, Article::new(1, "Rust编程入门".to_string(), "Rust是一门系统编程语言...".to_string(), 1));
        articles.insert(2, Article::new(2, "设计模式详解".to_string(), "设计模式是软件设计中的最佳实践...".to_string(), 1));
        articles.insert(3, Article::new(3, "Web开发技术".to_string(), "现代Web开发涉及前端和后端...".to_string(), 2));
        *article_id = 4;
    }

    pub fn get_user(&self, id: u32) -> Option<User> {
        self.users.lock().unwrap().get(&id).cloned()
    }

    pub fn get_all_users(&self) -> Vec<User> {
        self.users.lock().unwrap().values().cloned().collect()
    }

    pub fn get_article(&self, id: u32) -> Option<Article> {
        self.articles.lock().unwrap().get(&id).cloned()
    }

    pub fn get_all_articles(&self) -> Vec<Article> {
        self.articles.lock().unwrap().values().cloned().collect()
    }

    pub fn get_articles_by_author(&self, author_id: u32) -> Vec<Article> {
        self.articles.lock().unwrap()
            .values()
            .filter(|article| article.author_id == author_id)
            .cloned()
            .collect()
    }

    pub fn create_article(&self, title: String, content: String, author_id: u32) -> Article {
        let mut articles = self.articles.lock().unwrap();
        let mut next_id = self.next_article_id.lock().unwrap();
        
        let article = Article::new(*next_id, title, content, author_id);
        articles.insert(*next_id, article.clone());
        *next_id += 1;
        
        article
    }
}

/// 页面控制器基类
pub trait PageController {
    fn handle_get(&self, request: &HttpRequest) -> Result<HttpResponse, PageControllerError>;
    fn handle_post(&self, request: &HttpRequest) -> Result<HttpResponse, PageControllerError>;
}

/// 主页控制器
pub struct HomePageController {
    data_service: Arc<DataService>,
}

impl HomePageController {
    pub fn new(data_service: Arc<DataService>) -> Self {
        Self { data_service }
    }

    fn render_home_page(&self) -> String {
        let users_count = self.data_service.get_all_users().len();
        let articles_count = self.data_service.get_all_articles().len();

        format!(
            r#"
            <!DOCTYPE html>
            <html>
            <head>
                <title>首页</title>
                <style>
                    body {{ font-family: Arial, sans-serif; margin: 40px; background-color: #f5f5f5; }}
                    .container {{ max-width: 800px; margin: 0 auto; background: white; padding: 30px; border-radius: 8px; box-shadow: 0 2px 10px rgba(0,0,0,0.1); }}
                    .hero {{ text-align: center; margin-bottom: 40px; }}
                    .stats {{ display: flex; justify-content: space-around; margin: 30px 0; }}
                    .stat-card {{ background: #e3f2fd; padding: 20px; border-radius: 8px; text-align: center; min-width: 120px; }}
                    .navigation {{ text-align: center; margin-top: 30px; }}
                    .nav-link {{ display: inline-block; margin: 0 15px; padding: 10px 20px; background: #2196f3; color: white; text-decoration: none; border-radius: 4px; }}
                    .nav-link:hover {{ background: #1976d2; }}
                </style>
            </head>
            <body>
                <div class="container">
                    <div class="hero">
                        <h1>🏠 欢迎来到我们的网站</h1>
                        <p>这是一个展示页面控制器模式的示例网站</p>
                    </div>
                    
                    <div class="stats">
                        <div class="stat-card">
                            <h3>{}</h3>
                            <p>注册用户</p>
                        </div>
                        <div class="stat-card">
                            <h3>{}</h3>
                            <p>发布文章</p>
                        </div>
                    </div>
                    
                    <div class="navigation">
                        <a href="/users" class="nav-link">👥 用户列表</a>
                        <a href="/articles" class="nav-link">📰 文章列表</a>
                        <a href="/about" class="nav-link">ℹ️ 关于我们</a>
                    </div>
                </div>
            </body>
            </html>
            "#,
            users_count, articles_count
        )
    }
}

impl PageController for HomePageController {
    fn handle_get(&self, _request: &HttpRequest) -> Result<HttpResponse, PageControllerError> {
        println!("   🏠 处理首页GET请求");
        let content = self.render_home_page();
        Ok(HttpResponse::ok(content))
    }

    fn handle_post(&self, _request: &HttpRequest) -> Result<HttpResponse, PageControllerError> {
        Err(PageControllerError::ValidationError("首页不支持POST请求".to_string()))
    }
}

/// 用户列表页面控制器
pub struct UserListPageController {
    data_service: Arc<DataService>,
}

impl UserListPageController {
    pub fn new(data_service: Arc<DataService>) -> Self {
        Self { data_service }
    }

    fn render_user_list(&self, users: &[User]) -> String {
        let mut user_rows = String::new();
        for user in users {
            user_rows.push_str(&format!(
                "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td><a href='/users/{}'>查看详情</a></td></tr>",
                user.id, user.username, user.email, user.full_name, user.id
            ));
        }

        format!(
            r#"
            <!DOCTYPE html>
            <html>
            <head>
                <title>用户列表</title>
                <style>
                    body {{ font-family: Arial, sans-serif; margin: 40px; }}
                    .container {{ max-width: 1000px; margin: 0 auto; }}
                    table {{ border-collapse: collapse; width: 100%; }}
                    th, td {{ border: 1px solid #ddd; padding: 12px; text-align: left; }}
                    th {{ background-color: #f2f2f2; }}
                    .back-btn {{ display: inline-block; margin-bottom: 20px; padding: 8px 16px; background: #666; color: white; text-decoration: none; border-radius: 4px; }}
                </style>
            </head>
            <body>
                <div class="container">
                    <a href="/" class="back-btn">← 返回首页</a>
                    <h1>👥 用户列表</h1>
                    <table>
                        <thead>
                            <tr>
                                <th>ID</th>
                                <th>用户名</th>
                                <th>邮箱</th>
                                <th>姓名</th>
                                <th>操作</th>
                            </tr>
                        </thead>
                        <tbody>
                            {}
                        </tbody>
                    </table>
                </div>
            </body>
            </html>
            "#,
            user_rows
        )
    }
}

impl PageController for UserListPageController {
    fn handle_get(&self, _request: &HttpRequest) -> Result<HttpResponse, PageControllerError> {
        println!("   👥 处理用户列表GET请求");
        let users = self.data_service.get_all_users();
        let content = self.render_user_list(&users);
        Ok(HttpResponse::ok(content))
    }

    fn handle_post(&self, _request: &HttpRequest) -> Result<HttpResponse, PageControllerError> {
        Err(PageControllerError::ValidationError("用户列表页不支持POST请求".to_string()))
    }
}

/// 用户详情页面控制器
pub struct UserDetailPageController {
    data_service: Arc<DataService>,
}

impl UserDetailPageController {
    pub fn new(data_service: Arc<DataService>) -> Self {
        Self { data_service }
    }

    fn extract_user_id(&self, path: &str) -> Option<u32> {
        // 从 "/users/1" 中提取 1
        if let Some(id_str) = path.strip_prefix("/users/") {
            id_str.parse().ok()
        } else {
            None
        }
    }

    fn render_user_detail(&self, user: &User, articles: &[Article]) -> String {
        let mut article_list = String::new();
        for article in articles {
            article_list.push_str(&format!(
                "<li><a href='/articles/{}'>{}</a></li>",
                article.id, article.title
            ));
        }

        if article_list.is_empty() {
            article_list = "<li>暂无文章</li>".to_string();
        }

        format!(
            r#"
            <!DOCTYPE html>
            <html>
            <head>
                <title>用户详情 - {}</title>
                <style>
                    body {{ font-family: Arial, sans-serif; margin: 40px; }}
                    .container {{ max-width: 800px; margin: 0 auto; }}
                    .user-card {{ border: 1px solid #ddd; padding: 20px; border-radius: 8px; background: #f9f9f9; }}
                    .field {{ margin: 10px 0; }}
                    .label {{ font-weight: bold; }}
                    .articles {{ margin-top: 30px; }}
                    .back-btn {{ display: inline-block; margin-bottom: 20px; padding: 8px 16px; background: #666; color: white; text-decoration: none; border-radius: 4px; }}
                </style>
            </head>
            <body>
                <div class="container">
                    <a href="/users" class="back-btn">← 返回用户列表</a>
                    <h1>👤 用户详情</h1>
                    <div class="user-card">
                        <div class="field"><span class="label">ID:</span> {}</div>
                        <div class="field"><span class="label">用户名:</span> {}</div>
                        <div class="field"><span class="label">邮箱:</span> {}</div>
                        <div class="field"><span class="label">姓名:</span> {}</div>
                        <div class="field"><span class="label">注册时间:</span> {}</div>
                    </div>
                    
                    <div class="articles">
                        <h3>📝 该用户的文章 ({}篇)</h3>
                        <ul>
                            {}
                        </ul>
                    </div>
                </div>
            </body>
            </html>
            "#,
            user.username, user.id, user.username, user.email, user.full_name, user.created_at, articles.len(), article_list
        )
    }
}

impl PageController for UserDetailPageController {
    fn handle_get(&self, request: &HttpRequest) -> Result<HttpResponse, PageControllerError> {
        println!("   👤 处理用户详情GET请求: {}", request.path);
        
        let user_id = self.extract_user_id(&request.path)
            .ok_or_else(|| PageControllerError::ValidationError("无效的用户ID".to_string()))?;

        let user = self.data_service.get_user(user_id)
            .ok_or_else(|| PageControllerError::NotFound(format!("用户不存在: {}", user_id)))?;

        let articles = self.data_service.get_articles_by_author(user_id);
        let content = self.render_user_detail(&user, &articles);
        Ok(HttpResponse::ok(content))
    }

    fn handle_post(&self, _request: &HttpRequest) -> Result<HttpResponse, PageControllerError> {
        Err(PageControllerError::ValidationError("用户详情页不支持POST请求".to_string()))
    }
}

/// 文章列表页面控制器
pub struct ArticleListPageController {
    data_service: Arc<DataService>,
}

impl ArticleListPageController {
    pub fn new(data_service: Arc<DataService>) -> Self {
        Self { data_service }
    }

    fn render_article_list(&self, articles: &[Article]) -> String {
        let mut article_cards = String::new();
        for article in articles {
            let author = self.data_service.get_user(article.author_id)
                .map(|u| u.username)
                .unwrap_or_else(|| "未知".to_string());

            article_cards.push_str(&format!(
                r#"
                <div class="article-card">
                    <h3><a href="/articles/{}">{}</a></h3>
                    <p class="meta">作者: <a href="/users/{}">{}</a> | 发表时间: {}</p>
                    <p class="preview">{}</p>
                </div>
                "#,
                article.id,
                article.title,
                article.author_id,
                author,
                article.created_at,
                if article.content.len() > 100 {
                    format!("{}...", &article.content[..100])
                } else {
                    article.content.clone()
                }
            ));
        }

        format!(
            r#"
            <!DOCTYPE html>
            <html>
            <head>
                <title>文章列表</title>
                <style>
                    body {{ font-family: Arial, sans-serif; margin: 40px; }}
                    .container {{ max-width: 800px; margin: 0 auto; }}
                    .article-card {{ border: 1px solid #ddd; padding: 20px; margin: 20px 0; border-radius: 8px; }}
                    .article-card h3 {{ margin-top: 0; }}
                    .article-card h3 a {{ text-decoration: none; color: #333; }}
                    .article-card h3 a:hover {{ color: #2196f3; }}
                    .meta {{ color: #666; font-size: 14px; }}
                    .meta a {{ color: #2196f3; text-decoration: none; }}
                    .preview {{ margin: 15px 0 0 0; line-height: 1.6; }}
                    .back-btn {{ display: inline-block; margin-bottom: 20px; padding: 8px 16px; background: #666; color: white; text-decoration: none; border-radius: 4px; }}
                    .new-btn {{ display: inline-block; margin-left: 10px; padding: 8px 16px; background: #4CAF50; color: white; text-decoration: none; border-radius: 4px; }}
                </style>
            </head>
            <body>
                <div class="container">
                    <a href="/" class="back-btn">← 返回首页</a>
                    <a href="/articles/new" class="new-btn">✏️ 写文章</a>
                    <h1>📰 文章列表</h1>
                    {}
                </div>
            </body>
            </html>
            "#,
            article_cards
        )
    }
}

impl PageController for ArticleListPageController {
    fn handle_get(&self, _request: &HttpRequest) -> Result<HttpResponse, PageControllerError> {
        println!("   📰 处理文章列表GET请求");
        let articles = self.data_service.get_all_articles();
        let content = self.render_article_list(&articles);
        Ok(HttpResponse::ok(content))
    }

    fn handle_post(&self, _request: &HttpRequest) -> Result<HttpResponse, PageControllerError> {
        Err(PageControllerError::ValidationError("文章列表页不支持POST请求".to_string()))
    }
}

/// 新建文章页面控制器
pub struct NewArticlePageController {
    data_service: Arc<DataService>,
}

impl NewArticlePageController {
    pub fn new(data_service: Arc<DataService>) -> Self {
        Self { data_service }
    }

    fn render_new_article_form(&self, error: Option<&str>) -> String {
        let error_html = if let Some(err) = error {
            format!("<div class='error'>错误: {}</div>", err)
        } else {
            String::new()
        };

        format!(
            r#"
            <!DOCTYPE html>
            <html>
            <head>
                <title>写文章</title>
                <style>
                    body {{ font-family: Arial, sans-serif; margin: 40px; }}
                    .container {{ max-width: 600px; margin: 0 auto; }}
                    .form-group {{ margin: 20px 0; }}
                    label {{ display: block; margin-bottom: 5px; font-weight: bold; }}
                    input[type="text"], textarea {{ width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; font-size: 14px; }}
                    textarea {{ height: 200px; resize: vertical; }}
                    select {{ width: 100%; padding: 10px; border: 1px solid #ddd; border-radius: 4px; }}
                    .btn {{ background-color: #4CAF50; color: white; padding: 12px 24px; border: none; border-radius: 4px; cursor: pointer; font-size: 14px; }}
                    .btn:hover {{ background-color: #45a049; }}
                    .back-btn {{ display: inline-block; margin-bottom: 20px; padding: 8px 16px; background: #666; color: white; text-decoration: none; border-radius: 4px; }}
                    .error {{ background-color: #f8d7da; color: #721c24; padding: 10px; border-radius: 4px; margin-bottom: 20px; }}
                </style>
            </head>
            <body>
                <div class="container">
                    <a href="/articles" class="back-btn">← 返回文章列表</a>
                    <h1>✏️ 写文章</h1>
                    {}
                    <form method="post">
                        <div class="form-group">
                            <label for="title">文章标题:</label>
                            <input type="text" id="title" name="title" required>
                        </div>
                        <div class="form-group">
                            <label for="author_id">作者:</label>
                            <select id="author_id" name="author_id" required>
                                <option value="1">Alice Johnson</option>
                                <option value="2">Bob Smith</option>
                            </select>
                        </div>
                        <div class="form-group">
                            <label for="content">文章内容:</label>
                            <textarea id="content" name="content" placeholder="请输入文章内容..." required></textarea>
                        </div>
                        <button type="submit" class="btn">发布文章</button>
                    </form>
                </div>
            </body>
            </html>
            "#,
            error_html
        )
    }
}

impl PageController for NewArticlePageController {
    fn handle_get(&self, _request: &HttpRequest) -> Result<HttpResponse, PageControllerError> {
        println!("   ✏️ 处理新建文章GET请求");
        let content = self.render_new_article_form(None);
        Ok(HttpResponse::ok(content))
    }

    fn handle_post(&self, request: &HttpRequest) -> Result<HttpResponse, PageControllerError> {
        println!("   ✏️ 处理新建文章POST请求");
        
        let title = request.form_data.get("title")
            .ok_or_else(|| PageControllerError::ValidationError("缺少文章标题".to_string()))?;
        
        let content = request.form_data.get("content")
            .ok_or_else(|| PageControllerError::ValidationError("缺少文章内容".to_string()))?;
        
        let author_id_str = request.form_data.get("author_id")
            .ok_or_else(|| PageControllerError::ValidationError("缺少作者ID".to_string()))?;
        
        let author_id: u32 = author_id_str.parse()
            .map_err(|_| PageControllerError::ValidationError("无效的作者ID".to_string()))?;

        // 验证输入
        if title.trim().is_empty() {
            let content = self.render_new_article_form(Some("文章标题不能为空"));
            return Ok(HttpResponse::bad_request(content));
        }

        if content.trim().is_empty() {
            let content = self.render_new_article_form(Some("文章内容不能为空"));
            return Ok(HttpResponse::bad_request(content));
        }

        // 创建文章
        let article = self.data_service.create_article(
            title.clone(),
            content.clone(),
            author_id
        );

        println!("     ✅ 文章创建成功: {} (ID: {})", article.title, article.id);
        
        // 重定向到文章详情页
        Ok(HttpResponse::redirect(format!("/articles/{}", article.id)))
    }
}

/// 关于我们页面控制器
pub struct AboutPageController;

impl AboutPageController {
    pub fn new() -> Self {
        Self
    }

    fn render_about_page(&self) -> String {
        r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>关于我们</title>
            <style>
                body { font-family: Arial, sans-serif; margin: 40px; }
                .container { max-width: 600px; margin: 0 auto; }
                .back-btn { display: inline-block; margin-bottom: 20px; padding: 8px 16px; background: #666; color: white; text-decoration: none; border-radius: 4px; }
                .feature { margin: 20px 0; padding: 15px; background: #f0f8ff; border-left: 4px solid #2196f3; }
            </style>
        </head>
        <body>
            <div class="container">
                <a href="/" class="back-btn">← 返回首页</a>
                <h1>ℹ️ 关于我们</h1>
                <p>这是一个展示<strong>页面控制器模式</strong>的示例网站。</p>
                
                <div class="feature">
                    <h3>🎯 页面控制器模式特点</h3>
                    <ul>
                        <li>每个页面有独立的控制器类</li>
                        <li>控制器负责处理该页面的所有逻辑</li>
                        <li>简单直观，易于理解和维护</li>
                        <li>适合中小型Web应用</li>
                    </ul>
                </div>
                
                <div class="feature">
                    <h3>🏗️ 技术架构</h3>
                    <ul>
                        <li>Rust语言实现</li>
                        <li>内存数据存储</li>
                        <li>简单的HTML模板</li>
                        <li>RESTful URL设计</li>
                    </ul>
                </div>
                
                <div class="feature">
                    <h3>📱 功能展示</h3>
                    <ul>
                        <li>用户管理系统</li>
                        <li>文章发布系统</li>
                        <li>响应式页面设计</li>
                        <li>表单处理和验证</li>
                    </ul>
                </div>
                
                <p><em>本示例仅用于演示页面控制器模式的实现方法。</em></p>
            </div>
        </body>
        </html>
        "#.to_string()
    }
}

impl PageController for AboutPageController {
    fn handle_get(&self, _request: &HttpRequest) -> Result<HttpResponse, PageControllerError> {
        println!("   ℹ️ 处理关于我们GET请求");
        let content = self.render_about_page();
        Ok(HttpResponse::ok(content))
    }

    fn handle_post(&self, _request: &HttpRequest) -> Result<HttpResponse, PageControllerError> {
        Err(PageControllerError::ValidationError("关于我们页面不支持POST请求".to_string()))
    }
}

/// 页面控制器路由器
pub struct PageControllerRouter {
    data_service: Arc<DataService>,
}

impl PageControllerRouter {
    pub fn new() -> Self {
        Self {
            data_service: Arc::new(DataService::new()),
        }
    }

    /// 路由请求到相应的页面控制器
    pub fn route(&self, request: HttpRequest) -> HttpResponse {
        println!("🌐 路由请求: {} {}", request.method, request.path);

        let result = match (request.method.as_str(), request.path.as_str()) {
            ("GET", "/") => {
                let controller = HomePageController::new(self.data_service.clone());
                controller.handle_get(&request)
            },
            ("GET", "/users") => {
                let controller = UserListPageController::new(self.data_service.clone());
                controller.handle_get(&request)
            },
            ("GET", path) if path.starts_with("/users/") => {
                let controller = UserDetailPageController::new(self.data_service.clone());
                controller.handle_get(&request)
            },
            ("GET", "/articles") => {
                let controller = ArticleListPageController::new(self.data_service.clone());
                controller.handle_get(&request)
            },
            ("GET", "/articles/new") => {
                let controller = NewArticlePageController::new(self.data_service.clone());
                controller.handle_get(&request)
            },
            ("POST", "/articles/new") => {
                let controller = NewArticlePageController::new(self.data_service.clone());
                controller.handle_post(&request)
            },
            ("GET", "/about") => {
                let controller = AboutPageController::new();
                controller.handle_get(&request)
            },
            _ => Err(PageControllerError::NotFound("页面不存在".to_string())),
        };

        match result {
            Ok(response) => {
                println!("   ✅ 请求处理成功: {}", response.status);
                response
            },
            Err(e) => {
                println!("   ❌ 请求处理失败: {}", e);
                match e {
                    PageControllerError::NotFound(_) => HttpResponse::not_found(format!("404 - 页面不存在: {}", request.path)),
                    _ => HttpResponse::bad_request(e.to_string()),
                }
            }
        }
    }
}

/// 演示页面控制器模式
pub fn demo() {
    println!("=== 页面控制器模式演示 ===\n");

    let router = PageControllerRouter::new();

    println!("🌐 启动页面控制器Web应用");
    println!("每个页面都有独立的控制器类负责处理\n");

    // 测试各种页面请求
    let test_requests = vec![
        ("访问首页", HttpRequest::new("GET".to_string(), "/".to_string())),
        ("查看用户列表", HttpRequest::new("GET".to_string(), "/users".to_string())),
        ("查看用户详情", HttpRequest::new("GET".to_string(), "/users/1".to_string())),
        ("查看文章列表", HttpRequest::new("GET".to_string(), "/articles".to_string())),
        ("新建文章页面", HttpRequest::new("GET".to_string(), "/articles/new".to_string())),
        ("关于我们页面", HttpRequest::new("GET".to_string(), "/about".to_string())),
    ];

    for (description, request) in test_requests {
        println!("{}", "=".repeat(60));
        println!("📄 测试: {}", description);
        
        let response = router.route(request);
        println!("   响应状态: {}", response.status);
        println!("   响应体大小: {} 字节", response.body.len());
        
        if response.body.len() < 500 {
            println!("   响应内容: {}", response.body);
        }
    }

    println!("\n{}", "=".repeat(60));
    println!("📝 测试文章创建");
    
    let mut form_data = HashMap::new();
    form_data.insert("title".to_string(), "测试文章标题".to_string());
    form_data.insert("content".to_string(), "这是一篇测试文章的内容，用于演示页面控制器模式的POST请求处理。".to_string());
    form_data.insert("author_id".to_string(), "1".to_string());

    let create_request = HttpRequest::new("POST".to_string(), "/articles/new".to_string())
        .with_form_data(form_data);
    
    let response = router.route(create_request);
    println!("   创建文章响应状态: {}", response.status);
    if let Some(location) = response.headers.get("Location") {
        println!("   重定向到: {}", location);
    }

    println!("\n{}", "=".repeat(60));
    println!("❌ 测试错误处理");
    
    let error_requests = vec![
        ("无效路径", HttpRequest::new("GET".to_string(), "/invalid".to_string())),
        ("无效用户ID", HttpRequest::new("GET".to_string(), "/users/abc".to_string())),
        ("不存在的用户", HttpRequest::new("GET".to_string(), "/users/999".to_string())),
    ];

    for (description, request) in error_requests {
        println!("\n🚫 测试: {}", description);
        let response = router.route(request);
        println!("   错误响应状态: {}", response.status);
    }

    println!("\n=== 页面控制器模式演示完成 ===");

    println!("\n💡 页面控制器模式的优势:");
    println!("1. 简单直观 - 每个页面有独立的控制器");
    println!("2. 职责清晰 - 控制器只负责自己的页面");
    println!("3. 易于维护 - 页面逻辑集中在一个类中");
    println!("4. 便于测试 - 可以独立测试每个页面控制器");
    println!("5. 团队协作 - 不同开发者可以负责不同页面");

    println!("\n🏗️ 实现的页面控制器:");
    println!("• HomePageController - 首页控制器");
    println!("• UserListPageController - 用户列表控制器");
    println!("• UserDetailPageController - 用户详情控制器");
    println!("• ArticleListPageController - 文章列表控制器");
    println!("• NewArticlePageController - 新建文章控制器");
    println!("• AboutPageController - 关于我们控制器");

    println!("\n⚠️ 注意事项:");
    println!("1. 适合中小型应用 - 大型应用可能导致控制器过多");
    println!("2. 代码重复 - 不同控制器可能有相似逻辑");
    println!("3. 缺乏统一处理 - 横切关注点需要在每个控制器中处理");
    println!("4. 路由管理 - 需要维护URL到控制器的映射关系");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_service() {
        let service = DataService::new();
        
        let users = service.get_all_users();
        assert!(!users.is_empty());
        
        let user = service.get_user(1);
        assert!(user.is_some());
        
        let articles = service.get_all_articles();
        assert!(!articles.is_empty());
    }

    #[test]
    fn test_home_page_controller() {
        let data_service = Arc::new(DataService::new());
        let controller = HomePageController::new(data_service);
        let request = HttpRequest::new("GET".to_string(), "/".to_string());
        
        let result = controller.handle_get(&request);
        assert!(result.is_ok());
        
        let response = result.unwrap();
        assert_eq!(response.status, 200);
        assert!(response.body.contains("欢迎"));
    }

    #[test]
    fn test_page_controller_router() {
        let router = PageControllerRouter::new();
        
        // 测试首页
        let request = HttpRequest::new("GET".to_string(), "/".to_string());
        let response = router.route(request);
        assert_eq!(response.status, 200);
        
        // 测试404
        let request = HttpRequest::new("GET".to_string(), "/invalid".to_string());
        let response = router.route(request);
        assert_eq!(response.status, 404);
    }

    #[test]
    fn test_user_detail_controller() {
        let data_service = Arc::new(DataService::new());
        let controller = UserDetailPageController::new(data_service);
        
        // 测试有效用户
        let request = HttpRequest::new("GET".to_string(), "/users/1".to_string());
        let result = controller.handle_get(&request);
        assert!(result.is_ok());
        
        // 测试无效用户
        let request = HttpRequest::new("GET".to_string(), "/users/999".to_string());
        let result = controller.handle_get(&request);
        assert!(result.is_err());
    }
} 