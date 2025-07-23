// 文件路径: /d%3A/workspace/RustLearn/RustDesignPattern/src/EnterpriseAppPattern/OfflineConcurrencyPatterns/optimistic_offline_lock.rs

//! # 乐观离线锁模式 (Optimistic Offline Lock)
//!
//! ## 概述
//! 乐观离线锁是一种并发控制机制，用于处理长时间运行的业务事务。
//! 它假设冲突是罕见的，允许多个用户同时编辑同一数据，
//! 但在提交时检查是否发生冲突。
//!
//! ## 优点
//! - 避免了长时间持有锁的问题
//! - 提高了系统的并发性能
//! - 减少了死锁的可能性
//! - 适合在Web应用中使用
//!
//! ## 适用场景
//! - 用户会话时间较长的应用
//! - 需要支持多用户同时编辑的系统
//! - 对性能要求较高的应用
//! - 冲突发生概率较低的场景

use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

/// 版本控制信息
#[derive(Debug, Clone, PartialEq)]
pub struct Version {
    pub number: u64,
    pub timestamp: u64,
    pub user_id: String,
}

impl Version {
    pub fn new(user_id: String) -> Self {
        Self {
            number: 1,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            user_id,
        }
    }

    pub fn increment(&mut self, user_id: String) {
        self.number += 1;
        self.timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        self.user_id = user_id;
    }
}

/// 并发冲突错误
#[derive(Debug)]
pub enum ConcurrencyError {
    OptimisticLockFailure {
        expected_version: u64,
        actual_version: u64,
        entity_id: String,
    },
    EntityNotFound(String),
    DatabaseError(String),
}

impl fmt::Display for ConcurrencyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConcurrencyError::OptimisticLockFailure { expected_version, actual_version, entity_id } => {
                write!(f, "乐观锁冲突: 实体 {} 的期望版本为 {}, 但实际版本为 {}", 
                       entity_id, expected_version, actual_version)
            }
            ConcurrencyError::EntityNotFound(id) => {
                write!(f, "实体未找到: {}", id)
            }
            ConcurrencyError::DatabaseError(msg) => {
                write!(f, "数据库错误: {}", msg)
            }
        }
    }
}

impl std::error::Error for ConcurrencyError {}

/// 可版本控制的实体接口
pub trait Versionable {
    fn get_id(&self) -> String;
    fn get_version(&self) -> &Version;
    fn set_version(&mut self, version: Version);
}

/// 账户实体示例
#[derive(Debug, Clone)]
pub struct Account {
    pub id: String,
    pub name: String,
    pub balance: f64,
    pub email: String,
    pub version: Version,
}

impl Account {
    pub fn new(id: String, name: String, email: String, user_id: String) -> Self {
        Self {
            id: id.clone(),
            name,
            balance: 0.0,
            email,
            version: Version::new(user_id),
        }
    }

    pub fn deposit(&mut self, amount: f64) -> Result<(), String> {
        if amount <= 0.0 {
            return Err("存款金额必须大于0".to_string());
        }
        self.balance += amount;
        Ok(())
    }

    pub fn withdraw(&mut self, amount: f64) -> Result<(), String> {
        if amount <= 0.0 {
            return Err("取款金额必须大于0".to_string());
        }
        if self.balance < amount {
            return Err("余额不足".to_string());
        }
        self.balance -= amount;
        Ok(())
    }
}

impl Versionable for Account {
    fn get_id(&self) -> String {
        self.id.clone()
    }

    fn get_version(&self) -> &Version {
        &self.version
    }

    fn set_version(&mut self, version: Version) {
        self.version = version;
    }
}

impl fmt::Display for Account {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "账户[{}]: {} ({}), 余额: {:.2}, 版本: {}",
               self.id, self.name, self.email, self.balance, self.version.number)
    }
}

/// 乐观离线锁管理器
pub struct OptimisticOfflineLockManager<T: Versionable + Clone> {
    storage: Arc<Mutex<HashMap<String, T>>>,
    lock_timeout: u64, // 锁超时时间（秒）
}

impl<T: Versionable + Clone> OptimisticOfflineLockManager<T> {
    pub fn new(timeout_seconds: u64) -> Self {
        Self {
            storage: Arc::new(Mutex::new(HashMap::new())),
            lock_timeout: timeout_seconds,
        }
    }

    /// 加载实体
    pub fn load(&self, id: &str) -> Result<T, ConcurrencyError> {
        let storage = self.storage.lock().unwrap();
        storage.get(id)
            .cloned()
            .ok_or_else(|| ConcurrencyError::EntityNotFound(id.to_string()))
    }

