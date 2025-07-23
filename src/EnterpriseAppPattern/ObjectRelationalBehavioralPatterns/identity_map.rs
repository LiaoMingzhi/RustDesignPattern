// 文件路径: /d%3A/workspace/RustLearn/RustDesignPattern/src/EnterpriseAppPattern/ObjectRelationalBehavioralPatterns/identity_map.rs

//! # 身份映射模式 (Identity Map)
//!
//! ## 概述
//! 身份映射模式确保每个对象在一个工作单元中只加载一次，
//! 通过保持所有已加载对象的映射来实现这一点。
//!
//! ## 优点
//! - 避免重复加载同一对象
//! - 确保对象的身份一致性
//! - 提高性能，减少数据库访问
//! - 解决循环引用问题
//! - 支持缓存策略
//!
//! ## 适用场景
//! - 需要保证对象唯一性的系统
//! - 有复杂对象关系的应用
//! - 性能敏感的应用
//! - 需要避免重复查询的场景

use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, Mutex, Weak};
use std::any::{Any, TypeId};

/// 身份映射错误
#[derive(Debug)]
pub enum IdentityMapError {
    NotFound(String),
    TypeMismatch(String),
    WeakRefDropped(String),
}

impl fmt::Display for IdentityMapError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IdentityMapError::NotFound(msg) => write!(f, "对象未找到: {}", msg),
            IdentityMapError::TypeMismatch(msg) => write!(f, "类型不匹配: {}", msg),
            IdentityMapError::WeakRefDropped(msg) => write!(f, "弱引用已丢失: {}", msg),
        }
    }
}

impl std::error::Error for IdentityMapError {}

/// 可识别的对象接口
pub trait Identifiable: Any + Send + Sync {
    type Id: Clone + Eq + std::hash::Hash + fmt::Debug;
    
    fn get_id(&self) -> Self::Id;
    fn get_type_name(&self) -> &'static str;
}

/// 身份映射实现
pub struct IdentityMap {
    // 使用 TypeId 作为类型键，String 作为对象ID键
    objects: HashMap<TypeId, HashMap<String, Weak<dyn Any + Send + Sync>>>,
    // 统计信息
    hit_count: u64,
    miss_count: u64,
}

impl IdentityMap {
    pub fn new() -> Self {
        Self {
            objects: HashMap::new(),
            hit_count: 0,
            miss_count: 0,
        }
    }

    /// 添加对象到身份映射
    pub fn put<T: Identifiable + 'static>(&mut self, object: Arc<T>) {
        let type_id = TypeId::of::<T>();
        let object_id = format!("{:?}", object.get_id());
        
        self.objects
            .entry(type_id)
            .or_insert_with(HashMap::new)
            .insert(object_id, Arc::downgrade(&object) as Weak<dyn Any + Send + Sync>);
    }

    /// 从身份映射获取对象
    pub fn get<T: Identifiable + 'static>(&mut self, id: &T::Id) -> Option<Arc<T>> {
        let type_id = TypeId::of::<T>();
        let object_id = format!("{:?}", id);
        
        if let Some(type_map) = self.objects.get_mut(&type_id) {
            if let Some(weak_ref) = type_map.get(&object_id) {
                if let Some(object) = weak_ref.upgrade() {
                    self.hit_count += 1;
                    // 尝试向下转型为具体类型
                    if let Ok(typed_object) = object.downcast::<T>() {
                        return Some(typed_object);
                    }
                } else {
                    // 弱引用已失效，移除它
                    type_map.remove(&object_id);
                }
            }
        }
        
        self.miss_count += 1;
        None
    }

    /// 检查对象是否存在
    pub fn contains<T: Identifiable + 'static>(&mut self, id: &T::Id) -> bool {
        self.get::<T>(id).is_some()
    }

    /// 移除对象
    pub fn remove<T: Identifiable + 'static>(&mut self, id: &T::Id) -> bool {
        let type_id = TypeId::of::<T>();
        let object_id = format!("{:?}", id);
        
        if let Some(type_map) = self.objects.get_mut(&type_id) {
            type_map.remove(&object_id).is_some()
        } else {
            false
        }
    }

    /// 清空指定类型的所有对象
    pub fn clear_type<T: 'static>(&mut self) {
        let type_id = TypeId::of::<T>();
        self.objects.remove(&type_id);
    }

    /// 清空所有对象
    pub fn clear(&mut self) {
        self.objects.clear();
        self.hit_count = 0;
        self.miss_count = 0;
    }

    /// 清理失效的弱引用
    pub fn cleanup(&mut self) {
        for type_map in self.objects.values_mut() {
            type_map.retain(|_, weak_ref| weak_ref.strong_count() > 0);
        }
        // 移除空的类型映射
        self.objects.retain(|_, type_map| !type_map.is_empty());
    }

    /// 获取统计信息
    pub fn get_statistics(&self) -> IdentityMapStatistics {
        let total_types = self.objects.len();
        let total_objects: usize = self.objects.values()
            .map(|type_map| type_map.len())
            .sum();
        let hit_rate = if self.hit_count + self.miss_count > 0 {
            self.hit_count as f64 / (self.hit_count + self.miss_count) as f64
        } else {
            0.0
        };

        IdentityMapStatistics {
            total_types,
            total_objects,
            hit_count: self.hit_count,
            miss_count: self.miss_count,
            hit_rate,
        }
    }
}

