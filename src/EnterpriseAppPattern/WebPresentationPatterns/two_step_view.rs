//! 两步视图模式（Two Step View）
//! 
//! 两步视图模式将页面的生成分为两个阶段：
//! 1. 第一步：将领域数据转换为逻辑页面
//! 2. 第二步：将逻辑页面转换为具体的格式（HTML、JSON、XML等）
//! 
//! 这种模式特别适合需要支持多种客户端或多种输出格式的应用。
//! 
//! 文件位置：/d%3A/workspace/RustLearn/RustDesignPattern/src/EnterpriseAppPattern/WebPresentationPatterns/two_step_view.rs

use std::collections::HashMap;
use std::fmt;

// =================
// 第一步：逻辑页面结构
// =================

/// 逻辑页面元素类型
#[derive(Debug, Clone)]
pub enum LogicalElement {
    /// 文本内容
    Text {
        content: String,
        style: HashMap<String, String>,
    },
    /// 标题
    Heading {
        level: u8,
        content: String,
        id: Option<String>,
    },
    /// 列表
    List {
        items: Vec<LogicalElement>,
        ordered: bool,
    },
    /// 表格
    Table {
        headers: Vec<String>,
        rows: Vec<Vec<LogicalElement>>,
        caption: Option<String>,
    },
    /// 链接
    Link {
        url: String,
        text: String,
        external: bool,
    },
    /// 图像
    Image {
        src: String,
        alt: String,
        width: Option<u32>,
        height: Option<u32>,
    },
    /// 容器
    Container {
        children: Vec<LogicalElement>,
        layout: ContainerLayout,
        css_class: Option<String>,
    },
    /// 表单
    Form {
        fields: Vec<FormField>,
        action: String,
        method: HttpMethod,
    },
    /// 导航
    Navigation {
        items: Vec<NavigationItem>,
        current_path: Option<String>,
    },
}

/// 容器布局类型
#[derive(Debug, Clone)]
pub enum ContainerLayout {
    Vertical,
    Horizontal,
    Grid { columns: u32 },
    Flex,
}

/// 表单字段
#[derive(Debug, Clone)]
pub struct FormField {
    pub name: String,
    pub label: String,
    pub field_type: FieldType,
    pub required: bool,
    pub value: Option<String>,
    pub placeholder: Option<String>,
}

/// 表单字段类型
#[derive(Debug, Clone)]
pub enum FieldType {
    Text,
    Email,
    Password,
    Number,
    Select { options: Vec<(String, String)> },
    TextArea,
    Checkbox,
    Radio { options: Vec<(String, String)> },
    Hidden,
}

/// HTTP方法
#[derive(Debug, Clone)]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    DELETE,
}

/// 导航项
#[derive(Debug, Clone)]
pub struct NavigationItem {
    pub path: String,
    pub title: String,
    pub active: bool,
    pub children: Vec<NavigationItem>,
}

/// 逻辑页面结构
#[derive(Debug, Clone)]
pub struct LogicalPage {
    pub title: String,
    pub description: Option<String>,
    pub keywords: Vec<String>,
    pub elements: Vec<LogicalElement>,
    pub metadata: HashMap<String, String>,
}

impl LogicalPage {
    pub fn new(title: String) -> Self {
        Self {
            title,
            description: None,
            keywords: Vec::new(),
            elements: Vec::new(),
            metadata: HashMap::new(),
        }
    }
    
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }
    
    pub fn with_keywords(mut self, keywords: Vec<String>) -> Self {
        self.keywords = keywords;
        self
    }
    
    pub fn add_element(mut self, element: LogicalElement) -> Self {
        self.elements.push(element);
        self
    }
    
    pub fn add_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

// =================
// 第一步：数据到逻辑页面转换器
// =================

/// 页面构建器 - 负责第一步转换
pub trait PageBuilder {
    type Data;
    
    fn build_page(&self, data: &Self::Data) -> Result<LogicalPage, BuildError>;
}

