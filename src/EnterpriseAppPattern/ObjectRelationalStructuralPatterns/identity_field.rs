// 文件路径: /d%3A/workspace/RustLearn/RustDesignPattern/src/EnterpriseAppPattern/ObjectRelationalStructuralPatterns/identity_field.rs

//! # 身份字段模式 (Identity Field)
//!
//! ## 概述
//! 身份字段模式为数据库表中的每一行数据在对象中保存一个ID字段，
//! 用来维护对象与数据库记录之间的对应关系。
//!
//! ## 优点
//! - 提供对象与数据库记录的唯一映射
//! - 支持高效的对象查找和更新
//! - 便于实现对象缓存和身份映射
//! - 支持数据库外键关系
//!
//! ## 适用场景
//! - 所有需要持久化的领域对象
//! - 需要维护对象引用关系的系统
//! - 实现对象-关系映射的基础设施

use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, Mutex};

/// 身份标识符类型
pub trait IdentityId: Clone + fmt::Debug + fmt::Display + Eq + std::hash::Hash {
    fn is_valid(&self) -> bool;
}

/// 数字类型ID
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NumericId(pub u64);

impl NumericId {
    pub fn new(value: u64) -> Self {
        Self(value)
    }

    pub fn value(&self) -> u64 {
        self.0
    }
}

impl IdentityId for NumericId {
    fn is_valid(&self) -> bool {
        self.0 > 0
    }
}

impl fmt::Display for NumericId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// UUID类型ID
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UuidId(pub String);

impl UuidId {
    pub fn new(value: String) -> Self {
        Self(value)
    }

    pub fn generate() -> Self {
        // 简化的UUID生成（实际应用中应使用uuid crate）
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        Self(format!("uuid-{}", timestamp))
    }

    pub fn value(&self) -> &str {
        &self.0
    }
}

impl IdentityId for UuidId {
    fn is_valid(&self) -> bool {
        !self.0.is_empty() && self.0.len() >= 10
    }
}

impl fmt::Display for UuidId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// 可识别的实体接口
pub trait Identifiable<ID: IdentityId> {
    fn get_id(&self) -> Option<&ID>;
    fn set_id(&mut self, id: ID);
    fn is_new(&self) -> bool {
        self.get_id().is_none()
    }
}

/// 用户实体
#[derive(Debug, Clone)]
pub struct User {
    id: Option<NumericId>,
    pub username: String,
    pub email: String,
    pub full_name: String,
    pub age: u32,
    pub created_at: u64,
    pub updated_at: u64,
}

impl User {
    pub fn new(username: String, email: String, full_name: String, age: u32) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            id: None,
            username,
            email,
            full_name,
            age,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn update_info(&mut self, full_name: Option<String>, age: Option<u32>) {
        if let Some(name) = full_name {
            self.full_name = name;
        }
        if let Some(new_age) = age {
            self.age = new_age;
        }
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
    }
}

impl Identifiable<NumericId> for User {
    fn get_id(&self) -> Option<&NumericId> {
        self.id.as_ref()
    }

    fn set_id(&mut self, id: NumericId) {
        self.id = Some(id);
    }
}

impl fmt::Display for User {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let id_str = match &self.id {
            Some(id) => id.to_string(),
            None => "NEW".to_string(),
        };
        write!(f, "User[{}]: {} ({}) - {}", id_str, self.username, self.email, self.full_name)
    }
}

/// 订单实体
#[derive(Debug, Clone)]
pub struct Order {
    id: Option<UuidId>,
    pub user_id: NumericId,
    pub product_name: String,
    pub quantity: u32,
    pub unit_price: f64,
    pub total_amount: f64,
    pub status: OrderStatus,
    pub created_at: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum OrderStatus {
    Pending,
    Confirmed,
    Shipped,
    Delivered,
    Cancelled,
}

impl fmt::Display for OrderStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OrderStatus::Pending => write!(f, "待处理"),
            OrderStatus::Confirmed => write!(f, "已确认"),
            OrderStatus::Shipped => write!(f, "已发货"),
            OrderStatus::Delivered => write!(f, "已送达"),
            OrderStatus::Cancelled => write!(f, "已取消"),
        }
    }
}

