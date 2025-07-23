//! # 隐式锁模式（Implicit Lock Pattern）
//!
//! 隐式锁模式通过在应用程序框架层面自动处理锁定逻辑，
//! 使得开发者不需要显式地获取和释放锁。
//! 这种模式通常与事务边界、方法调用或对象访问绑定。
//!
//! ## 模式特点
//! - **自动锁定**: 框架自动处理锁的获取和释放
//! - **透明性**: 对业务代码透明，减少锁管理负担
//! - **一致性**: 确保锁定策略的一致性应用
//! - **错误减少**: 避免忘记释放锁的错误
//!
//! ## 使用场景
//! - 需要简化并发控制时
//! - 框架级别的锁管理时
//! - 减少开发者犯错的可能性时
//! - 统一锁定策略时

use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use std::fmt::{self, Display, Formatter};
use std::error::Error;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// 隐式锁错误类型
#[derive(Debug)]
pub enum ImplicitLockError {
    LockAcquisitionTimeout(String),
    LockNotHeld(String),
    DeadlockDetected(String),
    InvalidOperation(String),
    ConcurrencyError(String),
}

impl Display for ImplicitLockError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ImplicitLockError::LockAcquisitionTimeout(msg) => write!(f, "锁获取超时: {}", msg),
            ImplicitLockError::LockNotHeld(msg) => write!(f, "锁未持有: {}", msg),
            ImplicitLockError::DeadlockDetected(msg) => write!(f, "检测到死锁: {}", msg),
            ImplicitLockError::InvalidOperation(msg) => write!(f, "无效操作: {}", msg),
            ImplicitLockError::ConcurrencyError(msg) => write!(f, "并发错误: {}", msg),
        }
    }
}

impl Error for ImplicitLockError {}

/// 锁类型枚举
#[derive(Debug, Clone, PartialEq)]
pub enum LockType {
    Read,
    Write,
    Exclusive,
}

/// 隐式锁上下文
#[derive(Debug)]
pub struct LockContext {
    pub thread_id: String,
    pub locked_resources: Vec<String>,
    pub lock_types: HashMap<String, LockType>,
    pub acquired_at: u64,
}

impl LockContext {
    pub fn new(thread_id: String) -> Self {
        Self {
            thread_id,
            locked_resources: Vec::new(),
            lock_types: HashMap::new(),
            acquired_at: current_timestamp(),
        }
    }

    pub fn add_lock(&mut self, resource_id: String, lock_type: LockType) {
        self.locked_resources.push(resource_id.clone());
        self.lock_types.insert(resource_id, lock_type);
    }

    pub fn remove_lock(&mut self, resource_id: &str) {
        self.locked_resources.retain(|r| r != resource_id);
        self.lock_types.remove(resource_id);
    }

    pub fn has_lock(&self, resource_id: &str) -> bool {
        self.locked_resources.contains(&resource_id.to_string())
    }

    pub fn get_lock_type(&self, resource_id: &str) -> Option<&LockType> {
        self.lock_types.get(resource_id)
    }
}

/// 业务实体trait
pub trait BusinessEntity {
    fn get_id(&self) -> String;
    fn get_version(&self) -> u64;
    fn increment_version(&mut self);
    fn is_dirty(&self) -> bool;
    fn mark_clean(&mut self);
    fn mark_dirty(&mut self);
}

/// 账户实体
#[derive(Debug, Clone)]
pub struct Account {
    pub id: String,
    pub owner: String,
    pub balance: f64,
    pub account_type: AccountType,
    pub version: u64,
    pub is_dirty: bool,
    pub created_at: u64,
    pub last_transaction_at: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AccountType {
    Checking,
    Savings,
    Credit,
}

impl Account {
    pub fn new(id: String, owner: String, initial_balance: f64, account_type: AccountType) -> Self {
        let now = current_timestamp();
        Self {
            id,
            owner,
            balance: initial_balance,
            account_type,
            version: 1,
            is_dirty: false,
            created_at: now,
            last_transaction_at: now,
        }
    }

    pub fn deposit(&mut self, amount: f64) -> Result<(), ImplicitLockError> {
        if amount <= 0.0 {
            return Err(ImplicitLockError::InvalidOperation("存款金额必须大于0".to_string()));
        }
        self.balance += amount;
        self.last_transaction_at = current_timestamp();
        self.mark_dirty();
        Ok(())
    }

