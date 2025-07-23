/*
 * 文件路径: /d%3A/workspace/RustLearn/RustDesignPattern/src/EnterpriseAppPattern/BasePatterns/registry.rs
 * 
 * Registry（注册表）模式
 * 
 * 定义：
 * Registry模式提供一个全局访问点来获取系统中的对象和服务。
 * 它是一个众所周知的对象，其他对象可以用它来查找公共对象和服务。
 * 
 * 主要特点：
 * 1. 全局访问：提供系统级的对象访问点
 * 2. 延迟加载：对象可以在需要时才创建
 * 3. 单例保证：确保某些对象只有一个实例
 * 4. 解耦：避免硬编码的依赖关系
 * 5. 配置中心：集中管理系统配置
 * 
 * 适用场景：
 * - 需要全局访问某些服务或对象时
 * - 需要管理系统级配置时
 * - 需要实现服务定位器模式时
 * - 需要延迟初始化昂贵的对象时
 */

use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use std::any::{Any, TypeId};
use std::error::Error;
use std::fmt::{Display, Formatter};

/// Registry错误类型
#[derive(Debug)]
pub enum RegistryError {
    ServiceNotFound(String),
    ServiceAlreadyRegistered(String),
    TypeMismatch(String),
    InitializationError(String),
}

impl Display for RegistryError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            RegistryError::ServiceNotFound(msg) => write!(f, "服务未找到: {}", msg),
            RegistryError::ServiceAlreadyRegistered(msg) => write!(f, "服务已注册: {}", msg),
            RegistryError::TypeMismatch(msg) => write!(f, "类型不匹配: {}", msg),
            RegistryError::InitializationError(msg) => write!(f, "初始化错误: {}", msg),
        }
    }
}

impl Error for RegistryError {}

/// 服务生命周期枚举
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ServiceLifetime {
    Singleton,  // 单例
    Transient,  // 每次都创建新实例
    Scoped,     // 作用域内单例
}

/// 服务工厂trait
pub trait ServiceFactory: Send + Sync {
    fn create(&self) -> Result<Box<dyn Any + Send + Sync>, RegistryError>;
    fn lifetime(&self) -> ServiceLifetime;
}

/// 服务注册表
pub struct ServiceRegistry {
    services: RwLock<HashMap<TypeId, Arc<dyn ServiceFactory>>>,
    singletons: Mutex<HashMap<TypeId, Arc<dyn Any + Send + Sync>>>,
    scoped_instances: RwLock<HashMap<String, HashMap<TypeId, Arc<dyn Any + Send + Sync>>>>,
}

impl ServiceRegistry {
    /// 创建新的服务注册表
    pub fn new() -> Self {
        Self {
            services: RwLock::new(HashMap::new()),
            singletons: Mutex::new(HashMap::new()),
            scoped_instances: RwLock::new(HashMap::new()),
        }
    }
    
    /// 注册服务
    pub fn register<T: 'static + Send + Sync>(&self, factory: Arc<dyn ServiceFactory>) -> Result<(), RegistryError> {
        let type_id = TypeId::of::<T>();
        let mut services = self.services.write().unwrap();
        
        if services.contains_key(&type_id) {
            return Err(RegistryError::ServiceAlreadyRegistered(
                format!("类型 {:?} 已经注册", type_id)
            ));
        }
        
