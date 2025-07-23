//! # 分离接口模式（Separated Interface Pattern）
//!
//! 分离接口模式通过在单独的包中定义接口，让客户端不依赖于具体实现。
//! 这种模式允许实现的独立部署，减少依赖关系，提高系统的灵活性。
//!
//! ## 模式特点
//! - **依赖隔离**: 客户端只依赖接口，不依赖实现
//! - **独立部署**: 实现可以独立变更和部署
//! - **插件架构**: 支持运行时切换实现
//! - **测试友好**: 便于创建测试替身
//!
//! ## 使用场景
//! - 需要降低模块间耦合度
//! - 支持多种实现策略
//! - 需要独立部署不同组件
//! - 创建可插拔的系统架构

use std::collections::HashMap;
use std::fmt::{self, Display, Formatter};
use std::error::Error;
use std::any::Any;

/// 分离接口错误类型
#[derive(Debug)]
pub enum SeparatedInterfaceError {
    ProviderNotFound(String),
    ConfigurationError(String),
    InitializationError(String),
    ServiceError(String),
}

impl Display for SeparatedInterfaceError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            SeparatedInterfaceError::ProviderNotFound(msg) => write!(f, "提供者未找到: {}", msg),
            SeparatedInterfaceError::ConfigurationError(msg) => write!(f, "配置错误: {}", msg),
            SeparatedInterfaceError::InitializationError(msg) => write!(f, "初始化错误: {}", msg),
            SeparatedInterfaceError::ServiceError(msg) => write!(f, "服务错误: {}", msg),
        }
    }
}

impl Error for SeparatedInterfaceError {}

/// 日志级别
#[derive(Debug, Clone, PartialEq)]
pub enum LogLevel {
    Debug,
    Info,
    Warning,
    Error,
    Fatal,
}

impl Display for LogLevel {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            LogLevel::Debug => write!(f, "DEBUG"),
            LogLevel::Info => write!(f, "INFO"),
            LogLevel::Warning => write!(f, "WARN"),
            LogLevel::Error => write!(f, "ERROR"),
            LogLevel::Fatal => write!(f, "FATAL"),
        }
    }
}

/// 日志条目
#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: LogLevel,
    pub message: String,
    pub module: String,
    pub metadata: HashMap<String, String>,
}

impl LogEntry {
    pub fn new(level: LogLevel, message: String, module: String) -> Self {
        Self {
            timestamp: "2024-01-01 12:00:00".to_string(),
            level,
            message,
            module,
            metadata: HashMap::new(),
        }
    }

    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

/// 分离的日志接口
/// 
/// 这个接口在独立的包中定义，客户端只依赖这个接口
pub trait LoggerInterface: Send + Sync {
    fn log(&self, entry: &LogEntry) -> Result<(), SeparatedInterfaceError>;
    fn log_debug(&self, message: &str, module: &str) -> Result<(), SeparatedInterfaceError>;
    fn log_info(&self, message: &str, module: &str) -> Result<(), SeparatedInterfaceError>;
    fn log_warning(&self, message: &str, module: &str) -> Result<(), SeparatedInterfaceError>;
    fn log_error(&self, message: &str, module: &str) -> Result<(), SeparatedInterfaceError>;
    fn is_enabled(&self, level: &LogLevel) -> bool;
    fn get_name(&self) -> &str;
    fn flush(&self) -> Result<(), SeparatedInterfaceError>;
}

/// 分离的配置接口
pub trait ConfigurationInterface: Send + Sync {
    fn get_string(&self, key: &str) -> Option<String>;
    fn get_int(&self, key: &str) -> Option<i32>;
    fn get_bool(&self, key: &str) -> Option<bool>;
    fn set_value(&mut self, key: String, value: String);
    fn has_key(&self, key: &str) -> bool;
    fn get_all_keys(&self) -> Vec<String>;
}

/// 分离的缓存接口
pub trait CacheInterface: Send + Sync {
    fn get(&self, key: &str) -> Option<String>;
    fn set(&mut self, key: String, value: String, ttl_seconds: Option<u64>) -> Result<(), SeparatedInterfaceError>;
    fn delete(&mut self, key: &str) -> Result<bool, SeparatedInterfaceError>;
    fn exists(&self, key: &str) -> bool;
    fn clear(&mut self) -> Result<(), SeparatedInterfaceError>;
    fn get_stats(&self) -> CacheStats;
}

/// 缓存统计信息
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub entries: u64,
    pub memory_used: u64,
}

