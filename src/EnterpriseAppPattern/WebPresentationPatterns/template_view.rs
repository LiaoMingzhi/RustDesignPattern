//! 模板视图模式（Template View）
//! 
//! 模板视图模式将展示逻辑嵌入到模板中，模板定义了页面的结构和格式，
//! 而动态数据通过模板引擎注入到模板中。这种模式适合内容导向的网站，
//! 让设计师和开发者可以分离工作。
//! 
//! 文件位置：/d%3A/workspace/RustLearn/RustDesignPattern/src/EnterpriseAppPattern/WebPresentationPatterns/template_view.rs

use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

// =================
// 模板数据模型
// =================

/// 模板变量值
#[derive(Debug, Clone)]
pub enum TemplateValue {
    String(String),
    Number(f64),
    Boolean(bool),
    Array(Vec<TemplateValue>),
    Object(HashMap<String, TemplateValue>),
    Null,
}

impl TemplateValue {
    pub fn as_string(&self) -> String {
        match self {
            TemplateValue::String(s) => s.clone(),
            TemplateValue::Number(n) => {
                if n.fract() == 0.0 {
                    format!("{:.0}", n)
                } else {
                    format!("{}", n)
                }
            },
            TemplateValue::Boolean(b) => b.to_string(),
            TemplateValue::Array(_) => "[Array]".to_string(),
            TemplateValue::Object(_) => "[Object]".to_string(),
            TemplateValue::Null => "".to_string(),
        }
    }
    
    pub fn as_number(&self) -> Option<f64> {
        match self {
            TemplateValue::Number(n) => Some(*n),
            TemplateValue::String(s) => s.parse().ok(),
            TemplateValue::Boolean(b) => Some(if *b { 1.0 } else { 0.0 }),
            _ => None,
        }
    }
    
    pub fn as_boolean(&self) -> bool {
        match self {
            TemplateValue::Boolean(b) => *b,
            TemplateValue::String(s) => !s.is_empty(),
            TemplateValue::Number(n) => *n != 0.0,
            TemplateValue::Array(arr) => !arr.is_empty(),
            TemplateValue::Object(obj) => !obj.is_empty(),
            TemplateValue::Null => false,
        }
    }
    
    pub fn get_property(&self, key: &str) -> Option<&TemplateValue> {
        match self {
            TemplateValue::Object(obj) => obj.get(key),
            _ => None,
        }
    }
    
    pub fn get_index(&self, index: usize) -> Option<&TemplateValue> {
        match self {
            TemplateValue::Array(arr) => arr.get(index),
            _ => None,
        }
    }
}

impl From<String> for TemplateValue {
    fn from(s: String) -> Self {
        TemplateValue::String(s)
    }
}

impl From<&str> for TemplateValue {
    fn from(s: &str) -> Self {
        TemplateValue::String(s.to_string())
    }
}

impl From<f64> for TemplateValue {
    fn from(n: f64) -> Self {
        TemplateValue::Number(n)
    }
}

impl From<i32> for TemplateValue {
    fn from(n: i32) -> Self {
        TemplateValue::Number(n as f64)
    }
}

impl From<bool> for TemplateValue {
    fn from(b: bool) -> Self {
        TemplateValue::Boolean(b)
    }
}

impl fmt::Display for TemplateValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_string())
    }
}

/// 模板上下文 - 包含所有模板变量
#[derive(Debug, Clone)]
pub struct TemplateContext {
    variables: HashMap<String, TemplateValue>,
}

impl TemplateContext {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }
    
    pub fn set<T: Into<TemplateValue>>(&mut self, key: &str, value: T) {
        self.variables.insert(key.to_string(), value.into());
    }
    
    pub fn get(&self, key: &str) -> Option<&TemplateValue> {
        // 支持点号路径，如 "user.name"
        let parts: Vec<&str> = key.split('.').collect();
        let mut current = self.variables.get(parts[0])?;
        
        for part in &parts[1..] {
            current = current.get_property(part)?;
        }
        
        Some(current)
    }
    
    pub fn merge(&mut self, other: TemplateContext) {
        for (key, value) in other.variables {
            self.variables.insert(key, value);
        }
    }
    
    pub fn from_map(map: HashMap<String, TemplateValue>) -> Self {
        Self { variables: map }
    }
}

// =================
// 模板节点
// =================