/// 身份映射统计信息
#[derive(Debug, Clone)]
pub struct IdentityMapStatistics {
    pub total_types: usize,
    pub total_objects: usize,
    pub hit_count: u64,
    pub miss_count: u64,
    pub hit_rate: f64,
}

impl fmt::Display for IdentityMapStatistics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "IdentityMap统计 - 类型数: {}, 对象数: {}, 命中: {}, 未命中: {}, 命中率: {:.2}%",
               self.total_types, self.total_objects, self.hit_count, self.miss_count, self.hit_rate * 100.0)
    }
}

/// 示例用户实体
#[derive(Debug, Clone)]
pub struct User {
    pub id: u32,
    pub username: String,
    pub email: String,
    pub full_name: String,
}

impl User {
    pub fn new(id: u32, username: String, email: String, full_name: String) -> Self {
        Self { id, username, email, full_name }
    }
}

impl Identifiable for User {
    type Id = u32;
    
    fn get_id(&self) -> Self::Id {
        self.id
    }
    
    fn get_type_name(&self) -> &'static str {
        "User"
    }
}

impl fmt::Display for User {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "User[{}]: {} ({}) - {}", self.id, self.username, self.email, self.full_name)
    }
}

/// 示例订单实体
#[derive(Debug, Clone)]
pub struct Order {
    pub id: u32,
    pub user_id: u32,
    pub total_amount: f64,
    pub status: String,
}

impl Order {
    pub fn new(id: u32, user_id: u32, total_amount: f64, status: String) -> Self {
        Self { id, user_id, total_amount, status }
    }
}

impl Identifiable for Order {
    type Id = u32;
    
    fn get_id(&self) -> Self::Id {
        self.id
    }
    
    fn get_type_name(&self) -> &'static str {
        "Order"
    }
}

impl fmt::Display for Order {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Order[{}]: 用户{} - {:.2} ({})", self.id, self.user_id, self.total_amount, self.status)
    }
}

/// 带身份映射的数据访问层
pub struct DataAccessLayer {
    identity_map: IdentityMap,
    // 模拟数据库
    users_db: HashMap<u32, User>,
    orders_db: HashMap<u32, Order>,
    next_user_id: u32,
    next_order_id: u32,
}

impl DataAccessLayer {
    pub fn new() -> Self {
        Self {
            identity_map: IdentityMap::new(),
            users_db: HashMap::new(),
            orders_db: HashMap::new(),
            next_user_id: 1,
            next_order_id: 1,
        }
    }

