//! 数据库会话状态模式（Database Session State）
//! 
//! 数据库会话状态模式将会话数据存储在数据库中，适用于分布式环境和高可用性要求的场景。
//! 它提供了持久性的会话存储，支持服务器故障恢复和负载均衡环境下的会话共享。
//! 
//! 文件位置：/d%3A/workspace/RustLearn/RustDesignPattern/src/EnterpriseAppPattern/SessionStatePatterns/database_session_state.rs

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use std::fmt;

// =================
// 会话数据结构
// =================

/// 会话数据值类型
#[derive(Debug, Clone)]
pub enum SessionValue {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Binary(Vec<u8>),
    Json(String),
}

impl SessionValue {
    pub fn as_string(&self) -> Option<String> {
        match self {
            SessionValue::String(s) => Some(s.clone()),
            SessionValue::Integer(i) => Some(i.to_string()),
            SessionValue::Float(f) => Some(f.to_string()),
            SessionValue::Boolean(b) => Some(b.to_string()),
            SessionValue::Json(j) => Some(j.clone()),
            _ => None,
        }
    }
    
    pub fn as_integer(&self) -> Option<i64> {
        match self {
            SessionValue::Integer(i) => Some(*i),
            SessionValue::String(s) => s.parse().ok(),
            _ => None,
        }
    }
    
    pub fn as_boolean(&self) -> Option<bool> {
        match self {
            SessionValue::Boolean(b) => Some(*b),
            SessionValue::String(s) => match s.as_str() {
                "true" | "1" => Some(true),
                "false" | "0" => Some(false),
                _ => None,
            },
            SessionValue::Integer(i) => Some(*i != 0),
            _ => None,
        }
    }
}

impl fmt::Display for SessionValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SessionValue::String(s) => write!(f, "{}", s),
            SessionValue::Integer(i) => write!(f, "{}", i),
            SessionValue::Float(fl) => write!(f, "{}", fl),
            SessionValue::Boolean(b) => write!(f, "{}", b),
            SessionValue::Binary(data) => write!(f, "Binary({} bytes)", data.len()),
            SessionValue::Json(j) => write!(f, "{}", j),
        }
    }
}

/// 会话记录
#[derive(Debug, Clone)]
pub struct SessionRecord {
    pub session_id: String,
    pub user_id: Option<String>,
    pub data: HashMap<String, SessionValue>,
    pub created_at: u64,
    pub last_accessed: u64,
    pub expires_at: u64,
    pub ip_address: String,
    pub user_agent: String,
    pub is_active: bool,
}

impl SessionRecord {
    pub fn new(session_id: String, duration_seconds: u64) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        Self {
            session_id,
            user_id: None,
            data: HashMap::new(),
            created_at: now,
            last_accessed: now,
            expires_at: now + duration_seconds,
            ip_address: "127.0.0.1".to_string(),
            user_agent: "Unknown".to_string(),
            is_active: true,
        }
    }
    
    pub fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        now > self.expires_at
    }
    
    pub fn touch(&mut self, extend_seconds: u64) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        self.last_accessed = now;
        self.expires_at = now + extend_seconds;
    }
    
    pub fn set_value(&mut self, key: String, value: SessionValue) {
        self.data.insert(key, value);
    }
    
    pub fn get_value(&self, key: &str) -> Option<&SessionValue> {
        self.data.get(key)
    }
    
    pub fn remove_value(&mut self, key: &str) -> Option<SessionValue> {
        self.data.remove(key)
    }
    
    pub fn size(&self) -> usize {
        self.data.len()
    }
}

// =================
// 数据库接口
// =================

/// 会话数据库接口
pub trait SessionDatabase {
    type Error;
    
    /// 创建会话记录
    fn create_session(&mut self, session: SessionRecord) -> Result<(), Self::Error>;
    
    /// 根据ID获取会话
    fn get_session(&self, session_id: &str) -> Result<Option<SessionRecord>, Self::Error>;
    
    /// 更新会话记录
    fn update_session(&mut self, session: SessionRecord) -> Result<(), Self::Error>;
    
    /// 删除会话记录
    fn delete_session(&mut self, session_id: &str) -> Result<(), Self::Error>;
    
