/*
 * 文件路径: /d%3A/workspace/RustLearn/RustDesignPattern/src/EnterpriseAppPattern/ObjectRelationalBehavioralPatterns/unit_of_work.rs
 * 
 * Unit of Work（工作单元）模式
 * 
 * 定义：
 * Unit of Work维护一个受业务事务影响的对象列表，协调变更的写入并解决并发问题。
 * 它追踪在业务事务期间读取的对象，并在事务结束时一次性提交所有变更。
 * 
 * 主要特点：
 * 1. 追踪对象变更
 * 2. 维护身份映射
 * 3. 批量提交变更
 * 4. 处理对象间依赖
 * 5. 事务边界管理
 * 
 * 优势：
 * - 减少数据库往返次数
 * - 保证事务一致性
 * - 简化对象状态管理
 * - 自动解决依赖顺序
 * - 提高性能
 * 
 * 适用场景：
 * - 复杂的对象关系
 * - 需要事务保证的批量操作
 * - 对象状态变更追踪
 * - 性能敏感的应用
 */

use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::rc::Rc;
use std::cell::RefCell;

/// 工作单元错误类型
#[derive(Debug)]
pub enum UnitOfWorkError {
    ConcurrencyConflict(String),
    ValidationError(String),
    TransactionError(String),
    DatabaseError(String),
    NotFound(String),
}

impl Display for UnitOfWorkError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            UnitOfWorkError::ConcurrencyConflict(msg) => write!(f, "并发冲突: {}", msg),
            UnitOfWorkError::ValidationError(msg) => write!(f, "验证错误: {}", msg),
            UnitOfWorkError::TransactionError(msg) => write!(f, "事务错误: {}", msg),
            UnitOfWorkError::DatabaseError(msg) => write!(f, "数据库错误: {}", msg),
            UnitOfWorkError::NotFound(msg) => write!(f, "资源未找到: {}", msg),
        }
    }
}

impl Error for UnitOfWorkError {}

/// 领域对象标识符
pub type ObjectId = u32;

/// 对象版本（用于乐观锁）
pub type Version = u32;

/// 领域对象基类特征
pub trait DomainObject {
    fn get_id(&self) -> Option<ObjectId>;
    fn set_id(&mut self, id: ObjectId);
    fn get_version(&self) -> Version;
    fn set_version(&mut self, version: Version);
    fn validate(&self) -> Result<(), UnitOfWorkError>;
    fn clone_box(&self) -> Box<dyn DomainObject>;
}

/// 用户实体
#[derive(Debug, Clone)]
pub struct User {
    pub id: Option<ObjectId>,
    pub username: String,
    pub email: String,
    pub balance: f64,
    pub version: Version,
}

impl User {
    pub fn new(username: String, email: String) -> Self {
        Self {
            id: None,
            username,
            email,
            balance: 0.0,
            version: 0,
        }
    }
    
    pub fn deposit(&mut self, amount: f64) -> Result<(), UnitOfWorkError> {
        if amount <= 0.0 {
            return Err(UnitOfWorkError::ValidationError("存款金额必须大于0".to_string()));
        }
        self.balance += amount;
        Ok(())
    }
    
    pub fn withdraw(&mut self, amount: f64) -> Result<(), UnitOfWorkError> {
        if amount <= 0.0 {
            return Err(UnitOfWorkError::ValidationError("取款金额必须大于0".to_string()));
        }
        if self.balance < amount {
            return Err(UnitOfWorkError::ValidationError("余额不足".to_string()));
        }
        self.balance -= amount;
        Ok(())
    }
}

impl DomainObject for User {
    fn get_id(&self) -> Option<ObjectId> {
        self.id
    }
    
    fn set_id(&mut self, id: ObjectId) {
        self.id = Some(id);
    }
    
    fn get_version(&self) -> Version {
        self.version
    }
    
    fn set_version(&mut self, version: Version) {
        self.version = version;
    }
    
