//! # 插件模式（Plugin Pattern）
//!
//! 插件模式通过配置而非编程的方式来连接类，
//! 允许在运行时动态加载和配置系统的行为。
//! 这种模式支持系统的可扩展性和可配置性。
//!
//! ## 模式特点
//! - **运行时配置**: 通过配置文件动态决定系统行为
//! - **松耦合**: 插件与主系统解耦
//! - **可扩展性**: 支持第三方扩展
//! - **热插拔**: 支持运行时加载/卸载插件
//!
//! ## 使用场景
//! - 需要支持第三方扩展的系统
//! - 可配置的业务流程
//! - 多租户系统中的定制功能
//! - 插件架构的应用程序

use std::collections::HashMap;
use std::fmt::{self, Display, Formatter};
use std::error::Error;
use std::any::{Any, TypeId};

/// 插件系统错误类型
#[derive(Debug)]
pub enum PluginError {
    PluginNotFound(String),
    PluginLoadError(String),
    PluginConfigError(String),
    PluginExecutionError(String),
    InvalidInterface(String),
}

impl Display for PluginError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            PluginError::PluginNotFound(msg) => write!(f, "插件未找到: {}", msg),
            PluginError::PluginLoadError(msg) => write!(f, "插件加载错误: {}", msg),
            PluginError::PluginConfigError(msg) => write!(f, "插件配置错误: {}", msg),
            PluginError::PluginExecutionError(msg) => write!(f, "插件执行错误: {}", msg),
            PluginError::InvalidInterface(msg) => write!(f, "无效的插件接口: {}", msg),
        }
    }
}

impl Error for PluginError {}

/// 插件配置
#[derive(Debug, Clone)]
pub struct PluginConfig {
    pub name: String,
    pub version: String,
    pub enabled: bool,
    pub priority: i32,
    pub parameters: HashMap<String, String>,
}

impl PluginConfig {
    pub fn new(name: String, version: String) -> Self {
        Self {
            name,
            version,
            enabled: true,
            priority: 0,
            parameters: HashMap::new(),
        }
    }

    pub fn with_parameter(mut self, key: String, value: String) -> Self {
        self.parameters.insert(key, value);
        self
    }

    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    pub fn get_parameter(&self, key: &str) -> Option<&String> {
        self.parameters.get(key)
    }
}

/// 插件上下文
#[derive(Debug)]
pub struct PluginContext {
    pub plugin_name: String,
    pub config: PluginConfig,
    pub shared_data: HashMap<String, String>,
}

impl PluginContext {
    pub fn new(plugin_name: String, config: PluginConfig) -> Self {
        Self {
            plugin_name,
            config,
            shared_data: HashMap::new(),
        }
    }

    pub fn set_data(&mut self, key: String, value: String) {
        self.shared_data.insert(key, value);
    }

    pub fn get_data(&self, key: &str) -> Option<&String> {
        self.shared_data.get(key)
    }
}

/// 插件执行结果
#[derive(Debug, Clone)]
pub struct PluginResult {
    pub success: bool,
    pub message: String,
    pub data: HashMap<String, String>,
    pub execution_time_ms: u64,
}

impl PluginResult {
    pub fn success(message: String) -> Self {
        Self {
            success: true,
            message,
            data: HashMap::new(),
            execution_time_ms: 0,
        }
    }

    pub fn failure(message: String) -> Self {
        Self {
            success: false,
            message,
            data: HashMap::new(),
            execution_time_ms: 0,
        }
    }

    pub fn with_data(mut self, key: String, value: String) -> Self {
        self.data.insert(key, value);
        self
    }

    pub fn with_execution_time(mut self, time_ms: u64) -> Self {
        self.execution_time_ms = time_ms;
        self
    }
}

/// 插件接口 - 所有插件必须实现的基础接口
pub trait Plugin: Send + Sync {
    fn get_name(&self) -> &str;
    fn get_version(&self) -> &str;
    fn get_description(&self) -> &str;
    fn initialize(&mut self, context: &mut PluginContext) -> Result<(), PluginError>;
    fn execute(&self, context: &PluginContext, input: &str) -> Result<PluginResult, PluginError>;
    fn cleanup(&mut self) -> Result<(), PluginError>;
    fn get_supported_operations(&self) -> Vec<String>;
    fn is_compatible_with(&self, version: &str) -> bool;
}

