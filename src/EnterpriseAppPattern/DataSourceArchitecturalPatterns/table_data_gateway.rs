//! 表数据入口模式 (Table Data Gateway)
//! 
//! 一个充当表或视图访问网关的对象。一个实例处理表中的所有行。
//! 文件路径：/d%3A/workspace/RustLearn/RustDesignPattern/src/EnterpriseAppPattern/DataSourceArchitecturalPatterns/table_data_gateway.rs

use std::collections::HashMap;
use std::fmt;

// 用户实体
#[derive(Debug, Clone)]
pub struct User {
    pub id: Option<u32>,
    pub username: String,
    pub email: String,
    pub created_at: String,
}

impl User {
    pub fn new(username: String, email: String) -> Self {
        Self {
            id: None,
            username,
            email,
            created_at: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        }
    }
}

impl fmt::Display for User {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "User[id={:?}, username={}, email={}]", 
               self.id, self.username, self.email)
    }
}

// 模拟数据库错误
#[derive(Debug)]
pub enum DatabaseError {
    ConnectionError(String),
    QueryError(String),
    ValidationError(String),
    NotFound,
}

impl fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DatabaseError::ConnectionError(msg) => write!(f, "连接错误: {}", msg),
            DatabaseError::QueryError(msg) => write!(f, "查询错误: {}", msg),
            DatabaseError::ValidationError(msg) => write!(f, "验证错误: {}", msg),
            DatabaseError::NotFound => write!(f, "未找到记录"),
        }
    }
}

// 表数据入口 - 处理用户表的所有操作
pub struct UserTableDataGateway {
    // 模拟数据库连接和数据存储
    users: HashMap<u32, User>,
    next_id: u32,
}

impl UserTableDataGateway {
    pub fn new() -> Self {
        println!("初始化用户表数据入口");
        Self {
            users: HashMap::new(),
            next_id: 1,
        }
    }

    // 插入新用户
    pub fn insert(&mut self, mut user: User) -> Result<u32, DatabaseError> {
        // 验证用户数据
        if user.username.is_empty() {
            return Err(DatabaseError::ValidationError("用户名不能为空".to_string()));
        }
        if user.email.is_empty() {
            return Err(DatabaseError::ValidationError("邮箱不能为空".to_string()));
        }

        // 检查用户名是否已存在
        if self.users.values().any(|u| u.username == user.username) {
            return Err(DatabaseError::ValidationError("用户名已存在".to_string()));
        }

        let id = self.next_id;
        user.id = Some(id);
        self.users.insert(id, user.clone());
        self.next_id += 1;

        println!("插入用户: {}", user);
        Ok(id)
    }

    // 根据ID查找用户
    pub fn find_by_id(&self, id: u32) -> Result<User, DatabaseError> {
        match self.users.get(&id) {
            Some(user) => {
                println!("查找用户ID {}: {}", id, user);
                Ok(user.clone())
            },
            None => {
                println!("未找到用户ID: {}", id);
                Err(DatabaseError::NotFound)
            }
        }
    }

    // 根据用户名查找用户
    pub fn find_by_username(&self, username: &str) -> Result<User, DatabaseError> {
        for user in self.users.values() {
            if user.username == username {
                println!("查找用户名 '{}': {}", username, user);
                return Ok(user.clone());
            }
        }
        println!("未找到用户名: {}", username);
        Err(DatabaseError::NotFound)
    }

    // 查找所有用户
    pub fn find_all(&self) -> Vec<User> {
        let users: Vec<User> = self.users.values().cloned().collect();
        println!("查找所有用户，共 {} 条记录", users.len());
        users
    }

    // 根据邮箱域名查找用户
    pub fn find_by_email_domain(&self, domain: &str) -> Vec<User> {
        let users: Vec<User> = self.users.values()
            .filter(|user| user.email.ends_with(&format!("@{}", domain)))
            .cloned()
            .collect();
        println!("查找邮箱域名 '{}' 的用户，共 {} 条记录", domain, users.len());
        users
    }

    // 更新用户
    pub fn update(&mut self, user: User) -> Result<(), DatabaseError> {
        let id = user.id.ok_or(DatabaseError::ValidationError("用户ID不能为空".to_string()))?;
        
        if !self.users.contains_key(&id) {
            return Err(DatabaseError::NotFound);
        }

        // 验证数据
        if user.username.is_empty() {
            return Err(DatabaseError::ValidationError("用户名不能为空".to_string()));
        }

        // 检查用户名是否被其他用户使用
        if self.users.values().any(|u| u.id != user.id && u.username == user.username) {
            return Err(DatabaseError::ValidationError("用户名已被使用".to_string()));
        }

        self.users.insert(id, user.clone());
        println!("更新用户: {}", user);
        Ok(())
    }

