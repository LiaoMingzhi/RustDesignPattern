// 延迟加载模式实现
//! # 延迟加载模式 (Lazy Load)
//!
//! ## 概述
//! 延迟加载模式推迟对象的初始化直到真正需要使用时。
//! 这可以显著提高性能，特别是在处理大量数据或复杂对象图时。
//!
//! ## 优点
//! - 提高应用启动性能
//! - 减少内存使用
//! - 避免不必要的数据库查询
//! - 支持大对象图的处理
//!
//! ## 适用场景
//! - 大型对象或集合
//! - 复杂的对象关系
//! - 网络或数据库访问成本高的情况
//! - 内存敏感的应用

use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

/// 延迟加载错误
#[derive(Debug)]
pub enum LazyLoadError {
    LoadError(String),
    AlreadyLoaded,
    NotInitialized,
}

impl fmt::Display for LazyLoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LazyLoadError::LoadError(msg) => write!(f, "加载错误: {}", msg),
            LazyLoadError::AlreadyLoaded => write!(f, "已经加载"),
            LazyLoadError::NotInitialized => write!(f, "未初始化"),
        }
    }
}

impl std::error::Error for LazyLoadError {}

/// 延迟加载器接口
pub trait LazyLoader<T> {
    fn load(&self) -> Result<T, LazyLoadError>;
}

/// 虚拟代理 - 延迟加载的一种实现方式
pub struct VirtualProxy<T> {
    loader: Box<dyn LazyLoader<T>>,
    cached_value: RefCell<Option<T>>,
}

impl<T: Clone> VirtualProxy<T> {
    pub fn new(loader: Box<dyn LazyLoader<T>>) -> Self {
        Self {
            loader,
            cached_value: RefCell::new(None),
        }
    }

    /// 获取值（如果未加载则延迟加载）
    pub fn get(&self) -> Result<T, LazyLoadError> {
        if let Some(ref value) = *self.cached_value.borrow() {
            return Ok(value.clone());
        }

        let value = self.loader.load()?;
        *self.cached_value.borrow_mut() = Some(value.clone());
        Ok(value)
    }

    /// 检查是否已加载
    pub fn is_loaded(&self) -> bool {
        self.cached_value.borrow().is_some()
    }

    /// 重置缓存（强制重新加载）
    pub fn reset(&self) {
        *self.cached_value.borrow_mut() = None;
    }
}

/// 用户实体
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

impl fmt::Display for User {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "User[{}]: {} - {}", self.id, self.username, self.email)
    }
}

/// 订单实体
#[derive(Debug, Clone)]
pub struct Order {
    pub id: u32,
    pub user_id: u32,
    pub product_name: String,
    pub amount: f64,
    pub status: String,
}

impl Order {
    pub fn new(id: u32, user_id: u32, product_name: String, amount: f64, status: String) -> Self {
        Self { id, user_id, product_name, amount, status }
    }
}

impl fmt::Display for Order {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Order[{}]: {} - {:.2} ({})", self.id, self.product_name, self.amount, self.status)
    }
}

/// 订单详情（大对象，适合延迟加载）
#[derive(Debug, Clone)]
pub struct OrderDetails {
    pub order_id: u32,
    pub shipping_address: String,
    pub billing_address: String,
    pub payment_method: String,
    pub notes: String,
    pub tracking_number: Option<String>,
}

impl fmt::Display for OrderDetails {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "OrderDetails[{}]: 收货地址: {}, 支付方式: {}", 
               self.order_id, self.shipping_address, self.payment_method)
    }
}

/// 模拟数据库
pub struct MockDatabase {
    users: HashMap<u32, User>,
    orders: HashMap<u32, Order>,
    order_details: HashMap<u32, OrderDetails>,
    query_count: RefCell<u32>,
}

impl MockDatabase {
    pub fn new() -> Self {
        let mut db = Self {
            users: HashMap::new(),
            orders: HashMap::new(),
            order_details: HashMap::new(),
            query_count: RefCell::new(0),
        };
        
        // 初始化测试数据
        db.init_test_data();
        db
    }