/// 数据处理插件接口
pub trait DataProcessorPlugin: Plugin {
    fn process_data(&self, data: &str, context: &PluginContext) -> Result<String, PluginError>;
    fn get_supported_formats(&self) -> Vec<String>;
    fn validate_data(&self, data: &str) -> Result<bool, PluginError>;
}

/// 认证插件接口
pub trait AuthenticationPlugin: Plugin {
    fn authenticate(&self, username: &str, password: &str, context: &PluginContext) -> Result<bool, PluginError>;
    fn authorize(&self, user_id: &str, resource: &str, action: &str, context: &PluginContext) -> Result<bool, PluginError>;
    fn get_user_info(&self, user_id: &str, context: &PluginContext) -> Result<HashMap<String, String>, PluginError>;
}

/// JSON数据处理插件
pub struct JsonProcessorPlugin {
    name: String,
    version: String,
    initialized: bool,
}

impl JsonProcessorPlugin {
    pub fn new() -> Self {
        Self {
            name: "JSON处理器".to_string(),
            version: "1.0.0".to_string(),
            initialized: false,
        }
    }
}

impl Plugin for JsonProcessorPlugin {
    fn get_name(&self) -> &str {
        &self.name
    }

    fn get_version(&self) -> &str {
        &self.version
    }

    fn get_description(&self) -> &str {
        "JSON数据格式处理插件"
    }

    fn initialize(&mut self, context: &mut PluginContext) -> Result<(), PluginError> {
        println!("🔌 初始化JSON处理插件: {}", context.plugin_name);
        context.set_data("format".to_string(), "json".to_string());
        self.initialized = true;
        Ok(())
    }

    fn execute(&self, context: &PluginContext, input: &str) -> Result<PluginResult, PluginError> {
        if !self.initialized {
            return Err(PluginError::PluginExecutionError("插件未初始化".to_string()));
        }

        let start_time = std::time::Instant::now();
        
        // 简单的JSON处理逻辑
        let processed = if input.starts_with('{') && input.ends_with('}') {
            format!("{{\"processed\": true, \"original\": {}}}", input)
        } else {
            format!("{{\"processed\": true, \"original\": \"{}\"}}", input)
        };
        
        let elapsed = start_time.elapsed().as_millis() as u64;
        
        Ok(PluginResult::success("JSON处理完成".to_string())
            .with_data("output".to_string(), processed)
            .with_execution_time(elapsed))
    }

    fn cleanup(&mut self) -> Result<(), PluginError> {
        println!("🔌 清理JSON处理插件");
        self.initialized = false;
        Ok(())
    }

    fn get_supported_operations(&self) -> Vec<String> {
        vec!["parse".to_string(), "format".to_string(), "validate".to_string()]
    }

    fn is_compatible_with(&self, version: &str) -> bool {
        version >= "1.0" && version < "2.0"
    }
}

impl DataProcessorPlugin for JsonProcessorPlugin {
    fn process_data(&self, data: &str, _context: &PluginContext) -> Result<String, PluginError> {
        // 简单的JSON美化
        Ok(format!("{{\n  \"data\": \"{}\",\n  \"timestamp\": \"2024-01-01T12:00:00Z\"\n}}", data))
    }

    fn get_supported_formats(&self) -> Vec<String> {
        vec!["json".to_string(), "application/json".to_string()]
    }

    fn validate_data(&self, data: &str) -> Result<bool, PluginError> {
        // 简单的JSON验证
        Ok(data.trim().starts_with('{') && data.trim().ends_with('}'))
    }
}

/// XML数据处理插件
pub struct XmlProcessorPlugin {
    name: String,
    version: String,
    initialized: bool,
}

impl XmlProcessorPlugin {
    pub fn new() -> Self {
        Self {
            name: "XML处理器".to_string(),
            version: "1.0.0".to_string(),
            initialized: false,
        }
    }
}

impl Plugin for XmlProcessorPlugin {
    fn get_name(&self) -> &str {
        &self.name
    }

    fn get_version(&self) -> &str {
        &self.version
    }

    fn get_description(&self) -> &str {
        "XML数据格式处理插件"
    }

