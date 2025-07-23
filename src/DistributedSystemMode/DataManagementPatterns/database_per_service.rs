/*
 * 文件路径: /d%3A/workspace/RustLearn/RustDesignPattern/src/DistributedSystemMode/DataManagementPatterns/database_per_service.rs
 * 
 * Database per Service模式 (每服务一数据库)
 * 
 * 每个微服务拥有独立的数据库，避免共享数据库造成的耦合。
 * 这种模式保证了服务的独立性和可扩展性，但需要处理分布式数据一致性问题。
 * 
 * 主要特点：
 * 1. 数据隔离 - 每个服务拥有独立的数据存储
 * 2. 技术多样性 - 可以为不同服务选择最适合的数据库类型
 * 3. 独立扩展 - 每个数据库可以独立扩展
 * 4. 容错性 - 单个数据库故障不会影响其他服务
 * 5. 部署独立 - 数据库schema变更不会影响其他服务
 */

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};

// =================
// 数据库抽象层
// =================

/// 数据库操作结果
pub type DbResult<T> = Result<T, DatabaseError>;

/// 数据库错误类型
#[derive(Debug, Clone)]
pub enum DatabaseError {
    ConnectionFailed,
    QueryFailed(String),
    DataNotFound,
    ValidationError(String),
    TransactionFailed,
}

impl fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DatabaseError::ConnectionFailed => write!(f, "数据库连接失败"),
            DatabaseError::QueryFailed(msg) => write!(f, "查询失败: {}", msg),
            DatabaseError::DataNotFound => write!(f, "数据未找到"),
            DatabaseError::ValidationError(msg) => write!(f, "数据验证失败: {}", msg),
            DatabaseError::TransactionFailed => write!(f, "事务执行失败"),
        }
    }
}

/// 通用数据库接口
pub trait Database: Send + Sync {
    type Entity: Clone + Send + Sync;
    
    fn get(&self, id: &str) -> DbResult<Self::Entity>;
    fn save(&self, entity: &Self::Entity) -> DbResult<String>;
    fn update(&self, id: &str, entity: &Self::Entity) -> DbResult<()>;
    fn delete(&self, id: &str) -> DbResult<()>;
    fn find_all(&self) -> DbResult<Vec<Self::Entity>>;
    fn health_check(&self) -> bool;
}

// =================
// 用户服务数据库
// =================

#[derive(Debug, Clone)]
pub struct User {
    pub id: String,
    pub name: String,
    pub email: String,
    pub created_at: u64,
    pub status: UserStatus,
}

#[derive(Debug, Clone)]
pub enum UserStatus {
    Active,
    Inactive,
    Suspended,
}

/// 用户数据库 (关系型数据库模拟)
pub struct UserDatabase {
    data: Arc<RwLock<HashMap<String, User>>>,
    connection_pool_size: usize,
}

impl UserDatabase {
    pub fn new(pool_size: usize) -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
            connection_pool_size: pool_size,
        }
    }
    
    pub fn get_connection_info(&self) -> String {
        format!("PostgreSQL - 连接池大小: {}", self.connection_pool_size)
    }
}

impl Database for UserDatabase {
    type Entity = User;
    
    fn get(&self, id: &str) -> DbResult<Self::Entity> {
        let data = self.data.read().unwrap();
        data.get(id)
            .cloned()
            .ok_or(DatabaseError::DataNotFound)
    }
    
    fn save(&self, entity: &Self::Entity) -> DbResult<String> {
        if entity.name.is_empty() || entity.email.is_empty() {
            return Err(DatabaseError::ValidationError("姓名和邮箱不能为空".to_string()));
        }
        
        let mut data = self.data.write().unwrap();
        let id = format!("user_{}", data.len() + 1);
        let mut user = entity.clone();
        user.id = id.clone();
        user.created_at = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        
        data.insert(id.clone(), user);
        Ok(id)
    }
    
    fn update(&self, id: &str, entity: &Self::Entity) -> DbResult<()> {
        let mut data = self.data.write().unwrap();
        if data.contains_key(id) {
            let mut updated_user = entity.clone();
            updated_user.id = id.to_string();
            data.insert(id.to_string(), updated_user);
            Ok(())
        } else {
            Err(DatabaseError::DataNotFound)
        }
    }
    
    fn delete(&self, id: &str) -> DbResult<()> {
        let mut data = self.data.write().unwrap();
        data.remove(id)
            .map(|_| ())
            .ok_or(DatabaseError::DataNotFound)
    }
    
    fn find_all(&self) -> DbResult<Vec<Self::Entity>> {
        let data = self.data.read().unwrap();
        Ok(data.values().cloned().collect())
    }
    
    fn health_check(&self) -> bool {
        true
    }
}

// =================
// 订单服务数据库
// =================