    /// 查找用户的所有会话
    fn get_user_sessions(&self, user_id: &str) -> Result<Vec<SessionRecord>, Self::Error>;
    
    /// 清理过期会话
    fn cleanup_expired_sessions(&mut self) -> Result<usize, Self::Error>;
    
    /// 获取活跃会话数量
    fn get_active_session_count(&self) -> Result<usize, Self::Error>;
    
    /// 根据IP地址查找会话
    fn get_sessions_by_ip(&self, ip_address: &str) -> Result<Vec<SessionRecord>, Self::Error>;
}

/// 内存数据库实现（模拟真实数据库）
pub struct InMemorySessionDatabase {
    sessions: Arc<Mutex<HashMap<String, SessionRecord>>>,
    user_sessions: Arc<Mutex<HashMap<String, Vec<String>>>>, // user_id -> session_ids
}

impl InMemorySessionDatabase {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            user_sessions: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[derive(Debug)]
pub enum DatabaseError {
    SessionNotFound,
    ConnectionError,
    SerializationError,
    ConstraintViolation(String),
}

impl fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DatabaseError::SessionNotFound => write!(f, "会话未找到"),
            DatabaseError::ConnectionError => write!(f, "数据库连接错误"),
            DatabaseError::SerializationError => write!(f, "序列化错误"),
            DatabaseError::ConstraintViolation(msg) => write!(f, "约束违反: {}", msg),
        }
    }
}

impl SessionDatabase for InMemorySessionDatabase {
    type Error = DatabaseError;
    
    fn create_session(&mut self, session: SessionRecord) -> Result<(), Self::Error> {
        let mut sessions = self.sessions.lock().unwrap();
        let mut user_sessions = self.user_sessions.lock().unwrap();
        
        sessions.insert(session.session_id.clone(), session.clone());
        
        if let Some(user_id) = &session.user_id {
            user_sessions
                .entry(user_id.clone())
                .or_insert_with(Vec::new)
                .push(session.session_id.clone());
        }
        
        Ok(())
    }
    
    fn get_session(&self, session_id: &str) -> Result<Option<SessionRecord>, Self::Error> {
        let sessions = self.sessions.lock().unwrap();
        Ok(sessions.get(session_id).cloned())
    }
    
    fn update_session(&mut self, session: SessionRecord) -> Result<(), Self::Error> {
        let mut sessions = self.sessions.lock().unwrap();
        
        if sessions.contains_key(&session.session_id) {
            sessions.insert(session.session_id.clone(), session);
            Ok(())
        } else {
            Err(DatabaseError::SessionNotFound)
        }
    }
    
    fn delete_session(&mut self, session_id: &str) -> Result<(), Self::Error> {
        let mut sessions = self.sessions.lock().unwrap();
        let mut user_sessions = self.user_sessions.lock().unwrap();
        
        if let Some(session) = sessions.remove(session_id) {
            if let Some(user_id) = &session.user_id {
                if let Some(session_ids) = user_sessions.get_mut(user_id) {
                    session_ids.retain(|id| id != session_id);
                    if session_ids.is_empty() {
                        user_sessions.remove(user_id);
                    }
                }
            }
            Ok(())
        } else {
            Err(DatabaseError::SessionNotFound)
        }
    }
    
    fn get_user_sessions(&self, user_id: &str) -> Result<Vec<SessionRecord>, Self::Error> {
        let sessions = self.sessions.lock().unwrap();
        let user_sessions = self.user_sessions.lock().unwrap();
        
        let mut result = Vec::new();
        if let Some(session_ids) = user_sessions.get(user_id) {
            for session_id in session_ids {
                if let Some(session) = sessions.get(session_id) {
                    result.push(session.clone());
                }
            }
        }
        
        Ok(result)
    }
    
    fn cleanup_expired_sessions(&mut self) -> Result<usize, Self::Error> {
        let mut sessions = self.sessions.lock().unwrap();
        let mut user_sessions = self.user_sessions.lock().unwrap();
        
        let mut expired_sessions = Vec::new();
        for (session_id, session) in sessions.iter() {
            if session.is_expired() {
                expired_sessions.push(session_id.clone());
            }
        }
        
        let count = expired_sessions.len();
        for session_id in expired_sessions {
            if let Some(session) = sessions.remove(&session_id) {
                if let Some(user_id) = &session.user_id {
                    if let Some(session_ids) = user_sessions.get_mut(user_id) {
                        session_ids.retain(|id| id != &session_id);
                        if session_ids.is_empty() {
                            user_sessions.remove(user_id);
                        }
                    }
                }
            }
        }
        
        Ok(count)
    }
    