    fn validate(&self) -> Result<(), UnitOfWorkError> {
        if self.username.trim().is_empty() {
            return Err(UnitOfWorkError::ValidationError("用户名不能为空".to_string()));
        }
        if !self.email.contains('@') {
            return Err(UnitOfWorkError::ValidationError("邮箱格式不正确".to_string()));
        }
        if self.balance < 0.0 {
            return Err(UnitOfWorkError::ValidationError("余额不能为负数".to_string()));
        }
        Ok(())
    }
    
    fn clone_box(&self) -> Box<dyn DomainObject> {
        Box::new(self.clone())
    }
}

/// 账户实体
#[derive(Debug, Clone)]
pub struct Account {
    pub id: Option<ObjectId>,
    pub user_id: ObjectId,
    pub account_number: String,
    pub account_type: AccountType,
    pub balance: f64,
    pub version: Version,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AccountType {
    Checking,
    Savings,
    Credit,
}

impl Account {
    pub fn new(user_id: ObjectId, account_number: String, account_type: AccountType) -> Self {
        Self {
            id: None,
            user_id,
            account_number,
            account_type,
            balance: 0.0,
            version: 0,
        }
    }
    
    pub fn credit(&mut self, amount: f64) -> Result<(), UnitOfWorkError> {
        if amount <= 0.0 {
            return Err(UnitOfWorkError::ValidationError("金额必须大于0".to_string()));
        }
        self.balance += amount;
        Ok(())
    }
    
    pub fn debit(&mut self, amount: f64) -> Result<(), UnitOfWorkError> {
        if amount <= 0.0 {
            return Err(UnitOfWorkError::ValidationError("金额必须大于0".to_string()));
        }
        
        // 信用账户可以透支
        if self.account_type != AccountType::Credit && self.balance < amount {
            return Err(UnitOfWorkError::ValidationError("余额不足".to_string()));
        }
        
        self.balance -= amount;
        Ok(())
    }
}

impl DomainObject for Account {
    fn get_id(&self) -> Option<ObjectId> {
        self.id
    }
    
    fn set_id(&mut self, id: ObjectId) {
        self.id = Some(id);
    }
    
    fn get_version(&self) -> Version {
        self.version
    }
    
    fn set_version(&mut self, version: Version) {
        self.version = version;
    }
    
    fn validate(&self) -> Result<(), UnitOfWorkError> {
        if self.account_number.trim().is_empty() {
            return Err(UnitOfWorkError::ValidationError("账户号码不能为空".to_string()));
        }
        // 储蓄账户不能透支
        if self.account_type == AccountType::Savings && self.balance < 0.0 {
            return Err(UnitOfWorkError::ValidationError("储蓄账户余额不能为负数".to_string()));
        }
        Ok(())
    }
    
    fn clone_box(&self) -> Box<dyn DomainObject> {
        Box::new(self.clone())
    }
}

/// 对象状态枚举
#[derive(Debug, Clone, PartialEq)]
pub enum ObjectState {
    New,        // 新创建的对象
    Clean,      // 未修改的对象
    Dirty,      // 已修改的对象
    Removed,    // 标记为删除的对象
}

/// 对象注册信息
struct ObjectRegistration {
    object: Box<dyn DomainObject>,
    state: ObjectState,
    original_version: Version,
}

/// 工作单元实现
pub struct UnitOfWork {
    // 身份映射：缓存加载的对象
    identity_map: HashMap<ObjectId, Rc<RefCell<Box<dyn DomainObject>>>>,
    
    // 对象注册表：追踪对象状态
    new_objects: Vec<Box<dyn DomainObject>>,
    dirty_objects: HashMap<ObjectId, Box<dyn DomainObject>>,
    removed_objects: HashSet<ObjectId>,
    
    // 版本追踪（乐观锁）
    original_versions: HashMap<ObjectId, Version>,
    
    // 模拟数据库
    database: Rc<RefCell<MockDatabase>>,
    