    /// 保存实体（带版本检查）
    pub fn save(&self, entity: &mut T, user_id: String) -> Result<(), ConcurrencyError> {
        let mut storage = self.storage.lock().unwrap();
        let id = entity.get_id();
        
        if let Some(existing) = storage.get(&id) {
            // 检查版本冲突
            if existing.get_version().number != entity.get_version().number {
                return Err(ConcurrencyError::OptimisticLockFailure {
                    expected_version: entity.get_version().number,
                    actual_version: existing.get_version().number,
                    entity_id: id,
                });
            }
        }

        // 更新版本信息
        let mut new_version = entity.get_version().clone();
        new_version.increment(user_id);
        entity.set_version(new_version);

        // 保存到存储
        storage.insert(id, entity.clone());
        Ok(())
    }

    /// 创建新实体
    pub fn create(&self, entity: T) -> Result<(), ConcurrencyError> {
        let mut storage = self.storage.lock().unwrap();
        let id = entity.get_id();
        
        if storage.contains_key(&id) {
            return Err(ConcurrencyError::DatabaseError(
                format!("实体 {} 已存在", id)
            ));
        }

        storage.insert(id, entity);
        Ok(())
    }

    /// 删除实体（带版本检查）
    pub fn delete(&self, id: &str, expected_version: u64) -> Result<(), ConcurrencyError> {
        let mut storage = self.storage.lock().unwrap();
        
        if let Some(existing) = storage.get(id) {
            if existing.get_version().number != expected_version {
                return Err(ConcurrencyError::OptimisticLockFailure {
                    expected_version,
                    actual_version: existing.get_version().number,
                    entity_id: id.to_string(),
                });
            }
        } else {
            return Err(ConcurrencyError::EntityNotFound(id.to_string()));
        }

        storage.remove(id);
        Ok(())
    }

    /// 获取所有实体
    pub fn find_all(&self) -> Vec<T> {
        let storage = self.storage.lock().unwrap();
        storage.values().cloned().collect()
    }

    /// 检查版本冲突
    pub fn check_version(&self, id: &str, expected_version: u64) -> Result<bool, ConcurrencyError> {
        let storage = self.storage.lock().unwrap();
        
        if let Some(existing) = storage.get(id) {
            Ok(existing.get_version().number == expected_version)
        } else {
            Err(ConcurrencyError::EntityNotFound(id.to_string()))
        }
    }

    /// 获取统计信息
    pub fn get_statistics(&self) -> HashMap<String, usize> {
        let storage = self.storage.lock().unwrap();
        let mut stats = HashMap::new();
        stats.insert("total_entities".to_string(), storage.len());
        stats
    }
}

/// 业务事务管理器
pub struct BusinessTransactionManager {
    account_manager: OptimisticOfflineLockManager<Account>,
}

impl BusinessTransactionManager {
    pub fn new() -> Self {
        Self {
            account_manager: OptimisticOfflineLockManager::new(300), // 5分钟超时
        }
    }

    /// 创建账户
    pub fn create_account(&self, id: String, name: String, email: String, user_id: String) -> Result<(), ConcurrencyError> {
        let account = Account::new(id, name, email, user_id);
        self.account_manager.create(account)
    }

    /// 转账操作（使用乐观锁）
    pub fn transfer(&self, from_id: &str, to_id: &str, amount: f64, user_id: String) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            // 加载源账户和目标账户
            let mut from_account = self.account_manager.load(from_id)?;
            let mut to_account = self.account_manager.load(to_id)?;

            // 执行业务逻辑
            from_account.withdraw(amount)?;
            to_account.deposit(amount)?;

            // 尝试保存（可能失败）
            match self.account_manager.save(&mut from_account, user_id.clone()) {
                Ok(_) => {
                    match self.account_manager.save(&mut to_account, user_id.clone()) {
                        Ok(_) => {
                            println!("转账成功: {} -> {}, 金额: {:.2}", from_id, to_id, amount);
                            return Ok(());
                        }
                        Err(ConcurrencyError::OptimisticLockFailure { .. }) => {
                            // 目标账户版本冲突，需要重试
                            println!("目标账户版本冲突，正在重试...");
                            continue;
                        }
                        Err(e) => return Err(Box::new(e)),
                    }
                }
                Err(ConcurrencyError::OptimisticLockFailure { .. }) => {
                    // 源账户版本冲突，需要重试
                    println!("源账户版本冲突，正在重试...");
                    continue;
                }
                Err(e) => return Err(Box::new(e)),
            }
        }
    }

    /// 获取账户信息
    pub fn get_account(&self, id: &str) -> Result<Account, ConcurrencyError> {
        self.account_manager.load(id)
    }

    /// 更新账户信息
    pub fn update_account(&self, id: &str, name: Option<String>, email: Option<String>, user_id: String) -> Result<(), ConcurrencyError> {
        let mut account = self.account_manager.load(id)?;
        
        if let Some(new_name) = name {
            account.name = new_name;
        }
        if let Some(new_email) = email {
            account.email = new_email;
        }

        self.account_manager.save(&mut account, user_id)
    }

    /// 列出所有账户
    pub fn list_accounts(&self) -> Vec<Account> {
        self.account_manager.find_all()
    }
}