    fn initialize(&mut self, context: &mut PluginContext) -> Result<(), PluginError> {
        println!("🔌 初始化XML处理插件: {}", context.plugin_name);
        context.set_data("format".to_string(), "xml".to_string());
        self.initialized = true;
        Ok(())
    }

    fn execute(&self, context: &PluginContext, input: &str) -> Result<PluginResult, PluginError> {
        if !self.initialized {
            return Err(PluginError::PluginExecutionError("插件未初始化".to_string()));
        }

        let start_time = std::time::Instant::now();
        
        // 简单的XML处理逻辑
        let processed = format!("<root><data>{}</data><processed>true</processed></root>", input);
        
        let elapsed = start_time.elapsed().as_millis() as u64;
        
        Ok(PluginResult::success("XML处理完成".to_string())
            .with_data("output".to_string(), processed)
            .with_execution_time(elapsed))
    }

    fn cleanup(&mut self) -> Result<(), PluginError> {
        println!("🔌 清理XML处理插件");
        self.initialized = false;
        Ok(())
    }

    fn get_supported_operations(&self) -> Vec<String> {
        vec!["parse".to_string(), "format".to_string(), "transform".to_string()]
    }

    fn is_compatible_with(&self, version: &str) -> bool {
        version >= "1.0" && version < "2.0"
    }
}

impl DataProcessorPlugin for XmlProcessorPlugin {
    fn process_data(&self, data: &str, _context: &PluginContext) -> Result<String, PluginError> {
        // 简单的XML格式化
        Ok(format!("<data>\n  <content>{}</content>\n  <timestamp>2024-01-01T12:00:00Z</timestamp>\n</data>", data))
    }

    fn get_supported_formats(&self) -> Vec<String> {
        vec!["xml".to_string(), "application/xml".to_string(), "text/xml".to_string()]
    }

    fn validate_data(&self, data: &str) -> Result<bool, PluginError> {
        // 简单的XML验证
        Ok(data.trim().starts_with('<') && data.trim().ends_with('>'))
    }
}

/// 简单认证插件
pub struct SimpleAuthPlugin {
    name: String,
    version: String,
    initialized: bool,
    users: HashMap<String, String>, // 用户名 -> 密码
}

impl SimpleAuthPlugin {
    pub fn new() -> Self {
        let mut users = HashMap::new();
        users.insert("admin".to_string(), "admin123".to_string());
        users.insert("user".to_string(), "user123".to_string());
        users.insert("guest".to_string(), "guest123".to_string());
        
        Self {
            name: "简单认证".to_string(),
            version: "1.0.0".to_string(),
            initialized: false,
            users,
        }
    }
}

impl Plugin for SimpleAuthPlugin {
    fn get_name(&self) -> &str {
        &self.name
    }

    fn get_version(&self) -> &str {
        &self.version
    }

    fn get_description(&self) -> &str {
        "简单的用户名密码认证插件"
    }

    fn initialize(&mut self, context: &mut PluginContext) -> Result<(), PluginError> {
        println!("🔌 初始化简单认证插件: {}", context.plugin_name);
        context.set_data("auth_type".to_string(), "simple".to_string());
        self.initialized = true;
        Ok(())
    }

    fn execute(&self, _context: &PluginContext, input: &str) -> Result<PluginResult, PluginError> {
        if !self.initialized {
            return Err(PluginError::PluginExecutionError("插件未初始化".to_string()));
        }

        // 简单的认证逻辑（从输入中解析用户名和密码）
        let parts: Vec<&str> = input.split(':').collect();
        if parts.len() != 2 {
            return Ok(PluginResult::failure("输入格式错误，应为'username:password'".to_string()));
        }

        let username = parts[0];
        let password = parts[1];
        
        match self.users.get(username) {
            Some(stored_password) if stored_password == password => {
                Ok(PluginResult::success(format!("用户 {} 认证成功", username))
                    .with_data("user_id".to_string(), username.to_string())
                    .with_data("authenticated".to_string(), "true".to_string()))
            }
            _ => {
                Ok(PluginResult::failure(format!("用户 {} 认证失败", username))
                    .with_data("authenticated".to_string(), "false".to_string()))
            }
        }
    }