/// 构建错误
#[derive(Debug)]
pub enum BuildError {
    InvalidData(String),
    ConversionError(String),
    ValidationError(String),
}

impl fmt::Display for BuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BuildError::InvalidData(msg) => write!(f, "Invalid data: {}", msg),
            BuildError::ConversionError(msg) => write!(f, "Conversion error: {}", msg),
            BuildError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl std::error::Error for BuildError {}

// =================
// 第二步：格式渲染器
// =================

/// 格式渲染器 - 负责第二步转换
pub trait FormatRenderer {
    fn render(&self, page: &LogicalPage) -> Result<String, RenderError>;
    fn content_type(&self) -> &str;
}

/// 渲染错误
#[derive(Debug)]
pub enum RenderError {
    UnsupportedElement(String),
    RenderingFailed(String),
    InvalidStructure(String),
}

impl fmt::Display for RenderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RenderError::UnsupportedElement(elem) => write!(f, "Unsupported element: {}", elem),
            RenderError::RenderingFailed(msg) => write!(f, "Rendering failed: {}", msg),
            RenderError::InvalidStructure(msg) => write!(f, "Invalid structure: {}", msg),
        }
    }
}

impl std::error::Error for RenderError {}

// =================
// HTML 渲染器
// =================

/// HTML格式渲染器
pub struct HtmlRenderer {
    pretty_print: bool,
    include_meta: bool,
}

impl HtmlRenderer {
    pub fn new() -> Self {
        Self {
            pretty_print: true,
            include_meta: true,
        }
    }
    
    pub fn with_pretty_print(mut self, pretty: bool) -> Self {
        self.pretty_print = pretty;
        self
    }
    
    pub fn with_meta(mut self, include: bool) -> Self {
        self.include_meta = include;
        self
    }
    