        services.insert(type_id, factory);
        Ok(())
    }
    
    /// 获取服务实例
    pub fn get<T: 'static + Send + Sync>(&self, scope: Option<&str>) -> Result<Arc<T>, RegistryError> {
        let type_id = TypeId::of::<T>();
        
        // 获取服务工厂
        let services = self.services.read().unwrap();
        let factory = services.get(&type_id)
            .ok_or_else(|| RegistryError::ServiceNotFound(
                format!("未找到类型 {:?} 的服务", type_id)
            ))?;
        
        match factory.lifetime() {
            ServiceLifetime::Singleton => self.get_singleton::<T>(factory.clone()),
            ServiceLifetime::Transient => self.create_transient::<T>(factory.clone()),
            ServiceLifetime::Scoped => self.get_scoped::<T>(factory.clone(), scope.unwrap_or("default")),
        }
    }
    
    /// 获取单例实例
    fn get_singleton<T: 'static + Send + Sync>(&self, factory: Arc<dyn ServiceFactory>) -> Result<Arc<T>, RegistryError> {
        let type_id = TypeId::of::<T>();
        
        // 先检查是否已经存在
        {
            let singletons = self.singletons.lock().unwrap();
            if let Some(instance) = singletons.get(&type_id) {
                return instance.clone().downcast::<T>()
                    .map_err(|_| RegistryError::TypeMismatch("类型转换失败".to_string()));
            }
        }
        
        // 创建新实例
        let instance = factory.create()?;
        let typed_instance = instance.downcast::<T>()
            .map_err(|_| RegistryError::TypeMismatch("类型转换失败".to_string()))?;
        
        // 转换为 Arc 以共享所有权
        let arc_instance: Arc<T> = Arc::from(typed_instance);
        
        // 存储单例
        let mut singletons = self.singletons.lock().unwrap();
        singletons.insert(type_id, arc_instance.clone());
        
        Ok(arc_instance)
    }
    
    /// 创建瞬态实例
    fn create_transient<T: 'static + Send + Sync>(&self, factory: Arc<dyn ServiceFactory>) -> Result<Arc<T>, RegistryError> {
        let instance = factory.create()?;
        let typed_instance = instance.downcast::<T>()
            .map_err(|_| RegistryError::TypeMismatch("类型转换失败".to_string()))?;
        
        // 转换为 Arc
        Ok(Arc::from(typed_instance))
    }
    
    /// 获取作用域实例
    fn get_scoped<T: 'static + Send + Sync>(&self, factory: Arc<dyn ServiceFactory>, scope: &str) -> Result<Arc<T>, RegistryError> {
        let type_id = TypeId::of::<T>();
        
        // 检查作用域是否已有实例
        {
            let scoped_instances = self.scoped_instances.read().unwrap();
            if let Some(scope_map) = scoped_instances.get(scope) {
                if let Some(instance) = scope_map.get(&type_id) {
                    return instance.clone().downcast::<T>()
                        .map_err(|_| RegistryError::TypeMismatch("类型转换失败".to_string()));
                }
            }
        }
        
        // 创建新实例
        let instance = factory.create()?;
        let typed_instance = instance.downcast::<T>()
            .map_err(|_| RegistryError::TypeMismatch("类型转换失败".to_string()))?;
        
        // 转换为 Arc 以共享所有权
        let arc_instance: Arc<T> = Arc::from(typed_instance);
        
        // 存储到作用域
        let mut scoped_instances = self.scoped_instances.write().unwrap();
        let scope_map = scoped_instances.entry(scope.to_string()).or_insert_with(HashMap::new);
        scope_map.insert(type_id, arc_instance.clone());
        
        Ok(arc_instance)
    }
    
    /// 清理作用域
    pub fn clear_scope(&self, scope: &str) {
        let mut scoped_instances = self.scoped_instances.write().unwrap();
        scoped_instances.remove(scope);
    }
    
    /// 获取已注册服务数量
    pub fn service_count(&self) -> usize {
        self.services.read().unwrap().len()
    }
}

impl Default for ServiceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// 全局服务注册表
static GLOBAL_REGISTRY: std::sync::OnceLock<ServiceRegistry> = std::sync::OnceLock::new();

/// 获取全局注册表
pub fn global_registry() -> &'static ServiceRegistry {
    GLOBAL_REGISTRY.get_or_init(ServiceRegistry::new)
}

/// 配置注册表 - 专门用于管理配置
pub struct ConfigRegistry {
    configs: RwLock<HashMap<String, String>>,
}

impl ConfigRegistry {
    pub fn new() -> Self {
        Self {
            configs: RwLock::new(HashMap::new()),
        }
    }
    
    /// 设置配置值
    pub fn set(&self, key: &str, value: &str) {
        let mut configs = self.configs.write().unwrap();
        configs.insert(key.to_string(), value.to_string());
    }
    
    /// 获取配置值
    pub fn get(&self, key: &str) -> Option<String> {
        let configs = self.configs.read().unwrap();
        configs.get(key).cloned()
    }
    
    /// 获取配置值或默认值
    pub fn get_or_default(&self, key: &str, default: &str) -> String {
        self.get(key).unwrap_or_else(|| default.to_string())
    }
    