    fn cleanup(&mut self) -> Result<(), PluginError> {
        println!("🔌 清理简单认证插件");
        self.initialized = false;
        Ok(())
    }

    fn get_supported_operations(&self) -> Vec<String> {
        vec!["authenticate".to_string(), "authorize".to_string()]
    }

    fn is_compatible_with(&self, version: &str) -> bool {
        version >= "1.0" && version < "2.0"
    }
}

impl AuthenticationPlugin for SimpleAuthPlugin {
    fn authenticate(&self, username: &str, password: &str, _context: &PluginContext) -> Result<bool, PluginError> {
        match self.users.get(username) {
            Some(stored_password) => Ok(stored_password == password),
            None => Ok(false),
        }
    }

    fn authorize(&self, user_id: &str, resource: &str, action: &str, _context: &PluginContext) -> Result<bool, PluginError> {
        // 简单的授权逻辑
        match user_id {
            "admin" => Ok(true), // 管理员有所有权限
            "user" => Ok(action == "read" || (action == "write" && resource.starts_with("user_"))),
            "guest" => Ok(action == "read" && resource == "public"),
            _ => Ok(false),
        }
    }

    fn get_user_info(&self, user_id: &str, _context: &PluginContext) -> Result<HashMap<String, String>, PluginError> {
        if self.users.contains_key(user_id) {
            let mut info = HashMap::new();
            info.insert("user_id".to_string(), user_id.to_string());
            info.insert("role".to_string(), match user_id {
                "admin" => "管理员".to_string(),
                "user" => "普通用户".to_string(),
                "guest" => "访客".to_string(),
                _ => "未知".to_string(),
            });
            info.insert("status".to_string(), "active".to_string());
            Ok(info)
        } else {
            Err(PluginError::PluginNotFound(format!("用户 {} 不存在", user_id)))
        }
    }
}