    fn render_element(&self, element: &LogicalElement, depth: usize) -> Result<String, RenderError> {
        let indent = if self.pretty_print { "  ".repeat(depth) } else { String::new() };
        let newline = if self.pretty_print { "\n" } else { "" };
        
        match element {
            LogicalElement::Text { content, style } => {
                let style_attr = if !style.is_empty() {
                    let style_str: Vec<String> = style.iter()
                        .map(|(k, v)| format!("{}:{}", k, v))
                        .collect();
                    format!(" style=\"{}\"", style_str.join(";"))
                } else {
                    String::new()
                };
                Ok(format!("{}<span{}>{}</span>", indent, style_attr, self.escape_html(content)))
            },
            
            LogicalElement::Heading { level, content, id } => {
                let id_attr = id.as_ref()
                    .map(|i| format!(" id=\"{}\"", i))
                    .unwrap_or_default();
                Ok(format!("{}<h{}{}>{}</h{}>", indent, level, id_attr, self.escape_html(content), level))
            },
            
            LogicalElement::List { items, ordered } => {
                let tag = if *ordered { "ol" } else { "ul" };
                let mut result = format!("{}<{}>{}", indent, tag, newline);
                
                for item in items {
                    result.push_str(&format!("{}  <li>", indent));
                    result.push_str(&self.render_element(item, depth + 2)?);
                    result.push_str(&format!("</li>{}", newline));
                }
                
                result.push_str(&format!("{}</{}>", indent, tag));
                Ok(result)
            },
            
            LogicalElement::Table { headers, rows, caption } => {
                let mut result = format!("{}<table>{}", indent, newline);
                
                if let Some(cap) = caption {
                    result.push_str(&format!("{}  <caption>{}</caption>{}", indent, self.escape_html(cap), newline));
                }
                
                // 表头
                if !headers.is_empty() {
                    result.push_str(&format!("{}  <thead>{}", indent, newline));
                    result.push_str(&format!("{}    <tr>{}", indent, newline));
                    for header in headers {
                        result.push_str(&format!("{}      <th>{}</th>{}", indent, self.escape_html(header), newline));
                    }
                    result.push_str(&format!("{}    </tr>{}", indent, newline));
                    result.push_str(&format!("{}  </thead>{}", indent, newline));
                }
                
                // 表体
                if !rows.is_empty() {
                    result.push_str(&format!("{}  <tbody>{}", indent, newline));
                    for row in rows {
                        result.push_str(&format!("{}    <tr>{}", indent, newline));
                        for cell in row {
                            result.push_str(&format!("{}      <td>", indent));
                            result.push_str(&self.render_element(cell, depth + 3)?);
                            result.push_str(&format!("</td>{}", newline));
                        }
                        result.push_str(&format!("{}    </tr>{}", indent, newline));
                    }
                    result.push_str(&format!("{}  </tbody>{}", indent, newline));
                }
                
                result.push_str(&format!("{}</table>", indent));
                Ok(result)
            },
            
            LogicalElement::Link { url, text, external } => {
                let target = if *external { " target=\"_blank\"" } else { "" };
                Ok(format!("{}<a href=\"{}\"{}>{}</a>", indent, url, target, self.escape_html(text)))
            },
            
            LogicalElement::Image { src, alt, width, height } => {
                let mut attrs = format!(" src=\"{}\" alt=\"{}\"", src, alt);
                if let Some(w) = width {
                    attrs.push_str(&format!(" width=\"{}\"", w));
                }
                if let Some(h) = height {
                    attrs.push_str(&format!(" height=\"{}\"", h));
                }
                Ok(format!("{}<img{} />", indent, attrs))
            },
            
            LogicalElement::Container { children, layout: _, css_class } => {
                let class_attr = css_class.as_ref()
                    .map(|c| format!(" class=\"{}\"", c))
                    .unwrap_or_default();
                
                let mut result = format!("{}<div{}>{}", indent, class_attr, newline);
                for child in children {
                    result.push_str(&self.render_element(child, depth + 1)?);
                    result.push_str(newline);
                }
                result.push_str(&format!("{}</div>", indent));
                Ok(result)
            },
            
            LogicalElement::Form { fields, action, method } => {
                let method_str = match method {
                    HttpMethod::GET => "get",
                    HttpMethod::POST => "post",
                    HttpMethod::PUT => "post", // HTML表单通常使用POST + hidden字段
                    HttpMethod::DELETE => "post",
                };
                
                let mut result = format!("{}<form action=\"{}\" method=\"{}\">{}", indent, action, method_str, newline);
                
                for field in fields {
                    result.push_str(&self.render_form_field(field, depth + 1)?);
                    result.push_str(newline);
                }
                
                result.push_str(&format!("{}</form>", indent));
                Ok(result)
            },
            
            LogicalElement::Navigation { items, current_path } => {
                let mut result = format!("{}<nav>{}", indent, newline);
                result.push_str(&format!("{}  <ul>{}", indent, newline));
                
                for item in items {
                    let active_class = if Some(&item.path) == current_path.as_ref() {
                        " class=\"active\""
                    } else {
                        ""
                    };
                    
                    result.push_str(&format!("{}    <li{}><a href=\"{}\">{}</a></li>{}", 
                        indent, active_class, item.path, self.escape_html(&item.title), newline));
                }
                
                result.push_str(&format!("{}  </ul>{}", indent, newline));
                result.push_str(&format!("{}</nav>", indent));
                Ok(result)
            },
        }
    }
    