    // 事务状态
    is_in_transaction: bool,
}

impl UnitOfWork {
    pub fn new(database: Rc<RefCell<MockDatabase>>) -> Self {
        Self {
            identity_map: HashMap::new(),
            new_objects: Vec::new(),
            dirty_objects: HashMap::new(),
            removed_objects: HashSet::new(),
            original_versions: HashMap::new(),
            database,
            is_in_transaction: false,
        }
    }
    
    /// 开始事务
    pub fn begin_transaction(&mut self) -> Result<(), UnitOfWorkError> {
        if self.is_in_transaction {
            return Err(UnitOfWorkError::TransactionError("事务已经开始".to_string()));
        }
        
        self.is_in_transaction = true;
        println!("🔄 开始工作单元事务");
        Ok(())
    }
    
    /// 注册新对象
    pub fn register_new(&mut self, mut object: Box<dyn DomainObject>) -> Result<(), UnitOfWorkError> {
        if object.get_id().is_some() {
            return Err(UnitOfWorkError::ValidationError("新对象不应该有ID".to_string()));
        }
        
        object.validate()?;
        self.new_objects.push(object);
        println!("➕ 注册新对象");
        Ok(())
    }
    
    /// 注册脏对象（已修改）
    pub fn register_dirty(&mut self, object: Box<dyn DomainObject>) -> Result<(), UnitOfWorkError> {
        let id = object.get_id()
            .ok_or_else(|| UnitOfWorkError::ValidationError("脏对象必须有ID".to_string()))?;
        
        object.validate()?;
        
        // 检查是否已经在新对象列表中
        if self.new_objects.iter().any(|obj| obj.get_id() == Some(id)) {
            return Ok(()); // 新对象不需要标记为脏
        }
        
        // 如果不在原始版本映射中，添加当前版本
        if !self.original_versions.contains_key(&id) {
            self.original_versions.insert(id, object.get_version());
        }
        
        self.dirty_objects.insert(id, object);
        println!("✏️  注册脏对象 ID: {}", id);
        Ok(())
    }
    
    /// 注册删除对象
    pub fn register_removed(&mut self, object: &dyn DomainObject) -> Result<(), UnitOfWorkError> {
        let id = object.get_id()
            .ok_or_else(|| UnitOfWorkError::ValidationError("删除对象必须有ID".to_string()))?;
        
        // 从其他集合中移除
        self.dirty_objects.remove(&id);
        self.new_objects.retain(|obj| obj.get_id() != Some(id));
        
        // 如果是从数据库加载的对象，记录版本用于并发检查
        if !self.original_versions.contains_key(&id) {
            self.original_versions.insert(id, object.get_version());
        }
        
        self.removed_objects.insert(id);
        println!("🗑️  注册删除对象 ID: {}", id);
        Ok(())
    }
    
    /// 从身份映射获取对象
    pub fn get_object(&self, id: ObjectId) -> Option<Rc<RefCell<Box<dyn DomainObject>>>> {
        self.identity_map.get(&id).cloned()
    }
    
    /// 加载对象到身份映射
    pub fn load_object(&mut self, id: ObjectId) -> Result<Rc<RefCell<Box<dyn DomainObject>>>, UnitOfWorkError> {
        // 首先检查身份映射
        if let Some(obj) = self.identity_map.get(&id) {
            return Ok(obj.clone());
        }
        
        // 从数据库加载
        let object = self.database.borrow().load_object(id)
            .ok_or_else(|| UnitOfWorkError::NotFound(format!("对象 {} 不存在", id)))?;
        
        // 记录原始版本
        self.original_versions.insert(id, object.get_version());
        
        // 添加到身份映射
        let object_ref = Rc::new(RefCell::new(object));
        self.identity_map.insert(id, object_ref.clone());
        
        println!("📥 加载对象到身份映射 ID: {}", id);
        Ok(object_ref)
    }
    