/// 插件管理器
pub struct PluginManager {
    plugins: HashMap<String, Box<dyn Plugin>>,
    data_processors: HashMap<String, Box<dyn DataProcessorPlugin>>,
    auth_providers: HashMap<String, Box<dyn AuthenticationPlugin>>,
    configurations: HashMap<String, PluginConfig>,
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
            data_processors: HashMap::new(),
            auth_providers: HashMap::new(),
            configurations: HashMap::new(),
        }
    }

    /// 注册插件
    pub fn register_plugin(&mut self, plugin: Box<dyn Plugin>, config: PluginConfig) -> Result<(), PluginError> {
        let name = plugin.get_name().to_string();
        
        if !config.enabled {
            println!("⚠️  插件 {} 已禁用，跳过注册", name);
            return Ok(());
        }

        println!("📦 注册插件: {} v{}", name, plugin.get_version());
        
        self.configurations.insert(name.clone(), config);
        self.plugins.insert(name, plugin);
        
        Ok(())
    }

    /// 注册数据处理插件
    pub fn register_data_processor(&mut self, plugin: Box<dyn DataProcessorPlugin>, config: PluginConfig) -> Result<(), PluginError> {
        let name = plugin.get_name().to_string();
        
        if !config.enabled {
            println!("⚠️  数据处理插件 {} 已禁用，跳过注册", name);
            return Ok(());
        }

        println!("📦 注册数据处理插件: {} v{}", name, plugin.get_version());
        
        self.configurations.insert(name.clone(), config);
        self.data_processors.insert(name, plugin);
        
        Ok(())
    }

    /// 注册认证插件
    pub fn register_auth_provider(&mut self, plugin: Box<dyn AuthenticationPlugin>, config: PluginConfig) -> Result<(), PluginError> {
        let name = plugin.get_name().to_string();
        
        if !config.enabled {
            println!("⚠️  认证插件 {} 已禁用，跳过注册", name);
            return Ok(());
        }

        println!("📦 注册认证插件: {} v{}", name, plugin.get_version());
        
        self.configurations.insert(name.clone(), config);
        self.auth_providers.insert(name, plugin);
        
        Ok(())
    }

    /// 初始化所有插件
    pub fn initialize_all(&mut self) -> Result<(), PluginError> {
        println!("🚀 初始化所有插件...");
        
        // 按优先级排序初始化
        let mut plugin_names: Vec<_> = self.configurations.keys().cloned().collect();
        plugin_names.sort_by_key(|name| -self.configurations.get(name).unwrap().priority);
        
        for name in plugin_names {
            if let Some(config) = self.configurations.get(&name).cloned() {
                let mut context = PluginContext::new(name.clone(), config);
                
                if let Some(plugin) = self.plugins.get_mut(&name) {
                    plugin.initialize(&mut context)?;
                }
            }
        }
        
        println!("✅ 所有插件初始化完成");
        Ok(())
    }

    /// 执行插件
    pub fn execute_plugin(&self, plugin_name: &str, input: &str) -> Result<PluginResult, PluginError> {
        let plugin = self.plugins.get(plugin_name)
            .ok_or_else(|| PluginError::PluginNotFound(plugin_name.to_string()))?;
        
        let config = self.configurations.get(plugin_name)
            .ok_or_else(|| PluginError::PluginConfigError(format!("配置未找到: {}", plugin_name)))?;
        
        let context = PluginContext::new(plugin_name.to_string(), config.clone());
        plugin.execute(&context, input)
    }

    /// 处理数据
    pub fn process_data(&self, processor_name: &str, data: &str) -> Result<String, PluginError> {
        let processor = self.data_processors.get(processor_name)
            .ok_or_else(|| PluginError::PluginNotFound(processor_name.to_string()))?;
        
        let config = self.configurations.get(processor_name)
            .ok_or_else(|| PluginError::PluginConfigError(format!("配置未找到: {}", processor_name)))?;
        
        let context = PluginContext::new(processor_name.to_string(), config.clone());
        processor.process_data(data, &context)
    }

    /// 认证用户
    pub fn authenticate_user(&self, auth_provider: &str, username: &str, password: &str) -> Result<bool, PluginError> {
        let provider = self.auth_providers.get(auth_provider)
            .ok_or_else(|| PluginError::PluginNotFound(auth_provider.to_string()))?;
        
        let config = self.configurations.get(auth_provider)
            .ok_or_else(|| PluginError::PluginConfigError(format!("配置未找到: {}", auth_provider)))?;
        
        let context = PluginContext::new(auth_provider.to_string(), config.clone());
        provider.authenticate(username, password, &context)
    }

    /// 获取插件列表
    pub fn list_plugins(&self) -> Vec<PluginInfo> {
        let mut infos = Vec::new();
        
        for (name, plugin) in &self.plugins {
            if let Some(config) = self.configurations.get(name) {
                infos.push(PluginInfo {
                    name: name.clone(),
                    version: plugin.get_version().to_string(),
                    description: plugin.get_description().to_string(),
                    enabled: config.enabled,
                    priority: config.priority,
                    plugin_type: "通用插件".to_string(),
                });
            }
        }
        
        for (name, plugin) in &self.data_processors {
            if let Some(config) = self.configurations.get(name) {
                infos.push(PluginInfo {
                    name: name.clone(),
                    version: plugin.get_version().to_string(),
                    description: plugin.get_description().to_string(),
                    enabled: config.enabled,
                    priority: config.priority,
                    plugin_type: "数据处理插件".to_string(),
                });
            }
        }
        
        for (name, plugin) in &self.auth_providers {
            if let Some(config) = self.configurations.get(name) {
                infos.push(PluginInfo {
                    name: name.clone(),
                    version: plugin.get_version().to_string(),
                    description: plugin.get_description().to_string(),
                    enabled: config.enabled,
                    priority: config.priority,
                    plugin_type: "认证插件".to_string(),
                });
            }
        }
        
        // 按优先级排序
        infos.sort_by_key(|info| -info.priority);
        infos
    }

    /// 清理所有插件
    pub fn cleanup_all(&mut self) -> Result<(), PluginError> {
        println!("🧹 清理所有插件...");
        
        for (_, plugin) in self.plugins.iter_mut() {
            plugin.cleanup()?;
        }
        
        println!("✅ 所有插件清理完成");
        Ok(())
    }
}

/// 插件信息
#[derive(Debug, Clone)]
pub struct PluginInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub enabled: bool,
    pub priority: i32,
    pub plugin_type: String,
}