    fn render_form_field(&self, field: &FormField, depth: usize) -> Result<String, RenderError> {
        let indent = if self.pretty_print { "  ".repeat(depth) } else { String::new() };
        let newline = if self.pretty_print { "\n" } else { "" };
        
        let required_attr = if field.required { " required" } else { "" };
        let value_attr = field.value.as_ref()
            .map(|v| format!(" value=\"{}\"", self.escape_html(v)))
            .unwrap_or_default();
        let placeholder_attr = field.placeholder.as_ref()
            .map(|p| format!(" placeholder=\"{}\"", self.escape_html(p)))
            .unwrap_or_default();
        
        let mut result = format!("{}<div class=\"field\">{}", indent, newline);
        result.push_str(&format!("{}  <label for=\"{}\">{}</label>{}", 
            indent, field.name, self.escape_html(&field.label), newline));
        
        match &field.field_type {
            FieldType::Text => {
                result.push_str(&format!("{}  <input type=\"text\" id=\"{}\" name=\"{}\"{}{}{} />", 
                    indent, field.name, field.name, value_attr, placeholder_attr, required_attr));
            },
            FieldType::Email => {
                result.push_str(&format!("{}  <input type=\"email\" id=\"{}\" name=\"{}\"{}{}{} />", 
                    indent, field.name, field.name, value_attr, placeholder_attr, required_attr));
            },
            FieldType::Password => {
                result.push_str(&format!("{}  <input type=\"password\" id=\"{}\" name=\"{}\"{}{} />", 
                    indent, field.name, field.name, placeholder_attr, required_attr));
            },
            FieldType::Number => {
                result.push_str(&format!("{}  <input type=\"number\" id=\"{}\" name=\"{}\"{}{}{} />", 
                    indent, field.name, field.name, value_attr, placeholder_attr, required_attr));
            },
            FieldType::TextArea => {
                let content = field.value.as_deref().unwrap_or("");
                result.push_str(&format!("{}  <textarea id=\"{}\" name=\"{}\"{}{}>{}</textarea>", 
                    indent, field.name, field.name, placeholder_attr, required_attr, self.escape_html(content)));
            },
            FieldType::Select { options } => {
                result.push_str(&format!("{}  <select id=\"{}\" name=\"{}\"{}>{}", 
                    indent, field.name, field.name, required_attr, newline));
                for (value, text) in options {
                    let selected = if field.value.as_ref() == Some(value) { " selected" } else { "" };
                    result.push_str(&format!("{}    <option value=\"{}\"{}>{}</option>{}", 
                        indent, value, selected, self.escape_html(text), newline));
                }
                result.push_str(&format!("{}  </select>", indent));
            },
            FieldType::Checkbox => {
                let checked = field.value.as_deref() == Some("true");
                let checked_attr = if checked { " checked" } else { "" };
                result.push_str(&format!("{}  <input type=\"checkbox\" id=\"{}\" name=\"{}\" value=\"true\"{}{} />", 
                    indent, field.name, field.name, checked_attr, required_attr));
            },
            FieldType::Radio { options } => {
                for (value, text) in options {
                    let checked = field.value.as_ref() == Some(value);
                    let checked_attr = if checked { " checked" } else { "" };
                    result.push_str(&format!("{}  <input type=\"radio\" id=\"{}_{}_{}\" name=\"{}\" value=\"{}\"{}{} />", 
                        indent, field.name, value, field.name, field.name, value, checked_attr, required_attr));
                    result.push_str(&format!("  <label for=\"{}_{}_{}\">{}</label>{}", 
                        field.name, value, field.name, self.escape_html(text), newline));
                }
            },
            FieldType::Hidden => {
                result.push_str(&format!("{}  <input type=\"hidden\" name=\"{}\" value=\"{}\" />", 
                    indent, field.name, field.value.as_deref().unwrap_or("")));
            },
        }
        
        result.push_str(&format!("{}</div> {}", newline, indent));
        Ok(result)
    }
    
    fn escape_html(&self, text: &str) -> String {
        text.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#39;")
    }
}

