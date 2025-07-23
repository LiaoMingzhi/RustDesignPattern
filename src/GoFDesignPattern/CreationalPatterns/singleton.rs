//! 单例模式 (Singleton Pattern)
//! 
//! 保证一个类仅有一个实例，并提供一个访问它的全局访问点。
//! 文件路径：/d%3A/workspace/RustLearn/RustDesignPattern/src/GoFDesignPattern/CreationalPatterns/singleton.rs

use std::sync::{Arc, Mutex, OnceLock};
use std::collections::HashMap;
use std::cell::RefCell;

// 方法1: 使用 OnceLock (Rust 1.70+ 推荐方式)
static LOGGER_INSTANCE: OnceLock<Mutex<Logger>> = OnceLock::new();

// 单例类 - 日志记录器
#[derive(Debug)]
pub struct Logger {
    logs: Vec<String>,
}

impl Logger {
    // 私有构造函数
    fn new() -> Self {
        println!("创建Logger实例");
        Self {
            logs: Vec::new(),
        }
    }

    // 获取单例实例
    pub fn instance() -> &'static Mutex<Logger> {
        LOGGER_INSTANCE.get_or_init(|| {
            Mutex::new(Logger::new())
        })
    }

    pub fn log(&mut self, message: &str) {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let log_entry = format!("[{}] {}", timestamp, message);
        self.logs.push(log_entry.clone());
        println!("日志记录: {}", log_entry);
    }

    pub fn get_logs(&self) -> &Vec<String> {
        &self.logs
    }

    pub fn clear_logs(&mut self) {
        self.logs.clear();
        println!("日志已清空");
    }

    pub fn log_count(&self) -> usize {
        self.logs.len()
    }
}

// 方法2: 线程安全的单例 - 配置管理器
static CONFIG_MANAGER: OnceLock<Arc<Mutex<ConfigManager>>> = OnceLock::new();

#[derive(Debug)]
pub struct ConfigManager {
    settings: HashMap<String, String>,
}

impl ConfigManager {
    fn new() -> Self {
        println!("创建ConfigManager实例");
        let mut settings = HashMap::new();
        
        // 默认配置
        settings.insert("app_name".to_string(), "RustApp".to_string());
        settings.insert("version".to_string(), "1.0.0".to_string());
        settings.insert("debug".to_string(), "false".to_string());

        Self { settings }
    }

    pub fn instance() -> &'static Arc<Mutex<ConfigManager>> {
        CONFIG_MANAGER.get_or_init(|| {
            Arc::new(Mutex::new(ConfigManager::new()))
        })
    }

    pub fn get_setting(&self, key: &str) -> Option<String> {
        self.settings.get(key).cloned()
    }

    pub fn set_setting(&mut self, key: String, value: String) {
        println!("设置配置: {} = {}", key, value);
        self.settings.insert(key, value);
    }

    pub fn list_settings(&self) {
        println!("当前配置:");
        for (key, value) in &self.settings {
            println!("  {} = {}", key, value);
        }
    }

    pub fn remove_setting(&mut self, key: &str) -> Option<String> {
        if let Some(value) = self.settings.remove(key) {
            println!("移除配置: {}", key);
            Some(value)
        } else {
            println!("配置项不存在: {}", key);
            None
        }
    }
}

// 方法3: 使用OnceLock实现的单例 - 数据库连接池
static DATABASE_POOL: OnceLock<Mutex<DatabasePool>> = OnceLock::new();

#[derive(Debug)]
pub struct DatabasePool {
    connections: Vec<String>,
    max_connections: usize,
}

impl DatabasePool {
    fn new() -> Self {
        println!("创建DatabasePool实例");
        DatabasePool {
            connections: Vec::new(),
            max_connections: 10,
        }
    }