impl CacheStats {
    pub fn hit_rate(&self) -> f64 {
        if self.hits + self.misses == 0 {
            0.0
        } else {
            self.hits as f64 / (self.hits + self.misses) as f64
        }
    }
}

/// 控制台日志实现
pub struct ConsoleLogger {
    name: String,
    min_level: LogLevel,
}

impl ConsoleLogger {
    pub fn new(name: String, min_level: LogLevel) -> Self {
        Self { name, min_level }
    }

    fn should_log(&self, level: &LogLevel) -> bool {
        self.level_value(level) >= self.level_value(&self.min_level)
    }

    fn level_value(&self, level: &LogLevel) -> u8 {
        match level {
            LogLevel::Debug => 1,
            LogLevel::Info => 2,
            LogLevel::Warning => 3,
            LogLevel::Error => 4,
            LogLevel::Fatal => 5,
        }
    }
}

impl LoggerInterface for ConsoleLogger {
    fn log(&self, entry: &LogEntry) -> Result<(), SeparatedInterfaceError> {
        if !self.should_log(&entry.level) {
            return Ok(());
        }

        let mut output = format!("[{}] {} - {}: {}", 
                                entry.timestamp, entry.level, entry.module, entry.message);
        
        if !entry.metadata.is_empty() {
            output.push_str(" {");
            let metadata: Vec<String> = entry.metadata.iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect();
            output.push_str(&metadata.join(", "));
            output.push('}');
        }
        
        println!("{}", output);
        Ok(())
    }

    fn log_debug(&self, message: &str, module: &str) -> Result<(), SeparatedInterfaceError> {
        let entry = LogEntry::new(LogLevel::Debug, message.to_string(), module.to_string());
        self.log(&entry)
    }

    fn log_info(&self, message: &str, module: &str) -> Result<(), SeparatedInterfaceError> {
        let entry = LogEntry::new(LogLevel::Info, message.to_string(), module.to_string());
        self.log(&entry)
    }

    fn log_warning(&self, message: &str, module: &str) -> Result<(), SeparatedInterfaceError> {
        let entry = LogEntry::new(LogLevel::Warning, message.to_string(), module.to_string());
        self.log(&entry)
    }

    fn log_error(&self, message: &str, module: &str) -> Result<(), SeparatedInterfaceError> {
        let entry = LogEntry::new(LogLevel::Error, message.to_string(), module.to_string());
        self.log(&entry)
    }

    fn is_enabled(&self, level: &LogLevel) -> bool {
        self.should_log(level)
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn flush(&self) -> Result<(), SeparatedInterfaceError> {
        // 控制台输出通常是立即的，无需特殊处理
        Ok(())
    }
}

/// 文件日志实现
pub struct FileLogger {
    name: String,
    min_level: LogLevel,
    file_path: String,
    logs: Vec<String>, // 简化的内存存储，实际应用中会写入文件
}

impl FileLogger {
    pub fn new(name: String, min_level: LogLevel, file_path: String) -> Self {
        Self { 
            name, 
            min_level, 
            file_path,
            logs: Vec::new(),
        }
    }

    fn should_log(&self, level: &LogLevel) -> bool {
        match (level, &self.min_level) {
            (LogLevel::Fatal, _) => true,
            (LogLevel::Error, LogLevel::Fatal) => false,
            (LogLevel::Error, _) => true,
            (LogLevel::Warning, LogLevel::Error | LogLevel::Fatal) => false,
            (LogLevel::Warning, _) => true,
            (LogLevel::Info, LogLevel::Warning | LogLevel::Error | LogLevel::Fatal) => false,
            (LogLevel::Info, _) => true,
            (LogLevel::Debug, LogLevel::Debug) => true,
            (LogLevel::Debug, _) => false,
        }
    }
}

impl LoggerInterface for FileLogger {
    fn log(&self, entry: &LogEntry) -> Result<(), SeparatedInterfaceError> {
        if !self.should_log(&entry.level) {
            return Ok(());
        }

        let log_line = format!("[{}] {} - {}: {}", 
                              entry.timestamp, entry.level, entry.module, entry.message);
        
        // 在实际实现中，这里会写入文件
        println!("📁 [文件日志 {}] {}", self.file_path, log_line);
        Ok(())
    }

