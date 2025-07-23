//! # 粗粒度锁模式（Coarse Grained Lock Pattern）
//!
//! 粗粒度锁模式是一种离线并发控制模式，通过锁定一组相关对象来保证并发操作的一致性。
//! 与细粒度锁相比，粗粒度锁能够减少锁的数量，简化并发控制逻辑，但可能会降低并发性能。
//!
//! ## 模式特点
//! - **批量锁定**: 一次性锁定多个相关对象
//! - **减少死锁**: 减少锁的数量降低死锁风险
//! - **简化管理**: 简化锁的获取和释放逻辑
//! - **业务聚合**: 按照业务边界进行锁定

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use std::fmt::{self, Display, Formatter};
use std::error::Error;

/// 锁ID类型
pub type LockId = String;

/// 对象版本类型
pub type Version = u64;

/// 粗粒度锁错误类型
#[derive(Debug)]
pub enum CoarseGrainedLockError {
    LockConflict(String),
    LockNotFound(String),
    LockExpired(String),
    EntityNotFound(String),
    ValidationError(String),
    DatabaseError(String),
}

impl Display for CoarseGrainedLockError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            CoarseGrainedLockError::LockConflict(msg) => write!(f, "锁冲突: {}", msg),
            CoarseGrainedLockError::LockNotFound(msg) => write!(f, "锁未找到: {}", msg),
            CoarseGrainedLockError::LockExpired(msg) => write!(f, "锁已过期: {}", msg),
            CoarseGrainedLockError::EntityNotFound(msg) => write!(f, "实体未找到: {}", msg),
            CoarseGrainedLockError::ValidationError(msg) => write!(f, "验证错误: {}", msg),
            CoarseGrainedLockError::DatabaseError(msg) => write!(f, "数据库错误: {}", msg),
        }
    }
}

impl Error for CoarseGrainedLockError {}

/// 锁信息
#[derive(Debug, Clone)]
pub struct LockInfo {
    pub lock_id: LockId,
    pub owner_id: String,
    pub entities: Vec<String>,
    pub lock_type: LockType,
    pub created_at: u64,
    pub expires_at: u64,
    pub is_active: bool,
}

impl LockInfo {
    pub fn new(lock_id: LockId, owner_id: String, entities: Vec<String>, lock_type: LockType, timeout_seconds: u64) -> Self {
        let now = current_timestamp();
        Self {
            lock_id,
            owner_id,
            entities,
            lock_type,
            created_at: now,
            expires_at: now + timeout_seconds,
            is_active: true,
        }
    }

    /// 检查锁是否已过期
    pub fn is_expired(&self) -> bool {
        current_timestamp() > self.expires_at
    }

    /// 检查是否拥有指定实体的锁
    pub fn owns_entity(&self, entity_id: &str) -> bool {
        self.entities.contains(&entity_id.to_string())
    }

    /// 续期锁
    pub fn renew(&mut self, additional_seconds: u64) {
        self.expires_at = current_timestamp() + additional_seconds;
    }
}

impl Display for LockInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Lock[{}] - Owner: {}, Type: {:?}, Entities: {:?}, Active: {}", 
               self.lock_id, self.owner_id, self.lock_type, self.entities, self.is_active)
    }
}

/// 锁类型
#[derive(Debug, Clone, PartialEq)]
pub enum LockType {
    ReadOnly,    // 只读锁（共享锁）
    ReadWrite,   // 读写锁（排他锁）
    Exclusive,   // 独占锁
}

/// 业务实体trait
pub trait BusinessEntity {
    fn get_id(&self) -> String;
    fn get_version(&self) -> Version;
    fn validate(&self) -> Result<(), CoarseGrainedLockError>;
}

/// 客户实体
#[derive(Debug, Clone)]
pub struct Customer {
    pub id: String,
    pub name: String,
    pub email: String,
    pub phone: String,
    pub version: Version,
    pub credit_limit: f64,
    pub status: CustomerStatus,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CustomerStatus {
    Active,
    Inactive,
    Suspended,
}

impl Customer {
    pub fn new(id: String, name: String, email: String, phone: String) -> Self {
        Self {
            id,
            name,
            email,
            phone,
            version: 1,
            credit_limit: 10000.0,
            status: CustomerStatus::Active,
        }
    }