    fn get_active_session_count(&self) -> Result<usize, Self::Error> {
        let sessions = self.sessions.lock().unwrap();
        let count = sessions.values()
            .filter(|session| session.is_active && !session.is_expired())
            .count();
        Ok(count)
    }
    
    fn get_sessions_by_ip(&self, ip_address: &str) -> Result<Vec<SessionRecord>, Self::Error> {
        let sessions = self.sessions.lock().unwrap();
        let result = sessions.values()
            .filter(|session| session.ip_address == ip_address)
            .cloned()
            .collect();
        Ok(result)
    }
}

// =================
// 数据库会话管理器
// =================

/// 数据库会话管理器
pub struct DatabaseSessionManager<DB>
where
    DB: SessionDatabase<Error = DatabaseError>,
{
    database: DB,
    default_timeout: u64,
    max_sessions_per_user: usize,
    cleanup_interval: Duration,
    last_cleanup: SystemTime,
}

impl<DB> DatabaseSessionManager<DB>
where
    DB: SessionDatabase<Error = DatabaseError>,
{
    pub fn new(database: DB) -> Self {
        Self {
            database,
            default_timeout: 3600, // 1小时
            max_sessions_per_user: 5,
            cleanup_interval: Duration::from_secs(300), // 5分钟
            last_cleanup: SystemTime::now(),
        }
    }
    
    pub fn with_timeout(mut self, timeout_seconds: u64) -> Self {
        self.default_timeout = timeout_seconds;
        self
    }
    
    pub fn with_max_sessions_per_user(mut self, max_sessions: usize) -> Self {
        self.max_sessions_per_user = max_sessions;
        self
    }
    
    /// 创建新会话
    pub fn create_session(&mut self, user_id: Option<String>, ip_address: String, user_agent: String) -> Result<String, DB::Error> {
        let session_id = generate_session_id();
        let mut session = SessionRecord::new(session_id.clone(), self.default_timeout);
        
        session.user_id = user_id.clone();
        session.ip_address = ip_address;
        session.user_agent = user_agent;
        
        // 检查用户会话数量限制
        if let Some(ref uid) = user_id {
            let user_sessions = self.database.get_user_sessions(uid)?;
            if user_sessions.len() >= self.max_sessions_per_user {
                // 删除最旧的会话
                if let Some(oldest_session) = user_sessions.iter().min_by_key(|s| s.created_at) {
                    self.database.delete_session(&oldest_session.session_id)?;
                }
            }
        }
        
        self.database.create_session(session)?;
        self.maybe_cleanup();
        
        Ok(session_id)
    }
    
    /// 获取会话
    pub fn get_session(&mut self, session_id: &str) -> Result<Option<SessionRecord>, DB::Error> {
        if let Some(mut session) = self.database.get_session(session_id)? {
            if session.is_expired() {
                self.database.delete_session(session_id)?;
                Ok(None)
            } else {
                // 更新访问时间
                session.touch(self.default_timeout);
                self.database.update_session(session.clone())?;
                Ok(Some(session))
            }
        } else {
            Ok(None)
        }
    }
    
    /// 设置会话值
    pub fn set_session_value(&mut self, session_id: &str, key: String, value: SessionValue) -> Result<(), DB::Error> {
        if let Some(mut session) = self.database.get_session(session_id)? {
            if !session.is_expired() {
                session.set_value(key, value);
                session.touch(self.default_timeout);
                self.database.update_session(session)?;
                Ok(())
            } else {
                self.database.delete_session(session_id)?;
                Err(DatabaseError::SessionNotFound)
            }
        } else {
            Err(DatabaseError::SessionNotFound)
        }
    }
    
    /// 获取会话值
    pub fn get_session_value(&mut self, session_id: &str, key: &str) -> Result<Option<SessionValue>, DB::Error> {
        if let Some(session) = self.get_session(session_id)? {
            Ok(session.get_value(key).cloned())
        } else {
            Ok(None)
        }
    }
    
    /// 删除会话值
    pub fn remove_session_value(&mut self, session_id: &str, key: &str) -> Result<Option<SessionValue>, DB::Error> {
        if let Some(mut session) = self.database.get_session(session_id)? {
            if !session.is_expired() {
                let result = session.remove_value(key);
                session.touch(self.default_timeout);
                self.database.update_session(session)?;
                Ok(result)
            } else {
                self.database.delete_session(session_id)?;
                Err(DatabaseError::SessionNotFound)
            }
        } else {
            Err(DatabaseError::SessionNotFound)
        }
    }
    
    /// 销毁会话
    pub fn destroy_session(&mut self, session_id: &str) -> Result<(), DB::Error> {
        self.database.delete_session(session_id)
    }
    
    /// 销毁用户的所有会话
    pub fn destroy_user_sessions(&mut self, user_id: &str) -> Result<usize, DB::Error> {
        let sessions = self.database.get_user_sessions(user_id)?;
        let count = sessions.len();
        
        for session in sessions {
            self.database.delete_session(&session.session_id)?;
        }
        
        Ok(count)
    }
    
    /// 获取会话统计信息
    pub fn get_session_stats(&self) -> Result<SessionStats, DB::Error> {
        let active_count = self.database.get_active_session_count()?;
        
        Ok(SessionStats {
            active_sessions: active_count,
            total_sessions: active_count, // 简化实现
        })
    }
    
    /// 根据IP地址获取会话
    pub fn get_sessions_by_ip(&self, ip_address: &str) -> Result<Vec<SessionRecord>, DB::Error> {
        self.database.get_sessions_by_ip(ip_address)
    }
    
    /// 手动清理过期会话
    pub fn cleanup_expired_sessions(&mut self) -> Result<usize, DB::Error> {
        let count = self.database.cleanup_expired_sessions()?;
        self.last_cleanup = SystemTime::now();
        Ok(count)
    }
    
    /// 如果需要的话执行清理
    fn maybe_cleanup(&mut self) {
        if self.last_cleanup.elapsed().unwrap_or(Duration::ZERO) > self.cleanup_interval {
            let _ = self.cleanup_expired_sessions();
        }
    }
}