    pub fn withdraw(&mut self, amount: f64) -> Result<(), ImplicitLockError> {
        if amount <= 0.0 {
            return Err(ImplicitLockError::InvalidOperation("取款金额必须大于0".to_string()));
        }
        if self.balance < amount {
            return Err(ImplicitLockError::InvalidOperation("余额不足".to_string()));
        }
        self.balance -= amount;
        self.last_transaction_at = current_timestamp();
        self.mark_dirty();
        Ok(())
    }

    pub fn transfer_to(&mut self, other: &mut Account, amount: f64) -> Result<(), ImplicitLockError> {
        if amount <= 0.0 {
            return Err(ImplicitLockError::InvalidOperation("转账金额必须大于0".to_string()));
        }
        if self.balance < amount {
            return Err(ImplicitLockError::InvalidOperation("余额不足".to_string()));
        }

        self.withdraw(amount)?;
        other.deposit(amount)?;
        
        println!("💸 转账成功: {} -> {}, 金额: {:.2}", self.id, other.id, amount);
        Ok(())
    }
}

impl BusinessEntity for Account {
    fn get_id(&self) -> String {
        self.id.clone()
    }

    fn get_version(&self) -> u64 {
        self.version
    }

    fn increment_version(&mut self) {
        self.version += 1;
    }

    fn is_dirty(&self) -> bool {
        self.is_dirty
    }

    fn mark_clean(&mut self) {
        self.is_dirty = false;
    }

    fn mark_dirty(&mut self) {
        self.is_dirty = true;
        self.increment_version();
    }
}

impl Display for Account {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Account[{}] - Owner: {}, Balance: {:.2}, Type: {:?}, Version: {}", 
               self.id, self.owner, self.balance, self.account_type, self.version)
    }
}

/// 隐式锁管理器
pub struct ImplicitLockManager {
    locks: Arc<RwLock<HashMap<String, Arc<RwLock<()>>>>>, // 资源锁
    contexts: Arc<Mutex<HashMap<String, LockContext>>>,   // 线程上下文
    timeout_duration: Duration,
}

impl ImplicitLockManager {
    pub fn new() -> Self {
        Self {
            locks: Arc::new(RwLock::new(HashMap::new())),
            contexts: Arc::new(Mutex::new(HashMap::new())),
            timeout_duration: Duration::from_secs(30),
        }
    }

    /// 为当前线程创建锁上下文
    pub fn create_context(&self) -> String {
        let thread_id = format!("thread_{}", current_timestamp());
        let context = LockContext::new(thread_id.clone());
        
        let mut contexts = self.contexts.lock().unwrap();
        contexts.insert(thread_id.clone(), context);
        
        println!("🔄 创建锁上下文: {}", thread_id);
        thread_id
    }

    /// 清理锁上下文
    pub fn cleanup_context(&self, thread_id: &str) {
        let mut contexts = self.contexts.lock().unwrap();
        if let Some(context) = contexts.remove(thread_id) {
            // 释放该线程持有的所有锁
            for resource_id in &context.locked_resources {
                self.release_resource_lock(resource_id);
            }
            println!("🧹 清理锁上下文: {}", thread_id);
        }
    }

    /// 隐式获取资源锁
    fn acquire_resource_lock(&self, resource_id: &str, lock_type: LockType, thread_id: &str) -> Result<(), ImplicitLockError> {
        // 获取或创建资源锁
        let resource_lock = {
            let mut locks = self.locks.write().unwrap();
            locks.entry(resource_id.to_string())
                .or_insert_with(|| Arc::new(RwLock::new(())))
                .clone()
        };

        // 尝试获取锁
        let lock_acquired = match lock_type {
            LockType::Read => {
                // 尝试获取读锁
                resource_lock.try_read().is_ok()
            }
            LockType::Write | LockType::Exclusive => {
                // 尝试获取写锁
                resource_lock.try_write().is_ok()
            }
        };

        if !lock_acquired {
            return Err(ImplicitLockError::LockAcquisitionTimeout(
                format!("无法获取资源 {} 的 {:?} 锁", resource_id, lock_type)
            ));
        }

        // 更新线程上下文
        let mut contexts = self.contexts.lock().unwrap();
        if let Some(context) = contexts.get_mut(thread_id) {
            context.add_lock(resource_id.to_string(), lock_type.clone());
        }

        println!("🔒 隐式获取锁: {} ({:?})", resource_id, lock_type);
        Ok(())
    }

