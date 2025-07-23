// 文件路径: /d%3A/workspace/RustLearn/RustDesignPattern/src/EnterpriseAppPattern/OfflineConcurrencyPatterns/pessimistic_offline_lock.rs

//! # 悲观离线锁模式 (Pessimistic Offline Lock)
//!
//! ## 概述
//! 悲观离线锁预先假设会发生冲突，在用户开始编辑数据时就获取锁，
//! 防止其他用户同时修改相同的数据，直到当前用户完成编辑或锁超时。
//!
//! ## 优点
//! - 避免了数据冲突
//! - 确保数据一致性
//! - 适合冲突频繁的场景
//! - 用户体验良好（不会出现保存失败）
//!
//! ## 适用场景
//! - 冲突发生概率较高的应用
//! - 数据一致性要求严格的系统
//! - 用户编辑时间相对较短的场景
//! - 需要防止丢失更新的情况

use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH, Duration};

/// 锁信息
#[derive(Debug, Clone)]
pub struct LockInfo {
    pub entity_id: String,
    pub owner_id: String,
    pub lock_time: u64,
    pub expire_time: u64,
    pub lock_type: LockType,
}

/// 锁类型
#[derive(Debug, Clone, PartialEq)]
pub enum LockType {
    Read,
    Write,
    Exclusive,
}

impl fmt::Display for LockType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LockType::Read => write!(f, "读锁"),
            LockType::Write => write!(f, "写锁"),
            LockType::Exclusive => write!(f, "排他锁"),
        }
    }
}

/// 锁错误
#[derive(Debug)]
pub enum LockError {
    AlreadyLocked {
        entity_id: String,
        owner_id: String,
        lock_type: LockType,
    },
    LockNotFound(String),
    LockExpired(String),
    InvalidOwner {
        entity_id: String,
        expected_owner: String,
        actual_owner: String,
    },
    EntityNotFound(String),
    DatabaseError(String),
}

impl fmt::Display for LockError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LockError::AlreadyLocked { entity_id, owner_id, lock_type } => {
                write!(f, "实体 {} 已被用户 {} 使用{}锁定", entity_id, owner_id, lock_type)
            }
            LockError::LockNotFound(id) => {
                write!(f, "未找到实体 {} 的锁", id)
            }
            LockError::LockExpired(id) => {
                write!(f, "实体 {} 的锁已过期", id)
            }
            LockError::InvalidOwner { entity_id, expected_owner, actual_owner } => {
                write!(f, "实体 {} 的锁拥有者不匹配: 期望 {}, 实际 {}", 
                       entity_id, expected_owner, actual_owner)
            }
            LockError::EntityNotFound(id) => {
                write!(f, "实体未找到: {}", id)
            }
            LockError::DatabaseError(msg) => {
                write!(f, "数据库错误: {}", msg)
            }
        }
    }
}

impl std::error::Error for LockError {}

/// 可锁定的实体接口
pub trait Lockable {
    fn get_id(&self) -> String;
}

/// 文档实体示例
#[derive(Debug, Clone)]
pub struct Document {
    pub id: String,
    pub title: String,
    pub content: String,
    pub author: String,
    pub version: u32,
    pub created_at: u64,
    pub updated_at: u64,
}

impl Document {
    pub fn new(id: String, title: String, author: String) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        Self {
            id: id.clone(),
            title,
            content: String::new(),
            author,
            version: 1,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn update_content(&mut self, content: String) {
        self.content = content;
        self.version += 1;
        self.updated_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
    }
}

impl Lockable for Document {
    fn get_id(&self) -> String {
        self.id.clone()
    }
}

impl fmt::Display for Document {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "文档[{}]: {} (作者: {}, 版本: {})", 
               self.id, self.title, self.author, self.version)
    }
}

/// 悲观离线锁管理器
pub struct PessimisticOfflineLockManager<T: Lockable + Clone> {
    storage: Arc<Mutex<HashMap<String, T>>>,
    locks: Arc<Mutex<HashMap<String, LockInfo>>>,
    default_timeout: u64, // 默认锁超时时间（秒）
}