    pub fn update_credit_limit(&mut self, new_limit: f64) -> Result<(), CoarseGrainedLockError> {
        if new_limit < 0.0 {
            return Err(CoarseGrainedLockError::ValidationError("信用额度不能为负数".to_string()));
        }
        self.credit_limit = new_limit;
        self.version += 1;
        Ok(())
    }

    pub fn update_status(&mut self, new_status: CustomerStatus) {
        self.status = new_status;
        self.version += 1;
    }
}

impl BusinessEntity for Customer {
    fn get_id(&self) -> String {
        self.id.clone()
    }

    fn get_version(&self) -> Version {
        self.version
    }

    fn validate(&self) -> Result<(), CoarseGrainedLockError> {
        if self.name.trim().is_empty() {
            return Err(CoarseGrainedLockError::ValidationError("客户姓名不能为空".to_string()));
        }
        if self.email.trim().is_empty() {
            return Err(CoarseGrainedLockError::ValidationError("客户邮箱不能为空".to_string()));
        }
        Ok(())
    }
}

impl Display for Customer {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Customer[{}] - {}, Email: {}, Credit: {:.2}, Status: {:?}", 
               self.id, self.name, self.email, self.credit_limit, self.status)
    }
}

/// 订单实体
#[derive(Debug, Clone)]
pub struct Order {
    pub id: String,
    pub customer_id: String,
    pub items: Vec<OrderItem>,
    pub total_amount: f64,
    pub status: OrderStatus,
    pub version: Version,
}

#[derive(Debug, Clone)]
pub struct OrderItem {
    pub product_id: String,
    pub quantity: i32,
    pub unit_price: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum OrderStatus {
    Draft,
    Confirmed,
    Processing,
    Shipped,
    Delivered,
    Cancelled,
}

impl Order {
    pub fn new(id: String, customer_id: String) -> Self {
        Self {
            id,
            customer_id,
            items: Vec::new(),
            total_amount: 0.0,
            status: OrderStatus::Draft,
            version: 1,
        }
    }

    pub fn add_item(&mut self, product_id: String, quantity: i32, unit_price: f64) {
        self.items.push(OrderItem { product_id, quantity, unit_price });
        self.calculate_total();
        self.version += 1;
    }

    pub fn calculate_total(&mut self) {
        self.total_amount = self.items.iter()
            .map(|item| item.quantity as f64 * item.unit_price)
            .sum();
    }

    pub fn confirm(&mut self) -> Result<(), CoarseGrainedLockError> {
        if self.status != OrderStatus::Draft {
            return Err(CoarseGrainedLockError::ValidationError("只有草稿状态的订单才能确认".to_string()));
        }
        if self.items.is_empty() {
            return Err(CoarseGrainedLockError::ValidationError("订单必须包含至少一个商品".to_string()));
        }
        self.status = OrderStatus::Confirmed;
        self.version += 1;
        Ok(())
    }

    pub fn cancel(&mut self) -> Result<(), CoarseGrainedLockError> {
        match self.status {
            OrderStatus::Delivered => Err(CoarseGrainedLockError::ValidationError("已交付的订单无法取消".to_string())),
            OrderStatus::Cancelled => Err(CoarseGrainedLockError::ValidationError("订单已经被取消".to_string())),
            _ => {
                self.status = OrderStatus::Cancelled;
                self.version += 1;
                Ok(())
            }
        }
    }
}

impl BusinessEntity for Order {
    fn get_id(&self) -> String {
        self.id.clone()
    }

    fn get_version(&self) -> Version {
        self.version
    }

    fn validate(&self) -> Result<(), CoarseGrainedLockError> {
        if self.customer_id.trim().is_empty() {
            return Err(CoarseGrainedLockError::ValidationError("订单必须关联客户".to_string()));
        }
        if self.total_amount < 0.0 {
            return Err(CoarseGrainedLockError::ValidationError("订单总额不能为负数".to_string()));
        }
        Ok(())
    }
}

impl Display for Order {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Order[{}] - Customer: {}, Amount: {:.2}, Status: {:?}, Items: {}", 
               self.id, self.customer_id, self.total_amount, self.status, self.items.len())
    }
}

/// 粗粒度锁管理器
pub struct CoarseGrainedLockManager {
    locks: Arc<Mutex<HashMap<LockId, LockInfo>>>,
    entity_locks: Arc<Mutex<HashMap<String, LockId>>>, // 实体ID到锁ID的映射
    next_lock_id: Arc<Mutex<u64>>,
}