    pub fn instance() -> &'static Mutex<DatabasePool> {
        DATABASE_POOL.get_or_init(|| {
            Mutex::new(DatabasePool::new())
        })
    }

    pub fn get_connection(&mut self) -> Option<String> {
        if self.connections.len() < self.max_connections {
            let conn_id = format!("conn_{}", self.connections.len() + 1);
            self.connections.push(conn_id.clone());
            println!("获取数据库连接: {}", conn_id);
            Some(conn_id)
        } else {
            println!("连接池已满，无法获取新连接");
            None
        }
    }

    pub fn release_connection(&mut self, conn_id: &str) -> bool {
        if let Some(pos) = self.connections.iter().position(|x| x == conn_id) {
            self.connections.remove(pos);
            println!("释放数据库连接: {}", conn_id);
            true
        } else {
            println!("连接不存在: {}", conn_id);
            false
        }
    }

    pub fn get_active_connections(&self) -> usize {
        self.connections.len()
    }
}

// 方法4: 使用thread_local实现的单例（适用于单线程场景）
thread_local! {
    static LOCAL_CACHE: RefCell<HashMap<String, String>> = RefCell::new(HashMap::new());
}

pub struct ThreadLocalCache;

impl ThreadLocalCache {
    pub fn set(key: String, value: String) {
        LOCAL_CACHE.with(|cache| {
            cache.borrow_mut().insert(key.clone(), value.clone());
            println!("ThreadLocal缓存设置: {} = {}", key, value);
        });
    }

    pub fn get(key: &str) -> Option<String> {
        LOCAL_CACHE.with(|cache| {
            cache.borrow().get(key).cloned()
        })
    }

    pub fn clear() {
        LOCAL_CACHE.with(|cache| {
            cache.borrow_mut().clear();
            println!("ThreadLocal缓存已清空");
        });
    }

    pub fn size() -> usize {
        LOCAL_CACHE.with(|cache| {
            cache.borrow().len()
        })
    }
}

// 应用程序类 - 演示单例的使用
pub struct Application;