    /// 获取整数配置
    pub fn get_int(&self, key: &str) -> Result<i32, RegistryError> {
        self.get(key)
            .ok_or_else(|| RegistryError::ServiceNotFound(format!("配置 {} 不存在", key)))?
            .parse()
            .map_err(|_| RegistryError::TypeMismatch("无法转换为整数".to_string()))
    }
    
    /// 获取布尔配置
    pub fn get_bool(&self, key: &str) -> Result<bool, RegistryError> {
        self.get(key)
            .ok_or_else(|| RegistryError::ServiceNotFound(format!("配置 {} 不存在", key)))?
            .parse()
            .map_err(|_| RegistryError::TypeMismatch("无法转换为布尔值".to_string()))
    }
    
    /// 批量设置配置
    pub fn set_batch(&self, configs: HashMap<String, String>) {
        let mut config_map = self.configs.write().unwrap();
        for (key, value) in configs {
            config_map.insert(key, value);
        }
    }
    
    /// 获取所有配置
    pub fn get_all(&self) -> HashMap<String, String> {
        self.configs.read().unwrap().clone()
    }
    
    /// 清空所有配置
    pub fn clear(&self) {
        let mut configs = self.configs.write().unwrap();
        configs.clear();
    }
}

impl Default for ConfigRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// 全局配置注册表
static GLOBAL_CONFIG: std::sync::OnceLock<ConfigRegistry> = std::sync::OnceLock::new();

/// 获取全局配置注册表
pub fn global_config() -> &'static ConfigRegistry {
    GLOBAL_CONFIG.get_or_init(ConfigRegistry::new)
}

/// 示例服务接口
pub trait DatabaseService: Send + Sync {
    fn connect(&self) -> Result<String, String>;
    fn execute_query(&self, sql: &str) -> Result<Vec<String>, String>;
}

/// MySQL数据库服务实现
pub struct MySqlDatabaseService {
    connection_string: String,
}

impl MySqlDatabaseService {
    pub fn new(connection_string: String) -> Self {
        Self { connection_string }
    }
}

impl DatabaseService for MySqlDatabaseService {
    fn connect(&self) -> Result<String, String> {
        println!("连接到MySQL数据库: {}", self.connection_string);
        Ok(format!("MySQL连接已建立: {}", self.connection_string))
    }
    
    fn execute_query(&self, sql: &str) -> Result<Vec<String>, String> {
        println!("执行MySQL查询: {}", sql);
        Ok(vec![format!("MySQL结果: {}", sql)])
    }
}

/// PostgreSQL数据库服务实现
pub struct PostgreSqlDatabaseService {
    host: String,
    port: u16,
}

impl PostgreSqlDatabaseService {
    pub fn new(host: String, port: u16) -> Self {
        Self { host, port }
    }
}

impl DatabaseService for PostgreSqlDatabaseService {
    fn connect(&self) -> Result<String, String> {
        println!("连接到PostgreSQL数据库: {}:{}", self.host, self.port);
        Ok(format!("PostgreSQL连接已建立: {}:{}", self.host, self.port))
    }
    
    fn execute_query(&self, sql: &str) -> Result<Vec<String>, String> {
        println!("执行PostgreSQL查询: {}", sql);
        Ok(vec![format!("PostgreSQL结果: {}", sql)])
    }
}

/// 数据库服务工厂
pub struct DatabaseServiceFactory {
    db_type: String,
    lifetime: ServiceLifetime,
}

impl DatabaseServiceFactory {
    pub fn new(db_type: String, lifetime: ServiceLifetime) -> Self {
        Self { db_type, lifetime }
    }
}

impl ServiceFactory for DatabaseServiceFactory {
    fn create(&self) -> Result<Box<dyn Any + Send + Sync>, RegistryError> {
        let service: Box<dyn Any + Send + Sync> = match self.db_type.as_str() {
            "mysql" => {
                let connection_string = global_config()
                    .get("mysql.connection_string")
                    .unwrap_or_else(|| "mysql://localhost:3306/test".to_string());
                Box::new(MySqlDatabaseService::new(connection_string))
            }
            "postgresql" => {
                let host = global_config().get_or_default("postgres.host", "localhost");
                let port: u16 = global_config().get_int("postgres.port")
                    .unwrap_or(5432) as u16;
                Box::new(PostgreSqlDatabaseService::new(host, port))
            }
            _ => return Err(RegistryError::InitializationError(
                format!("不支持的数据库类型: {}", self.db_type)
            )),
        };
        
        Ok(service)
    }
    