    fn log_debug(&self, message: &str, module: &str) -> Result<(), SeparatedInterfaceError> {
        let entry = LogEntry::new(LogLevel::Debug, message.to_string(), module.to_string());
        self.log(&entry)
    }

    fn log_info(&self, message: &str, module: &str) -> Result<(), SeparatedInterfaceError> {
        let entry = LogEntry::new(LogLevel::Info, message.to_string(), module.to_string());
        self.log(&entry)
    }

    fn log_warning(&self, message: &str, module: &str) -> Result<(), SeparatedInterfaceError> {
        let entry = LogEntry::new(LogLevel::Warning, message.to_string(), module.to_string());
        self.log(&entry)
    }

    fn log_error(&self, message: &str, module: &str) -> Result<(), SeparatedInterfaceError> {
        let entry = LogEntry::new(LogLevel::Error, message.to_string(), module.to_string());
        self.log(&entry)
    }

    fn is_enabled(&self, level: &LogLevel) -> bool {
        self.should_log(level)
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn flush(&self) -> Result<(), SeparatedInterfaceError> {
        println!("📁 [文件日志 {}] 刷新缓冲区", self.file_path);
        Ok(())
    }
}

/// 内存配置实现
pub struct MemoryConfiguration {
    data: HashMap<String, String>,
}

impl MemoryConfiguration {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    pub fn with_defaults() -> Self {
        let mut config = Self::new();
        config.set_value("app.name".to_string(), "演示应用".to_string());
        config.set_value("app.version".to_string(), "1.0.0".to_string());
        config.set_value("server.port".to_string(), "8080".to_string());
        config.set_value("database.enabled".to_string(), "true".to_string());
        config.set_value("cache.size".to_string(), "1000".to_string());
        config
    }
}

impl ConfigurationInterface for MemoryConfiguration {
    fn get_string(&self, key: &str) -> Option<String> {
        self.data.get(key).cloned()
    }

    fn get_int(&self, key: &str) -> Option<i32> {
        self.data.get(key)?.parse().ok()
    }

    fn get_bool(&self, key: &str) -> Option<bool> {
        match self.data.get(key)?.as_str() {
            "true" | "1" | "yes" | "on" => Some(true),
            "false" | "0" | "no" | "off" => Some(false),
            _ => None,
        }
    }

    fn set_value(&mut self, key: String, value: String) {
        self.data.insert(key, value);
    }

    fn has_key(&self, key: &str) -> bool {
        self.data.contains_key(key)
    }

    fn get_all_keys(&self) -> Vec<String> {
        self.data.keys().cloned().collect()
    }
}

/// 内存缓存实现
pub struct MemoryCache {
    data: HashMap<String, String>,
    stats: CacheStats,
}

impl MemoryCache {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
            stats: CacheStats {
                hits: 0,
                misses: 0,
                entries: 0,
                memory_used: 0,
            },
        }
    }
}

impl CacheInterface for MemoryCache {
    fn get(&self, key: &str) -> Option<String> {
        self.data.get(key).cloned()
    }

    fn set(&mut self, key: String, value: String, _ttl_seconds: Option<u64>) -> Result<(), SeparatedInterfaceError> {
        let is_new = !self.data.contains_key(&key);
        self.data.insert(key, value);
        
        if is_new {
            self.stats.entries += 1;
        }
        
        // 简化的内存使用计算
        self.stats.memory_used = self.data.iter()
            .map(|(k, v)| k.len() + v.len())
            .sum::<usize>() as u64;
        
        Ok(())
    }

    fn delete(&mut self, key: &str) -> Result<bool, SeparatedInterfaceError> {
        let removed = self.data.remove(key).is_some();
        if removed {
            self.stats.entries -= 1;
            self.stats.memory_used = self.data.iter()
                .map(|(k, v)| k.len() + v.len())
                .sum::<usize>() as u64;
        }
        Ok(removed)
    }

    fn exists(&self, key: &str) -> bool {
        self.data.contains_key(key)
    }

    fn clear(&mut self) -> Result<(), SeparatedInterfaceError> {
        self.data.clear();
        self.stats.entries = 0;
        self.stats.memory_used = 0;
        Ok(())
    }

    fn get_stats(&self) -> CacheStats {
        self.stats.clone()
    }
}