    /// 释放资源锁
    fn release_resource_lock(&self, resource_id: &str) {
        // 实际的锁释放会在Drop时自动处理
        println!("🔓 隐式释放锁: {}", resource_id);
    }

    /// 检查是否持有锁
    pub fn holds_lock(&self, resource_id: &str, thread_id: &str) -> bool {
        let contexts = self.contexts.lock().unwrap();
        if let Some(context) = contexts.get(thread_id) {
            context.has_lock(resource_id)
        } else {
            false
        }
    }

    /// 获取锁统计信息
    pub fn get_lock_statistics(&self) -> LockStatistics {
        let locks = self.locks.read().unwrap();
        let contexts = self.contexts.lock().unwrap();

        let total_resources = locks.len();
        let active_contexts = contexts.len();
        let total_locks_held: usize = contexts.values()
            .map(|ctx| ctx.locked_resources.len())
            .sum();

        LockStatistics {
            total_resources,
            active_contexts,
            total_locks_held,
        }
    }
}

/// 锁统计信息
#[derive(Debug)]
pub struct LockStatistics {
    pub total_resources: usize,
    pub active_contexts: usize,
    pub total_locks_held: usize,
}

impl Display for LockStatistics {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "锁统计 - 资源数: {}, 活跃上下文: {}, 持有锁数: {}", 
               self.total_resources, self.active_contexts, self.total_locks_held)
    }
}

/// 隐式锁装饰器 - 为方法调用自动添加锁
pub struct ImplicitLockDecorator {
    lock_manager: Arc<ImplicitLockManager>,
}

impl ImplicitLockDecorator {
    pub fn new(lock_manager: Arc<ImplicitLockManager>) -> Self {
        Self { lock_manager }
    }

    /// 带隐式锁的读操作
    pub fn with_read_lock<T, F>(&self, resource_id: &str, operation: F) -> Result<T, ImplicitLockError>
    where
        F: FnOnce() -> Result<T, ImplicitLockError>,
    {
        let thread_id = self.lock_manager.create_context();
        
        // 自动获取读锁
        self.lock_manager.acquire_resource_lock(resource_id, LockType::Read, &thread_id)?;
        
        // 执行操作
        let result = operation();
        
        // 自动清理上下文（释放锁）
        self.lock_manager.cleanup_context(&thread_id);
        
        result
    }

    /// 带隐式锁的写操作
    pub fn with_write_lock<T, F>(&self, resource_id: &str, operation: F) -> Result<T, ImplicitLockError>
    where
        F: FnOnce() -> Result<T, ImplicitLockError>,
    {
        let thread_id = self.lock_manager.create_context();
        
        // 自动获取写锁
        self.lock_manager.acquire_resource_lock(resource_id, LockType::Write, &thread_id)?;
        
        // 执行操作
        let result = operation();
        
        // 自动清理上下文（释放锁）
        self.lock_manager.cleanup_context(&thread_id);
        
        result
    }

    /// 带隐式锁的多资源操作
    pub fn with_multiple_locks<T, F>(&self, resources: &[(String, LockType)], operation: F) -> Result<T, ImplicitLockError>
    where
        F: FnOnce() -> Result<T, ImplicitLockError>,
    {
        let thread_id = self.lock_manager.create_context();
        
        // 按照资源ID排序以避免死锁
        let mut sorted_resources = resources.to_vec();
        sorted_resources.sort_by(|a, b| a.0.cmp(&b.0));
        
        // 依次获取所有锁
        for (resource_id, lock_type) in &sorted_resources {
            self.lock_manager.acquire_resource_lock(resource_id, lock_type.clone(), &thread_id)?;
        }
        
        // 执行操作
        let result = operation();
        
        // 自动清理上下文（释放所有锁）
        self.lock_manager.cleanup_context(&thread_id);
        
        result
    }
}

/// 银行服务 - 使用隐式锁
pub struct BankService {
    accounts: Arc<Mutex<HashMap<String, Account>>>,
    lock_decorator: ImplicitLockDecorator,
}

impl BankService {
    pub fn new() -> Self {
        let lock_manager = Arc::new(ImplicitLockManager::new());
        Self {
            accounts: Arc::new(Mutex::new(HashMap::new())),
            lock_decorator: ImplicitLockDecorator::new(lock_manager),
        }
    }