    // 删除用户
    pub fn delete(&mut self, id: u32) -> Result<(), DatabaseError> {
        match self.users.remove(&id) {
            Some(user) => {
                println!("删除用户: {}", user);
                Ok(())
            },
            None => {
                println!("删除失败，未找到用户ID: {}", id);
                Err(DatabaseError::NotFound)
            }
        }
    }

    // 获取用户总数
    pub fn count(&self) -> usize {
        let count = self.users.len();
        println!("用户总数: {}", count);
        count
    }

    // 批量插入用户
    pub fn batch_insert(&mut self, users: Vec<User>) -> Result<Vec<u32>, DatabaseError> {
        let mut ids = Vec::new();
        
        for user in users {
            match self.insert(user) {
                Ok(id) => ids.push(id),
                Err(e) => {
                    println!("批量插入时出错: {}", e);
                    return Err(e);
                }
            }
        }
        
        println!("批量插入完成，共插入 {} 条记录", ids.len());
        Ok(ids)
    }

    // 根据条件删除用户
    pub fn delete_by_email_domain(&mut self, domain: &str) -> usize {
        let mut deleted_count = 0;
        let ids_to_delete: Vec<u32> = self.users.iter()
            .filter(|(_, user)| user.email.ends_with(&format!("@{}", domain)))
            .map(|(id, _)| *id)
            .collect();

        for id in ids_to_delete {
            if self.users.remove(&id).is_some() {
                deleted_count += 1;
            }
        }

        println!("删除邮箱域名 '{}' 的用户，共删除 {} 条记录", domain, deleted_count);
        deleted_count
    }
}

// 订单表数据入口示例
#[derive(Debug, Clone)]
pub struct Order {
    pub id: Option<u32>,
    pub user_id: u32,
    pub product_name: String,
    pub amount: f64,
    pub status: OrderStatus,
    pub created_at: String,
}

#[derive(Debug, Clone)]
pub enum OrderStatus {
    Pending,    // 待处理
    Paid,      // 已支付
    Shipped,   // 已发货
    Delivered, // 已交付
    Cancelled, // 已取消
}

impl fmt::Display for OrderStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let status = match self {
            OrderStatus::Pending => "待处理",
            OrderStatus::Paid => "已支付",
            OrderStatus::Shipped => "已发货",
            OrderStatus::Delivered => "已交付",
            OrderStatus::Cancelled => "已取消",
        };
        write!(f, "{}", status)
    }
}

impl fmt::Display for Order {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Order[id={:?}, user_id={}, product={}, amount={:.2}, status={}]", 
               self.id, self.user_id, self.product_name, self.amount, self.status)
    }
}

pub struct OrderTableDataGateway {
    orders: HashMap<u32, Order>,
    next_id: u32,
}

impl OrderTableDataGateway {
    pub fn new() -> Self {
        println!("初始化订单表数据入口");
        Self {
            orders: HashMap::new(),
            next_id: 1,
        }
    }

    pub fn insert(&mut self, mut order: Order) -> Result<u32, DatabaseError> {
        if order.product_name.is_empty() {
            return Err(DatabaseError::ValidationError("商品名称不能为空".to_string()));
        }
        if order.amount <= 0.0 {
            return Err(DatabaseError::ValidationError("订单金额必须大于0".to_string()));
        }

        let id = self.next_id;
        order.id = Some(id);
        order.created_at = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        
        self.orders.insert(id, order.clone());
        self.next_id += 1;

        println!("创建订单: {}", order);
        Ok(id)
    }

    pub fn find_by_user_id(&self, user_id: u32) -> Vec<Order> {
        let orders: Vec<Order> = self.orders.values()
            .filter(|order| order.user_id == user_id)
            .cloned()
            .collect();
        println!("查找用户ID {} 的订单，共 {} 条", user_id, orders.len());
        orders
    }

    pub fn find_by_status(&self, status: &OrderStatus) -> Vec<Order> {
        let orders: Vec<Order> = self.orders.values()
            .filter(|order| std::mem::discriminant(&order.status) == std::mem::discriminant(status))
            .cloned()
            .collect();
        println!("查找状态为 '{}' 的订单，共 {} 条", status, orders.len());
        orders
    }

    pub fn update_status(&mut self, order_id: u32, new_status: OrderStatus) -> Result<(), DatabaseError> {
        match self.orders.get_mut(&order_id) {
            Some(order) => {
                let old_status = order.status.clone();
                order.status = new_status.clone();
                println!("订单 {} 状态从 '{}' 更新为 '{}'", order_id, old_status, new_status);
                Ok(())
            },
            None => {
                println!("未找到订单ID: {}", order_id);
                Err(DatabaseError::NotFound)
            }
        }
    }