    /// 提交所有变更
    pub fn commit(&mut self) -> Result<(), UnitOfWorkError> {
        if !self.is_in_transaction {
            return Err(UnitOfWorkError::TransactionError("没有活动的事务".to_string()));
        }
        
        println!("💾 开始提交工作单元变更");
        
        // 1. 验证所有对象
        self.validate_all_objects()?;
        
        // 2. 检查并发冲突（乐观锁）
        self.check_concurrency_conflicts()?;
        
        // 3. 按依赖顺序提交变更
        self.commit_in_order()?;
        
        // 4. 清理工作单元
        self.clear();
        
        self.is_in_transaction = false;
        println!("✅ 工作单元事务提交成功");
        Ok(())
    }
    
    /// 回滚事务
    pub fn rollback(&mut self) {
        println!("↩️  回滚工作单元事务");
        
        // 清理所有注册的变更
        self.clear();
        self.is_in_transaction = false;
        
        println!("✅ 工作单元事务已回滚");
    }
    
    /// 验证所有对象
    fn validate_all_objects(&self) -> Result<(), UnitOfWorkError> {
        // 验证新对象
        for object in &self.new_objects {
            object.validate()?;
        }
        
        // 验证脏对象
        for object in self.dirty_objects.values() {
            object.validate()?;
        }
        
        Ok(())
    }
    
    /// 检查并发冲突
    fn check_concurrency_conflicts(&self) -> Result<(), UnitOfWorkError> {
        let db = self.database.borrow();
        
        // 检查脏对象的版本冲突
        for (id, object) in &self.dirty_objects {
            let original_version = self.original_versions.get(id).unwrap_or(&0);
            let current_db_version = db.get_version(*id);
            
            if *original_version != current_db_version {
                return Err(UnitOfWorkError::ConcurrencyConflict(
                    format!("对象 {} 版本冲突: 期望 {}, 实际 {}", 
                           id, original_version, current_db_version)
                ));
            }
        }
        
        // 检查删除对象的版本冲突
        for id in &self.removed_objects {
            let original_version = self.original_versions.get(id).unwrap_or(&0);
            let current_db_version = db.get_version(*id);
            
            if *original_version != current_db_version {
                return Err(UnitOfWorkError::ConcurrencyConflict(
                    format!("删除对象 {} 版本冲突: 期望 {}, 实际 {}", 
                           id, original_version, current_db_version)
                ));
            }
        }
        
        Ok(())
    }
    
    /// 按依赖顺序提交变更
    fn commit_in_order(&mut self) -> Result<(), UnitOfWorkError> {
        let mut db = self.database.borrow_mut();
        
        // 1. 插入新对象
        for mut object in self.new_objects.drain(..) {
            let id = db.insert_object(&mut *object)?;
            object.set_id(id);
            object.set_version(1);
            
            // 更新身份映射
            self.identity_map.insert(id, Rc::new(RefCell::new(object)));
            println!("➕ 插入新对象 ID: {}", id);
        }
        
        // 2. 更新脏对象
        for (id, mut object) in self.dirty_objects.drain() {
            // 增加版本号
            let new_version = object.get_version() + 1;
            object.set_version(new_version);
            
            db.update_object(id, &*object)?;
            
            // 更新身份映射
            if let Some(cached_object) = self.identity_map.get(&id) {
                *cached_object.borrow_mut() = object;
            }
            
            println!("✏️  更新对象 ID: {}, 新版本: {}", id, new_version);
        }
        
        // 3. 删除对象
        for id in self.removed_objects.drain() {
            db.delete_object(id)?;
            self.identity_map.remove(&id);
            println!("🗑️  删除对象 ID: {}", id);
        }
        
        Ok(())
    }
    
    /// 清理工作单元状态
    fn clear(&mut self) {
        self.new_objects.clear();
        self.dirty_objects.clear();
        self.removed_objects.clear();
        self.original_versions.clear();
        // 注意：身份映射在事务外仍然有效
    }
    