    fn init_test_data(&mut self) {
        // 用户数据
        self.users.insert(1, User::new(1, "alice".to_string(), "alice@example.com".to_string(), "Alice Johnson".to_string()));
        self.users.insert(2, User::new(2, "bob".to_string(), "bob@example.com".to_string(), "Bob Smith".to_string()));
        
        // 订单数据
        self.orders.insert(1, Order::new(1, 1, "笔记本电脑".to_string(), 8999.99, "已发货".to_string()));
        self.orders.insert(2, Order::new(2, 1, "鼠标".to_string(), 199.99, "已完成".to_string()));
        self.orders.insert(3, Order::new(3, 2, "键盘".to_string(), 599.99, "处理中".to_string()));
        
        // 订单详情数据
        self.order_details.insert(1, OrderDetails {
            order_id: 1,
            shipping_address: "北京市朝阳区某某路123号".to_string(),
            billing_address: "北京市朝阳区某某路123号".to_string(),
            payment_method: "信用卡".to_string(),
            notes: "请在工作日送达".to_string(),
            tracking_number: Some("SF1234567890".to_string()),
        });
        
        self.order_details.insert(2, OrderDetails {
            order_id: 2,
            shipping_address: "上海市浦东新区某某路456号".to_string(),
            billing_address: "上海市浦东新区某某路456号".to_string(),
            payment_method: "支付宝".to_string(),
            notes: "".to_string(),
            tracking_number: Some("YTO9876543210".to_string()),
        });
        
        self.order_details.insert(3, OrderDetails {
            order_id: 3,
            shipping_address: "广州市天河区某某路789号".to_string(),
            billing_address: "广州市天河区某某路789号".to_string(),
            payment_method: "微信支付".to_string(),
            notes: "需要发票".to_string(),
            tracking_number: None,
        });
    }

    pub fn find_user(&self, id: u32) -> Option<User> {
        self.increment_query_count();
        println!("    📀 执行数据库查询: 查找用户 {}", id);
        self.users.get(&id).cloned()
    }

    pub fn find_orders_by_user(&self, user_id: u32) -> Vec<Order> {
        self.increment_query_count();
        println!("    📀 执行数据库查询: 查找用户 {} 的订单", user_id);
        self.orders.values()
            .filter(|order| order.user_id == user_id)
            .cloned()
            .collect()
    }

    pub fn find_order_details(&self, order_id: u32) -> Option<OrderDetails> {
        self.increment_query_count();
        println!("    📀 执行数据库查询: 查找订单 {} 的详情", order_id);
        // 模拟复杂查询的延迟
        std::thread::sleep(std::time::Duration::from_millis(100));
        self.order_details.get(&order_id).cloned()
    }

    fn increment_query_count(&self) {
        *self.query_count.borrow_mut() += 1;
    }

    pub fn get_query_count(&self) -> u32 {
        *self.query_count.borrow()
    }

    pub fn reset_query_count(&self) {
        *self.query_count.borrow_mut() = 0;
    }
}

/// 用户订单加载器
pub struct UserOrdersLoader {
    user_id: u32,
    database: Rc<MockDatabase>,
}

impl UserOrdersLoader {
    pub fn new(user_id: u32, database: Rc<MockDatabase>) -> Self {
        Self { user_id, database }
    }
}

impl LazyLoader<Vec<Order>> for UserOrdersLoader {
    fn load(&self) -> Result<Vec<Order>, LazyLoadError> {
        let orders = self.database.find_orders_by_user(self.user_id);
        Ok(orders)
    }
}

/// 订单详情加载器
pub struct OrderDetailsLoader {
    order_id: u32,
    database: Rc<MockDatabase>,
}

impl OrderDetailsLoader {
    pub fn new(order_id: u32, database: Rc<MockDatabase>) -> Self {
        Self { order_id, database }
    }
}