#[derive(Debug, Clone)]
pub struct Order {
    pub id: String,
    pub user_id: String,
    pub product_id: String,
    pub quantity: u32,
    pub price: f64,
    pub status: OrderStatus,
    pub created_at: u64,
}

#[derive(Debug, Clone)]
pub enum OrderStatus {
    Pending,
    Confirmed,
    Shipped,
    Delivered,
    Cancelled,
}

/// 订单数据库 (文档数据库模拟)
pub struct OrderDatabase {
    data: Arc<RwLock<HashMap<String, Order>>>,
    collection_name: String,
}

impl OrderDatabase {
    pub fn new(collection: &str) -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
            collection_name: collection.to_string(),
        }
    }
    
    pub fn get_connection_info(&self) -> String {
        format!("MongoDB - 集合: {}", self.collection_name)
    }
    
    pub fn find_by_user_id(&self, user_id: &str) -> DbResult<Vec<Order>> {
        let data = self.data.read().unwrap();
        let orders: Vec<Order> = data.values()
            .filter(|order| order.user_id == user_id)
            .cloned()
            .collect();
        Ok(orders)
    }
}

impl Database for OrderDatabase {
    type Entity = Order;
    
    fn get(&self, id: &str) -> DbResult<Self::Entity> {
        let data = self.data.read().unwrap();
        data.get(id)
            .cloned()
            .ok_or(DatabaseError::DataNotFound)
    }
    
    fn save(&self, entity: &Self::Entity) -> DbResult<String> {
        if entity.user_id.is_empty() || entity.product_id.is_empty() {
            return Err(DatabaseError::ValidationError("用户ID和产品ID不能为空".to_string()));
        }
        
        let mut data = self.data.write().unwrap();
        let id = format!("order_{}", data.len() + 1);
        let mut order = entity.clone();
        order.id = id.clone();
        order.created_at = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        
        data.insert(id.clone(), order);
        Ok(id)
    }
    
    fn update(&self, id: &str, entity: &Self::Entity) -> DbResult<()> {
        let mut data = self.data.write().unwrap();
        if data.contains_key(id) {
            let mut updated_order = entity.clone();
            updated_order.id = id.to_string();
            data.insert(id.to_string(), updated_order);
            Ok(())
        } else {
            Err(DatabaseError::DataNotFound)
        }
    }
    
    fn delete(&self, id: &str) -> DbResult<()> {
        let mut data = self.data.write().unwrap();
        data.remove(id)
            .map(|_| ())
            .ok_or(DatabaseError::DataNotFound)
    }
    
    fn find_all(&self) -> DbResult<Vec<Self::Entity>> {
        let data = self.data.read().unwrap();
        Ok(data.values().cloned().collect())
    }
    
    fn health_check(&self) -> bool {
        true
    }
}

// =================
// 产品服务数据库
// =================

#[derive(Debug, Clone)]
pub struct Product {
    pub id: String,
    pub name: String,
    pub description: String,
    pub price: f64,
    pub stock: u32,
    pub category: String,
    pub tags: Vec<String>,
}

/// 产品数据库 (键值数据库模拟)
pub struct ProductDatabase {
    data: Arc<RwLock<HashMap<String, Product>>>,
    cache_ttl: u64,
}

impl ProductDatabase {
    pub fn new(ttl_seconds: u64) -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
            cache_ttl: ttl_seconds,
        }
    }
    
    pub fn get_connection_info(&self) -> String {
        format!("Redis - 缓存TTL: {}秒", self.cache_ttl)
    }
    
    pub fn find_by_category(&self, category: &str) -> DbResult<Vec<Product>> {
        let data = self.data.read().unwrap();
        let products: Vec<Product> = data.values()
            .filter(|product| product.category == category)
            .cloned()
            .collect();
        Ok(products)
    }
}

impl Database for ProductDatabase {
    type Entity = Product;
    
    fn get(&self, id: &str) -> DbResult<Self::Entity> {
        let data = self.data.read().unwrap();
        data.get(id)
            .cloned()
            .ok_or(DatabaseError::DataNotFound)
    }
    
    fn save(&self, entity: &Self::Entity) -> DbResult<String> {
        if entity.name.is_empty() || entity.price <= 0.0 {
            return Err(DatabaseError::ValidationError("产品名称不能为空，价格必须大于0".to_string()));
        }
        
        let mut data = self.data.write().unwrap();
        let id = format!("product_{}", data.len() + 1);
        let mut product = entity.clone();
        product.id = id.clone();
        
        data.insert(id.clone(), product);
        Ok(id)
    }
    
    fn update(&self, id: &str, entity: &Self::Entity) -> DbResult<()> {
        let mut data = self.data.write().unwrap();
        if data.contains_key(id) {
            let mut updated_product = entity.clone();
            updated_product.id = id.to_string();
            data.insert(id.to_string(), updated_product);
            Ok(())
        } else {
            Err(DatabaseError::DataNotFound)
        }
    }
    