    fn lifetime(&self) -> ServiceLifetime {
        self.lifetime
    }
}

/// 缓存服务接口
pub trait CacheService: Send + Sync {
    fn get(&self, key: &str) -> Option<String>;
    fn set(&self, key: &str, value: &str);
    fn delete(&self, key: &str);
}

/// Redis缓存服务实现
pub struct RedisCacheService {
    host: String,
    port: u16,
    cache: Arc<Mutex<HashMap<String, String>>>,
}

impl RedisCacheService {
    pub fn new(host: String, port: u16) -> Self {
        Self {
            host,
            port,
            cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl CacheService for RedisCacheService {
    fn get(&self, key: &str) -> Option<String> {
        let cache = self.cache.lock().unwrap();
        cache.get(key).cloned()
    }
    
    fn set(&self, key: &str, value: &str) {
        let mut cache = self.cache.lock().unwrap();
        cache.insert(key.to_string(), value.to_string());
        println!("Redis设置 {}:{} -> {}", self.host, self.port, key);
    }
    
    fn delete(&self, key: &str) {
        let mut cache = self.cache.lock().unwrap();
        cache.remove(key);
        println!("Redis删除 {}:{} -> {}", self.host, self.port, key);
    }
}

/// 缓存服务工厂
pub struct CacheServiceFactory {
    lifetime: ServiceLifetime,
}

impl CacheServiceFactory {
    pub fn new(lifetime: ServiceLifetime) -> Self {
        Self { lifetime }
    }
}

impl ServiceFactory for CacheServiceFactory {
    fn create(&self) -> Result<Box<dyn Any + Send + Sync>, RegistryError> {
        let host = global_config().get_or_default("redis.host", "localhost");
        let port: u16 = global_config().get_int("redis.port")
            .unwrap_or(6379) as u16;
        
        let service: Box<dyn Any + Send + Sync> = Box::new(RedisCacheService::new(host, port));
        Ok(service)
    }
    
    fn lifetime(&self) -> ServiceLifetime {
        self.lifetime
    }
}

/// Registry模式演示
pub fn demo() {
    println!("=== Registry（注册表）模式演示 ===\n");
    
    // 1. 配置注册表演示
    println!("1. 配置注册表演示:");
    let config = global_config();
    
    // 设置配置
    config.set("mysql.connection_string", "mysql://user:pass@localhost:3306/mydb");
    config.set("postgres.host", "db.example.com");
    config.set("postgres.port", "5432");
    config.set("redis.host", "cache.example.com");
    config.set("redis.port", "6379");
    config.set("app.debug", "true");
    config.set("app.max_connections", "100");
    
    println!("MySQL连接字符串: {}", config.get("mysql.connection_string").unwrap());
    println!("PostgreSQL主机: {}", config.get("postgres.host").unwrap());
    println!("Redis端口: {}", config.get_int("redis.port").unwrap());
    println!("调试模式: {}", config.get_bool("app.debug").unwrap());
    println!("不存在的配置（使用默认值）: {}", config.get_or_default("app.timeout", "30"));
    
    println!("{}", "=".repeat(50));
    
    // 2. 服务注册表演示
    println!("2. 服务注册表演示:");
    let registry = global_registry();
    
    // 注册数据库服务（单例）
    let db_factory = Arc::new(DatabaseServiceFactory::new(
        "mysql".to_string(),
        ServiceLifetime::Singleton
    ));
    registry.register::<MySqlDatabaseService>(db_factory).unwrap();
    
    // 注册缓存服务（瞬态）
    let cache_factory = Arc::new(CacheServiceFactory::new(ServiceLifetime::Transient));
    registry.register::<RedisCacheService>(cache_factory).unwrap();
    
    println!("已注册服务数量: {}", registry.service_count());
    
    // 获取数据库服务（单例）
    println!("\n获取数据库服务:");
    let db_service1: Arc<MySqlDatabaseService> = registry.get(None).unwrap();
    let db_service2: Arc<MySqlDatabaseService> = registry.get(None).unwrap();
    
    match db_service1.connect() {
        Ok(conn) => println!("数据库服务1: {}", conn),
        Err(e) => println!("连接失败: {}", e),
    }
    
    let query_result = db_service2.execute_query("SELECT * FROM users");
    match query_result {
        Ok(results) => println!("查询结果: {:?}", results),
        Err(e) => println!("查询失败: {}", e),
    }
    
    // 验证单例
    println!("数据库服务是否为同一实例: {}", 
             Arc::ptr_eq(&db_service1, &db_service2));
    
    // 获取缓存服务（瞬态）
    println!("\n获取缓存服务:");
    let cache_service1: Arc<RedisCacheService> = registry.get(None).unwrap();
    let cache_service2: Arc<RedisCacheService> = registry.get(None).unwrap();
    
    cache_service1.set("user:123", "John Doe");
    if let Some(value) = cache_service1.get("user:123") {
        println!("缓存值: {}", value);
    }
    
    // 验证瞬态
    println!("缓存服务是否为同一实例: {}", 
             Arc::ptr_eq(&cache_service1, &cache_service2));
    
    println!("{}", "=".repeat(50));
    
    // 3. 作用域服务演示
    println!("3. 作用域服务演示:");
    
    // 注册作用域服务
    let scoped_db_factory = Arc::new(DatabaseServiceFactory::new(
        "postgresql".to_string(),
        ServiceLifetime::Scoped
    ));
    registry.register::<PostgreSqlDatabaseService>(scoped_db_factory).unwrap();
    
    // 在请求作用域1中获取服务
    let db_in_request1_a: Arc<PostgreSqlDatabaseService> = registry.get(Some("request-1")).unwrap();
    let db_in_request1_b: Arc<PostgreSqlDatabaseService> = registry.get(Some("request-1")).unwrap();
    
    // 在请求作用域2中获取服务
    let db_in_request2: Arc<PostgreSqlDatabaseService> = registry.get(Some("request-2")).unwrap();
    
    println!("请求1中的两个实例是否相同: {}", 
             Arc::ptr_eq(&db_in_request1_a, &db_in_request1_b));
    println!("请求1和请求2的实例是否相同: {}", 
             Arc::ptr_eq(&db_in_request1_a, &db_in_request2));
    
    // 清理作用域
    registry.clear_scope("request-1");
    println!("已清理请求1的作用域");
    
    println!("{}", "=".repeat(50));
    
    // 4. 错误处理演示
    println!("4. 错误处理演示:");
    
    // 尝试获取未注册的服务
    match registry.get::<String>(None) {
        Ok(_) => println!("不应该成功"),
        Err(e) => println!("预期错误: {}", e),
    }
    
    // 尝试重复注册服务
    let duplicate_factory = Arc::new(DatabaseServiceFactory::new(
        "mysql".to_string(),
        ServiceLifetime::Singleton
    ));
    match registry.register::<MySqlDatabaseService>(duplicate_factory) {
        Ok(_) => println!("不应该成功"),
        Err(e) => println!("预期错误: {}", e),
    }
    
    // 尝试获取无效的配置类型
    config.set("invalid.number", "not_a_number");
    match config.get_int("invalid.number") {
        Ok(_) => println!("不应该成功"),
        Err(e) => println!("预期错误: {}", e),
    }
    
    println!("\n=== Registry模式演示完成 ===");
    
    // 输出模式总结
    println!("\n【Registry模式总结】");
    println!("优点:");
    println!("1. 全局访问：提供系统级的对象访问点");
    println!("2. 解耦：避免硬编码的依赖关系");
    println!("3. 灵活配置：可以动态注册和获取服务");
    println!("4. 生命周期管理：支持单例、瞬态、作用域等模式");
    println!("5. 延迟初始化：对象在需要时才创建");
    
    println!("\n缺点:");
    println!("1. 全局状态：可能导致隐式依赖");
    println!("2. 测试困难：全局状态难以隔离测试");
    println!("3. 类型安全：运行时类型检查可能失败");
    
    println!("\n适用场景:");
    println!("1. 需要全局访问服务或配置时");
    println!("2. 实现依赖注入容器时");
    println!("3. 需要管理对象生命周期时");
    println!("4. 需要延迟初始化昂贵对象时");
} 