/// 模板节点类型
#[derive(Debug, Clone)]
pub enum TemplateNode {
    Text(String),
    Variable(String),
    If {
        condition: String,
        then_nodes: Vec<TemplateNode>,
        else_nodes: Vec<TemplateNode>,
    },
    For {
        variable: String,
        iterable: String,
        body: Vec<TemplateNode>,
    },
    Include(String),
}

/// 模板解析器
pub struct TemplateParser;

impl TemplateParser {
    pub fn parse(template: &str) -> Result<Vec<TemplateNode>, TemplateError> {
        let mut nodes = Vec::new();
        let mut chars = template.chars().peekable();
        let mut text_buffer = String::new();
        
        while let Some(ch) = chars.next() {
            if ch == '{' && chars.peek() == Some(&'{') {
                chars.next(); // 消费第二个 '{'
                
                // 如果有文本缓冲区，添加为文本节点
                if !text_buffer.is_empty() {
                    nodes.push(TemplateNode::Text(text_buffer.clone()));
                    text_buffer.clear();
                }
                
                // 解析模板标签
                let mut tag_content = String::new();
                let mut depth = 1;
                
                while let Some(ch) = chars.next() {
                    if ch == '{' && chars.peek() == Some(&'{') {
                        depth += 1;
                        tag_content.push(ch);
                    } else if ch == '}' && chars.peek() == Some(&'}') {
                        depth -= 1;
                        if depth == 0 {
                            chars.next(); // 消费第二个 '}'
                            break;
                        }
                        tag_content.push(ch);
                    } else {
                        tag_content.push(ch);
                    }
                }
                
                // 解析标签内容
                let node = Self::parse_tag(&tag_content.trim())?;
                nodes.push(node);
            } else {
                text_buffer.push(ch);
            }
        }
        
        // 处理剩余的文本
        if !text_buffer.is_empty() {
            nodes.push(TemplateNode::Text(text_buffer));
        }
        
        Ok(nodes)
    }
    
    fn parse_tag(content: &str) -> Result<TemplateNode, TemplateError> {
        let content = content.trim();
        
        if content.starts_with("if ") {
            let condition = content[3..].trim().to_string();
            Ok(TemplateNode::If {
                condition,
                then_nodes: Vec::new(), // 简化实现
                else_nodes: Vec::new(),
            })
        } else if content.starts_with("for ") {
            // 解析 "for item in items" 格式
            let parts: Vec<&str> = content.split_whitespace().collect();
            if parts.len() >= 4 && parts[2] == "in" {
                Ok(TemplateNode::For {
                    variable: parts[1].to_string(),
                    iterable: parts[3].to_string(),
                    body: Vec::new(), // 简化实现
                })
            } else {
                Err(TemplateError::ParseError(format!("无效的for语法: {}", content)))
            }
        } else if content.starts_with("include ") {
            let template_name = content[8..].trim().to_string();
            Ok(TemplateNode::Include(template_name))
        } else {
            // 变量替换
            Ok(TemplateNode::Variable(content.to_string()))
        }
    }
}

// =================
// 模板引擎
// =================

/// 模板引擎
pub struct TemplateEngine {
    templates: HashMap<String, Vec<TemplateNode>>,
    base_path: String,
}

impl TemplateEngine {
    pub fn new() -> Self {
        Self {
            templates: HashMap::new(),
            base_path: "templates/".to_string(),
        }
    }
    
    pub fn with_base_path(mut self, path: String) -> Self {
        self.base_path = path;
        self
    }
    
    /// 注册模板
    pub fn register_template(&mut self, name: &str, template_content: &str) -> Result<(), TemplateError> {
        let nodes = TemplateParser::parse(template_content)?;
        self.templates.insert(name.to_string(), nodes);
        Ok(())
    }
    
    /// 渲染模板
    pub fn render(&self, template_name: &str, context: &TemplateContext) -> Result<String, TemplateError> {
        let nodes = self.templates.get(template_name)
            .ok_or_else(|| TemplateError::TemplateNotFound(template_name.to_string()))?;
        
        self.render_nodes(nodes, context)
    }
    
    fn render_nodes(&self, nodes: &[TemplateNode], context: &TemplateContext) -> Result<String, TemplateError> {
        let mut result = String::new();
        
        for node in nodes {
            match node {
                TemplateNode::Text(text) => {
                    result.push_str(text);
                },
                TemplateNode::Variable(var_name) => {
                    if let Some(value) = context.get(var_name) {
                        result.push_str(&self.format_value(value));
                    }
                },
                TemplateNode::If { condition, then_nodes, else_nodes: _ } => {
                    if self.evaluate_condition(condition, context)? {
                        result.push_str(&self.render_nodes(then_nodes, context)?);
                    }
                },
                TemplateNode::For { variable, iterable, body } => {
                    if let Some(array_value) = context.get(iterable) {
                        if let TemplateValue::Array(items) = array_value {
                            for item in items {
                                let mut item_context = context.clone();
                                item_context.set(variable, item.clone());
                                result.push_str(&self.render_nodes(body, &item_context)?);
                            }
                        }
                    }
                },
                TemplateNode::Include(template_name) => {
                    result.push_str(&self.render(template_name, context)?);
                },
            }
        }
        
        Ok(result)
    }
    