impl CoarseGrainedLockManager {
    pub fn new() -> Self {
        Self {
            locks: Arc::new(Mutex::new(HashMap::new())),
            entity_locks: Arc::new(Mutex::new(HashMap::new())),
            next_lock_id: Arc::new(Mutex::new(1)),
        }
    }

    /// 获取锁
    pub fn acquire_lock(
        &self, 
        entities: Vec<String>, 
        owner_id: String, 
        lock_type: LockType,
        timeout_seconds: u64
    ) -> Result<LockId, CoarseGrainedLockError> {
        let mut locks = self.locks.lock().unwrap();
        let mut entity_locks = self.entity_locks.lock().unwrap();

        // 清理过期锁
        self.cleanup_expired_locks(&mut locks, &mut entity_locks);

        // 检查锁冲突
        self.check_lock_conflicts(&entities, &lock_type, &entity_locks, &locks)?;

        // 生成锁ID
        let lock_id = format!("lock_{}", *self.next_lock_id.lock().unwrap());
        *self.next_lock_id.lock().unwrap() += 1;

        // 创建锁
        let lock_info = LockInfo::new(lock_id.clone(), owner_id, entities.clone(), lock_type, timeout_seconds);

        // 注册锁
        locks.insert(lock_id.clone(), lock_info);
        for entity_id in entities {
            entity_locks.insert(entity_id, lock_id.clone());
        }

        println!("🔒 获取粗粒度锁成功: {}", lock_id);
        Ok(lock_id)
    }

    /// 释放锁
    pub fn release_lock(&self, lock_id: &LockId, owner_id: &str) -> Result<(), CoarseGrainedLockError> {
        let mut locks = self.locks.lock().unwrap();
        let mut entity_locks = self.entity_locks.lock().unwrap();

        let lock_info = locks.get(lock_id)
            .ok_or_else(|| CoarseGrainedLockError::LockNotFound(lock_id.clone()))?;

        // 验证锁的拥有者
        if lock_info.owner_id != owner_id {
            return Err(CoarseGrainedLockError::LockConflict(
                format!("锁 {} 属于用户 {}, 不能被用户 {} 释放", lock_id, lock_info.owner_id, owner_id)
            ));
        }

        // 释放实体锁
        for entity_id in &lock_info.entities {
            entity_locks.remove(entity_id);
        }

        // 删除锁
        locks.remove(lock_id);

        println!("🔓 释放粗粒度锁成功: {}", lock_id);
        Ok(())
    }

    /// 续期锁
    pub fn renew_lock(&self, lock_id: &LockId, owner_id: &str, additional_seconds: u64) -> Result<(), CoarseGrainedLockError> {
        let mut locks = self.locks.lock().unwrap();

        let lock_info = locks.get_mut(lock_id)
            .ok_or_else(|| CoarseGrainedLockError::LockNotFound(lock_id.clone()))?;

        // 验证锁的拥有者
        if lock_info.owner_id != owner_id {
            return Err(CoarseGrainedLockError::LockConflict(
                format!("锁 {} 属于用户 {}, 不能被用户 {} 续期", lock_id, lock_info.owner_id, owner_id)
            ));
        }

        // 检查锁是否已过期
        if lock_info.is_expired() {
            return Err(CoarseGrainedLockError::LockExpired(format!("锁 {} 已过期", lock_id)));
        }

        lock_info.renew(additional_seconds);
        println!("⏰ 锁续期成功: {}, 新过期时间: {}", lock_id, lock_info.expires_at);
        Ok(())
    }

    /// 检查实体是否被锁定
    pub fn is_locked(&self, entity_id: &str) -> bool {
        let entity_locks = self.entity_locks.lock().unwrap();
        entity_locks.contains_key(entity_id)
    }

    /// 获取实体的锁信息
    pub fn get_entity_lock(&self, entity_id: &str) -> Option<LockInfo> {
        let entity_locks = self.entity_locks.lock().unwrap();
        let locks = self.locks.lock().unwrap();

        if let Some(lock_id) = entity_locks.get(entity_id) {
            locks.get(lock_id).cloned()
        } else {
            None
        }
    }

    /// 列出活跃锁
    pub fn list_active_locks(&self) -> Vec<LockInfo> {
        let locks = self.locks.lock().unwrap();
        locks.values()
            .filter(|lock| lock.is_active && !lock.is_expired())
            .cloned()
            .collect()
    }