/// 服务提供者注册表
pub struct ServiceRegistry {
    logger_providers: HashMap<String, Box<dyn LoggerInterface>>,
    config_providers: HashMap<String, Box<dyn ConfigurationInterface>>,
    cache_providers: HashMap<String, Box<dyn CacheInterface>>,
}

impl ServiceRegistry {
    pub fn new() -> Self {
        Self {
            logger_providers: HashMap::new(),
            config_providers: HashMap::new(),
            cache_providers: HashMap::new(),
        }
    }

    pub fn register_logger(&mut self, name: String, logger: Box<dyn LoggerInterface>) {
        self.logger_providers.insert(name, logger);
    }

    pub fn register_config(&mut self, name: String, config: Box<dyn ConfigurationInterface>) {
        self.config_providers.insert(name, config);
    }

    pub fn register_cache(&mut self, name: String, cache: Box<dyn CacheInterface>) {
        self.cache_providers.insert(name, cache);
    }

    pub fn get_logger(&self, name: &str) -> Result<&dyn LoggerInterface, SeparatedInterfaceError> {
        self.logger_providers.get(name)
            .map(|logger| logger.as_ref())
            .ok_or_else(|| SeparatedInterfaceError::ProviderNotFound(format!("日志提供者 {} 未找到", name)))
    }

    pub fn get_config(&self, name: &str) -> Result<&dyn ConfigurationInterface, SeparatedInterfaceError> {
        self.config_providers.get(name)
            .map(|config| config.as_ref())
            .ok_or_else(|| SeparatedInterfaceError::ProviderNotFound(format!("配置提供者 {} 未找到", name)))
    }

    pub fn get_cache(&mut self, name: &str) -> Result<&mut Box<dyn CacheInterface>, SeparatedInterfaceError> {
        match self.cache_providers.get_mut(name) {
            Some(cache) => Ok(cache),
            None => Err(SeparatedInterfaceError::ProviderNotFound(format!("缓存提供者 {} 未找到", name)))
        }
    }

    pub fn list_providers(&self) -> ServiceProviderList {
        ServiceProviderList {
            loggers: self.logger_providers.keys().cloned().collect(),
            configs: self.config_providers.keys().cloned().collect(),
            caches: self.cache_providers.keys().cloned().collect(),
        }
    }
}

/// 服务提供者列表
#[derive(Debug)]
pub struct ServiceProviderList {
    pub loggers: Vec<String>,
    pub configs: Vec<String>,
    pub caches: Vec<String>,
}

impl Display for ServiceProviderList {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "注册的服务提供者:")?;
        writeln!(f, "  日志提供者: {:?}", self.loggers)?;
        writeln!(f, "  配置提供者: {:?}", self.configs)?;
        write!(f, "  缓存提供者: {:?}", self.caches)
    }
}

/// 使用分离接口的应用服务
pub struct ApplicationService {
    logger: Option<String>,
    config: Option<String>,
    cache: Option<String>,
}

impl ApplicationService {
    pub fn new() -> Self {
        Self {
            logger: None,
            config: None,
            cache: None,
        }
    }

    pub fn with_logger(mut self, logger_name: String) -> Self {
        self.logger = Some(logger_name);
        self
    }

    pub fn with_config(mut self, config_name: String) -> Self {
        self.config = Some(config_name);
        self
    }

    pub fn with_cache(mut self, cache_name: String) -> Self {
        self.cache = Some(cache_name);
        self
    }

    pub fn process_request(&self, registry: &mut ServiceRegistry, request_id: &str) 
        -> Result<String, SeparatedInterfaceError> {
        
        // 获取日志服务
        if let Some(logger_name) = &self.logger {
            let logger = registry.get_logger(logger_name)?;
            logger.log_info(&format!("开始处理请求: {}", request_id), "ApplicationService")?;
        }

        // 获取配置
        let mut app_name = "默认应用".to_string();
        if let Some(config_name) = &self.config {
            let config = registry.get_config(config_name)?;
            if let Some(name) = config.get_string("app.name") {
                app_name = name;
            }
        }

        // 检查缓存
        let cache_key = format!("request:{}", request_id);
        let mut from_cache = false;
        
        if let Some(cache_name) = &self.cache {
            let cache = registry.get_cache(cache_name)?;
            if let Some(cached_result) = cache.get(&cache_key) {
                if let Some(logger_name) = &self.logger {
                    let logger = registry.get_logger(logger_name)?;
                    logger.log_info(&format!("从缓存获取结果: {}", request_id), "ApplicationService")?;
                }
                return Ok(cached_result);
            }
        }

        // 处理业务逻辑
        let result = format!("处理完成 - 应用: {}, 请求ID: {}, 时间: 2024-01-01 12:00:00", 
                            app_name, request_id);

        // 存储到缓存
        if let Some(cache_name) = &self.cache {
            let cache = registry.get_cache(cache_name)?;
            cache.set(cache_key, result.clone(), Some(300))?; // 5分钟TTL
        }

        // 记录日志
        if let Some(logger_name) = &self.logger {
            let logger = registry.get_logger(logger_name)?;
            logger.log_info(&format!("请求处理完成: {}", request_id), "ApplicationService")?;
        }

        Ok(result)
    }
}

