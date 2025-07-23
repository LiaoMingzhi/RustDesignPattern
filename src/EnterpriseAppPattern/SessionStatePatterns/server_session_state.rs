/*
 * 文件路径: /d%3A/workspace/RustLearn/RustDesignPattern/src/EnterpriseAppPattern/SessionStatePatterns/server_session_state.rs
 * 
 * Server Session State（服务器会话状态）模式
 * 
 * 定义：
 * 将会话状态存储在服务器端的内存中，通过会话ID来标识和检索用户的会话数据。
 * 这是传统Web应用中最常用的会话管理方式。
 * 
 * 主要特点：
 * 1. 状态存储在服务器内存中
 * 2. 客户端只保存会话ID（通常在Cookie中）
 * 3. 服务器负责会话数据的管理
 * 4. 支持复杂的会话数据结构
 * 5. 需要考虑内存使用和会话清理
 * 
 * 适用场景：
 * - 需要存储大量会话数据时
 * - 需要高安全性的应用
 * - 传统的Web应用
 * - 单体应用架构
 */

use std::collections::HashMap;
use std::sync::{Arc, RwLock, Mutex};
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::thread;

/// 服务器会话状态错误
#[derive(Debug)]
pub enum ServerSessionError {
    SessionNotFound(String),
    SessionExpired(String),
    InvalidSessionId(String),
    StorageError(String),
}

impl Display for ServerSessionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ServerSessionError::SessionNotFound(msg) => write!(f, "会话未找到: {}", msg),
            ServerSessionError::SessionExpired(msg) => write!(f, "会话已过期: {}", msg),
            ServerSessionError::InvalidSessionId(msg) => write!(f, "无效的会话ID: {}", msg),
            ServerSessionError::StorageError(msg) => write!(f, "存储错误: {}", msg),
        }
    }
}

impl Error for ServerSessionError {}

/// 会话数据
#[derive(Debug, Clone)]
pub struct SessionData {
    session_id: String,
    data: HashMap<String, String>,
    created_at: SystemTime,
    last_accessed: SystemTime,
    timeout_duration: Duration,
}

impl SessionData {
    /// 创建新的会话数据
    pub fn new(session_id: String, timeout_minutes: u64) -> Self {
        let now = SystemTime::now();
        Self {
            session_id,
            data: HashMap::new(),
            created_at: now,
            last_accessed: now,
            timeout_duration: Duration::from_secs(timeout_minutes * 60),
        }
    }
    
    /// 获取会话ID
    pub fn session_id(&self) -> &str {
        &self.session_id
    }
    
    /// 设置数据
    pub fn set(&mut self, key: &str, value: &str) {
        self.data.insert(key.to_string(), value.to_string());
        self.touch();
    }
    
    /// 获取数据
    pub fn get(&mut self, key: &str) -> Option<&String> {
        self.touch();
        self.data.get(key)
    }
    
    /// 获取数据（不更新访问时间）
    pub fn get_without_touch(&self, key: &str) -> Option<&String> {
        self.data.get(key)
    }
    
    /// 删除数据
    pub fn remove(&mut self, key: &str) -> Option<String> {
        self.touch();
        self.data.remove(key)
    }
    
    /// 获取所有数据
    pub fn get_all(&mut self) -> &HashMap<String, String> {
        self.touch();
        &self.data
    }
    
    /// 检查会话是否过期
    pub fn is_expired(&self) -> bool {
        SystemTime::now()
            .duration_since(self.last_accessed)
            .unwrap_or(Duration::from_secs(0)) > self.timeout_duration
    }
    
    /// 更新最后访问时间
    pub fn touch(&mut self) {
        self.last_accessed = SystemTime::now();
    }
    
    /// 获取会话存在时间
    pub fn session_age(&self) -> Duration {
        SystemTime::now()
            .duration_since(self.created_at)
            .unwrap_or(Duration::from_secs(0))
    }
    
    /// 获取剩余有效时间
    pub fn remaining_time(&self) -> Duration {
        let elapsed = SystemTime::now()
            .duration_since(self.last_accessed)
            .unwrap_or(Duration::from_secs(0));
        
        if elapsed < self.timeout_duration {
            self.timeout_duration - elapsed
        } else {
            Duration::from_secs(0)
        }
    }
}

/// 会话存储接口
pub trait SessionStorage: Send + Sync + std::fmt::Debug {
    fn create_session(&self, timeout_minutes: u64) -> Result<String, ServerSessionError>;
    fn get_session(&self, session_id: &str) -> Result<Option<SessionData>, ServerSessionError>;
    fn save_session(&self, session: SessionData) -> Result<(), ServerSessionError>;
    fn delete_session(&self, session_id: &str) -> Result<(), ServerSessionError>;
    fn cleanup_expired_sessions(&self) -> Result<usize, ServerSessionError>;
    fn get_session_count(&self) -> usize;
}