    fn delete(&self, id: &str) -> DbResult<()> {
        let mut data = self.data.write().unwrap();
        data.remove(id)
            .map(|_| ())
            .ok_or(DatabaseError::DataNotFound)
    }
    
    fn find_all(&self) -> DbResult<Vec<Self::Entity>> {
        let data = self.data.read().unwrap();
        Ok(data.values().cloned().collect())
    }
    
    fn health_check(&self) -> bool {
        true
    }
}

// =================
// 服务管理器
// =================

/// 多数据库服务管理器
pub struct MultiDatabaseServiceManager {
    user_db: Arc<UserDatabase>,
    order_db: Arc<OrderDatabase>,
    product_db: Arc<ProductDatabase>,
}

impl MultiDatabaseServiceManager {
    pub fn new() -> Self {
        Self {
            user_db: Arc::new(UserDatabase::new(10)),
            order_db: Arc::new(OrderDatabase::new("orders")),
            product_db: Arc::new(ProductDatabase::new(3600)),
        }
    }
    
    pub fn get_user_service(&self) -> Arc<UserDatabase> {
        Arc::clone(&self.user_db)
    }
    
    pub fn get_order_service(&self) -> Arc<OrderDatabase> {
        Arc::clone(&self.order_db)
    }
    
    pub fn get_product_service(&self) -> Arc<ProductDatabase> {
        Arc::clone(&self.product_db)
    }
    
    pub fn health_check_all(&self) -> HashMap<String, bool> {
        let mut results = HashMap::new();
        results.insert("user_db".to_string(), self.user_db.health_check());
        results.insert("order_db".to_string(), self.order_db.health_check());
        results.insert("product_db".to_string(), self.product_db.health_check());
        results
    }
    
    pub fn get_database_info(&self) -> HashMap<String, String> {
        let mut info = HashMap::new();
        info.insert("user_db".to_string(), self.user_db.get_connection_info());
        info.insert("order_db".to_string(), self.order_db.get_connection_info());
        info.insert("product_db".to_string(), self.product_db.get_connection_info());
        info
    }
}

// =================
// 演示函数
// =================

/// Database per Service模式演示
pub fn demo_database_per_service() {
    println!("=== Database per Service模式演示 ===\n");
    
    let manager = MultiDatabaseServiceManager::new();
    println!("创建多数据库服务管理器");
    
    // 显示数据库连接信息
    println!("\n1. 数据库连接信息:");
    let db_info = manager.get_database_info();
    for (service, info) in db_info {
        println!("  {}: {}", service, info);
    }
    
    // 用户服务操作
    println!("\n2. 用户服务操作:");
    let user_db = manager.get_user_service();
    
    let user1 = User {
        id: String::new(),
        name: "张三".to_string(),
        email: "zhangsan@example.com".to_string(),
        created_at: 0,
        status: UserStatus::Active,
    };
    
    let user_id = user_db.save(&user1).unwrap();
    println!("创建用户: {}", user_id);
    
    let saved_user = user_db.get(&user_id).unwrap();
    println!("查询用户: {} - {}", saved_user.name, saved_user.email);
    
    // 产品服务操作
    println!("\n3. 产品服务操作:");
    let product_db = manager.get_product_service();
    
    let product1 = Product {
        id: String::new(),
        name: "智能手机".to_string(),
        description: "最新款智能手机".to_string(),
        price: 2999.0,
        stock: 100,
        category: "电子产品".to_string(),
        tags: vec!["手机".to_string(), "智能".to_string()],
    };
    
    let product_id = product_db.save(&product1).unwrap();
    println!("创建产品: {}", product_id);
    
    let saved_product = product_db.get(&product_id).unwrap();
    println!("查询产品: {} - ¥{}", saved_product.name, saved_product.price);
    
    // 订单服务操作
    println!("\n4. 订单服务操作:");
    let order_db = manager.get_order_service();
    
    let order1 = Order {
        id: String::new(),
        user_id: user_id.clone(),
        product_id: product_id.clone(),
        quantity: 1,
        price: 2999.0,
        status: OrderStatus::Pending,
        created_at: 0,
    };
    
    let order_id = order_db.save(&order1).unwrap();
    println!("创建订单: {}", order_id);
    
    // 健康检查
    println!("\n5. 数据库健康检查:");
    let health_status = manager.health_check_all();
    for (service, status) in health_status {
        println!("  {}: {}", service, if status { "✓ 健康" } else { "✗ 异常" });
    }
    
    println!("\n【Database per Service模式特点】");
    println!("✓ 数据隔离 - 每个服务拥有独立的数据存储");
    println!("✓ 技术多样性 - 可以为不同服务选择最适合的数据库类型");
    println!("✓ 独立扩展 - 每个数据库可以独立扩展");
    println!("✓ 容错性 - 单个数据库故障不会影响其他服务");
    println!("✓ 部署独立 - 数据库schema变更不会影响其他服务");
} 