impl<T: Lockable + Clone> PessimisticOfflineLockManager<T> {
    pub fn new(default_timeout_seconds: u64) -> Self {
        Self {
            storage: Arc::new(Mutex::new(HashMap::new())),
            locks: Arc::new(Mutex::new(HashMap::new())),
            default_timeout: default_timeout_seconds,
        }
    }

    /// 获取锁
    pub fn acquire_lock(&self, entity_id: &str, owner_id: &str, lock_type: LockType, timeout_seconds: Option<u64>) -> Result<(), LockError> {
        let mut locks = self.locks.lock().unwrap();
        
        // 检查是否已有锁
        if let Some(existing_lock) = locks.get(entity_id) {
            // 检查锁是否过期
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            
            if existing_lock.expire_time > now {
                // 锁未过期，检查是否为同一用户
                if existing_lock.owner_id != owner_id {
                    return Err(LockError::AlreadyLocked {
                        entity_id: entity_id.to_string(),
                        owner_id: existing_lock.owner_id.clone(),
                        lock_type: existing_lock.lock_type.clone(),
                    });
                }
            } else {
                // 锁已过期，移除过期锁
                locks.remove(entity_id);
            }
        }

        // 创建新锁
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let timeout = timeout_seconds.unwrap_or(self.default_timeout);
        let lock_info = LockInfo {
            entity_id: entity_id.to_string(),
            owner_id: owner_id.to_string(),
            lock_time: now,
            expire_time: now + timeout,
            lock_type,
        };

        locks.insert(entity_id.to_string(), lock_info);
        Ok(())
    }

    /// 释放锁
    pub fn release_lock(&self, entity_id: &str, owner_id: &str) -> Result<(), LockError> {
        let mut locks = self.locks.lock().unwrap();
        
        if let Some(lock_info) = locks.get(entity_id) {
            if lock_info.owner_id != owner_id {
                return Err(LockError::InvalidOwner {
                    entity_id: entity_id.to_string(),
                    expected_owner: owner_id.to_string(),
                    actual_owner: lock_info.owner_id.clone(),
                });
            }
            
            locks.remove(entity_id);
            Ok(())
        } else {
            Err(LockError::LockNotFound(entity_id.to_string()))
        }
    }

    /// 检查锁状态
    fn check_lock(&self, entity_id: &str, owner_id: &str) -> Result<(), LockError> {
        let locks = self.locks.lock().unwrap();
        
        if let Some(lock_info) = locks.get(entity_id) {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            
            if lock_info.expire_time <= now {
                return Err(LockError::LockExpired(entity_id.to_string()));
            }
            
            if lock_info.owner_id != owner_id {
                return Err(LockError::InvalidOwner {
                    entity_id: entity_id.to_string(),
                    expected_owner: owner_id.to_string(),
                    actual_owner: lock_info.owner_id.clone(),
                });
            }
            
            Ok(())
        } else {
            Err(LockError::LockNotFound(entity_id.to_string()))
        }
    }

    /// 创建实体
    pub fn create(&self, entity: T) -> Result<(), LockError> {
        let mut storage = self.storage.lock().unwrap();
        let id = entity.get_id();
        
        if storage.contains_key(&id) {
            return Err(LockError::DatabaseError(
                format!("实体 {} 已存在", id)
            ));
        }

        storage.insert(id, entity);
        Ok(())
    }

    /// 加载实体（需要先获取锁）
    pub fn load(&self, entity_id: &str, owner_id: &str) -> Result<T, LockError> {
        // 检查锁
        self.check_lock(entity_id, owner_id)?;
        
        let storage = self.storage.lock().unwrap();
        storage.get(entity_id)
            .cloned()
            .ok_or_else(|| LockError::EntityNotFound(entity_id.to_string()))
    }

    /// 保存实体（需要持有锁）
    pub fn save(&self, entity: &T, owner_id: &str) -> Result<(), LockError> {
        let entity_id = entity.get_id();
        
        // 检查锁
        self.check_lock(&entity_id, owner_id)?;
        
        let mut storage = self.storage.lock().unwrap();
        storage.insert(entity_id, entity.clone());
        Ok(())
    }