impl FormatRenderer for HtmlRenderer {
    fn render(&self, page: &LogicalPage) -> Result<String, RenderError> {
        let mut result = String::new();
        let newline = if self.pretty_print { "\n" } else { "" };
        
        result.push_str("<!DOCTYPE html>");
        result.push_str(newline);
        result.push_str("<html>");
        result.push_str(newline);
        result.push_str("<head>");
        result.push_str(newline);
        
        if self.include_meta {
            result.push_str(&format!("  <meta charset=\"utf-8\">{}", newline));
            result.push_str(&format!("  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">{}", newline));
        }
        
        result.push_str(&format!("  <title>{}</title>{}", self.escape_html(&page.title), newline));
        
        if let Some(desc) = &page.description {
            result.push_str(&format!("  <meta name=\"description\" content=\"{}\">{}", self.escape_html(desc), newline));
        }
        
        if !page.keywords.is_empty() {
            let keywords = page.keywords.join(", ");
            result.push_str(&format!("  <meta name=\"keywords\" content=\"{}\">{}", self.escape_html(&keywords), newline));
        }
        
        result.push_str("</head>");
        result.push_str(newline);
        result.push_str("<body>");
        result.push_str(newline);
        
        for element in &page.elements {
            result.push_str(&self.render_element(element, 1)?);
            result.push_str(newline);
        }
        
        result.push_str("</body>");
        result.push_str(newline);
        result.push_str("</html>");
        
        Ok(result)
    }
    
    fn content_type(&self) -> &str {
        "text/html"
    }
}

// =================
// JSON 渲染器
// =================

/// JSON格式渲染器
pub struct JsonRenderer {
    pretty_print: bool,
}

impl JsonRenderer {
    pub fn new() -> Self {
        Self {
            pretty_print: true,
        }
    }
    
    pub fn with_pretty_print(mut self, pretty: bool) -> Self {
        self.pretty_print = pretty;
        self
    }
    
    fn render_element_json(&self, element: &LogicalElement) -> serde_json::Value {
        use serde_json::{json, Value};
        
        match element {
            LogicalElement::Text { content, style } => {
                json!({
                    "type": "text",
                    "content": content,
                    "style": style
                })
            },
            LogicalElement::Heading { level, content, id } => {
                json!({
                    "type": "heading",
                    "level": level,
                    "content": content,
                    "id": id
                })
            },
            LogicalElement::List { items, ordered } => {
                json!({
                    "type": "list",
                    "ordered": ordered,
                    "items": items.iter().map(|i| self.render_element_json(i)).collect::<Vec<Value>>()
                })
            },
            LogicalElement::Table { headers, rows, caption } => {
                json!({
                    "type": "table",
                    "caption": caption,
                    "headers": headers,
                    "rows": rows.iter().map(|row| {
                        row.iter().map(|cell| self.render_element_json(cell)).collect::<Vec<Value>>()
                    }).collect::<Vec<Vec<Value>>>()
                })
            },
            LogicalElement::Link { url, text, external } => {
                json!({
                    "type": "link",
                    "url": url,
                    "text": text,
                    "external": external
                })
            },
            LogicalElement::Image { src, alt, width, height } => {
                json!({
                    "type": "image",
                    "src": src,
                    "alt": alt,
                    "width": width,
                    "height": height
                })
            },
            LogicalElement::Container { children, layout, css_class } => {
                json!({
                    "type": "container",
                    "layout": format!("{:?}", layout),
                    "css_class": css_class,
                    "children": children.iter().map(|c| self.render_element_json(c)).collect::<Vec<Value>>()
                })
            },
            LogicalElement::Form { fields, action, method } => {
                json!({
                    "type": "form",
                    "action": action,
                    "method": format!("{:?}", method),
                    "fields": fields.iter().map(|f| json!({
                        "name": f.name,
                        "label": f.label,
                        "type": format!("{:?}", f.field_type),
                        "required": f.required,
                        "value": f.value,
                        "placeholder": f.placeholder
                    })).collect::<Vec<Value>>()
                })
            },
            LogicalElement::Navigation { items, current_path } => {
                json!({
                    "type": "navigation",
                    "current_path": current_path,
                    "items": items.iter().map(|i| json!({
                        "path": i.path,
                        "title": i.title,
                        "active": i.active
                    })).collect::<Vec<Value>>()
                })
            },
        }
    }
}

impl FormatRenderer for JsonRenderer {
    fn render(&self, page: &LogicalPage) -> Result<String, RenderError> {
        use serde_json::json;
        
        let page_json = json!({
            "title": page.title,
            "description": page.description,
            "keywords": page.keywords,
            "metadata": page.metadata,
            "elements": page.elements.iter().map(|e| self.render_element_json(e)).collect::<Vec<_>>()
        });
        
        if self.pretty_print {
            serde_json::to_string_pretty(&page_json)
        } else {
            serde_json::to_string(&page_json)
        }.map_err(|e| RenderError::RenderingFailed(e.to_string()))
    }
    
    fn content_type(&self) -> &str {
        "application/json"
    }
}

// =================
// 两步视图处理器
// =================

/// 两步视图处理器 - 协调两个步骤
pub struct TwoStepViewProcessor<T> {
    page_builder: Box<dyn PageBuilder<Data = T>>,
    renderers: HashMap<String, Box<dyn FormatRenderer>>,
}

impl<T> TwoStepViewProcessor<T> {
    pub fn new(page_builder: Box<dyn PageBuilder<Data = T>>) -> Self {
        Self {
            page_builder,
            renderers: HashMap::new(),
        }
    }
    
    pub fn add_renderer(mut self, format: String, renderer: Box<dyn FormatRenderer>) -> Self {
        self.renderers.insert(format, renderer);
        self
    }
    
    pub fn process(&self, data: &T, format: &str) -> Result<(String, String), ProcessError> {
        // 第一步：构建逻辑页面
        let logical_page = self.page_builder.build_page(data)
            .map_err(ProcessError::BuildError)?;
        
        // 第二步：渲染为指定格式
        let renderer = self.renderers.get(format)
            .ok_or_else(|| ProcessError::UnsupportedFormat(format.to_string()))?;
        
        let content = renderer.render(&logical_page)
            .map_err(ProcessError::RenderError)?;
        
        let content_type = renderer.content_type().to_string();
        
        Ok((content, content_type))
    }
    
    pub fn supported_formats(&self) -> Vec<String> {
        self.renderers.keys().cloned().collect()
    }
}

/// 处理错误
#[derive(Debug)]
pub enum ProcessError {
    BuildError(BuildError),
    RenderError(RenderError),
    UnsupportedFormat(String),
}

impl fmt::Display for ProcessError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessError::BuildError(e) => write!(f, "Build error: {}", e),
            ProcessError::RenderError(e) => write!(f, "Render error: {}", e),
            ProcessError::UnsupportedFormat(format) => write!(f, "Unsupported format: {}", format),
        }
    }
}