    /// 获取工作单元统计信息
    pub fn get_statistics(&self) -> UnitOfWorkStatistics {
        UnitOfWorkStatistics {
            new_objects_count: self.new_objects.len(),
            dirty_objects_count: self.dirty_objects.len(),
            removed_objects_count: self.removed_objects.len(),
            cached_objects_count: self.identity_map.len(),
            is_in_transaction: self.is_in_transaction,
        }
    }
}

/// 工作单元统计信息
#[derive(Debug)]
pub struct UnitOfWorkStatistics {
    pub new_objects_count: usize,
    pub dirty_objects_count: usize,
    pub removed_objects_count: usize,
    pub cached_objects_count: usize,
    pub is_in_transaction: bool,
}

impl Display for UnitOfWorkStatistics {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "工作单元统计: 新对象 {}, 脏对象 {}, 删除对象 {}, 缓存对象 {}, 事务状态: {}", 
               self.new_objects_count, 
               self.dirty_objects_count,
               self.removed_objects_count,
               self.cached_objects_count,
               if self.is_in_transaction { "进行中" } else { "空闲" })
    }
}

/// 模拟数据库
pub struct MockDatabase {
    users: HashMap<ObjectId, Box<dyn DomainObject>>,
    accounts: HashMap<ObjectId, Box<dyn DomainObject>>,
    next_id: ObjectId,
}

impl MockDatabase {
    pub fn new() -> Self {
        Self {
            users: HashMap::new(),
            accounts: HashMap::new(),
            next_id: 1,
        }
    }
    
    pub fn insert_object(&mut self, object: &mut dyn DomainObject) -> Result<ObjectId, UnitOfWorkError> {
        let id = self.next_id;
        self.next_id += 1;
        
        object.set_id(id);
        object.set_version(1);
        
        // 根据对象类型存储（简化实现）
        let cloned_object = object.clone_box();
        
        // 这里简化处理，实际应该根据对象类型判断
        self.users.insert(id, cloned_object);
        
        Ok(id)
    }
    
    pub fn update_object(&mut self, id: ObjectId, object: &dyn DomainObject) -> Result<(), UnitOfWorkError> {
        if self.users.contains_key(&id) {
            self.users.insert(id, object.clone_box());
        } else if self.accounts.contains_key(&id) {
            self.accounts.insert(id, object.clone_box());
        } else {
            return Err(UnitOfWorkError::NotFound(format!("对象 {} 不存在", id)));
        }
        
        Ok(())
    }
    
    pub fn delete_object(&mut self, id: ObjectId) -> Result<(), UnitOfWorkError> {
        let removed = self.users.remove(&id).is_some() || 
                     self.accounts.remove(&id).is_some();
        
        if !removed {
            return Err(UnitOfWorkError::NotFound(format!("对象 {} 不存在", id)));
        }
        
        Ok(())
    }
    
    pub fn load_object(&self, id: ObjectId) -> Option<Box<dyn DomainObject>> {
        self.users.get(&id)
            .or_else(|| self.accounts.get(&id))
            .map(|obj| obj.clone_box())
    }
    
    pub fn get_version(&self, id: ObjectId) -> Version {
        self.load_object(id)
            .map(|obj| obj.get_version())
            .unwrap_or(0)
    }
    
    pub fn get_object_count(&self) -> usize {
        self.users.len() + self.accounts.len()
    }
}