    /// 删除实体（需要持有锁）
    pub fn delete(&self, entity_id: &str, owner_id: &str) -> Result<(), LockError> {
        // 检查锁
        self.check_lock(entity_id, owner_id)?;
        
        let mut storage = self.storage.lock().unwrap();
        if storage.remove(entity_id).is_none() {
            return Err(LockError::EntityNotFound(entity_id.to_string()));
        }
        
        // 同时释放锁
        let _ = self.release_lock(entity_id, owner_id);
        Ok(())
    }

    /// 获取所有锁信息
    pub fn get_all_locks(&self) -> Vec<LockInfo> {
        let locks = self.locks.lock().unwrap();
        locks.values().cloned().collect()
    }

    /// 清理过期锁
    pub fn cleanup_expired_locks(&self) -> usize {
        let mut locks = self.locks.lock().unwrap();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let mut expired_keys = Vec::new();
        for (key, lock_info) in locks.iter() {
            if lock_info.expire_time <= now {
                expired_keys.push(key.clone());
            }
        }
        
        let count = expired_keys.len();
        for key in expired_keys {
            locks.remove(&key);
        }
        
        count
    }

    /// 续期锁
    pub fn renew_lock(&self, entity_id: &str, owner_id: &str, additional_seconds: u64) -> Result<(), LockError> {
        let mut locks = self.locks.lock().unwrap();
        
        if let Some(lock_info) = locks.get_mut(entity_id) {
            if lock_info.owner_id != owner_id {
                return Err(LockError::InvalidOwner {
                    entity_id: entity_id.to_string(),
                    expected_owner: owner_id.to_string(),
                    actual_owner: lock_info.owner_id.clone(),
                });
            }
            
            lock_info.expire_time += additional_seconds;
            Ok(())
        } else {
            Err(LockError::LockNotFound(entity_id.to_string()))
        }
    }
}

/// 文档编辑服务
pub struct DocumentEditingService {
    doc_manager: PessimisticOfflineLockManager<Document>,
}

impl DocumentEditingService {
    pub fn new() -> Self {
        Self {
            doc_manager: PessimisticOfflineLockManager::new(300), // 5分钟默认超时
        }
    }

    /// 创建文档
    pub fn create_document(&self, id: String, title: String, author: String) -> Result<(), LockError> {
        let document = Document::new(id, title, author);
        self.doc_manager.create(document)
    }

    /// 开始编辑文档（获取锁）
    pub fn start_editing(&self, doc_id: &str, user_id: &str, timeout_minutes: Option<u64>) -> Result<Document, LockError> {
        let timeout_seconds = timeout_minutes.map(|m| m * 60);
        
        // 获取写锁
        self.doc_manager.acquire_lock(doc_id, user_id, LockType::Write, timeout_seconds)?;
        
        // 加载文档
        self.doc_manager.load(doc_id, user_id)
    }

    /// 保存文档
    pub fn save_document(&self, document: &Document, user_id: &str) -> Result<(), LockError> {
        self.doc_manager.save(document, user_id)
    }

    /// 完成编辑（释放锁）
    pub fn finish_editing(&self, doc_id: &str, user_id: &str) -> Result<(), LockError> {
        self.doc_manager.release_lock(doc_id, user_id)
    }

    /// 取消编辑（释放锁，不保存）
    pub fn cancel_editing(&self, doc_id: &str, user_id: &str) -> Result<(), LockError> {
        self.doc_manager.release_lock(doc_id, user_id)
    }

    /// 续期编辑锁
    pub fn extend_editing_time(&self, doc_id: &str, user_id: &str, additional_minutes: u64) -> Result<(), LockError> {
        self.doc_manager.renew_lock(doc_id, user_id, additional_minutes * 60)
    }

    /// 获取所有锁状态
    pub fn get_lock_status(&self) -> Vec<LockInfo> {
        self.doc_manager.get_all_locks()
    }