    /// 获取锁统计信息
    pub fn get_lock_statistics(&self) -> LockStatistics {
        let locks = self.locks.lock().unwrap();
        let entity_locks = self.entity_locks.lock().unwrap();

        let total_locks = locks.len();
        let active_locks = locks.values().filter(|lock| lock.is_active && !lock.is_expired()).count();
        let expired_locks = locks.values().filter(|lock| lock.is_expired()).count();
        let locked_entities = entity_locks.len();

        LockStatistics {
            total_locks,
            active_locks,
            expired_locks,
            locked_entities,
        }
    }

    /// 检查锁冲突
    fn check_lock_conflicts(
        &self,
        entities: &[String],
        lock_type: &LockType,
        entity_locks: &HashMap<String, LockId>,
        locks: &HashMap<LockId, LockInfo>
    ) -> Result<(), CoarseGrainedLockError> {
        for entity_id in entities {
            if let Some(existing_lock_id) = entity_locks.get(entity_id) {
                if let Some(existing_lock) = locks.get(existing_lock_id) {
                    if !existing_lock.is_expired() {
                        // 检查锁类型兼容性
                        let conflict = match (&existing_lock.lock_type, lock_type) {
                            (LockType::ReadOnly, LockType::ReadOnly) => false, // 读锁兼容
                            _ => true, // 其他情况都冲突
                        };

                        if conflict {
                            return Err(CoarseGrainedLockError::LockConflict(
                                format!("实体 {} 已被锁定 (锁ID: {})", entity_id, existing_lock_id)
                            ));
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// 清理过期锁
    fn cleanup_expired_locks(
        &self,
        locks: &mut HashMap<LockId, LockInfo>,
        entity_locks: &mut HashMap<String, LockId>
    ) {
        let expired_locks: Vec<LockId> = locks.values()
            .filter(|lock| lock.is_expired())
            .map(|lock| lock.lock_id.clone())
            .collect();

        for lock_id in expired_locks {
            if let Some(lock_info) = locks.remove(&lock_id) {
                for entity_id in &lock_info.entities {
                    entity_locks.remove(entity_id);
                }
                println!("🗑️ 清理过期锁: {}", lock_id);
            }
        }
    }
}

/// 锁统计信息
#[derive(Debug)]
pub struct LockStatistics {
    pub total_locks: usize,
    pub active_locks: usize,
    pub expired_locks: usize,
    pub locked_entities: usize,
}

impl Display for LockStatistics {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "锁统计 - 总锁数: {}, 活跃锁: {}, 过期锁: {}, 锁定实体: {}", 
               self.total_locks, self.active_locks, self.expired_locks, self.locked_entities)
    }
}

/// 业务服务
pub struct BusinessService {
    lock_manager: CoarseGrainedLockManager,
    customers: Arc<Mutex<HashMap<String, Customer>>>,
    orders: Arc<Mutex<HashMap<String, Order>>>,
}

impl BusinessService {
    pub fn new() -> Self {
        Self {
            lock_manager: CoarseGrainedLockManager::new(),
            customers: Arc::new(Mutex::new(HashMap::new())),
            orders: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// 创建客户
    pub fn create_customer(&self, id: String, name: String, email: String, phone: String) -> Result<(), CoarseGrainedLockError> {
        let customer = Customer::new(id.clone(), name, email, phone);
        customer.validate()?;

        let mut customers = self.customers.lock().unwrap();
        customers.insert(id, customer);
        Ok(())
    }

    /// 创建订单
    pub fn create_order(&self, id: String, customer_id: String) -> Result<(), CoarseGrainedLockError> {
        let order = Order::new(id.clone(), customer_id);
        order.validate()?;

        let mut orders = self.orders.lock().unwrap();
        orders.insert(id, order);
        Ok(())
    }

    /// 处理客户订单（需要同时锁定客户和订单）
    pub fn process_customer_order(
        &self,
        customer_id: &str,
        order_id: &str,
        operator_id: &str
    ) -> Result<(), CoarseGrainedLockError> {
        // 获取粗粒度锁，同时锁定客户和订单
        let entities = vec![customer_id.to_string(), order_id.to_string()];
        let lock_id = self.lock_manager.acquire_lock(
            entities,
            operator_id.to_string(),
            LockType::Exclusive,
            300 // 5分钟超时
        )?;

        // 在锁保护下执行业务操作
        let result = self.perform_order_processing(customer_id, order_id);

        // 释放锁
        self.lock_manager.release_lock(&lock_id, operator_id)?;

        result
    }

    /// 执行订单处理业务逻辑
    fn perform_order_processing(&self, customer_id: &str, order_id: &str) -> Result<(), CoarseGrainedLockError> {
        let mut customers = self.customers.lock().unwrap();
        let mut orders = self.orders.lock().unwrap();

        // 获取客户和订单
        let customer = customers.get_mut(customer_id)
            .ok_or_else(|| CoarseGrainedLockError::EntityNotFound(format!("客户 {} 不存在", customer_id)))?;
        
        let order = orders.get_mut(order_id)
            .ok_or_else(|| CoarseGrainedLockError::EntityNotFound(format!("订单 {} 不存在", order_id)))?;

        // 验证订单属于该客户
        if order.customer_id != customer_id {
            return Err(CoarseGrainedLockError::ValidationError("订单不属于指定客户".to_string()));
        }

        // 检查客户信用额度
        if order.total_amount > customer.credit_limit {
            return Err(CoarseGrainedLockError::ValidationError("订单金额超出客户信用额度".to_string()));
        }

        // 确认订单
        order.confirm()?;

        // 更新客户信用额度
        customer.credit_limit -= order.total_amount;
        customer.version += 1;

        println!("✅ 订单处理完成: 客户 {} 的订单 {} 已确认，剩余信用额度: {:.2}", 
                 customer_id, order_id, customer.credit_limit);

        Ok(())
    }

    /// 获取客户信息
    pub fn get_customer(&self, customer_id: &str) -> Option<Customer> {
        let customers = self.customers.lock().unwrap();
        customers.get(customer_id).cloned()
    }

    /// 获取订单信息
    pub fn get_order(&self, order_id: &str) -> Option<Order> {
        let orders = self.orders.lock().unwrap();
        orders.get(order_id).cloned()
    }

    /// 获取锁统计信息
    pub fn get_lock_statistics(&self) -> LockStatistics {
        self.lock_manager.get_lock_statistics()
    }
}

/// 获取当前时间戳
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

/// 演示粗粒度锁模式
pub fn demo() {
    println!("=== 粗粒度锁模式演示 ===\n");

    let service = BusinessService::new();

    // 创建测试数据
    println!("1. 创建测试数据");
    let _ = service.create_customer("cust001".to_string(), "张三".to_string(), "zhang@example.com".to_string(), "13800138000".to_string());
    let _ = service.create_customer("cust002".to_string(), "李四".to_string(), "li@example.com".to_string(), "13900139000".to_string());
    
    let _ = service.create_order("order001".to_string(), "cust001".to_string());
    let _ = service.create_order("order002".to_string(), "cust002".to_string());

    // 添加订单项目
    {
        let mut orders = service.orders.lock().unwrap();
        if let Some(order) = orders.get_mut("order001") {
            order.add_item("product001".to_string(), 2, 1500.0);
            order.add_item("product002".to_string(), 1, 800.0);
        }
        if let Some(order) = orders.get_mut("order002") {
            order.add_item("product003".to_string(), 3, 2000.0);
        }
    }

    println!("   创建了 2 个客户和 2 个订单");

    // 显示初始状态
    println!("\n2. 初始状态");
    if let Some(customer) = service.get_customer("cust001") {
        println!("   {}", customer);
    }
    if let Some(order) = service.get_order("order001") {
        println!("   {}", order);
    }

    // 演示粗粒度锁的使用
    println!("\n3. 使用粗粒度锁处理订单");
    match service.process_customer_order("cust001", "order001", "operator1") {
        Ok(_) => println!("   订单处理成功"),
        Err(e) => println!("   订单处理失败: {}", e),
    }

    // 显示处理后状态
    println!("\n4. 处理后状态");
    if let Some(customer) = service.get_customer("cust001") {
        println!("   {}", customer);
    }
    if let Some(order) = service.get_order("order001") {
        println!("   {}", order);
    }

    // 模拟并发冲突
    println!("\n5. 模拟并发冲突");
    let lock_manager = &service.lock_manager;
    
    // 操作员1获取锁
    match lock_manager.acquire_lock(
        vec!["cust002".to_string(), "order002".to_string()],
        "operator1".to_string(),
        LockType::Exclusive,
        300
    ) {
        Ok(lock_id) => {
            println!("   操作员1获取锁成功: {}", lock_id);
            
            // 操作员2尝试获取同样的锁（应该失败）
            match lock_manager.acquire_lock(
                vec!["cust002".to_string()],
                "operator2".to_string(),
                LockType::Exclusive,
                300
            ) {
                Ok(_) => println!("   操作员2获取锁成功（不应该发生）"),
                Err(e) => println!("   操作员2获取锁失败: {}", e),
            }
            
            // 释放锁
            let _ = lock_manager.release_lock(&lock_id, "operator1");
            println!("   操作员1释放锁成功");
        }
        Err(e) => println!("   操作员1获取锁失败: {}", e),
    }

    // 显示锁统计信息
    println!("\n6. 锁统计信息");
    let stats = service.get_lock_statistics();
    println!("   {}", stats);

    // 列出活跃锁
    println!("\n7. 活跃锁列表");
    let active_locks = lock_manager.list_active_locks();
    if active_locks.is_empty() {
        println!("   当前没有活跃锁");
    } else {
        for lock in active_locks {
            println!("   {}", lock);
        }
    }

    println!("\n=== 粗粒度锁模式演示完成 ===");

    println!("\n💡 粗粒度锁模式的优势:");
    println!("1. 简化锁管理 - 减少锁的数量，简化获取和释放逻辑");
    println!("2. 降低死锁风险 - 减少锁的交互，降低死锁发生概率");
    println!("3. 业务聚合 - 按照业务边界进行锁定，符合业务逻辑");
    println!("4. 一致性保证 - 确保相关对象的一致性操作");

    println!("\n⚠️ 设计考虑:");
    println!("1. 性能权衡 - 可能降低系统并发性能");
    println!("2. 锁粒度 - 需要平衡锁的粒度和并发性");
    println!("3. 超时管理 - 合理设置锁的超时时间");
    println!("4. 异常处理 - 确保在异常情况下能够正确释放锁");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coarse_grained_lock_basic() {
        let manager = CoarseGrainedLockManager::new();
        
        // 获取锁
        let entities = vec!["entity1".to_string(), "entity2".to_string()];
        let lock_id = manager.acquire_lock(entities, "user1".to_string(), LockType::Exclusive, 300).unwrap();
        
        // 检查实体是否被锁定
        assert!(manager.is_locked("entity1"));
        assert!(manager.is_locked("entity2"));
        
        // 释放锁
        assert!(manager.release_lock(&lock_id, "user1").is_ok());
        
        // 检查实体是否已解锁
        assert!(!manager.is_locked("entity1"));
        assert!(!manager.is_locked("entity2"));
    }

    #[test]
    fn test_lock_conflict() {
        let manager = CoarseGrainedLockManager::new();
        
        // 用户1获取锁
        let entities1 = vec!["entity1".to_string()];
        let lock_id1 = manager.acquire_lock(entities1, "user1".to_string(), LockType::Exclusive, 300).unwrap();
        
        // 用户2尝试获取同一实体的锁（应该失败）
        let entities2 = vec!["entity1".to_string()];
        assert!(manager.acquire_lock(entities2, "user2".to_string(), LockType::Exclusive, 300).is_err());
        
        // 释放锁后用户2应该能够获取锁
        assert!(manager.release_lock(&lock_id1, "user1").is_ok());
        assert!(manager.acquire_lock(vec!["entity1".to_string()], "user2".to_string(), LockType::Exclusive, 300).is_ok());
    }

    #[test]
    fn test_read_lock_sharing() {
        let manager = CoarseGrainedLockManager::new();
        
        // 用户1获取读锁
        let lock_id1 = manager.acquire_lock(vec!["entity1".to_string()], "user1".to_string(), LockType::ReadOnly, 300).unwrap();
        
        // 用户2也可以获取读锁
        let lock_id2 = manager.acquire_lock(vec!["entity1".to_string()], "user2".to_string(), LockType::ReadOnly, 300).unwrap();
        
        assert!(manager.is_locked("entity1"));
        
        // 但是用户3不能获取写锁
        assert!(manager.acquire_lock(vec!["entity1".to_string()], "user3".to_string(), LockType::ReadWrite, 300).is_err());
        
        // 释放所有读锁
        assert!(manager.release_lock(&lock_id1, "user1").is_ok());
        assert!(manager.release_lock(&lock_id2, "user2").is_ok());
    }
} 