impl Order {
    pub fn new(user_id: NumericId, product_name: String, quantity: u32, unit_price: f64) -> Self {
        let total_amount = quantity as f64 * unit_price;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            id: None,
            user_id,
            product_name,
            quantity,
            unit_price,
            total_amount,
            status: OrderStatus::Pending,
            created_at: now,
        }
    }

    pub fn confirm(&mut self) {
        self.status = OrderStatus::Confirmed;
    }

    pub fn ship(&mut self) {
        if self.status == OrderStatus::Confirmed {
            self.status = OrderStatus::Shipped;
        }
    }

    pub fn deliver(&mut self) {
        if self.status == OrderStatus::Shipped {
            self.status = OrderStatus::Delivered;
        }
    }

    pub fn cancel(&mut self) {
        if matches!(self.status, OrderStatus::Pending | OrderStatus::Confirmed) {
            self.status = OrderStatus::Cancelled;
        }
    }
}

impl Identifiable<UuidId> for Order {
    fn get_id(&self) -> Option<&UuidId> {
        self.id.as_ref()
    }

    fn set_id(&mut self, id: UuidId) {
        self.id = Some(id);
    }
}

impl fmt::Display for Order {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let id_str = match &self.id {
            Some(id) => id.to_string(),
            None => "NEW".to_string(),
        };
        write!(f, "Order[{}]: {} x{} - {:.2} ({})", 
               id_str, self.product_name, self.quantity, self.total_amount, self.status)
    }
}

/// ID生成器接口
pub trait IdGenerator<ID: IdentityId> {
    fn generate(&self) -> ID;
}

/// 数字ID生成器
pub struct NumericIdGenerator {
    next_id: Arc<Mutex<u64>>,
}

impl NumericIdGenerator {
    pub fn new(start_from: u64) -> Self {
        Self {
            next_id: Arc::new(Mutex::new(start_from)),
        }
    }
}

impl IdGenerator<NumericId> for NumericIdGenerator {
    fn generate(&self) -> NumericId {
        let mut next_id = self.next_id.lock().unwrap();
        let id = *next_id;
        *next_id += 1;
        NumericId::new(id)
    }
}

/// UUID生成器
pub struct UuidIdGenerator;

impl UuidIdGenerator {
    pub fn new() -> Self {
        Self
    }
}

impl IdGenerator<UuidId> for UuidIdGenerator {
    fn generate(&self) -> UuidId {
        UuidId::generate()
    }
}

/// 身份映射 (Identity Map)
pub struct IdentityMap<ID: IdentityId, T: Identifiable<ID> + Clone> {
    entities: HashMap<ID, T>,
}

impl<ID: IdentityId, T: Identifiable<ID> + Clone> IdentityMap<ID, T> {
    pub fn new() -> Self {
        Self {
            entities: HashMap::new(),
        }
    }

    pub fn get(&self, id: &ID) -> Option<&T> {
        self.entities.get(id)
    }

    pub fn put(&mut self, entity: T) {
        if let Some(id) = entity.get_id() {
            self.entities.insert(id.clone(), entity);
        }
    }

    pub fn remove(&mut self, id: &ID) -> Option<T> {
        self.entities.remove(id)
    }

    pub fn contains(&self, id: &ID) -> bool {
        self.entities.contains_key(id)
    }

    pub fn size(&self) -> usize {
        self.entities.len()
    }

    pub fn get_all(&self) -> Vec<&T> {
        self.entities.values().collect()
    }

    pub fn clear(&mut self) {
        self.entities.clear();
    }
}

/// 实体仓储（带身份字段支持）
pub struct EntityRepository<ID: IdentityId, T: Identifiable<ID> + Clone> {
    storage: HashMap<ID, T>,
    identity_map: IdentityMap<ID, T>,
    id_generator: Box<dyn IdGenerator<ID>>,
}

impl<ID: IdentityId, T: Identifiable<ID> + Clone> EntityRepository<ID, T> {
    pub fn new(id_generator: Box<dyn IdGenerator<ID>>) -> Self {
        Self {
            storage: HashMap::new(),
            identity_map: IdentityMap::new(),
            id_generator,
        }
    }