/// 内存会话存储
#[derive(Debug)]
pub struct MemorySessionStorage {
    sessions: Arc<RwLock<HashMap<String, SessionData>>>,
    session_counter: Arc<Mutex<u64>>,
}

impl MemorySessionStorage {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            session_counter: Arc::new(Mutex::new(0)),
        }
    }
    
    /// 生成会话ID
    fn generate_session_id(&self) -> String {
        let mut counter = self.session_counter.lock().unwrap();
        *counter += 1;
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        format!("SESSION_{}_{}", timestamp, *counter)
    }
}

impl SessionStorage for MemorySessionStorage {
    fn create_session(&self, timeout_minutes: u64) -> Result<String, ServerSessionError> {
        let session_id = self.generate_session_id();
        let session = SessionData::new(session_id.clone(), timeout_minutes);
        
        let mut sessions = self.sessions.write()
            .map_err(|_| ServerSessionError::StorageError("无法获取写锁".to_string()))?;
        
        sessions.insert(session_id.clone(), session);
        Ok(session_id)
    }
    
    fn get_session(&self, session_id: &str) -> Result<Option<SessionData>, ServerSessionError> {
        let sessions = self.sessions.read()
            .map_err(|_| ServerSessionError::StorageError("无法获取读锁".to_string()))?;
        
        if let Some(session) = sessions.get(session_id) {
            if session.is_expired() {
                // 会话已过期，但不在这里删除（由清理任务处理）
                return Err(ServerSessionError::SessionExpired(session_id.to_string()));
            }
            Ok(Some(session.clone()))
        } else {
            Ok(None)
        }
    }
    
    fn save_session(&self, session: SessionData) -> Result<(), ServerSessionError> {
        let mut sessions = self.sessions.write()
            .map_err(|_| ServerSessionError::StorageError("无法获取写锁".to_string()))?;
        
        sessions.insert(session.session_id.clone(), session);
        Ok(())
    }
    
    fn delete_session(&self, session_id: &str) -> Result<(), ServerSessionError> {
        let mut sessions = self.sessions.write()
            .map_err(|_| ServerSessionError::StorageError("无法获取写锁".to_string()))?;
        
        sessions.remove(session_id);
        Ok(())
    }
    
    fn cleanup_expired_sessions(&self) -> Result<usize, ServerSessionError> {
        let mut sessions = self.sessions.write()
            .map_err(|_| ServerSessionError::StorageError("无法获取写锁".to_string()))?;
        
        let initial_count = sessions.len();
        sessions.retain(|_, session| !session.is_expired());
        let final_count = sessions.len();
        
        Ok(initial_count - final_count)
    }
    
    fn get_session_count(&self) -> usize {
        self.sessions.read().unwrap().len()
    }
}

impl Default for MemorySessionStorage {
    fn default() -> Self {
        Self::new()
    }
}

/// 会话管理器
#[derive(Debug)]
pub struct SessionManager {
    storage: Arc<dyn SessionStorage>,
    default_timeout: u64,
}

impl SessionManager {
    pub fn new(storage: Arc<dyn SessionStorage>, default_timeout_minutes: u64) -> Self {
        Self {
            storage,
            default_timeout: default_timeout_minutes,
        }
    }
    
    /// 创建新会话
    pub fn create_session(&self) -> Result<String, ServerSessionError> {
        self.create_session_with_timeout(self.default_timeout)
    }
    
    /// 创建带自定义超时的会话
    pub fn create_session_with_timeout(&self, timeout_minutes: u64) -> Result<String, ServerSessionError> {
        self.storage.create_session(timeout_minutes)
    }
    
    /// 获取会话
    pub fn get_session(&self, session_id: &str) -> Result<Option<SessionData>, ServerSessionError> {
        if session_id.is_empty() {
            return Err(ServerSessionError::InvalidSessionId("会话ID不能为空".to_string()));
        }
        
        self.storage.get_session(session_id)
    }
    
    /// 设置会话数据
    pub fn set_session_data(&self, session_id: &str, key: &str, value: &str) -> Result<(), ServerSessionError> {
        let mut session = self.get_session(session_id)?
            .ok_or_else(|| ServerSessionError::SessionNotFound(session_id.to_string()))?;
        
        session.set(key, value);
        self.storage.save_session(session)
    }
    