/// 演示分离接口模式
pub fn demo() {
    println!("=== 分离接口模式演示 ===\n");

    // 创建服务注册表
    let mut registry = ServiceRegistry::new();

    println!("1. 注册不同的服务实现");
    
    // 注册不同的日志实现
    registry.register_logger(
        "console".to_string(),
        Box::new(ConsoleLogger::new("控制台日志".to_string(), LogLevel::Info))
    );
    
    registry.register_logger(
        "file".to_string(),
        Box::new(FileLogger::new("文件日志".to_string(), LogLevel::Debug, "/var/log/app.log".to_string()))
    );

    // 注册配置实现
    registry.register_config(
        "memory".to_string(),
        Box::new(MemoryConfiguration::with_defaults())
    );

    // 注册缓存实现
    registry.register_cache(
        "memory".to_string(),
        Box::new(MemoryCache::new())
    );

    println!("   ✅ 服务提供者注册完成");
    println!("   {}", registry.list_providers());

    // 演示接口的可插拔性
    println!("\n2. 演示接口的可插拔性");
    
    // 使用控制台日志的应用服务
    println!("\n   📱 使用控制台日志:");
    let console_app = ApplicationService::new()
        .with_logger("console".to_string())
        .with_config("memory".to_string())
        .with_cache("memory".to_string());

    match console_app.process_request(&mut registry, "REQ-001") {
        Ok(result) => println!("     ✅ 处理结果: {}", result),
        Err(e) => println!("     ❌ 处理失败: {}", e),
    }

    // 使用文件日志的应用服务
    println!("\n   📁 使用文件日志:");
    let file_app = ApplicationService::new()
        .with_logger("file".to_string())
        .with_config("memory".to_string())
        .with_cache("memory".to_string());

    match file_app.process_request(&mut registry, "REQ-002") {
        Ok(result) => println!("     ✅ 处理结果: {}", result),
        Err(e) => println!("     ❌ 处理失败: {}", e),
    }

    // 演示缓存命中
    println!("\n3. 演示缓存命中");
    match console_app.process_request(&mut registry, "REQ-001") {
        Ok(result) => println!("   ✅ 缓存命中结果: {}", result),
        Err(e) => println!("   ❌ 缓存访问失败: {}", e),
    }

    // 演示不同日志级别
    println!("\n4. 演示不同日志级别");
    if let Ok(logger) = registry.get_logger("console") {
        println!("   🔧 控制台日志器:");
        let _ = logger.log_debug("这是调试信息", "Demo");
        let _ = logger.log_info("这是信息日志", "Demo");
        let _ = logger.log_warning("这是警告信息", "Demo");
        let _ = logger.log_error("这是错误信息", "Demo");
    }

    // 演示配置访问
    println!("\n5. 演示配置访问");
    if let Ok(config) = registry.get_config("memory") {
        println!("   🔧 配置信息:");
        println!("     应用名称: {:?}", config.get_string("app.name"));
        println!("     服务端口: {:?}", config.get_int("server.port"));
        println!("     数据库启用: {:?}", config.get_bool("database.enabled"));
        println!("     所有配置键: {:?}", config.get_all_keys());
    }

    // 演示缓存统计
    println!("\n6. 演示缓存统计");
    if let Ok(cache) = registry.get_cache("memory") {
        let stats = cache.get_stats();
        println!("   📊 缓存统计:");
        println!("     条目数: {}", stats.entries);
        println!("     内存使用: {} 字节", stats.memory_used);
        println!("     命中次数: {}", stats.hits);
        println!("     未命中次数: {}", stats.misses);
        println!("     命中率: {:.2}%", stats.hit_rate() * 100.0);
    }

    // 演示运行时切换实现
    println!("\n7. 演示运行时切换实现");
    let flexible_app = ApplicationService::new()
        .with_config("memory".to_string())
        .with_cache("memory".to_string());

    // 不使用日志
    println!("   🔇 无日志模式:");
    match flexible_app.process_request(&mut registry, "REQ-003") {
        Ok(result) => println!("     ✅ 处理结果: {}", result),
        Err(e) => println!("     ❌ 处理失败: {}", e),
    }

    // 仅使用缓存，不使用日志和配置
    println!("\n   💾 仅缓存模式:");
    let cache_only_app = ApplicationService::new()
        .with_cache("memory".to_string());

    match cache_only_app.process_request(&mut registry, "REQ-004") {
        Ok(result) => println!("     ✅ 处理结果: {}", result),
        Err(e) => println!("     ❌ 处理失败: {}", e),
    }

    println!("\n=== 分离接口模式演示完成 ===");

    println!("\n💡 分离接口模式的优势:");
    println!("1. 依赖隔离 - 客户端只依赖接口，不依赖具体实现");
    println!("2. 可插拔性 - 可以在运行时切换不同的实现");
    println!("3. 测试友好 - 容易创建Mock对象进行单元测试");
    println!("4. 独立部署 - 实现可以独立更新而不影响客户端");

    println!("\n⚠️ 设计考虑:");
    println!("1. 抽象成本 - 需要定义和维护额外的接口");
    println!("2. 运行时开销 - 动态分发可能有轻微性能影响");
    println!("3. 复杂性 - 增加了系统的抽象层次");
    println!("4. 版本管理 - 接口变更需要谨慎处理兼容性");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_console_logger() {
        let logger = ConsoleLogger::new("测试".to_string(), LogLevel::Info);
        assert_eq!(logger.get_name(), "测试");
        assert!(logger.is_enabled(&LogLevel::Info));
        assert!(!logger.is_enabled(&LogLevel::Debug));
        
        let result = logger.log_info("测试消息", "测试模块");
        assert!(result.is_ok());
    }

    #[test]
    fn test_memory_configuration() {
        let mut config = MemoryConfiguration::new();
        config.set_value("test.key".to_string(), "test.value".to_string());
        config.set_value("test.number".to_string(), "42".to_string());
        config.set_value("test.flag".to_string(), "true".to_string());
        
        assert_eq!(config.get_string("test.key"), Some("test.value".to_string()));
        assert_eq!(config.get_int("test.number"), Some(42));
        assert_eq!(config.get_bool("test.flag"), Some(true));
        assert!(config.has_key("test.key"));
    }

    #[test]
    fn test_memory_cache() {
        let mut cache = MemoryCache::new();
        
        // 测试设置和获取
        let result = cache.set("key1".to_string(), "value1".to_string(), None);
        assert!(result.is_ok());
        assert_eq!(cache.get("key1"), Some("value1".to_string()));
        
        // 测试删除
        let deleted = cache.delete("key1").unwrap();
        assert!(deleted);
        assert_eq!(cache.get("key1"), None);
    }

    #[test]
    fn test_service_registry() {
        let mut registry = ServiceRegistry::new();
        
        let logger = Box::new(ConsoleLogger::new("测试".to_string(), LogLevel::Info));
        registry.register_logger("test".to_string(), logger);
        
        let retrieved_logger = registry.get_logger("test");
        assert!(retrieved_logger.is_ok());
        assert_eq!(retrieved_logger.unwrap().get_name(), "测试");
    }

    #[test]
    fn test_application_service() {
        let mut registry = ServiceRegistry::new();
        
        registry.register_logger(
            "test".to_string(),
            Box::new(ConsoleLogger::new("测试".to_string(), LogLevel::Info))
        );
        registry.register_config(
            "test".to_string(),
            Box::new(MemoryConfiguration::with_defaults())
        );
        registry.register_cache(
            "test".to_string(),
            Box::new(MemoryCache::new())
        );
        
        let app = ApplicationService::new()
            .with_logger("test".to_string())
            .with_config("test".to_string())
            .with_cache("test".to_string());
        
        let result = app.process_request(&mut registry, "TEST-001");
        assert!(result.is_ok());
    }
} 