    /// 保存实体
    pub fn save(&mut self, mut entity: T) -> Result<T, String> {
        if entity.is_new() {
            // 新实体，生成ID
            let id = self.id_generator.generate();
            entity.set_id(id.clone());
            
            self.storage.insert(id.clone(), entity.clone());
            self.identity_map.put(entity.clone());
            
            Ok(entity)
        } else {
            // 更新现有实体
            if let Some(id) = entity.get_id() {
                self.storage.insert(id.clone(), entity.clone());
                self.identity_map.put(entity.clone());
                Ok(entity)
            } else {
                Err("实体ID无效".to_string())
            }
        }
    }

    /// 根据ID查找实体
    pub fn find_by_id(&mut self, id: &ID) -> Option<T> {
        // 首先检查身份映射
        if let Some(entity) = self.identity_map.get(id) {
            return Some(entity.clone());
        }

        // 从存储中加载
        if let Some(entity) = self.storage.get(id) {
            let entity = entity.clone();
            self.identity_map.put(entity.clone());
            Some(entity)
        } else {
            None
        }
    }

    /// 删除实体
    pub fn delete(&mut self, id: &ID) -> bool {
        self.identity_map.remove(id);
        self.storage.remove(id).is_some()
    }

    /// 获取所有实体
    pub fn find_all(&self) -> Vec<T> {
        self.storage.values().cloned().collect()
    }

    /// 检查实体是否存在
    pub fn exists(&self, id: &ID) -> bool {
        self.storage.contains_key(id)
    }

    /// 获取实体数量
    pub fn count(&self) -> usize {
        self.storage.len()
    }

    /// 清空缓存
    pub fn clear_cache(&mut self) {
        self.identity_map.clear();
    }
}