// =================
// 会话统计信息
// =================

#[derive(Debug)]
pub struct SessionStats {
    pub active_sessions: usize,
    pub total_sessions: usize,
}

impl fmt::Display for SessionStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "活跃会话: {}, 总会话: {}", self.active_sessions, self.total_sessions)
    }
}

// =================
// 购物车会话示例
// =================

/// 购物车会话服务
pub struct ShoppingCartSessionService<DB>
where
    DB: SessionDatabase<Error = DatabaseError>,
{
    session_manager: DatabaseSessionManager<DB>,
}

impl<DB> ShoppingCartSessionService<DB>
where
    DB: SessionDatabase<Error = DatabaseError>,
{
    pub fn new(database: DB) -> Self {
        Self {
            session_manager: DatabaseSessionManager::new(database)
                .with_timeout(7200) // 2小时
                .with_max_sessions_per_user(3),
        }
    }
    
    /// 创建购物会话
    pub fn create_shopping_session(&mut self, user_id: Option<String>, ip_address: String) -> Result<String, DB::Error> {
        let session_id = self.session_manager.create_session(
            user_id, 
            ip_address, 
            "Shopping Cart Client".to_string()
        )?;
        
        // 初始化购物车
        self.session_manager.set_session_value(
            &session_id,
            "cart_items".to_string(),
            SessionValue::Json("[]".to_string())
        )?;
        
        self.session_manager.set_session_value(
            &session_id,
            "cart_total".to_string(),
            SessionValue::Float(0.0)
        )?;
        
        Ok(session_id)
    }
    
    /// 添加商品到购物车
    pub fn add_to_cart(&mut self, session_id: &str, product_id: String, quantity: i64, price: f64) -> Result<(), DB::Error> {
        // 获取当前购物车
        let cart_json = self.session_manager.get_session_value(session_id, "cart_items")?
            .and_then(|v| v.as_string())
            .unwrap_or_else(|| "[]".to_string());
        
        let current_total = self.session_manager.get_session_value(session_id, "cart_total")?
            .and_then(|v| match v {
                SessionValue::Float(f) => Some(f),
                _ => None,
            })
            .unwrap_or(0.0);
        
        // 简化的购物车项目JSON格式
        let new_item = format!(
            r#"{{"product_id":"{}","quantity":{},"price":{}}}"#,
            product_id, quantity, price
        );
        
        let updated_cart = if cart_json == "[]" {
            format!("[{}]", new_item)
        } else {
            format!("{},{}]", &cart_json[..cart_json.len()-1], new_item)
        };
        
        let new_total = current_total + (price * quantity as f64);
        
        // 更新会话数据
        self.session_manager.set_session_value(
            session_id,
            "cart_items".to_string(),
            SessionValue::Json(updated_cart)
        )?;
        
        self.session_manager.set_session_value(
            session_id,
            "cart_total".to_string(),
            SessionValue::Float(new_total)
        )?;
        
        Ok(())
    }
    
    /// 获取购物车信息
    pub fn get_cart_info(&mut self, session_id: &str) -> Result<Option<CartInfo>, DB::Error> {
        if let Some(_) = self.session_manager.get_session(session_id)? {
            let items = self.session_manager.get_session_value(session_id, "cart_items")?
                .and_then(|v| v.as_string())
                .unwrap_or_else(|| "[]".to_string());
            
            let total = self.session_manager.get_session_value(session_id, "cart_total")?
                .and_then(|v| match v {
                    SessionValue::Float(f) => Some(f),
                    _ => None,
                })
                .unwrap_or(0.0);
            
            let item_count = items.matches("product_id").count();
            
            Ok(Some(CartInfo {
                items_json: items,
                total_amount: total,
                item_count,
            }))
        } else {
            Ok(None)
        }
    }
    
    /// 清空购物车
    pub fn clear_cart(&mut self, session_id: &str) -> Result<(), DB::Error> {
        self.session_manager.set_session_value(
            session_id,
            "cart_items".to_string(),
            SessionValue::Json("[]".to_string())
        )?;
        
        self.session_manager.set_session_value(
            session_id,
            "cart_total".to_string(),
            SessionValue::Float(0.0)
        )?;
        
        Ok(())
    }
    
    /// 结账
    pub fn checkout(&mut self, session_id: &str) -> Result<CheckoutResult, DB::Error> {
        let cart_info = self.get_cart_info(session_id)?;
        
        if let Some(cart) = cart_info {
            if cart.item_count > 0 {
                // 模拟订单创建
                let order_id = format!("ORDER-{}", generate_session_id()[..8].to_uppercase());
                
                // 清空购物车
                self.clear_cart(session_id)?;
                
                // 记录订单ID
                self.session_manager.set_session_value(
                    session_id,
                    "last_order_id".to_string(),
                    SessionValue::String(order_id.clone())
                )?;
                
                Ok(CheckoutResult {
                    success: true,
                    order_id: Some(order_id),
                    total_amount: cart.total_amount,
                    message: "订单创建成功".to_string(),
                })
            } else {
                Ok(CheckoutResult {
                    success: false,
                    order_id: None,
                    total_amount: 0.0,
                    message: "购物车为空".to_string(),
                })
            }
        } else {
            Err(DatabaseError::SessionNotFound)
        }
    }
    
    pub fn get_session_stats(&self) -> Result<SessionStats, DB::Error> {
        self.session_manager.get_session_stats()
    }
    
    pub fn cleanup_expired_sessions(&mut self) -> Result<usize, DB::Error> {
        self.session_manager.cleanup_expired_sessions()
    }
}