    pub fn get_total_amount_by_user(&self, user_id: u32) -> f64 {
        let total: f64 = self.orders.values()
            .filter(|order| order.user_id == user_id)
            .map(|order| order.amount)
            .sum();
        println!("用户ID {} 的订单总金额: {:.2}", user_id, total);
        total
    }
}

pub fn demo() {
    println!("=== 表数据入口模式演示 ===");

    // 1. 用户表操作演示
    println!("\n1. 用户表数据入口演示:");
    let mut user_gateway = UserTableDataGateway::new();

    // 插入用户
    let user1 = User::new("张三".to_string(), "zhangsan@example.com".to_string());
    let user2 = User::new("李四".to_string(), "lisi@company.com".to_string());
    let user3 = User::new("王五".to_string(), "wangwu@example.com".to_string());

    match user_gateway.insert(user1) {
        Ok(id) => println!("✓ 用户插入成功，ID: {}", id),
        Err(e) => println!("✗ 用户插入失败: {}", e),
    }

    match user_gateway.insert(user2) {
        Ok(id) => println!("✓ 用户插入成功，ID: {}", id),
        Err(e) => println!("✗ 用户插入失败: {}", e),
    }

    match user_gateway.insert(user3) {
        Ok(id) => println!("✓ 用户插入成功，ID: {}", id),
        Err(e) => println!("✗ 用户插入失败: {}", e),
    }

    // 查询操作
    println!("\n查询操作:");
    if let Ok(user) = user_gateway.find_by_id(1) {
        println!("找到用户: {}", user);
    }

    if let Ok(user) = user_gateway.find_by_username("李四") {
        println!("找到用户: {}", user);
    }

    let all_users = user_gateway.find_all();
    println!("所有用户:");
    for user in &all_users {
        println!("  - {}", user);
    }

    let example_users = user_gateway.find_by_email_domain("example.com");
    println!("example.com 域名用户:");
    for user in &example_users {
        println!("  - {}", user);
    }

    // 更新操作
    println!("\n更新操作:");
    if let Ok(mut user) = user_gateway.find_by_id(1) {
        user.email = "zhangsan_new@example.com".to_string();
        match user_gateway.update(user) {
            Ok(_) => println!("✓ 用户更新成功"),
            Err(e) => println!("✗ 用户更新失败: {}", e),
        }
    }

    // 2. 订单表操作演示
    println!("\n2. 订单表数据入口演示:");
    let mut order_gateway = OrderTableDataGateway::new();

    // 创建订单
    let order1 = Order {
        id: None,
        user_id: 1,
        product_name: "笔记本电脑".to_string(),
        amount: 5999.0,
        status: OrderStatus::Pending,
        created_at: String::new(),
    };

    let order2 = Order {
        id: None,
        user_id: 1,
        product_name: "鼠标".to_string(),
        amount: 199.0,
        status: OrderStatus::Paid,
        created_at: String::new(),
    };

    let order3 = Order {
        id: None,
        user_id: 2,
        product_name: "键盘".to_string(),
        amount: 299.0,
        status: OrderStatus::Shipped,
        created_at: String::new(),
    };

    order_gateway.insert(order1).ok();
    order_gateway.insert(order2).ok();
    order_gateway.insert(order3).ok();

    // 查询订单
    println!("\n订单查询:");
    let user1_orders = order_gateway.find_by_user_id(1);
    println!("用户1的订单:");
    for order in &user1_orders {
        println!("  - {}", order);
    }

    let pending_orders = order_gateway.find_by_status(&OrderStatus::Pending);
    println!("待处理订单:");
    for order in &pending_orders {
        println!("  - {}", order);
    }

    // 更新订单状态
    println!("\n订单状态更新:");
    order_gateway.update_status(1, OrderStatus::Paid).ok();
    order_gateway.update_status(2, OrderStatus::Shipped).ok();

    // 统计信息
    println!("\n统计信息:");
    user_gateway.count();
    order_gateway.get_total_amount_by_user(1);

    println!("\n表数据入口模式的优点:");
    println!("1. 将数据库访问逻辑集中在一个地方");
    println!("2. 为表提供简单的CRUD接口");
    println!("3. 隐藏SQL复杂性，提供类型安全的方法");
    println!("4. 便于测试和模拟数据访问层");
    println!("5. 支持批量操作和复杂查询");

    println!("\n适用场景:");
    println!("1. 简单的数据访问需求");
    println!("2. 面向表的操作较多");
    println!("3. 需要集中管理数据访问逻辑");
    println!("4. 数据库表结构相对稳定");
} 