    /// 查找用户（带身份映射）
    pub fn find_user(&mut self, id: u32) -> Option<Arc<User>> {
        // 首先检查身份映射
        if let Some(user) = self.identity_map.get::<User>(&id) {
            println!("   🎯 从身份映射中获取用户: {}", user);
            return Some(user);
        }

        // 身份映射中没有，从数据库加载
        if let Some(user_data) = self.users_db.get(&id) {
            println!("   💾 从数据库加载用户: {}", user_data);
            let user = Arc::new(user_data.clone());
            self.identity_map.put(user.clone());
            Some(user)
        } else {
            None
        }
    }

    /// 查找订单（带身份映射）
    pub fn find_order(&mut self, id: u32) -> Option<Arc<Order>> {
        // 首先检查身份映射
        if let Some(order) = self.identity_map.get::<Order>(&id) {
            println!("   🎯 从身份映射中获取订单: {}", order);
            return Some(order);
        }

        // 身份映射中没有，从数据库加载
        if let Some(order_data) = self.orders_db.get(&id) {
            println!("   💾 从数据库加载订单: {}", order_data);
            let order = Arc::new(order_data.clone());
            self.identity_map.put(order.clone());
            Some(order)
        } else {
            None
        }
    }

    /// 创建用户
    pub fn create_user(&mut self, username: String, email: String, full_name: String) -> Arc<User> {
        let id = self.next_user_id;
        self.next_user_id += 1;
        
        let user = User::new(id, username, email, full_name);
        self.users_db.insert(id, user.clone());
        
        let user_arc = Arc::new(user);
        self.identity_map.put(user_arc.clone());
        user_arc
    }

    /// 创建订单
    pub fn create_order(&mut self, user_id: u32, total_amount: f64, status: String) -> Arc<Order> {
        let id = self.next_order_id;
        self.next_order_id += 1;
        
        let order = Order::new(id, user_id, total_amount, status);
        self.orders_db.insert(id, order.clone());
        
        let order_arc = Arc::new(order);
        self.identity_map.put(order_arc.clone());
        order_arc
    }

    /// 获取所有用户（演示批量操作）
    pub fn find_all_users(&mut self) -> Vec<Arc<User>> {
        let mut users = Vec::new();
        
        // 先收集所有用户ID，避免借用冲突
        let user_ids: Vec<u32> = self.users_db.keys().copied().collect();
        
        for user_id in user_ids {
            if let Some(user) = self.find_user(user_id) {
                users.push(user);
            }
        }
        
        users
    }

    /// 获取用户的所有订单
    pub fn find_user_orders(&mut self, user_id: u32) -> Vec<Arc<Order>> {
        let mut orders = Vec::new();
        
        // 先收集符合条件的订单ID，避免借用冲突
        let order_ids: Vec<u32> = self.orders_db.values()
            .filter(|order| order.user_id == user_id)
            .map(|order| order.id)
            .collect();
        
        for order_id in order_ids {
            if let Some(order_arc) = self.find_order(order_id) {
                orders.push(order_arc);
            }
        }
        
        orders
    }

    /// 清理身份映射
    pub fn cleanup_identity_map(&mut self) {
        self.identity_map.cleanup();
    }

    /// 获取身份映射统计
    pub fn get_identity_map_statistics(&self) -> IdentityMapStatistics {
        self.identity_map.get_statistics()
    }

    /// 清空身份映射
    pub fn clear_identity_map(&mut self) {
        self.identity_map.clear();
    }
}