/// Unit of Work模式演示
pub fn demo() {
    println!("=== Unit of Work（工作单元）模式演示 ===\n");
    
    // 1. 初始化数据库和工作单元
    println!("1. 初始化数据库和工作单元:");
    let database = Rc::new(RefCell::new(MockDatabase::new()));
    let mut uow = UnitOfWork::new(database.clone());
    
    println!("数据库和工作单元初始化完成");
    
    println!("{}", "=".repeat(50));
    
    // 2. 创建和注册新对象
    println!("2. 创建和注册新对象:");
    
    match uow.begin_transaction() {
        Ok(_) => {
            // 创建用户
            let user1 = Box::new(User::new("Alice".to_string(), "alice@example.com".to_string()));
            let user2 = Box::new(User::new("Bob".to_string(), "bob@example.com".to_string()));
            
            // 注册新对象
            match uow.register_new(user1) {
                Ok(_) => println!("✅ 用户1注册成功"),
                Err(e) => println!("❌ 用户1注册失败: {}", e),
            }
            
            match uow.register_new(user2) {
                Ok(_) => println!("✅ 用户2注册成功"),
                Err(e) => println!("❌ 用户2注册失败: {}", e),
            }
            
            // 显示工作单元统计
            let stats = uow.get_statistics();
            println!("📊 {}", stats);
            
            // 提交事务
            match uow.commit() {
                Ok(_) => println!("✅ 新对象创建事务提交成功"),
                Err(e) => println!("❌ 事务提交失败: {}", e),
            }
        }
        Err(e) => println!("❌ 开始事务失败: {}", e),
    }
    
    println!("{}", "=".repeat(50));
    
    // 3. 加载和修改对象
    println!("3. 加载和修改对象:");
    
    match uow.begin_transaction() {
        Ok(_) => {
            // 加载对象
            match uow.load_object(1) {
                Ok(user_ref) => {
                    let mut user = user_ref.borrow_mut();
                    
                    // 假设这是User类型（实际中需要更好的类型处理）
                    println!("📥 加载用户 ID: 1");
                    
                    // 模拟修改用户（这里简化处理）
                    println!("💰 用户余额操作...");
                    
                    // 创建修改后的用户对象进行注册
                    let mut modified_user = User::new("Alice".to_string(), "alice@example.com".to_string());
                    modified_user.id = Some(1);
                    modified_user.version = 1;
                    modified_user.balance = 1000.0;
                    
                    match uow.register_dirty(Box::new(modified_user)) {
                        Ok(_) => println!("✅ 用户修改已注册"),
                        Err(e) => println!("❌ 注册脏对象失败: {}", e),
                    }
                }
                Err(e) => println!("❌ 加载对象失败: {}", e),
            }
            
            // 显示工作单元统计
            let stats = uow.get_statistics();
            println!("📊 {}", stats);
            
            // 提交修改
            match uow.commit() {
                Ok(_) => println!("✅ 对象修改事务提交成功"),
                Err(e) => println!("❌ 事务提交失败: {}", e),
            }
        }
        Err(e) => println!("❌ 开始事务失败: {}", e),
    }
    
    println!("{}", "=".repeat(50));
    
    // 4. 复杂业务场景演示
    println!("4. 复杂业务场景演示:");
    
    match uow.begin_transaction() {
        Ok(_) => {
            // 创建多个对象和操作
            let user3 = Box::new(User::new("Charlie".to_string(), "charlie@example.com".to_string()));
            let account1 = Box::new(Account::new(1, "ACC001".to_string(), AccountType::Checking));
            let account2 = Box::new(Account::new(1, "ACC002".to_string(), AccountType::Savings));
            
            // 注册新对象
            let _ = uow.register_new(user3);
            let _ = uow.register_new(account1);
            let _ = uow.register_new(account2);
            
            // 模拟修改已存在的对象
            let mut modified_user = User::new("Alice Updated".to_string(), "alice.new@example.com".to_string());
            modified_user.id = Some(1);
            modified_user.version = 2; // 假设版本已更新
            modified_user.balance = 1500.0;
            
            let _ = uow.register_dirty(Box::new(modified_user));
            
            // 显示事务前的统计
            let stats = uow.get_statistics();
            println!("📊 提交前: {}", stats);
            
            // 提交复杂事务
            match uow.commit() {
                Ok(_) => {
                    println!("✅ 复杂业务事务提交成功");
                    println!("📈 数据库对象总数: {}", database.borrow().get_object_count());
                }
                Err(e) => println!("❌ 复杂事务提交失败: {}", e),
            }
        }
        Err(e) => println!("❌ 开始复杂事务失败: {}", e),
    }
    
    println!("{}", "=".repeat(50));
    
    // 5. 并发冲突演示
    println!("5. 并发冲突演示:");
    
    // 模拟并发冲突场景
    match uow.begin_transaction() {
        Ok(_) => {
            // 加载对象
            if let Ok(user_ref) = uow.load_object(1) {
                // 模拟其他事务修改了数据库中的对象版本
                println!("🔄 模拟其他事务修改对象版本...");
                
                // 创建一个版本冲突的修改
                let mut conflicted_user = User::new("Alice Conflicted".to_string(), "alice@example.com".to_string());
                conflicted_user.id = Some(1);
                conflicted_user.version = 1; // 过期版本
                conflicted_user.balance = 2000.0;
                
                let _ = uow.register_dirty(Box::new(conflicted_user));
                
                // 尝试提交（应该失败）
                match uow.commit() {
                    Ok(_) => println!("⚠️  并发冲突未被检测到（不应该发生）"),
                    Err(e) => println!("✅ 正确检测到并发冲突: {}", e),
                }
            }
        }
        Err(e) => println!("❌ 开始并发测试事务失败: {}", e),
    }
    
    println!("{}", "=".repeat(50));
    
    // 6. 事务回滚演示
    println!("6. 事务回滚演示:");
    
    match uow.begin_transaction() {
        Ok(_) => {
            // 创建一些操作
            let invalid_user = Box::new(User::new("".to_string(), "invalid-email".to_string()));
            
            match uow.register_new(invalid_user) {
                Ok(_) => {
                    println!("注册了无效用户（验证将在提交时失败）");
                    
                    // 添加一些有效操作
                    let valid_user = Box::new(User::new("Valid User".to_string(), "valid@example.com".to_string()));
                    let _ = uow.register_new(valid_user);
                    
                    let stats = uow.get_statistics();
                    println!("📊 回滚前: {}", stats);
                    
                    // 尝试提交（应该失败）
                    match uow.commit() {
                        Ok(_) => println!("⚠️  无效数据提交成功（不应该发生）"),
                        Err(e) => {
                            println!("❌ 提交失败（预期）: {}", e);
                            println!("🔙 执行回滚...");
                            uow.rollback();
                            
                            let stats_after = uow.get_statistics();
                            println!("📊 回滚后: {}", stats_after);
                        }
                    }
                }
                Err(e) => println!("注册失败（预期）: {}", e),
            }
        }
        Err(e) => println!("❌ 开始回滚测试事务失败: {}", e),
    }
    
    println!("\n=== Unit of Work模式演示完成 ===");
    
    // 输出模式总结
    println!("\n【Unit of Work模式总结】");
    println!("核心特点:");
    println!("1. 对象状态追踪：自动追踪新建、修改、删除的对象");
    println!("2. 身份映射：确保同一对象在内存中只有一个实例");
    println!("3. 批量提交：减少数据库交互次数，提高性能");
    println!("4. 事务管理：保证数据的一致性和完整性");
    println!("5. 并发控制：通过版本号实现乐观锁机制");
    
    println!("\n优势:");
    println!("1. 性能提升：批量操作减少数据库往返");
    println!("2. 一致性保证：事务边界确保数据完整性");
    println!("3. 内存效率：身份映射避免重复加载");
    println!("4. 简化编程：自动管理对象状态变化");
    println!("5. 并发安全：乐观锁处理并发冲突");
    
    println!("\n适用场景:");
    println!("1. 复杂的对象关系和依赖");
    println!("2. 需要事务保证的批量操作");
    println!("3. 对性能有较高要求的应用");
    println!("4. 多用户并发访问的系统");
    println!("5. ORM框架的核心实现");
}