    /// 获取会话数据
    pub fn get_session_data(&self, session_id: &str, key: &str) -> Result<Option<String>, ServerSessionError> {
        let mut session = self.get_session(session_id)?
            .ok_or_else(|| ServerSessionError::SessionNotFound(session_id.to_string()))?;
        
        let value = session.get(key).cloned();
        self.storage.save_session(session)?;
        Ok(value)
    }
    
    /// 删除会话数据
    pub fn remove_session_data(&self, session_id: &str, key: &str) -> Result<Option<String>, ServerSessionError> {
        let mut session = self.get_session(session_id)?
            .ok_or_else(|| ServerSessionError::SessionNotFound(session_id.to_string()))?;
        
        let value = session.remove(key);
        self.storage.save_session(session)?;
        Ok(value)
    }
    
    /// 销毁会话
    pub fn destroy_session(&self, session_id: &str) -> Result<(), ServerSessionError> {
        self.storage.delete_session(session_id)
    }
    
    /// 清理过期会话
    pub fn cleanup_expired_sessions(&self) -> Result<usize, ServerSessionError> {
        self.storage.cleanup_expired_sessions()
    }
    
    /// 获取会话统计信息
    pub fn get_session_stats(&self) -> SessionStats {
        SessionStats {
            total_sessions: self.storage.get_session_count(),
        }
    }
}

/// 会话统计信息
#[derive(Debug, Clone)]
pub struct SessionStats {
    pub total_sessions: usize,
}

impl Display for SessionStats {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "总会话数: {}", self.total_sessions)
    }
}

/// 会话清理器（后台任务）
pub struct SessionCleaner {
    manager: Arc<SessionManager>,
    cleanup_interval: Duration,
    running: Arc<Mutex<bool>>,
}

impl SessionCleaner {
    pub fn new(manager: Arc<SessionManager>, cleanup_interval_minutes: u64) -> Self {
        Self {
            manager,
            cleanup_interval: Duration::from_secs(cleanup_interval_minutes * 60),
            running: Arc::new(Mutex::new(false)),
        }
    }
    
    /// 启动清理任务
    pub fn start(&self) {
        let mut running = self.running.lock().unwrap();
        if *running {
            return; // 已经在运行
        }
        *running = true;
        
        let manager = Arc::clone(&self.manager);
        let cleanup_interval = self.cleanup_interval;
        let running_flag = Arc::clone(&self.running);
        
        thread::spawn(move || {
            while *running_flag.lock().unwrap() {
                thread::sleep(cleanup_interval);
                
                if !*running_flag.lock().unwrap() {
                    break;
                }
                
                match manager.cleanup_expired_sessions() {
                    Ok(cleaned_count) => {
                        if cleaned_count > 0 {
                            println!("清理了 {} 个过期会话", cleaned_count);
                        }
                    }
                    Err(e) => {
                        eprintln!("清理过期会话时出错: {}", e);
                    }
                }
            }
        });
    }
    
    /// 停止清理任务
    pub fn stop(&self) {
        let mut running = self.running.lock().unwrap();
        *running = false;
    }
}

/// 用户会话示例
#[derive(Debug, Clone)]
pub struct UserSession {
    manager: Arc<SessionManager>,
    session_id: String,
}

impl UserSession {
    /// 创建新的用户会话
    pub fn new(manager: Arc<SessionManager>) -> Result<Self, ServerSessionError> {
        let session_id = manager.create_session()?;
        Ok(Self { manager, session_id })
    }
    
    /// 从会话ID恢复用户会话
    pub fn from_session_id(manager: Arc<SessionManager>, session_id: String) -> Result<Self, ServerSessionError> {
        // 验证会话是否存在
        manager.get_session(&session_id)?
            .ok_or_else(|| ServerSessionError::SessionNotFound(session_id.clone()))?;
        
        Ok(Self { manager, session_id })
    }
    
    /// 获取会话ID
    pub fn session_id(&self) -> &str {
        &self.session_id
    }
    
    /// 登录用户
    pub fn login(&self, user_id: &str, username: &str, role: &str) -> Result<(), ServerSessionError> {
        self.manager.set_session_data(&self.session_id, "user_id", user_id)?;
        self.manager.set_session_data(&self.session_id, "username", username)?;
        self.manager.set_session_data(&self.session_id, "role", role)?;
        self.manager.set_session_data(&self.session_id, "logged_in", "true")?;
        Ok(())
    }
    
    /// 检查是否已登录
    pub fn is_logged_in(&self) -> Result<bool, ServerSessionError> {
        match self.manager.get_session_data(&self.session_id, "logged_in")? {
            Some(value) => Ok(value == "true"),
            None => Ok(false),
        }
    }
    