impl LazyLoader<OrderDetails> for OrderDetailsLoader {
    fn load(&self) -> Result<OrderDetails, LazyLoadError> {
        self.database.find_order_details(self.order_id)
            .ok_or_else(|| LazyLoadError::LoadError(format!("订单详情未找到: {}", self.order_id)))
    }
}

/// 带延迟加载的用户实体
pub struct UserWithLazyOrders {
    pub user: User,
    lazy_orders: VirtualProxy<Vec<Order>>,
}

impl UserWithLazyOrders {
    pub fn new(user: User, database: Rc<MockDatabase>) -> Self {
        let loader = Box::new(UserOrdersLoader::new(user.id, database));
        let lazy_orders = VirtualProxy::new(loader);
        
        Self { user, lazy_orders }
    }

    /// 获取订单（延迟加载）
    pub fn get_orders(&self) -> Result<Vec<Order>, LazyLoadError> {
        self.lazy_orders.get()
    }

    /// 检查订单是否已加载
    pub fn orders_loaded(&self) -> bool {
        self.lazy_orders.is_loaded()
    }
}

impl fmt::Display for UserWithLazyOrders {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} (订单已加载: {})", self.user, self.orders_loaded())
    }
}

/// 带延迟加载的订单实体
pub struct OrderWithLazyDetails {
    pub order: Order,
    lazy_details: VirtualProxy<OrderDetails>,
}

impl OrderWithLazyDetails {
    pub fn new(order: Order, database: Rc<MockDatabase>) -> Self {
        let loader = Box::new(OrderDetailsLoader::new(order.id, database));
        let lazy_details = VirtualProxy::new(loader);
        
        Self { order, lazy_details }
    }

    /// 获取订单详情（延迟加载）
    pub fn get_details(&self) -> Result<OrderDetails, LazyLoadError> {
        self.lazy_details.get()
    }

    /// 检查详情是否已加载
    pub fn details_loaded(&self) -> bool {
        self.lazy_details.is_loaded()
    }
}

impl fmt::Display for OrderWithLazyDetails {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} (详情已加载: {})", self.order, self.details_loaded())
    }
}