impl Application {
    pub fn run() {
        println!("应用程序启动...");

        // 使用日志记录器单例
        {
            let logger = Logger::instance();
            let mut logger_guard = logger.lock().unwrap();
            logger_guard.log("应用程序启动");
            logger_guard.log("初始化配置");
        }

        // 使用配置管理器单例
        {
            let config = ConfigManager::instance();
            let mut config_guard = config.lock().unwrap();
            config_guard.set_setting("max_users".to_string(), "1000".to_string());
            config_guard.list_settings();
        }

        // 使用数据库连接池单例
        {
            let db_pool = DatabasePool::instance();
            let mut pool_guard = db_pool.lock().unwrap();
            let conn1 = pool_guard.get_connection();
            let _conn2 = pool_guard.get_connection();
            
            println!("活动连接数: {}", pool_guard.get_active_connections());

            if let Some(conn_id) = conn1 {
                pool_guard.release_connection(&conn_id);
            }
        }

        // 使用线程本地缓存
        ThreadLocalCache::set("user_id".to_string(), "12345".to_string());
        ThreadLocalCache::set("session_token".to_string(), "abc123".to_string());
        
        if let Some(user_id) = ThreadLocalCache::get("user_id") {
            println!("ThreadLocal缓存读取: user_id = {}", user_id);
        }

        {
            let logger = Logger::instance();
            let mut logger_guard = logger.lock().unwrap();
            logger_guard.log("应用程序运行中");
            println!("总日志数: {}", logger_guard.log_count());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logger_singleton() {
        let logger1 = Logger::instance();
        let logger2 = Logger::instance();

        // 验证是同一个实例
        assert!(std::ptr::eq(logger1, logger2));

        {
            let mut logger_guard = logger1.lock().unwrap();
            logger_guard.log("测试日志1");
        }
        {
            let mut logger_guard = logger2.lock().unwrap();
            logger_guard.log("测试日志2");
        }

        // 验证状态共享
        {
            let logger_guard = logger1.lock().unwrap();
            assert!(logger_guard.log_count() >= 2);
        }
    }

    #[test]
    fn test_config_manager_singleton() {
        let config1 = ConfigManager::instance();
        let config2 = ConfigManager::instance();

        // 验证是同一个Arc
        assert!(Arc::ptr_eq(config1, config2));

        {
            let mut config_guard = config1.lock().unwrap();
            config_guard.set_setting("test_key".to_string(), "test_value".to_string());
        }

        {
            let config_guard = config2.lock().unwrap();
            assert_eq!(
                config_guard.get_setting("test_key"),
                Some("test_value".to_string())
            );
        }
    }

    #[test]
    fn test_database_pool_singleton() {
        let pool1 = DatabasePool::instance();
        let pool2 = DatabasePool::instance();

        // 验证是同一个实例
        assert!(std::ptr::eq(pool1, pool2));

        {
            let mut pool_guard = pool1.lock().unwrap();
            let conn = pool_guard.get_connection();
            assert!(conn.is_some());
        }
        
        {
            let pool_guard = pool2.lock().unwrap();
            assert_eq!(pool_guard.get_active_connections(), 1);
        }
    }

    #[test]
    fn test_thread_local_cache() {
        ThreadLocalCache::set("key1".to_string(), "value1".to_string());
        ThreadLocalCache::set("key2".to_string(), "value2".to_string());
        
        assert_eq!(ThreadLocalCache::get("key1"), Some("value1".to_string()));
        assert_eq!(ThreadLocalCache::get("key2"), Some("value2".to_string()));
        assert_eq!(ThreadLocalCache::get("key3"), None);
        assert_eq!(ThreadLocalCache::size(), 2);
        
        ThreadLocalCache::clear();
        assert_eq!(ThreadLocalCache::size(), 0);
    }
}

pub fn demo() {
    println!("=== 单例模式演示 ===");

    // 演示四种不同的单例实现
    println!("\n1. 日志记录器单例:");
    let logger1 = Logger::instance();
    let logger2 = Logger::instance();
    
    println!("logger1 和 logger2 是同一个实例: {}", 
             std::ptr::eq(logger1, logger2));

    {
        let mut logger_guard = logger1.lock().unwrap();
        logger_guard.log("第一条日志");
    }
    {
        let mut logger_guard = logger2.lock().unwrap();
        logger_guard.log("第二条日志");
    }
    
    {
        let logger_guard = logger1.lock().unwrap();
        println!("日志总数: {}", logger_guard.log_count());
    }

    println!("\n2. 配置管理器单例 (线程安全):");
    let config1 = ConfigManager::instance();
    let config2 = ConfigManager::instance();
    
    println!("config1 和 config2 是同一个Arc: {}", 
             Arc::ptr_eq(config1, config2));

    {
        let mut config = config1.lock().unwrap();
        config.set_setting("theme".to_string(), "dark".to_string());
        config.list_settings();
    }

    println!("\n3. 数据库连接池单例:");
    let pool = DatabasePool::instance();
    
    let connections: Vec<_> = {
        let mut pool_guard = pool.lock().unwrap();
        (0..3)
            .map(|_| pool_guard.get_connection())
            .collect()
    };
    
    {
        let pool_guard = pool.lock().unwrap();
        println!("当前活动连接数: {}", pool_guard.get_active_connections());
    }

    // 释放连接
    {
        let mut pool_guard = pool.lock().unwrap();
        for conn in connections.into_iter().flatten() {
            pool_guard.release_connection(&conn);
        }
        println!("释放后活动连接数: {}", pool_guard.get_active_connections());
    }

    println!("\n4. 线程本地单例缓存:");
    ThreadLocalCache::set("app_state".to_string(), "running".to_string());
    ThreadLocalCache::set("user_name".to_string(), "张三".to_string());
    
    if let Some(state) = ThreadLocalCache::get("app_state") {
        println!("应用状态: {}", state);
    }
    if let Some(name) = ThreadLocalCache::get("user_name") {
        println!("用户名: {}", name);
    }
    println!("缓存大小: {}", ThreadLocalCache::size());

    println!("\n5. 完整应用程序演示:");
    Application::run();

    println!("\n单例模式的优点:");
    println!("1. 确保一个类只有一个实例");
    println!("2. 提供全局访问点");
    println!("3. 延迟初始化，节省资源");
    println!("4. 控制实例数量");
    
    println!("\n在Rust中实现单例的现代方法:");
    println!("1. OnceLock: 线程安全的延迟初始化");
    println!("2. thread_local: 线程本地存储");
    println!("3. Arc<Mutex<T>>: 共享可变状态");
    println!("4. 避免使用static mut，提高安全性");
} 