    fn format_value(&self, value: &TemplateValue) -> String {
        match value {
            TemplateValue::String(s) => self.escape_html(s),
            _ => self.escape_html(&value.as_string()),
        }
    }
    
    fn escape_html(&self, text: &str) -> String {
        text.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#39;")
    }
    
    fn evaluate_condition(&self, condition: &str, context: &TemplateContext) -> Result<bool, TemplateError> {
        // 简化的条件评估
        if let Some(value) = context.get(condition) {
            Ok(value.as_boolean())
        } else {
            Ok(false)
        }
    }
}

// =================
// 模板错误
// =================

#[derive(Debug)]
pub enum TemplateError {
    ParseError(String),
    TemplateNotFound(String),
    RenderError(String),
    VariableNotFound(String),
}

impl fmt::Display for TemplateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TemplateError::ParseError(msg) => write!(f, "模板解析错误: {}", msg),
            TemplateError::TemplateNotFound(name) => write!(f, "模板未找到: {}", name),
            TemplateError::RenderError(msg) => write!(f, "模板渲染错误: {}", msg),
            TemplateError::VariableNotFound(var) => write!(f, "变量未找到: {}", var),
        }
    }
}

// =================
// 模板助手函数
// =================

/// 模板助手函数
pub struct TemplateHelpers;

impl TemplateHelpers {
    /// 格式化日期
    pub fn format_date(date: &str, format: &str) -> String {
        // 简化实现
        match format {
            "yyyy-MM-dd" => date.to_string(),
            "MM/dd/yyyy" => {
                if date.len() >= 10 {
                    let year = &date[0..4];
                    let month = &date[5..7];
                    let day = &date[8..10];
                    format!("{}/{}/{}", month, day, year)
                } else {
                    date.to_string()
                }
            },
            _ => date.to_string(),
        }
    }
    
    /// 截断文本
    pub fn truncate(text: &str, length: usize) -> String {
        if text.len() <= length {
            text.to_string()
        } else {
            format!("{}...", &text[..length])
        }
    }
    
    /// 格式化货币
    pub fn format_currency(amount: f64, currency: &str) -> String {
        match currency {
            "USD" => format!("${:.2}", amount),
            "CNY" => format!("¥{:.2}", amount),
            "EUR" => format!("€{:.2}", amount),
            _ => format!("{:.2} {}", amount, currency),
        }
    }
    
    /// 复数形式
    pub fn pluralize(count: i32, singular: &str, plural: &str) -> String {
        if count == 1 {
            format!("{} {}", count, singular)
        } else {
            format!("{} {}", count, plural)
        }
    }
}

// =================
// 模板视图
// =================

/// 模板视图 - 封装模板引擎的视图类
pub struct TemplateView {
    engine: TemplateEngine,
    global_context: TemplateContext,
}

impl TemplateView {
    pub fn new() -> Self {
        Self {
            engine: TemplateEngine::new(),
            global_context: TemplateContext::new(),
        }
    }
    
    /// 设置全局变量
    pub fn set_global<T: Into<TemplateValue>>(&mut self, key: &str, value: T) {
        self.global_context.set(key, value);
    }
    
    /// 注册模板
    pub fn register_template(&mut self, name: &str, content: &str) -> Result<(), TemplateError> {
        self.engine.register_template(name, content)
    }
    
    /// 渲染视图
    pub fn render(&self, template_name: &str, mut context: TemplateContext) -> Result<String, TemplateError> {
        // 合并全局上下文
        context.merge(self.global_context.clone());
        self.engine.render(template_name, &context)
    }
}

// =================
// 网站示例
// =================

/// 博客网站示例
pub struct BlogSite {
    template_view: TemplateView,
}

impl BlogSite {
    pub fn new() -> Self {
        let mut template_view = TemplateView::new();
        
        // 设置全局变量
        template_view.set_global("site_name", "我的博客");
        template_view.set_global("site_url", "https://myblog.com");
        
        // 注册模板
        Self::register_templates(&mut template_view);
        
        Self { template_view }
    }
    