    /// 获取当前用户信息
    pub fn get_user_info(&self) -> Result<Option<UserInfo>, ServerSessionError> {
        if !self.is_logged_in()? {
            return Ok(None);
        }
        
        let user_id = self.manager.get_session_data(&self.session_id, "user_id")?;
        let username = self.manager.get_session_data(&self.session_id, "username")?;
        let role = self.manager.get_session_data(&self.session_id, "role")?;
        
        if let (Some(user_id), Some(username), Some(role)) = (user_id, username, role) {
            Ok(Some(UserInfo { user_id, username, role }))
        } else {
            Ok(None)
        }
    }
    
    /// 设置用户偏好
    pub fn set_preference(&self, key: &str, value: &str) -> Result<(), ServerSessionError> {
        let pref_key = format!("pref_{}", key);
        self.manager.set_session_data(&self.session_id, &pref_key, value)
    }
    
    /// 获取用户偏好
    pub fn get_preference(&self, key: &str) -> Result<Option<String>, ServerSessionError> {
        let pref_key = format!("pref_{}", key);
        self.manager.get_session_data(&self.session_id, &pref_key)
    }
    
    /// 退出登录
    pub fn logout(&self) -> Result<(), ServerSessionError> {
        self.manager.destroy_session(&self.session_id)
    }
    
    /// 获取会话信息
    pub fn get_session_info(&self) -> Result<SessionInfo, ServerSessionError> {
        let session = self.manager.get_session(&self.session_id)?
            .ok_or_else(|| ServerSessionError::SessionNotFound(self.session_id.clone()))?;
        
        Ok(SessionInfo {
            session_id: session.session_id.clone(),
            session_age: session.session_age(),
            remaining_time: session.remaining_time(),
            is_expired: session.is_expired(),
        })
    }
}

/// 用户信息
#[derive(Debug, Clone)]
pub struct UserInfo {
    pub user_id: String,
    pub username: String,
    pub role: String,
}

impl Display for UserInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "用户: {} (ID: {}, 角色: {})", self.username, self.user_id, self.role)
    }
}

/// 会话信息
#[derive(Debug, Clone)]
pub struct SessionInfo {
    pub session_id: String,
    pub session_age: Duration,
    pub remaining_time: Duration,
    pub is_expired: bool,
}

impl Display for SessionInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "会话ID: {}, 存在时间: {:?}, 剩余时间: {:?}, 已过期: {}", 
               self.session_id, self.session_age, self.remaining_time, self.is_expired)
    }
}