    /// 创建账户
    pub fn create_account(&self, account: Account) -> Result<(), ImplicitLockError> {
        let account_id = account.get_id();
        
        // 使用隐式写锁保护账户创建
        self.lock_decorator.with_write_lock(&account_id, || {
            let mut accounts = self.accounts.lock().unwrap();
            if accounts.contains_key(&account_id) {
                return Err(ImplicitLockError::InvalidOperation("账户已存在".to_string()));
            }
            accounts.insert(account_id.clone(), account);
            println!("🏦 创建账户: {}", account_id);
            Ok(())
        })
    }

    /// 查询账户余额（只读操作）
    pub fn get_balance(&self, account_id: &str) -> Result<f64, ImplicitLockError> {
        // 使用隐式读锁保护余额查询
        self.lock_decorator.with_read_lock(account_id, || {
            let accounts = self.accounts.lock().unwrap();
            let account = accounts.get(account_id)
                .ok_or_else(|| ImplicitLockError::InvalidOperation("账户不存在".to_string()))?;
            Ok(account.balance)
        })
    }

    /// 存款操作
    pub fn deposit(&self, account_id: &str, amount: f64) -> Result<(), ImplicitLockError> {
        // 使用隐式写锁保护存款操作
        self.lock_decorator.with_write_lock(account_id, || {
            let mut accounts = self.accounts.lock().unwrap();
            let account = accounts.get_mut(account_id)
                .ok_or_else(|| ImplicitLockError::InvalidOperation("账户不存在".to_string()))?;
            account.deposit(amount)?;
            println!("💰 存款: {} 金额: {:.2}, 余额: {:.2}", account_id, amount, account.balance);
            Ok(())
        })
    }

    /// 取款操作
    pub fn withdraw(&self, account_id: &str, amount: f64) -> Result<(), ImplicitLockError> {
        // 使用隐式写锁保护取款操作
        self.lock_decorator.with_write_lock(account_id, || {
            let mut accounts = self.accounts.lock().unwrap();
            let account = accounts.get_mut(account_id)
                .ok_or_else(|| ImplicitLockError::InvalidOperation("账户不存在".to_string()))?;
            account.withdraw(amount)?;
            println!("💸 取款: {} 金额: {:.2}, 余额: {:.2}", account_id, amount, account.balance);
            Ok(())
        })
    }

    /// 转账操作（需要锁定两个账户）
    pub fn transfer(&self, from_account_id: &str, to_account_id: &str, amount: f64) -> Result<(), ImplicitLockError> {
        // 使用隐式多资源锁保护转账操作
        let resources = vec![
            (from_account_id.to_string(), LockType::Write),
            (to_account_id.to_string(), LockType::Write),
        ];

        self.lock_decorator.with_multiple_locks(&resources, || {
            let mut accounts = self.accounts.lock().unwrap();
            
            let from_account = accounts.get_mut(from_account_id)
                .ok_or_else(|| ImplicitLockError::InvalidOperation("源账户不存在".to_string()))?;
            
            if from_account.balance < amount {
                return Err(ImplicitLockError::InvalidOperation("余额不足".to_string()));
            }

            // 先从源账户扣款
            from_account.withdraw(amount)?;
            
            // 再向目标账户存款
            let to_account = accounts.get_mut(to_account_id)
                .ok_or_else(|| ImplicitLockError::InvalidOperation("目标账户不存在".to_string()))?;
            to_account.deposit(amount)?;

            println!("🔄 转账完成: {} -> {}, 金额: {:.2}", from_account_id, to_account_id, amount);
            Ok(())
        })
    }

    /// 获取账户信息
    pub fn get_account_info(&self, account_id: &str) -> Result<Account, ImplicitLockError> {
        // 使用隐式读锁保护账户信息查询
        self.lock_decorator.with_read_lock(account_id, || {
            let accounts = self.accounts.lock().unwrap();
            let account = accounts.get(account_id)
                .ok_or_else(|| ImplicitLockError::InvalidOperation("账户不存在".to_string()))?;
            Ok(account.clone())
        })
    }