    fn register_templates(view: &mut TemplateView) {
        // 布局模板
        let layout_template = r#"
<!DOCTYPE html>
<html>
<head>
    <title>{{title}} - {{site_name}}</title>
    <meta charset="utf-8">
    <style>
        body { font-family: Arial, sans-serif; margin: 0; padding: 20px; }
        .header { background: #333; color: white; padding: 10px; margin-bottom: 20px; }
        .content { max-width: 800px; margin: 0 auto; }
        .footer { margin-top: 40px; padding-top: 20px; border-top: 1px solid #ccc; color: #666; }
        .post { margin-bottom: 30px; padding-bottom: 20px; border-bottom: 1px solid #eee; }
        .post-title { color: #333; margin-bottom: 10px; }
        .post-meta { color: #666; font-size: 14px; margin-bottom: 15px; }
        .post-content { line-height: 1.6; }
    </style>
</head>
<body>
    <div class="header">
        <h1>{{site_name}}</h1>
        <nav>
            <a href="/" style="color: white; margin-right: 15px;">首页</a>
            <a href="/about" style="color: white;">关于</a>
        </nav>
    </div>
    
    <div class="content">
        {{content}}
    </div>
    
    <div class="footer">
        <p>&copy; 2024 {{site_name}}. All rights reserved.</p>
    </div>
</body>
</html>"#;
        
        // 首页模板
        let home_template = r#"
<h2>最新文章</h2>
{{posts}}"#;
        
        // 文章列表模板
        let post_list_template = r#"
<div class="post">
    <h3 class="post-title">{{title}}</h3>
    <div class="post-meta">
        发布于 {{date}} | 作者: {{author}} | {{comment_count}} 条评论
    </div>
    <div class="post-content">
        {{excerpt}}
        <p><a href="/post/{{id}}">阅读全文 →</a></p>
    </div>
</div>"#;
        
        // 文章详情模板
        let post_detail_template = r#"
<article class="post">
    <h2 class="post-title">{{title}}</h2>
    <div class="post-meta">
        发布于 {{date}} | 作者: {{author}} | 分类: {{category}}
    </div>
    <div class="post-content">
        {{content}}
    </div>
    
    <div class="post-tags">
        <strong>标签:</strong>
        {{tags}}
    </div>
</article>

<div class="comments">
    <h3>评论 ({{comment_count}})</h3>
    {{comments}}
</div>"#;
        
        view.register_template("layout", layout_template).unwrap();
        view.register_template("home", home_template).unwrap();
        view.register_template("post_list", post_list_template).unwrap();
        view.register_template("post_detail", post_detail_template).unwrap();
    }
    
    /// 渲染首页
    pub fn render_home(&self) -> Result<String, TemplateError> {
        let mut context = TemplateContext::new();
        context.set("title", "首页");
        
        // 模拟文章列表
        let posts_html = self.render_post_list()?;
        context.set("posts", posts_html);
        
        let content = self.template_view.render("home", context.clone())?;
        context.set("content", content);
        
        self.template_view.render("layout", context)
    }
    
    fn render_post_list(&self) -> Result<String, TemplateError> {
        let mut result = String::new();
        
        let posts = vec![
            ("1", "Rust设计模式探索", "探索Rust语言中的设计模式实践...", "2024-01-15", "张三", 5),
            ("2", "企业应用架构模式", "深入理解企业级应用的架构设计...", "2024-01-10", "李四", 12),
            ("3", "模板视图模式详解", "学习Web开发中的模板视图模式...", "2024-01-05", "王五", 8),
        ];
        
        for (id, title, excerpt, date, author, comment_count) in posts {
            let mut context = TemplateContext::new();
            context.set("id", id.to_string());
            context.set("title", title);
            context.set("excerpt", excerpt);
            context.set("date", TemplateHelpers::format_date(date, "yyyy年MM月dd日"));
            context.set("author", author);
            context.set("comment_count", TemplateHelpers::pluralize(comment_count, "条评论", "条评论"));
            
            result.push_str(&self.template_view.render("post_list", context)?);
        }
        
        Ok(result)
    }
    
    /// 渲染文章详情
    pub fn render_post_detail(&self, post_id: &str) -> Result<String, TemplateError> {
        let mut context = TemplateContext::new();
        
        // 模拟从数据库获取文章
        let (title, content, date, author, category) = match post_id {
            "1" => (
                "Rust设计模式探索",
                "<p>Rust语言以其独特的所有权系统和类型安全特性，为设计模式的实现提供了新的视角...</p><p>本文将深入探讨几种常见的设计模式在Rust中的实现方式。</p>",
                "2024-01-15",
                "张三",
                "编程技术"
            ),
            "2" => (
                "企业应用架构模式",
                "<p>企业级应用的架构设计是一个复杂而重要的话题...</p><p>我们将从分层架构开始，逐步介绍各种企业应用架构模式。</p>",
                "2024-01-10",
                "李四",
                "架构设计"
            ),
            _ => return Err(TemplateError::TemplateNotFound(format!("文章不存在: {}", post_id))),
        };
        
        context.set("title", title);
        context.set("content", content);
        context.set("date", TemplateHelpers::format_date(date, "yyyy年MM月dd日"));
        context.set("author", author);
        context.set("category", category);
        context.set("tags", "Rust, 设计模式, 编程");
        context.set("comment_count", 3);
        
        // 模拟评论
        context.set("comments", "<div>评论功能暂未实现...</div>");
        
        let content_html = self.template_view.render("post_detail", context.clone())?;
        context.set("content", content_html);
        
        self.template_view.render("layout", context)
    }
}

/// 模板视图模式演示
pub fn demo_template_view_pattern() {
    println!("=== 模板视图模式演示 ===\n");
    
    println!("1. 创建博客网站:");
    
    let blog = BlogSite::new();
    
    println!("2. 渲染首页:");
    
    match blog.render_home() {
        Ok(html) => {
            println!("首页HTML生成成功 ({} 字符)", html.len());
            
            // 显示HTML片段
            let preview = TemplateHelpers::truncate(&html, 200);
            println!("HTML预览:\n{}\n", preview);
        },
        Err(e) => println!("渲染失败: {}", e),
    }
    
    println!("3. 渲染文章详情页:");
    
    match blog.render_post_detail("1") {
        Ok(html) => {
            println!("文章详情页生成成功 ({} 字符)", html.len());
            
            // 显示HTML片段
            let preview = TemplateHelpers::truncate(&html, 200);
            println!("HTML预览:\n{}\n", preview);
        },
        Err(e) => println!("渲染失败: {}", e),
    }
    
    println!("4. 测试模板助手函数:");
    
    println!("  日期格式化:");
    println!("    {} -> {}", "2024-01-15", TemplateHelpers::format_date("2024-01-15", "MM/dd/yyyy"));
    
    println!("  货币格式化:");
    println!("    {} -> {}", 99.99, TemplateHelpers::format_currency(99.99, "USD"));
    println!("    {} -> {}", 999.50, TemplateHelpers::format_currency(999.50, "CNY"));
    
    println!("  文本截断:");
    let long_text = "这是一段很长的文本，用来测试截断功能的效果";
    println!("    '{}' -> '{}'", long_text, TemplateHelpers::truncate(long_text, 15));
    
    println!("  复数形式:");
    println!("    {} -> {}", 1, TemplateHelpers::pluralize(1, "item", "items"));
    println!("    {} -> {}", 5, TemplateHelpers::pluralize(5, "item", "items"));
    
    println!("\n5. 模板引擎特性演示:");
    
    let mut engine = TemplateEngine::new();
    
    // 注册一个简单的模板
    let simple_template = r#"
Hello {{name}}!
{{greeting}}
Your age is {{age}}.
"#;
    
    engine.register_template("simple", simple_template).unwrap();
    
    let mut context = TemplateContext::new();
    context.set("name", "张三");
    context.set("greeting", "欢迎来到模板世界!");
    context.set("age", 25);
    
    match engine.render("simple", &context) {
        Ok(result) => {
            println!("简单模板渲染结果:");
            println!("{}", result);
        },
        Err(e) => println!("渲染失败: {}", e),
    }
    
    println!("=== 模板视图模式特点 ===");
    println!("✓ 分离关注点 - 视图逻辑与业务逻辑分离");
    println!("✓ 设计师友好 - 设计师可以独立修改模板");
    println!("✓ 模板重用 - 布局和组件可以重复使用");
    println!("✓ 动态内容 - 支持变量替换和条件渲染");
    println!("✓ 安全性 - 自动HTML转义防止XSS攻击");
    println!("✓ 扩展性 - 支持助手函数和自定义标签");
    println!("✓ 缓存优化 - 模板编译后可以缓存提高性能");
}

/// 模板视图模式演示（包装函数）
pub fn demo() {
    demo_template_view_pattern();
}