/// 演示身份映射模式
pub fn demo() {
    println!("=== 身份映射模式演示 ===\n");

    let mut dal = DataAccessLayer::new();

    println!("1. 创建测试数据");
    let user1 = dal.create_user("alice".to_string(), "alice@example.com".to_string(), "Alice Johnson".to_string());
    let user2 = dal.create_user("bob".to_string(), "bob@example.com".to_string(), "Bob Smith".to_string());
    
    let order1 = dal.create_order(user1.id, 299.99, "PENDING".to_string());
    let order2 = dal.create_order(user1.id, 599.50, "COMPLETED".to_string());
    let order3 = dal.create_order(user2.id, 899.99, "PROCESSING".to_string());

    println!("   创建了 {} 个用户和 {} 个订单", 2, 3);

    println!("\n2. 身份映射机制演示");
    
    // 第一次访问 - 从数据库加载
    println!("   第一次查找用户1:");
    let found_user1 = dal.find_user(user1.id).unwrap();
    
    // 第二次访问 - 从身份映射获取
    println!("   第二次查找用户1:");
    let found_user1_again = dal.find_user(user1.id).unwrap();
    
    // 验证是同一个对象实例
    println!("   对象实例相同: {}", Arc::ptr_eq(&found_user1, &found_user1_again));

    println!("\n3. 批量操作中的身份映射效果");
    println!("   获取所有用户:");
    let all_users = dal.find_all_users();
    for user in &all_users {
        println!("     - {}", user);
    }

    println!("\n   获取用户1的所有订单:");
    let user1_orders = dal.find_user_orders(user1.id);
    for order in &user1_orders {
        println!("     - {}", order);
    }

    println!("\n4. 身份映射统计信息");
    let stats = dal.get_identity_map_statistics();
    println!("   {}", stats);

    println!("\n5. 测试对象唯一性");
    // 多次获取同一对象
    for i in 1..=5 {
        let user = dal.find_user(user1.id).unwrap();
        println!("   第{}次获取用户1: {:p}", i, user.as_ref());
    }

    // 显示最终统计
    let final_stats = dal.get_identity_map_statistics();
    println!("\n   最终统计: {}", final_stats);

    println!("\n6. 内存清理演示");
    println!("   清理前对象数: {}", dal.get_identity_map_statistics().total_objects);
    
    // 清理失效引用
    dal.cleanup_identity_map();
    println!("   清理后对象数: {}", dal.get_identity_map_statistics().total_objects);
    
    // 清空身份映射
    dal.clear_identity_map();
    println!("   清空后对象数: {}", dal.get_identity_map_statistics().total_objects);

    println!("\n7. 清空后重新访问");
    let user_after_clear = dal.find_user(user1.id).unwrap();
    println!("   重新加载用户1: {}", user_after_clear);

    println!("\n=== 身份映射模式演示完成 ===");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity_map_basic_operations() {
        let mut identity_map = IdentityMap::new();
        let user = Arc::new(User::new(1, "test".to_string(), "test@example.com".to_string(), "Test User".to_string()));
        
        // 添加对象
        identity_map.put(user.clone());
        
        // 获取对象
        let retrieved = identity_map.get::<User>(&1).unwrap();
        assert!(Arc::ptr_eq(&user, &retrieved));
        
        // 检查存在性
        assert!(identity_map.contains::<User>(&1));
        assert!(!identity_map.contains::<User>(&2));
    }

    #[test]
    fn test_identity_map_statistics() {
        let mut identity_map = IdentityMap::new();
        let user = Arc::new(User::new(1, "test".to_string(), "test@example.com".to_string(), "Test User".to_string()));
        
        identity_map.put(user.clone());
        
        // 测试命中
        let _ = identity_map.get::<User>(&1).unwrap();
        // 测试未命中
        let _ = identity_map.get::<User>(&2);
        
        let stats = identity_map.get_statistics();
        assert_eq!(stats.hit_count, 1);
        assert_eq!(stats.miss_count, 1);
        assert_eq!(stats.hit_rate, 0.5);
    }

    #[test]
    fn test_data_access_layer() {
        let mut dal = DataAccessLayer::new();
        
        let user = dal.create_user("test".to_string(), "test@example.com".to_string(), "Test User".to_string());
        let order = dal.create_order(user.id, 100.0, "PENDING".to_string());
        
        // 第一次获取（从数据库）
        let found_user1 = dal.find_user(user.id).unwrap();
        // 第二次获取（从身份映射）
        let found_user2 = dal.find_user(user.id).unwrap();
        
        assert!(Arc::ptr_eq(&found_user1, &found_user2));
        
        let stats = dal.get_identity_map_statistics();
        assert!(stats.hit_count > 0);
    }
} 