/// 演示身份字段模式
pub fn demo() {
    println!("=== 身份字段模式演示 ===\n");

    // 创建ID生成器
    let numeric_generator = Box::new(NumericIdGenerator::new(1));
    let uuid_generator = Box::new(UuidIdGenerator::new());

    // 创建仓储
    let mut user_repo: EntityRepository<NumericId, User> = EntityRepository::new(numeric_generator);
    let mut order_repo: EntityRepository<UuidId, Order> = EntityRepository::new(uuid_generator);

    println!("1. 创建用户");
    let users_data = vec![
        ("alice", "alice@example.com", "Alice Johnson", 25),
        ("bob", "bob@example.com", "Bob Smith", 32),
        ("charlie", "charlie@example.com", "Charlie Brown", 28),
    ];

    let mut created_users = Vec::new();
    for (username, email, full_name, age) in users_data {
        let user = User::new(username.to_string(), email.to_string(), full_name.to_string(), age);
        match user_repo.save(user) {
            Ok(saved_user) => {
                println!("   ✅ 创建用户: {}", saved_user);
                created_users.push(saved_user);
            }
            Err(e) => println!("   ❌ 创建用户失败: {}", e),
        }
    }

    println!("\n2. 创建订单");
    if let Some(first_user) = created_users.first() {
        if let Some(user_id) = first_user.get_id() {
            let orders_data = vec![
                ("笔记本电脑", 1, 8999.00),
                ("无线鼠标", 2, 199.00),
                ("机械键盘", 1, 599.00),
            ];

            for (product, quantity, price) in orders_data {
                let order = Order::new(
                    user_id.clone(),
                    product.to_string(),
                    quantity,
                    price,
                );
                match order_repo.save(order) {
                    Ok(saved_order) => println!("   ✅ 创建订单: {}", saved_order),
                    Err(e) => println!("   ❌ 创建订单失败: {}", e),
                }
            }
        }
    }

    println!("\n3. 查找实体");
    // 查找用户
    if let Some(first_user) = created_users.first() {
        if let Some(user_id) = first_user.get_id() {
            match user_repo.find_by_id(user_id) {
                Some(user) => println!("   根据ID查找用户: {}", user),
                None => println!("   用户未找到"),
            }
        }
    }

    // 查找所有订单
    println!("   所有订单:");
    for order in order_repo.find_all() {
        println!("     - {}", order);
    }

    println!("\n4. 更新实体");
    if let Some(mut user) = created_users.first().cloned() {
        user.update_info(Some("Alice Johnson Updated".to_string()), Some(26));
        match user_repo.save(user) {
            Ok(updated_user) => println!("   ✅ 更新用户: {}", updated_user),
            Err(e) => println!("   ❌ 更新用户失败: {}", e),
        }
    }

    println!("\n5. 身份映射演示");
    let mut identity_map: IdentityMap<NumericId, User> = IdentityMap::new();
    
    // 添加用户到身份映射
    for user in &created_users {
        identity_map.put(user.clone());
    }
    
    println!("   身份映射中的用户数量: {}", identity_map.size());
    
    // 从身份映射中查找
    if let Some(first_user) = created_users.first() {
        if let Some(user_id) = first_user.get_id() {
            if let Some(cached_user) = identity_map.get(user_id) {
                println!("   从身份映射查找: {}", cached_user);
            }
        }
    }

    println!("\n6. ID类型演示");
    // 数字ID
    let numeric_id = NumericId::new(123);
    println!("   数字ID: {} (有效: {})", numeric_id, numeric_id.is_valid());
    
    let invalid_numeric = NumericId::new(0);
    println!("   无效数字ID: {} (有效: {})", invalid_numeric, invalid_numeric.is_valid());
    
    // UUID
    let uuid_id = UuidId::generate();
    println!("   UUID: {} (有效: {})", uuid_id, uuid_id.is_valid());
    
    let invalid_uuid = UuidId::new("".to_string());
    println!("   无效UUID: {} (有效: {})", invalid_uuid, invalid_uuid.is_valid());

    println!("\n7. 统计信息");
    println!("   用户数量: {}", user_repo.count());
    println!("   订单数量: {}", order_repo.count());

    println!("\n8. 删除实体");
    if let Some(last_user) = created_users.last() {
        if let Some(user_id) = last_user.get_id() {
            if user_repo.delete(user_id) {
                println!("   ✅ 删除用户成功");
            } else {
                println!("   ❌ 删除用户失败");
            }
        }
    }
    
    println!("   删除后用户数量: {}", user_repo.count());

    println!("\n=== 身份字段模式演示完成 ===");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_numeric_id() {
        let id = NumericId::new(123);
        assert_eq!(id.value(), 123);
        assert!(id.is_valid());
        
        let invalid_id = NumericId::new(0);
        assert!(!invalid_id.is_valid());
    }

    #[test]
    fn test_uuid_id() {
        let id = UuidId::generate();
        assert!(id.is_valid());
        assert!(!id.value().is_empty());
        
        let invalid_id = UuidId::new("".to_string());
        assert!(!invalid_id.is_valid());
    }

    #[test]
    fn test_user_identity() {
        let mut user = User::new(
            "test".to_string(),
            "test@example.com".to_string(),
            "Test User".to_string(),
            25,
        );
        
        assert!(user.is_new());
        
        let id = NumericId::new(1);
        user.set_id(id.clone());
        
        assert!(!user.is_new());
        assert_eq!(user.get_id(), Some(&id));
    }

    #[test]
    fn test_identity_map() {
        let mut map: IdentityMap<NumericId, User> = IdentityMap::new();
        
        let mut user = User::new(
            "test".to_string(),
            "test@example.com".to_string(),
            "Test User".to_string(),
            25,
        );
        user.set_id(NumericId::new(1));
        
        map.put(user.clone());
        
        assert_eq!(map.size(), 1);
        assert!(map.contains(&NumericId::new(1)));
        assert_eq!(map.get(&NumericId::new(1)).unwrap().username, "test");
    }

    #[test]
    fn test_entity_repository() {
        let generator = Box::new(NumericIdGenerator::new(1));
        let mut repo: EntityRepository<NumericId, User> = EntityRepository::new(generator);
        
        let user = User::new(
            "test".to_string(),
            "test@example.com".to_string(),
            "Test User".to_string(),
            25,
        );
        
        let saved_user = repo.save(user).unwrap();
        assert!(!saved_user.is_new());
        
        let found_user = repo.find_by_id(saved_user.get_id().unwrap()).unwrap();
        assert_eq!(found_user.username, "test");
        
        assert_eq!(repo.count(), 1);
    }
} 