/// 演示乐观离线锁模式
pub fn demo() {
    println!("=== 乐观离线锁模式演示 ===\n");
    
    let tx_manager = BusinessTransactionManager::new();
    
    // 创建账户
    println!("1. 创建账户");
    let _ = tx_manager.create_account("acc1".to_string(), "张三".to_string(), "zhang@example.com".to_string(), "user1".to_string());
    let _ = tx_manager.create_account("acc2".to_string(), "李四".to_string(), "li@example.com".to_string(), "user2".to_string());
    
    // 初始存款
    println!("2. 初始存款");
    if let Ok(mut account) = tx_manager.get_account("acc1") {
        let _ = account.deposit(1000.0);
        let _ = tx_manager.account_manager.save(&mut account, "user1".to_string());
    }
    
    if let Ok(mut account) = tx_manager.get_account("acc2") {
        let _ = account.deposit(500.0);
        let _ = tx_manager.account_manager.save(&mut account, "user2".to_string());
    }
    
    // 显示账户状态
    println!("3. 账户状态");
    for account in tx_manager.list_accounts() {
        println!("   {}", account);
    }
    
    // 模拟并发冲突
    println!("\n4. 模拟并发冲突");
    let mut account1 = tx_manager.get_account("acc1").unwrap();
    let mut account2 = tx_manager.get_account("acc1").unwrap(); // 同时加载同一账户
    
    // 用户1修改账户
    account1.name = "张三(已更新)".to_string();
    match tx_manager.account_manager.save(&mut account1, "user1".to_string()) {
        Ok(_) => println!("   用户1保存成功"),
        Err(e) => println!("   用户1保存失败: {}", e),
    }
    
    // 用户2尝试修改同一账户（应该失败）
    account2.name = "张三(用户2更新)".to_string();
    match tx_manager.account_manager.save(&mut account2, "user2".to_string()) {
        Ok(_) => println!("   用户2保存成功"),
        Err(e) => println!("   用户2保存失败: {}", e),
    }
    
    // 执行转账
    println!("\n5. 执行转账");
    match tx_manager.transfer("acc1", "acc2", 200.0, "user1".to_string()) {
        Ok(_) => println!("   转账完成"),
        Err(e) => println!("   转账失败: {}", e),
    }
    
    // 显示最终状态
    println!("\n6. 最终账户状态");
    for account in tx_manager.list_accounts() {
        println!("   {}", account);
    }
    
    println!("\n=== 乐观离线锁模式演示完成 ===");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_optimistic_lock_success() {
        let manager = OptimisticOfflineLockManager::new(300);
        let account = Account::new("test1".to_string(), "Test".to_string(), "test@example.com".to_string(), "user1".to_string());
        
        // 创建账户
        assert!(manager.create(account).is_ok());
        
        // 加载并修改
        let mut loaded = manager.load("test1").unwrap();
        loaded.name = "Updated".to_string();
        
        // 保存应该成功
        assert!(manager.save(&mut loaded, "user1".to_string()).is_ok());
    }

    #[test]
    fn test_optimistic_lock_conflict() {
        let manager = OptimisticOfflineLockManager::new(300);
        let account = Account::new("test2".to_string(), "Test".to_string(), "test@example.com".to_string(), "user1".to_string());
        
        // 创建账户
        assert!(manager.create(account).is_ok());
        
        // 两个用户同时加载
        let mut account1 = manager.load("test2").unwrap();
        let mut account2 = manager.load("test2").unwrap();
        
        // 用户1先保存
        account1.name = "User1 Update".to_string();
        assert!(manager.save(&mut account1, "user1".to_string()).is_ok());
        
        // 用户2尝试保存应该失败
        account2.name = "User2 Update".to_string();
        assert!(manager.save(&mut account2, "user2".to_string()).is_err());
    }
} 