    /// 获取所有账户摘要
    pub fn get_accounts_summary(&self) -> Result<Vec<(String, f64)>, ImplicitLockError> {
        // 简化处理，实际应用中可能需要更复杂的锁策略
        let accounts = self.accounts.lock().unwrap();
        let summary: Vec<(String, f64)> = accounts.iter()
            .map(|(id, account)| (id.clone(), account.balance))
            .collect();
        Ok(summary)
    }
}

/// 获取当前时间戳
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

/// 演示隐式锁模式
pub fn demo() {
    println!("=== 隐式锁模式演示 ===\n");

    let bank_service = BankService::new();

    // 创建测试账户
    println!("1. 创建测试账户");
    let account1 = Account::new("ACC001".to_string(), "张三".to_string(), 10000.0, AccountType::Checking);
    let account2 = Account::new("ACC002".to_string(), "李四".to_string(), 5000.0, AccountType::Savings);
    let account3 = Account::new("ACC003".to_string(), "王五".to_string(), 8000.0, AccountType::Checking);

    let _ = bank_service.create_account(account1);
    let _ = bank_service.create_account(account2);
    let _ = bank_service.create_account(account3);

    println!("   创建了3个账户");

    // 演示隐式锁保护的读操作
    println!("\n2. 查询账户余额（隐式读锁保护）");
    match bank_service.get_balance("ACC001") {
        Ok(balance) => println!("   ACC001余额: {:.2}", balance),
        Err(e) => println!("   查询失败: {}", e),
    }

    match bank_service.get_balance("ACC002") {
        Ok(balance) => println!("   ACC002余额: {:.2}", balance),
        Err(e) => println!("   查询失败: {}", e),
    }

    // 演示隐式锁保护的写操作
    println!("\n3. 存取款操作（隐式写锁保护）");
    
    // 存款
    match bank_service.deposit("ACC001", 2000.0) {
        Ok(_) => println!("   存款操作成功"),
        Err(e) => println!("   存款操作失败: {}", e),
    }

    // 取款
    match bank_service.withdraw("ACC002", 1000.0) {
        Ok(_) => println!("   取款操作成功"),
        Err(e) => println!("   取款操作失败: {}", e),
    }

    // 演示隐式多资源锁保护的转账操作
    println!("\n4. 转账操作（隐式多资源锁保护）");
    match bank_service.transfer("ACC001", "ACC003", 3000.0) {
        Ok(_) => println!("   转账操作成功"),
        Err(e) => println!("   转账操作失败: {}", e),
    }

    // 查看操作后的账户状态
    println!("\n5. 查看操作后的账户状态");
    for account_id in &["ACC001", "ACC002", "ACC003"] {
        match bank_service.get_account_info(account_id) {
            Ok(account) => println!("   {}", account),
            Err(e) => println!("   查询{}失败: {}", account_id, e),
        }
    }

    // 演示并发操作模拟
    println!("\n6. 并发操作模拟");

    // 创建一个新的共享 BankService 实例用于并发演示
    let shared_bank_service = Arc::new(BankService::new());
    
    // 在共享服务中创建测试账户
    let shared_account1 = Account::new("SHARED001".to_string(), "并发用户1".to_string(), 1000.0, AccountType::Checking);
    let shared_account2 = Account::new("SHARED002".to_string(), "并发用户2".to_string(), 1000.0, AccountType::Savings);
    let shared_account3 = Account::new("SHARED003".to_string(), "并发用户3".to_string(), 1000.0, AccountType::Checking);
    
    let _ = shared_bank_service.create_account(shared_account1);
    let _ = shared_bank_service.create_account(shared_account2);
    let _ = shared_bank_service.create_account(shared_account3);

    // 启动并发操作
    let handles: Vec<_> = (0..3).map(|i| {
        let service = Arc::clone(&shared_bank_service);
        std::thread::spawn(move || {
            let account_id = format!("SHARED{:03}", (i % 3) + 1);
            
            // 模拟并发存款
            if let Err(e) = service.deposit(&account_id, 100.0) {
                println!("   线程{}存款失败: {}", i, e);
            } else {
                println!("   线程{}存款成功: {} +100.0", i, account_id);
            }
            
            // 模拟并发查询
            if let Ok(balance) = service.get_balance(&account_id) {
                println!("   线程{}查询{}余额: {:.2}", i, account_id, balance);
            }
            
            thread::sleep(Duration::from_millis(100));
        })
    }).collect();

    // 等待所有线程完成
    for handle in handles {
        let _ = handle.join();
    }

    println!("   并发操作完成");

    // 现在继续使用原来的 bank_service
    println!("\n7. 错误处理演示");
    match bank_service.get_balance("NOTEXIST") {  // 现在可以正常使用
        Ok(_) => println!("   不应该成功"),
        Err(e) => println!("   ✅ 正确捕获错误: {}", e),
    }

    // 显示账户摘要
    println!("\n8. 最终账户摘要");
    match bank_service.get_accounts_summary() {
        Ok(summary) => {
            let total_balance: f64 = summary.iter().map(|(_, balance)| balance).sum();
            println!("   账户总数: {}", summary.len());
            println!("   总余额: {:.2}", total_balance);
            for (id, balance) in summary {
                println!("   {}: {:.2}", id, balance);
            }
        }
        Err(e) => println!("   获取摘要失败: {}", e),
    }

    println!("\n=== 隐式锁模式演示完成 ===");

    println!("\n💡 隐式锁模式的优势:");
    println!("1. 自动管理 - 框架自动处理锁的获取和释放");
    println!("2. 减少错误 - 避免忘记释放锁或错误的锁使用");
    println!("3. 代码简洁 - 业务代码中不需要显式的锁管理");
    println!("4. 一致性 - 确保锁定策略在整个应用中的一致性");
    println!("5. 透明性 - 对开发者透明，专注于业务逻辑");

    println!("\n⚠️ 设计考虑:");
    println!("1. 性能开销 - 可能引入额外的框架层面开销");
    println!("2. 调试困难 - 锁的获取和释放对开发者不可见");
    println!("3. 灵活性限制 - 可能无法处理特殊的锁定需求");
    println!("4. 框架依赖 - 需要框架层面的支持和实现");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_account_operations() {
        let mut account = Account::new("TEST001".to_string(), "测试用户".to_string(), 1000.0, AccountType::Checking);
        
        // 测试存款
        assert!(account.deposit(500.0).is_ok());
        assert_eq!(account.balance, 1500.0);
        assert!(account.is_dirty());
        
        // 测试取款
        assert!(account.withdraw(200.0).is_ok());
        assert_eq!(account.balance, 1300.0);
        
        // 测试余额不足
        assert!(account.withdraw(2000.0).is_err());
    }

    #[test]
    fn test_bank_service_basic_operations() {
        let bank_service = BankService::new();
        
        let account = Account::new("TEST001".to_string(), "测试用户".to_string(), 1000.0, AccountType::Checking);
        assert!(bank_service.create_account(account).is_ok());
        
        // 测试查询余额
        assert_eq!(bank_service.get_balance("TEST001").unwrap(), 1000.0);
        
        // 测试存款
        assert!(bank_service.deposit("TEST001", 500.0).is_ok());
        assert_eq!(bank_service.get_balance("TEST001").unwrap(), 1500.0);
        
        // 测试取款
        assert!(bank_service.withdraw("TEST001", 200.0).is_ok());
        assert_eq!(bank_service.get_balance("TEST001").unwrap(), 1300.0);
    }

    #[test]
    fn test_transfer_operation() {
        let bank_service = BankService::new();
        
        let account1 = Account::new("TEST001".to_string(), "用户1".to_string(), 1000.0, AccountType::Checking);
        let account2 = Account::new("TEST002".to_string(), "用户2".to_string(), 500.0, AccountType::Savings);
        
        assert!(bank_service.create_account(account1).is_ok());
        assert!(bank_service.create_account(account2).is_ok());
        
        // 测试转账
        assert!(bank_service.transfer("TEST001", "TEST002", 300.0).is_ok());
        assert_eq!(bank_service.get_balance("TEST001").unwrap(), 700.0);
        assert_eq!(bank_service.get_balance("TEST002").unwrap(), 800.0);
        
        // 测试余额不足的转账
        assert!(bank_service.transfer("TEST001", "TEST002", 1000.0).is_err());
    }

    #[test]
    fn test_lock_context() {
        let mut context = LockContext::new("test_thread".to_string());
        
        context.add_lock("resource1".to_string(), LockType::Read);
        assert!(context.has_lock("resource1"));
        assert_eq!(context.get_lock_type("resource1"), Some(&LockType::Read));
        
        context.remove_lock("resource1");
        assert!(!context.has_lock("resource1"));
    }
} 