impl std::error::Error for ProcessError {}

// =================
// 示例实现
// =================

/// 博客文章数据
#[derive(Debug, Clone)]
pub struct BlogPost {
    pub title: String,
    pub content: String,
    pub author: String,
    pub tags: Vec<String>,
    pub created_at: String,
    pub comments: Vec<Comment>,
}

/// 评论数据
#[derive(Debug, Clone)]
pub struct Comment {
    pub author: String,
    pub content: String,
    pub created_at: String,
}

/// 博客文章页面构建器
pub struct BlogPostPageBuilder;

impl PageBuilder for BlogPostPageBuilder {
    type Data = BlogPost;
    
    fn build_page(&self, data: &Self::Data) -> Result<LogicalPage, BuildError> {
        let mut page = LogicalPage::new(data.title.clone())
            .with_description(format!("博客文章: {}", data.title))
            .with_keywords(data.tags.clone());
        
        // 导航
        page = page.add_element(LogicalElement::Navigation {
            items: vec![
                NavigationItem { path: "/".to_string(), title: "首页".to_string(), active: false, children: vec![] },
                NavigationItem { path: "/blog".to_string(), title: "博客".to_string(), active: true, children: vec![] },
                NavigationItem { path: "/about".to_string(), title: "关于".to_string(), active: false, children: vec![] },
            ],
            current_path: Some("/blog".to_string()),
        });
        
        // 文章标题
        page = page.add_element(LogicalElement::Heading {
            level: 1,
            content: data.title.clone(),
            id: Some("article-title".to_string()),
        });
        
        // 文章元信息
        let meta_text = format!("作者: {} | 发布时间: {} | 标签: {}", 
            data.author, data.created_at, data.tags.join(", "));
        let mut meta_style = HashMap::new();
        meta_style.insert("color".to_string(), "#666".to_string());
        meta_style.insert("font-size".to_string(), "14px".to_string());
        
        page = page.add_element(LogicalElement::Text {
            content: meta_text,
            style: meta_style,
        });
        
        // 文章内容
        page = page.add_element(LogicalElement::Text {
            content: data.content.clone(),
            style: HashMap::new(),
        });
        
        // 评论部分
        if !data.comments.is_empty() {
            page = page.add_element(LogicalElement::Heading {
                level: 2,
                content: format!("评论 ({})", data.comments.len()),
                id: Some("comments".to_string()),
            });
            
            let comment_elements: Vec<LogicalElement> = data.comments.iter().map(|comment| {
                LogicalElement::Container {
                    children: vec![
                        LogicalElement::Text {
                            content: format!("{} - {}", comment.author, comment.created_at),
                            style: {
                                let mut style = HashMap::new();
                                style.insert("font-weight".to_string(), "bold".to_string());
                                style
                            },
                        },
                        LogicalElement::Text {
                            content: comment.content.clone(),
                            style: HashMap::new(),
                        },
                    ],
                    layout: ContainerLayout::Vertical,
                    css_class: Some("comment".to_string()),
                }
            }).collect();
            
            page = page.add_element(LogicalElement::Container {
                children: comment_elements,
                layout: ContainerLayout::Vertical,
                css_class: Some("comments-list".to_string()),
            });
        }
        
        // 评论表单
        page = page.add_element(LogicalElement::Heading {
            level: 3,
            content: "发表评论".to_string(),
            id: Some("comment-form".to_string()),
        });
        
        page = page.add_element(LogicalElement::Form {
            fields: vec![
                FormField {
                    name: "author".to_string(),
                    label: "姓名".to_string(),
                    field_type: FieldType::Text,
                    required: true,
                    value: None,
                    placeholder: Some("请输入您的姓名".to_string()),
                },
                FormField {
                    name: "email".to_string(),
                    label: "邮箱".to_string(),
                    field_type: FieldType::Email,
                    required: true,
                    value: None,
                    placeholder: Some("请输入您的邮箱".to_string()),
                },
                FormField {
                    name: "content".to_string(),
                    label: "评论内容".to_string(),
                    field_type: FieldType::TextArea,
                    required: true,
                    value: None,
                    placeholder: Some("请输入评论内容".to_string()),
                },
            ],
            action: "/comments/create".to_string(),
            method: HttpMethod::POST,
        });
        
        Ok(page)
    }
}