impl Display for PluginInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} v{} ({}) - {} [优先级: {}, 状态: {}]",
               self.name, self.version, self.plugin_type, self.description,
               self.priority, if self.enabled { "启用" } else { "禁用" })
    }
}

/// 演示插件模式
pub fn demo() {
    println!("=== 插件模式演示 ===\n");

    // 创建插件管理器
    let mut manager = PluginManager::new();

    println!("1. 注册各种插件");
    
    // 注册JSON处理插件
    let json_config = PluginConfig::new("JSON处理器".to_string(), "1.0.0".to_string())
        .with_priority(10)
        .with_parameter("encoding".to_string(), "utf-8".to_string());
    
    manager.register_data_processor(Box::new(JsonProcessorPlugin::new()), json_config).unwrap();

    // 注册XML处理插件
    let xml_config = PluginConfig::new("XML处理器".to_string(), "1.0.0".to_string())
        .with_priority(5)
        .with_parameter("schema_validation".to_string(), "false".to_string());
    
    manager.register_data_processor(Box::new(XmlProcessorPlugin::new()), xml_config).unwrap();

    // 注册认证插件
    let auth_config = PluginConfig::new("简单认证".to_string(), "1.0.0".to_string())
        .with_priority(20)
        .with_parameter("session_timeout".to_string(), "3600".to_string());
    
    manager.register_auth_provider(Box::new(SimpleAuthPlugin::new()), auth_config).unwrap();

    // 注册一个禁用的插件
    let disabled_config = PluginConfig::new("禁用插件".to_string(), "1.0.0".to_string());
    let mut disabled_config = disabled_config;
    disabled_config.enabled = false;
    
    manager.register_data_processor(Box::new(JsonProcessorPlugin::new()), disabled_config).unwrap();

    println!("\n2. 初始化所有插件");
    manager.initialize_all().unwrap();

    println!("\n3. 列出所有已注册的插件");
    let plugins = manager.list_plugins();
    for plugin in &plugins {
        println!("   🔌 {}", plugin);
    }

    println!("\n4. 演示数据处理插件");
    
    // JSON数据处理
    println!("\n   📄 JSON数据处理:");
    let json_data = r#"{"name": "测试数据", "value": 123}"#;
    match manager.process_data("JSON处理器", json_data) {
        Ok(result) => println!("     ✅ 处理结果:\n{}", result),
        Err(e) => println!("     ❌ 处理失败: {}", e),
    }

    // XML数据处理
    println!("\n   📄 XML数据处理:");
    let xml_data = "测试XML数据";
    match manager.process_data("XML处理器", xml_data) {
        Ok(result) => println!("     ✅ 处理结果:\n{}", result),
        Err(e) => println!("     ❌ 处理失败: {}", e),
    }

    println!("\n5. 演示认证插件");
    
    let test_credentials = vec![
        ("admin", "admin123", true),
        ("user", "user123", true),
        ("guest", "guest123", true),
        ("admin", "wrongpass", false),
        ("nonexistent", "password", false),
    ];

    for (username, password, expected) in test_credentials {
        match manager.authenticate_user("简单认证", username, password) {
            Ok(result) => {
                let status = if result { "✅ 成功" } else { "❌ 失败" };
                let match_expected = if result == expected { "符合预期" } else { "不符合预期" };
                println!("     👤 用户 {} 认证{} ({})", username, status, match_expected);
            }
            Err(e) => println!("     ❌ 认证错误: {}", e),
        }
    }

    println!("\n6. 演示通用插件执行");
    
    // 尝试执行认证插件的通用接口
    let auth_input = "admin:admin123";
    if let Ok(result) = manager.execute_plugin("简单认证", auth_input) {
        println!("     🔑 认证结果: {}", result.message);
        println!("     📊 执行时间: {}ms", result.execution_time_ms);
        for (key, value) in &result.data {
            println!("     📝 数据 {}: {}", key, value);
        }
    }

    println!("\n7. 演示插件错误处理");
    
    // 尝试使用不存在的插件
    match manager.process_data("不存在的插件", "数据") {
        Ok(_) => println!("     意外成功"),
        Err(e) => println!("     ✅ 正确捕获错误: {}", e),
    }

    // 尝试使用被禁用的插件
    match manager.process_data("禁用插件", "数据") {
        Ok(_) => println!("     意外成功"),
        Err(e) => println!("     ✅ 正确捕获错误: {}", e),
    }

    println!("\n8. 清理插件资源");
    manager.cleanup_all().unwrap();

    println!("\n=== 插件模式演示完成 ===");

    println!("\n💡 插件模式的优势:");
    println!("1. 运行时配置 - 通过配置文件控制系统行为");
    println!("2. 松耦合 - 插件与核心系统解耦");
    println!("3. 可扩展性 - 支持第三方开发插件");
    println!("4. 热插拔 - 支持动态加载和卸载");

    println!("\n⚠️ 设计考虑:");
    println!("1. 接口设计 - 需要设计稳定的插件接口");
    println!("2. 版本兼容 - 处理插件版本兼容性问题");
    println!("3. 安全性 - 插件的安全性和权限控制");
    println!("4. 性能影响 - 动态调用可能影响性能");
    println!("5. 依赖管理 - 插件间的依赖关系管理");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_config() {
        let config = PluginConfig::new("测试插件".to_string(), "1.0.0".to_string())
            .with_parameter("key1".to_string(), "value1".to_string())
            .with_priority(5);
        
        assert_eq!(config.name, "测试插件");
        assert_eq!(config.version, "1.0.0");
        assert_eq!(config.priority, 5);
        assert_eq!(config.get_parameter("key1"), Some(&"value1".to_string()));
    }

    #[test]
    fn test_json_processor_plugin() {
        let mut plugin = JsonProcessorPlugin::new();
        let config = PluginConfig::new("JSON处理器".to_string(), "1.0.0".to_string());
        let mut context = PluginContext::new("JSON处理器".to_string(), config);
        
        // 测试初始化
        let init_result = plugin.initialize(&mut context);
        assert!(init_result.is_ok());
        
        // 测试执行
        let result = plugin.execute(&context, r#"{"test": "data"}"#);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.success);
        
        // 测试数据处理
        let processed = plugin.process_data("test data", &context);
        assert!(processed.is_ok());
        
        // 测试清理
        let cleanup_result = plugin.cleanup();
        assert!(cleanup_result.is_ok());
    }

    #[test]
    fn test_simple_auth_plugin() {
        let mut plugin = SimpleAuthPlugin::new();
        let config = PluginConfig::new("简单认证".to_string(), "1.0.0".to_string());
        let mut context = PluginContext::new("简单认证".to_string(), config.clone());
        
        // 初始化
        plugin.initialize(&mut context).unwrap();
        
        // 测试认证
        assert!(plugin.authenticate("admin", "admin123", &context).unwrap());
        assert!(!plugin.authenticate("admin", "wrongpass", &context).unwrap());
        assert!(!plugin.authenticate("nonexistent", "password", &context).unwrap());
        
        // 测试授权
        assert!(plugin.authorize("admin", "any_resource", "any_action", &context).unwrap());
        assert!(plugin.authorize("user", "user_data", "read", &context).unwrap());
        assert!(plugin.authorize("user", "user_data", "write", &context).unwrap());
        assert!(!plugin.authorize("user", "admin_data", "write", &context).unwrap());
        
        // 测试获取用户信息
        let info = plugin.get_user_info("admin", &context).unwrap();
        assert_eq!(info.get("role"), Some(&"管理员".to_string()));
    }

    #[test]
    fn test_plugin_manager() {
        let mut manager = PluginManager::new();
        
        // 注册插件
        let config = PluginConfig::new("JSON处理器".to_string(), "1.0.0".to_string());
        let result = manager.register_data_processor(Box::new(JsonProcessorPlugin::new()), config);
        assert!(result.is_ok());
        
        // 初始化
        let init_result = manager.initialize_all();
        assert!(init_result.is_ok());
        
        // 处理数据
        let process_result = manager.process_data("JSON处理器", "test data");
        assert!(process_result.is_ok());
        
        // 列出插件
        let plugins = manager.list_plugins();
        assert_eq!(plugins.len(), 1);
        assert_eq!(plugins[0].name, "JSON处理器");
        
        // 清理
        let cleanup_result = manager.cleanup_all();
        assert!(cleanup_result.is_ok());
    }
} 