#[derive(Debug)]
pub struct CartInfo {
    pub items_json: String,
    pub total_amount: f64,
    pub item_count: usize,
}

#[derive(Debug)]
pub struct CheckoutResult {
    pub success: bool,
    pub order_id: Option<String>,
    pub total_amount: f64,
    pub message: String,
}

// =================
// 辅助函数
// =================

/// 生成会话ID
fn generate_session_id() -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    
    let mut hasher = DefaultHasher::new();
    now.hash(&mut hasher);
    thread_local_random().hash(&mut hasher);
    
    format!("sess_{:x}", hasher.finish())
}

fn thread_local_random() -> u64 {
    // 简单的线程安全随机数生成
    use std::sync::OnceLock;
    use std::sync::Mutex;
    
    static SEED: OnceLock<Mutex<u64>> = OnceLock::new();
    let seed_mutex = SEED.get_or_init(|| Mutex::new(12345));
    
    let mut seed = seed_mutex.lock().unwrap();
    *seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
    *seed
}

/// 数据库会话状态模式演示
pub fn demo_database_session_state() {
    println!("=== 数据库会话状态模式演示 ===\n");
    
    let database = InMemorySessionDatabase::new();
    let mut cart_service = ShoppingCartSessionService::new(database);
    
    println!("1. 创建购物会话:");
    
    let session_id1 = cart_service.create_shopping_session(
        Some("user123".to_string()),
        "192.168.1.100".to_string()
    ).unwrap();
    
    let session_id2 = cart_service.create_shopping_session(
        None,
        "192.168.1.101".to_string()
    ).unwrap();
    
    println!("  用户会话ID: {}", session_id1);
    println!("  匿名会话ID: {}\n", session_id2);
    
    println!("2. 添加商品到购物车:");
    
    cart_service.add_to_cart(&session_id1, "PROD001".to_string(), 2, 99.99).unwrap();
    cart_service.add_to_cart(&session_id1, "PROD002".to_string(), 1, 149.99).unwrap();
    cart_service.add_to_cart(&session_id2, "PROD003".to_string(), 3, 29.99).unwrap();
    
    println!("  用户会话添加了2个商品");
    println!("  匿名会话添加了1个商品\n");
    
    println!("3. 查看购物车信息:");
    
    if let Some(cart1) = cart_service.get_cart_info(&session_id1).unwrap() {
        println!("  用户购物车:");
        println!("    商品数量: {}", cart1.item_count);
        println!("    总金额: ${:.2}", cart1.total_amount);
    }
    
    if let Some(cart2) = cart_service.get_cart_info(&session_id2).unwrap() {
        println!("  匿名购物车:");
        println!("    商品数量: {}", cart2.item_count);
        println!("    总金额: ${:.2}\n", cart2.total_amount);
    }
    
    println!("4. 结账流程:");
    
    let checkout_result = cart_service.checkout(&session_id1).unwrap();
    println!("  用户结账结果: {}", checkout_result.message);
    if let Some(order_id) = checkout_result.order_id {
        println!("  订单ID: {}", order_id);
        println!("  订单金额: ${:.2}\n", checkout_result.total_amount);
    }
    
    println!("5. 会话统计:");
    
    let stats = cart_service.get_session_stats().unwrap();
    println!("  {}\n", stats);
    
    println!("6. 清理过期会话:");
    
    let cleaned = cart_service.cleanup_expired_sessions().unwrap();
    println!("  清理了 {} 个过期会话\n", cleaned);
    
    println!("7. 会话持久性演示:");
    
    // 模拟服务器重启 - 会话数据仍在数据库中
    let database2 = InMemorySessionDatabase::new(); // 在真实场景中，这会连接到同一个数据库
    let mut cart_service2 = ShoppingCartSessionService::new(database2);
    
    // 尝试访问之前的会话（在真实环境中会成功）
    match cart_service2.get_cart_info(&session_id2) {
        Ok(Some(cart)) => {
            println!("  服务器重启后成功恢复会话");
            println!("  恢复的购物车商品数: {}", cart.item_count);
        },
        Ok(None) => {
            println!("  会话不存在（模拟环境限制）");
        },
        Err(_) => {
            println!("  会话访问失败");
        }
    }
    
    println!("\n=== 数据库会话状态模式特点 ===");
    println!("✓ 持久性 - 会话数据存储在数据库中，服务器重启不丢失");
    println!("✓ 可扩展性 - 支持负载均衡和分布式部署");
    println!("✓ 高可用性 - 数据库集群提供故障恢复能力");
    println!("✓ 会话共享 - 多个服务器实例可以访问同一会话");
    println!("✓ 精确管理 - 支持会话过期、清理和统计");
    println!("✓ 安全性 - 可以实现会话验证和IP绑定");
} 