/// 两步视图模式演示
pub fn demo() {
    println!("=== 两步视图模式演示 ===\n");
    
    println!("两步视图模式将页面生成分为两个步骤：");
    println!("1. 第一步：将领域数据转换为逻辑页面结构");
    println!("2. 第二步：将逻辑页面渲染为具体格式（HTML、JSON等）");
    println!();
    
    // 创建示例数据
    let blog_post = BlogPost {
        title: "Rust设计模式：两步视图模式".to_string(),
        content: "两步视图模式是一种优雅的Web表现层模式，它将页面生成过程分为两个清晰的步骤。第一步专注于数据结构化，第二步专注于格式化输出。这种分离使得我们可以轻松支持多种输出格式，同时保持代码的清晰和可维护性。".to_string(),
        author: "Rust开发者".to_string(),
        tags: vec!["Rust".to_string(), "设计模式".to_string(), "Web开发".to_string()],
        created_at: "2024-01-15 10:30:00".to_string(),
        comments: vec![
            Comment {
                author: "张三".to_string(),
                content: "很好的文章，学到了很多！".to_string(),
                created_at: "2024-01-15 11:00:00".to_string(),
            },
            Comment {
                author: "李四".to_string(),
                content: "两步视图模式确实很实用，特别是在需要支持多种客户端的时候。".to_string(),
                created_at: "2024-01-15 11:30:00".to_string(),
            },
        ],
    };
    
    // 创建两步视图处理器
    let processor = TwoStepViewProcessor::new(Box::new(BlogPostPageBuilder))
        .add_renderer("html".to_string(), Box::new(HtmlRenderer::new().with_pretty_print(true)))
        .add_renderer("json".to_string(), Box::new(JsonRenderer::new().with_pretty_print(true)));
    
    println!("支持的输出格式: {:?}\n", processor.supported_formats());
    
    // 生成HTML格式
    println!("1. 生成HTML格式:");
    match processor.process(&blog_post, "html") {
        Ok((content, content_type)) => {
            println!("Content-Type: {}", content_type);
            println!("HTML长度: {} 字符", content.len());
            
            // 显示HTML片段
            let lines: Vec<&str> = content.lines().collect();
            let preview_lines = lines.iter().take(15).cloned().collect::<Vec<_>>();
            println!("HTML预览:");
            for line in preview_lines {
                println!("  {}", line);
            }
            if lines.len() > 15 {
                println!("  ... ({} 行总计)", lines.len());
            }
        },
        Err(e) => println!("生成HTML失败: {}", e),
    }
    
    println!("\n{}", "=".repeat(50));
    
    // 生成JSON格式
    println!("2. 生成JSON格式:");
    match processor.process(&blog_post, "json") {
        Ok((content, content_type)) => {
            println!("Content-Type: {}", content_type);
            println!("JSON长度: {} 字符", content.len());
            
            // 显示JSON片段
            let lines: Vec<&str> = content.lines().collect();
            let preview_lines = lines.iter().take(20).cloned().collect::<Vec<_>>();
            println!("JSON预览:");
            for line in preview_lines {
                println!("  {}", line);
            }
            if lines.len() > 20 {
                println!("  ... ({} 行总计)", lines.len());
            }
        },
        Err(e) => println!("生成JSON失败: {}", e),
    }
    
    println!("\n{}", "=".repeat(50));
    
    // 测试不支持的格式
    println!("3. 测试不支持的格式:");
    match processor.process(&blog_post, "xml") {
        Ok(_) => println!("意外成功"),
        Err(e) => println!("预期错误: {}", e),
    }
    
    println!("\n=== 两步视图模式特点 ===");
    println!("✓ 关注点分离 - 数据结构化与格式化分离");
    println!("✓ 多格式支持 - 一份数据，多种输出格式");
    println!("✓ 可扩展性 - 易于添加新的输出格式");
    println!("✓ 可重用性 - 逻辑页面结构可在不同格式间复用");
    println!("✓ 可维护性 - 每个步骤职责单一，易于维护");
    println!("✓ 可测试性 - 两个步骤可以独立测试");
    
    println!("\n=== 适用场景 ===");
    println!("• 需要支持多种客户端（Web、移动端、API）");
    println!("• 同一内容需要多种显示格式");
    println!("• 内容管理系统");
    println!("• RESTful API与Web界面共存");
    println!("• 需要SEO优化的同时提供API服务");
    
    println!("\n=== 最佳实践 ===");
    println!("• 逻辑页面结构保持格式无关");
    println!("• 渲染器专注于格式化，不包含业务逻辑");
    println!("• 使用依赖注入管理渲染器");
    println!("• 考虑缓存逻辑页面结构以提高性能");
    println!("• 为不同格式提供合适的错误处理");
} 