/// 演示延迟加载模式
pub fn demo() {
    println!("=== 延迟加载模式演示 ===\n");

    let database = Rc::new(MockDatabase::new());
    
    println!("1. 创建带延迟加载的用户对象");
    database.reset_query_count();
    
    let user_data = database.find_user(1).unwrap();
    let user_with_lazy_orders = UserWithLazyOrders::new(user_data, database.clone());
    
    println!("   创建用户对象: {}", user_with_lazy_orders);
    println!("   数据库查询次数: {}", database.get_query_count());

    println!("\n2. 延迟加载用户订单");
    println!("   第一次获取订单（触发加载）:");
    let orders = user_with_lazy_orders.get_orders().unwrap();
    for order in &orders {
        println!("     - {}", order);
    }
    println!("   加载后状态: {}", user_with_lazy_orders);
    println!("   数据库查询次数: {}", database.get_query_count());

    println!("\n   第二次获取订单（使用缓存）:");
    let orders_again = user_with_lazy_orders.get_orders().unwrap();
    println!("   获取到 {} 个订单", orders_again.len());
    println!("   数据库查询次数: {}", database.get_query_count());

    println!("\n3. 创建带延迟加载的订单对象");
    let order_data = orders[0].clone();
    let order_with_lazy_details = OrderWithLazyDetails::new(order_data, database.clone());
    
    println!("   创建订单对象: {}", order_with_lazy_details);

    println!("\n4. 延迟加载订单详情");
    println!("   获取订单详情（触发加载）:");
    match order_with_lazy_details.get_details() {
        Ok(details) => {
            println!("     - {}", details);
            println!("     - 备注: {}", details.notes);
            if let Some(tracking) = &details.tracking_number {
                println!("     - 快递单号: {}", tracking);
            }
        }
        Err(e) => println!("     加载失败: {}", e),
    }
    
    println!("   加载后状态: {}", order_with_lazy_details);
    println!("   数据库查询次数: {}", database.get_query_count());

    println!("\n5. 性能对比演示");
    database.reset_query_count();
    
    // 创建多个用户对象但不访问订单
    println!("   创建5个用户对象（不加载订单）:");
    let mut users = Vec::new();
    for user_id in 1..=2 {
        if let Some(user_data) = database.find_user(user_id) {
            let user_with_lazy = UserWithLazyOrders::new(user_data, database.clone());
            users.push(user_with_lazy);
        }
    }
    
    println!("   创建完成，数据库查询次数: {}", database.get_query_count());

    // 只访问其中一个用户的订单
    println!("\n   只访问第一个用户的订单:");
    let first_user_orders = users[0].get_orders().unwrap();
    println!("   获取到 {} 个订单", first_user_orders.len());
    println!("   数据库查询次数: {}", database.get_query_count());

    println!("\n6. 虚拟代理功能测试");
    
    // 测试重置功能
    println!("   重置第一个用户的订单缓存:");
    users[0].lazy_orders.reset();
    println!("   重置后订单加载状态: {}", users[0].orders_loaded());
    
    // 重新加载
    println!("   重新加载订单:");
    let reloaded_orders = users[0].get_orders().unwrap();
    println!("   重新加载到 {} 个订单", reloaded_orders.len());
    println!("   最终数据库查询次数: {}", database.get_query_count());

    println!("\n=== 延迟加载模式演示完成 ===");
    
    println!("\n💡 延迟加载的优势:");
    println!("1. 性能提升 - 只在需要时才加载数据");
    println!("2. 内存节省 - 避免加载不使用的数据");
    println!("3. 网络优化 - 减少不必要的数据库查询");
    println!("4. 响应速度 - 应用启动更快");
    
    println!("\n⚠️  注意事项:");
    println!("1. N+1 查询问题 - 批量操作时要注意");
    println!("2. 缓存一致性 - 数据更新后要处理缓存");
    println!("3. 错误处理 - 延迟加载可能失败");
    println!("4. 线程安全 - 多线程环境下要注意同步");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_virtual_proxy() {
        struct TestLoader(String);
        
        impl LazyLoader<String> for TestLoader {
            fn load(&self) -> Result<String, LazyLoadError> {
                Ok(self.0.clone())
            }
        }
        
        let proxy = VirtualProxy::new(Box::new(TestLoader("test_value".to_string())));
        
        // 初始状态未加载
        assert!(!proxy.is_loaded());
        
        // 第一次获取，触发加载
        let value1 = proxy.get().unwrap();
        assert_eq!(value1, "test_value");
        assert!(proxy.is_loaded());
        
        // 第二次获取，使用缓存
        let value2 = proxy.get().unwrap();
        assert_eq!(value2, "test_value");
        
        // 重置缓存
        proxy.reset();
        assert!(!proxy.is_loaded());
    }

    #[test]
    fn test_user_with_lazy_orders() {
        let database = Rc::new(MockDatabase::new());
        let user = User::new(1, "test".to_string(), "test@example.com".to_string(), "Test User".to_string());
        let user_with_lazy = UserWithLazyOrders::new(user, database.clone());
        
        // 初始状态
        assert!(!user_with_lazy.orders_loaded());
        
        // 加载订单
        let orders = user_with_lazy.get_orders().unwrap();
        assert!(!orders.is_empty());
        assert!(user_with_lazy.orders_loaded());
    }

    #[test]
    fn test_order_with_lazy_details() {
        let database = Rc::new(MockDatabase::new());
        let order = Order::new(1, 1, "test".to_string(), 100.0, "test".to_string());
        let order_with_lazy = OrderWithLazyDetails::new(order, database.clone());
        
        // 初始状态
        assert!(!order_with_lazy.details_loaded());
        
        // 加载详情
        let details = order_with_lazy.get_details().unwrap();
        assert_eq!(details.order_id, 1);
        assert!(order_with_lazy.details_loaded());
    }
} 