/// 服务器会话状态模式演示
pub fn demo() {
    println!("=== Server Session State（服务器会话状态）模式演示 ===\n");
    
    // 1. 创建会话管理器
    println!("1. 创建会话管理器:");
    let storage = Arc::new(MemorySessionStorage::new());
    let manager = Arc::new(SessionManager::new(storage, 30)); // 30分钟超时
    
    println!("会话管理器创建完成，默认超时: 30分钟");
    println!("初始会话统计: {}", manager.get_session_stats());
    
    println!("{}", "=".repeat(50));
    
    // 2. 创建和管理会话
    println!("2. 创建和管理会话:");
    
    // 创建第一个会话
    match UserSession::new(Arc::clone(&manager)) {
        Ok(session1) => {
            println!("创建会话1: {}", session1.session_id());
            
            // 用户登录
            if let Err(e) = session1.login("user_001", "张三", "admin") {
                println!("登录失败: {}", e);
            } else {
                println!("用户登录成功");
                
                // 检查登录状态
                match session1.is_logged_in() {
                    Ok(true) => println!("用户已登录"),
                    Ok(false) => println!("用户未登录"),
                    Err(e) => println!("检查登录状态失败: {}", e),
                }
                
                // 获取用户信息
                match session1.get_user_info() {
                    Ok(Some(user_info)) => println!("用户信息: {}", user_info),
                    Ok(None) => println!("未找到用户信息"),
                    Err(e) => println!("获取用户信息失败: {}", e),
                }
                
                // 设置用户偏好
                let _ = session1.set_preference("language", "zh-CN");
                let _ = session1.set_preference("theme", "dark");
                
                // 获取用户偏好
                if let Ok(Some(lang)) = session1.get_preference("language") {
                    println!("用户语言偏好: {}", lang);
                }
                if let Ok(Some(theme)) = session1.get_preference("theme") {
                    println!("用户主题偏好: {}", theme);
                }
                
                // 获取会话信息
                match session1.get_session_info() {
                    Ok(session_info) => println!("会话信息: {}", session_info),
                    Err(e) => println!("获取会话信息失败: {}", e),
                }
            }
        }
        Err(e) => println!("创建会话失败: {}", e),
    }
    
    // 创建第二个会话
    match UserSession::new(Arc::clone(&manager)) {
        Ok(session2) => {
            println!("\n创建会话2: {}", session2.session_id());
            let _ = session2.login("user_002", "李四", "user");
            
            if let Ok(Some(user_info)) = session2.get_user_info() {
                println!("会话2用户信息: {}", user_info);
            }
        }
        Err(e) => println!("创建会话2失败: {}", e),
    }
    
    println!("\n当前会话统计: {}", manager.get_session_stats());
    
    println!("{}", "=".repeat(50));
    
    // 3. 会话恢复
    println!("3. 会话恢复演示:");
    
    // 模拟从Cookie或其他方式获取会话ID
    let session_id = manager.create_session().unwrap();
    println!("创建会话ID: {}", session_id);
    
    // 设置一些会话数据
    let _ = manager.set_session_data(&session_id, "cart_items", "3");
    let _ = manager.set_session_data(&session_id, "last_page", "/products");
    
    // 从会话ID恢复会话
    match UserSession::from_session_id(Arc::clone(&manager), session_id.clone()) {
        Ok(restored_session) => {
            println!("会话恢复成功: {}", restored_session.session_id());
            
            // 获取之前设置的数据
            if let Ok(Some(cart_items)) = manager.get_session_data(&session_id, "cart_items") {
                println!("购物车商品数量: {}", cart_items);
            }
            if let Ok(Some(last_page)) = manager.get_session_data(&session_id, "last_page") {
                println!("上次访问页面: {}", last_page);
            }
        }
        Err(e) => println!("会话恢复失败: {}", e),
    }
    
    println!("{}", "=".repeat(50));
    
    // 4. 会话清理
    println!("4. 会话清理演示:");
    
    // 创建一个短超时的会话用于测试过期
    let short_session_id = manager.storage.create_session(0).unwrap(); // 0分钟超时，立即过期
    println!("创建短超时会话: {}", short_session_id);
    
    // 等待一小段时间确保会话过期
    thread::sleep(Duration::from_millis(100));
    
    // 尝试访问过期会话
    match manager.get_session(&short_session_id) {
        Ok(Some(_)) => println!("会话仍然有效"),
        Ok(None) => println!("会话不存在"),
        Err(ServerSessionError::SessionExpired(_)) => println!("会话已过期（预期结果）"),
        Err(e) => println!("其他错误: {}", e),
    }
    
    // 手动清理过期会话
    match manager.cleanup_expired_sessions() {
        Ok(cleaned_count) => println!("清理了 {} 个过期会话", cleaned_count),
        Err(e) => println!("清理失败: {}", e),
    }
    
    println!("清理后会话统计: {}", manager.get_session_stats());
    
    println!("{}", "=".repeat(50));
    
    // 5. 错误处理演示
    println!("5. 错误处理演示:");
    
    // 尝试访问不存在的会话
    match manager.get_session("invalid_session_id") {
        Ok(Some(_)) => println!("不应该找到会话"),
        Ok(None) => println!("预期结果：会话不存在"),
        Err(e) => println!("错误: {}", e),
    }
    
    // 尝试使用空会话ID
    match manager.get_session("") {
        Ok(_) => println!("不应该成功"),
        Err(e) => println!("预期错误: {}", e),
    }
    
    // 尝试操作不存在的会话数据
    match manager.get_session_data("nonexistent", "key") {
        Ok(_) => println!("不应该成功"),
        Err(e) => println!("预期错误: {}", e),
    }
    
    println!("\n=== Server Session State模式演示完成 ===");
    
    // 输出模式总结
    println!("\n【Server Session State模式总结】");
    println!("优点:");
    println!("1. 安全性高：敏感数据存储在服务器端");
    println!("2. 支持复杂数据：可以存储任意类型的会话数据");
    println!("3. 客户端轻量：只需传输会话ID");
    println!("4. 服务器控制：完全控制会话生命周期");
    
    println!("\n缺点:");
    println!("1. 内存消耗：占用服务器内存资源");
    println!("2. 扩展性问题：难以在多个服务器间共享");
    println!("3. 单点故障：服务器重启会丢失所有会话");
    println!("4. 集群复杂：需要会话复制或共享存储");
    
    println!("\n适用场景:");
    println!("1. 需要高安全性的应用");
    println!("2. 会话数据复杂的应用");
    println!("3. 单体应用架构");
    println!("4. 用户数量可控的系统");
}

/// 服务器会话状态模式演示（包装函数）
pub fn demo_server_session_state() {
    demo();
} 