    /// 清理过期锁
    pub fn cleanup_expired_locks(&self) -> usize {
        self.doc_manager.cleanup_expired_locks()
    }
}

/// 演示悲观离线锁模式
pub fn demo() {
    println!("=== 悲观离线锁模式演示 ===\n");
    
    let edit_service = DocumentEditingService::new();
    
    // 创建文档
    println!("1. 创建文档");
    let _ = edit_service.create_document("doc1".to_string(), "项目计划".to_string(), "张三".to_string());
    let _ = edit_service.create_document("doc2".to_string(), "会议纪要".to_string(), "李四".to_string());
    
    // 用户1开始编辑文档1
    println!("\n2. 用户1开始编辑文档1");
    match edit_service.start_editing("doc1", "user1", Some(10)) {
        Ok(mut doc) => {
            println!("   成功获取编辑锁: {}", doc);
            
            // 修改文档内容
            doc.update_content("这是更新后的项目计划内容...".to_string());
            
            // 保存文档
            if let Err(e) = edit_service.save_document(&doc, "user1") {
                println!("   保存失败: {}", e);
            } else {
                println!("   文档保存成功");
            }
        }
        Err(e) => println!("   获取锁失败: {}", e),
    }
    
    // 用户2尝试编辑同一文档（应该失败）
    println!("\n3. 用户2尝试编辑文档1");
    match edit_service.start_editing("doc1", "user2", Some(5)) {
        Ok(doc) => println!("   意外成功: {}", doc),
        Err(e) => println!("   预期失败: {}", e),
    }
    
    // 显示锁状态
    println!("\n4. 当前锁状态");
    for lock in edit_service.get_lock_status() {
        println!("   实体: {}, 拥有者: {}, 类型: {}, 过期时间: {}", 
                 lock.entity_id, lock.owner_id, lock.lock_type, lock.expire_time);
    }
    
    // 用户1续期锁
    println!("\n5. 用户1续期锁");
    match edit_service.extend_editing_time("doc1", "user1", 5) {
        Ok(_) => println!("   续期成功"),
        Err(e) => println!("   续期失败: {}", e),
    }
    
    // 用户1完成编辑
    println!("\n6. 用户1完成编辑");
    match edit_service.finish_editing("doc1", "user1") {
        Ok(_) => println!("   编辑完成，锁已释放"),
        Err(e) => println!("   释放锁失败: {}", e),
    }
    
    // 用户2现在可以编辑了
    println!("\n7. 用户2现在尝试编辑文档1");
    match edit_service.start_editing("doc1", "user2", Some(5)) {
        Ok(doc) => {
            println!("   成功获取编辑锁: {}", doc);
            let _ = edit_service.finish_editing("doc1", "user2");
        }
        Err(e) => println!("   获取锁失败: {}", e),
    }
    
    // 清理过期锁
    println!("\n8. 清理过期锁");
    let cleaned = edit_service.cleanup_expired_locks();
    println!("   清理了 {} 个过期锁", cleaned);
    
    println!("\n=== 悲观离线锁模式演示完成 ===");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lock_acquisition() {
        let manager = PessimisticOfflineLockManager::<Document>::new(300);
        
        // 获取锁应该成功
        assert!(manager.acquire_lock("doc1", "user1", LockType::Write, None).is_ok());
        
        // 同一用户再次获取锁应该成功
        assert!(manager.acquire_lock("doc1", "user1", LockType::Write, None).is_ok());
        
        // 不同用户获取锁应该失败
        assert!(manager.acquire_lock("doc1", "user2", LockType::Write, None).is_err());
    }

    #[test]
    fn test_lock_release() {
        let manager = PessimisticOfflineLockManager::<Document>::new(300);
        
        // 获取锁
        assert!(manager.acquire_lock("doc1", "user1", LockType::Write, None).is_ok());
        
        // 释放锁
        assert!(manager.release_lock("doc1", "user1").is_ok());
        
        // 其他用户现在可以获取锁
        assert!(manager.acquire_lock("doc1", "user2", LockType::Write, None).is_ok());
    }
} 