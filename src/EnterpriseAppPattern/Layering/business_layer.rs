/*
 * 文件路径: /d%3A/workspace/RustLearn/RustDesignPattern/src/EnterpriseAppPattern/Layering/business_layer.rs
 * 
 * Business Layer（业务层）模式
 * 
 * 定义：
 * 业务层包含系统的核心业务逻辑，它处理业务规则、业务流程和业务计算。
 * 业务层是整个应用的核心，独立于表现层和数据访问层。
 * 
 * 主要职责：
 * 1. 实现业务规则和业务逻辑
 * 2. 协调不同的领域对象
 * 3. 管理事务边界
 * 4. 业务流程控制
 * 5. 业务数据验证
 * 
 * 设计原则：
 * 1. 独立于技术实现
 * 2. 体现业务专家的知识
 * 3. 可测试性
 * 4. 业务逻辑集中化
 * 
 * 适用场景：
 * - 复杂的业务规则
 * - 多步骤的业务流程
 * - 跨多个实体的业务操作
 * - 需要事务控制的操作
 */

use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Display, Formatter};

/// 业务层错误类型
#[derive(Debug)]
pub enum BusinessError {
    ValidationError(String),
    BusinessRuleViolation(String),
    ResourceNotFound(String),
    InsufficientPermission(String),
    DataIntegrityError(String),
    ExternalServiceError(String),
}

impl Display for BusinessError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            BusinessError::ValidationError(msg) => write!(f, "验证错误: {}", msg),
            BusinessError::BusinessRuleViolation(msg) => write!(f, "业务规则违反: {}", msg),
            BusinessError::ResourceNotFound(msg) => write!(f, "资源未找到: {}", msg),
            BusinessError::InsufficientPermission(msg) => write!(f, "权限不足: {}", msg),
            BusinessError::DataIntegrityError(msg) => write!(f, "数据完整性错误: {}", msg),
            BusinessError::ExternalServiceError(msg) => write!(f, "外部服务错误: {}", msg),
        }
    }
}

impl Error for BusinessError {}

/// 业务操作结果
#[derive(Debug)]
pub struct BusinessResult<T> {
    pub data: Option<T>,
    pub success: bool,
    pub message: String,
    pub warnings: Vec<String>,
}

impl<T> BusinessResult<T> {
    pub fn success(data: T) -> Self {
        Self {
            data: Some(data),
            success: true,
            message: "操作成功".to_string(),
            warnings: Vec::new(),
        }
    }
    
    pub fn success_with_message(data: T, message: String) -> Self {
        Self {
            data: Some(data),
            success: true,
            message,
            warnings: Vec::new(),
        }
    }
    
    pub fn failure(message: String) -> Self {
        Self {
            data: None,
            success: false,
            message,
            warnings: Vec::new(),
        }
    }
    
    pub fn with_warning(mut self, warning: String) -> Self {
        self.warnings.push(warning);
        self
    }
}

/// 用户实体
#[derive(Debug, Clone)]
pub struct User {
    pub id: String,
    pub email: String,
    pub name: String,
    pub status: UserStatus,
    pub role: UserRole,
    pub balance: f64,
    pub created_at: u64,
    pub last_login: Option<u64>,
}

/// 用户状态
#[derive(Debug, Clone, PartialEq)]
pub enum UserStatus {
    Active,
    Inactive,
    Suspended,
    Pending,
}

/// 用户角色
#[derive(Debug, Clone, PartialEq)]
pub enum UserRole {
    Admin,
    Manager,
    User,
    Guest,
}

/// 订单实体
#[derive(Debug, Clone)]
pub struct Order {
    pub id: String,
    pub user_id: String,
    pub items: Vec<OrderItem>,
    pub total_amount: f64,
    pub status: OrderStatus,
    pub created_at: u64,
    pub updated_at: u64,
}

/// 订单项
#[derive(Debug, Clone)]
pub struct OrderItem {
    pub product_id: String,
    pub product_name: String,
    pub quantity: i32,
    pub unit_price: f64,
    pub total_price: f64,
}

/// 订单状态
#[derive(Debug, Clone, PartialEq)]
pub enum OrderStatus {
    Pending,
    Confirmed,
    Processing,
    Shipped,
    Delivered,
    Cancelled,
    Refunded,
}

/// 产品实体
#[derive(Debug, Clone)]
pub struct Product {
    pub id: String,
    pub name: String,
    pub price: f64,
    pub stock: i32,
    pub category: String,
    pub is_active: bool,
}

/// 业务规则引擎
pub struct BusinessRuleEngine;

impl BusinessRuleEngine {
    /// 验证用户下单资格
    pub fn validate_order_eligibility(user: &User, order_amount: f64) -> Result<(), BusinessError> {
        // 规则1: 用户必须是活跃状态
        if user.status != UserStatus::Active {
            return Err(BusinessError::BusinessRuleViolation(
                "只有活跃用户才能下单".to_string()
            ));
        }
        
        // 规则2: 订单金额必须大于0
        if order_amount <= 0.0 {
            return Err(BusinessError::ValidationError(
                "订单金额必须大于0".to_string()
            ));
        }
        
        // 规则3: 单笔订单限额检查
        let max_order_amount = match user.role {
            UserRole::Admin => 100000.0,
            UserRole::Manager => 50000.0,
            UserRole::User => 10000.0,
            UserRole::Guest => 500.0,
        };
        
        if order_amount > max_order_amount {
            return Err(BusinessError::BusinessRuleViolation(
                format!("订单金额超过限额: {}", max_order_amount)
            ));
        }
        
        Ok(())
    }
    
    /// 计算折扣
    pub fn calculate_discount(user: &User, order_amount: f64) -> f64 {
        let mut discount: f64 = 0.0;
        
        // VIP用户享受5%折扣
        if user.role == UserRole::Manager || user.role == UserRole::Admin {
            discount += 0.05;
        }
        
        // 大订单折扣
        if order_amount > 1000.0 {
            discount += 0.02;
        }
        
        if order_amount > 5000.0 {
            discount += 0.03;
        }
        
        // 最大折扣10%
        discount.min(0.10)
    }
    
    /// 验证库存充足性
    pub fn validate_stock_availability(items: &[OrderItem], products: &HashMap<String, Product>) -> Result<(), BusinessError> {
        for item in items {
            if let Some(product) = products.get(&item.product_id) {
                if !product.is_active {
                    return Err(BusinessError::BusinessRuleViolation(
                        format!("产品 {} 已下架", product.name)
                    ));
                }
                
                if product.stock < item.quantity {
                    return Err(BusinessError::BusinessRuleViolation(
                        format!("产品 {} 库存不足，当前库存: {}, 需要: {}", 
                               product.name, product.stock, item.quantity)
                    ));
                }
            } else {
                return Err(BusinessError::ResourceNotFound(
                    format!("产品不存在: {}", item.product_id)
                ));
            }
        }
        
        Ok(())
    }
    
    /// 检查权限
    pub fn check_permission(user: &User, operation: &str) -> Result<(), BusinessError> {
        let allowed = match operation {
            "view_orders" => true, // 所有用户都可以查看订单
            "cancel_order" => user.role != UserRole::Guest,
            "refund_order" => user.role == UserRole::Admin || user.role == UserRole::Manager,
            "manage_products" => user.role == UserRole::Admin,
            "view_reports" => user.role != UserRole::Guest,
            _ => false,
        };
        
        if !allowed {
            return Err(BusinessError::InsufficientPermission(
                format!("用户 {} 没有权限执行操作: {}", user.name, operation)
            ));
        }
        
        Ok(())
    }
}

/// 用户业务服务
pub struct UserBusinessService {
    // 在实际应用中，这里会注入数据访问层
    users: std::sync::Mutex<HashMap<String, User>>,
}

impl UserBusinessService {
    pub fn new() -> Self {
        Self {
            users: std::sync::Mutex::new(HashMap::new()),
        }
    }
    
    /// 注册新用户
    pub fn register_user(&self, email: String, name: String, password: String) -> Result<BusinessResult<User>, BusinessError> {
        // 业务验证
        if email.is_empty() || name.is_empty() || password.is_empty() {
            return Err(BusinessError::ValidationError("邮箱、姓名和密码都不能为空".to_string()));
        }
        
        if password.len() < 6 {
            return Err(BusinessError::ValidationError("密码长度不能少于6位".to_string()));
        }
        
        let mut users = self.users.lock().unwrap();
        
        // 检查邮箱是否已存在
        for user in users.values() {
            if user.email == email {
                return Err(BusinessError::BusinessRuleViolation("邮箱已被注册".to_string()));
            }
        }
        
        // 创建新用户
        let user_id = format!("user_{}", current_timestamp());
        let user = User {
            id: user_id.clone(),
            email,
            name,
            status: UserStatus::Active,
            role: UserRole::User,
            balance: 0.0,
            created_at: current_timestamp(),
            last_login: None,
        };
        
        users.insert(user_id, user.clone());
        
        Ok(BusinessResult::success_with_message(user, "用户注册成功".to_string()))
    }
    
    /// 用户登录
    pub fn login_user(&self, email: String, password: String) -> Result<BusinessResult<User>, BusinessError> {
        let mut users = self.users.lock().unwrap();
        
        // 查找用户
        let mut found_user = None;
        for user in users.values_mut() {
            if user.email == email {
                // 简化的密码验证（实际应该验证哈希）
                if password == "password123" { // 模拟密码验证
                    if user.status != UserStatus::Active {
                        return Err(BusinessError::BusinessRuleViolation("用户账户未激活".to_string()));
                    }
                    
                    user.last_login = Some(current_timestamp());
                    found_user = Some(user.clone());
                } else {
                    return Err(BusinessError::ValidationError("密码错误".to_string()));
                }
                break;
            }
        }
        
        match found_user {
            Some(user) => Ok(BusinessResult::success_with_message(user, "登录成功".to_string())),
            None => Err(BusinessError::ResourceNotFound("用户不存在".to_string())),
        }
    }
    
    /// 充值余额
    pub fn recharge_balance(&self, user_id: &str, amount: f64) -> Result<BusinessResult<f64>, BusinessError> {
        if amount <= 0.0 {
            return Err(BusinessError::ValidationError("充值金额必须大于0".to_string()));
        }
        
        if amount > 50000.0 {
            return Err(BusinessError::BusinessRuleViolation("单次充值金额不能超过50000".to_string()));
        }
        
        let mut users = self.users.lock().unwrap();
        
        if let Some(user) = users.get_mut(user_id) {
            if user.status != UserStatus::Active {
                return Err(BusinessError::BusinessRuleViolation("只有活跃用户才能充值".to_string()));
            }
            
            user.balance += amount;
            Ok(BusinessResult::success_with_message(
                user.balance,
                format!("充值成功，当前余额: {:.2}", user.balance)
            ))
        } else {
            Err(BusinessError::ResourceNotFound("用户不存在".to_string()))
        }
    }
    
    /// 获取用户信息
    pub fn get_user(&self, user_id: &str) -> Result<User, BusinessError> {
        let users = self.users.lock().unwrap();
        users.get(user_id)
            .cloned()
            .ok_or_else(|| BusinessError::ResourceNotFound("用户不存在".to_string()))
    }
    
    /// 更新用户状态
    pub fn update_user_status(&self, user_id: &str, new_status: UserStatus, operator: &User) -> Result<BusinessResult<()>, BusinessError> {
        // 权限检查
        BusinessRuleEngine::check_permission(operator, "manage_users")?;
        
        let mut users = self.users.lock().unwrap();
        
        if let Some(user) = users.get_mut(user_id) {
            let old_status = user.status.clone();
            user.status = new_status.clone();
            
            let message = format!("用户状态已从 {:?} 更新为 {:?}", old_status, new_status);
            Ok(BusinessResult::success_with_message((), message))
        } else {
            Err(BusinessError::ResourceNotFound("用户不存在".to_string()))
        }
    }
}

impl Default for UserBusinessService {
    fn default() -> Self {
        Self::new()
    }
}

/// 订单业务服务
pub struct OrderBusinessService {
    orders: std::sync::Mutex<HashMap<String, Order>>,
    products: std::sync::Mutex<HashMap<String, Product>>,
}

impl OrderBusinessService {
    pub fn new() -> Self {
        let mut products = HashMap::new();
        
        // 初始化一些示例产品
        products.insert("prod_001".to_string(), Product {
            id: "prod_001".to_string(),
            name: "智能手机".to_string(),
            price: 2999.0,
            stock: 50,
            category: "电子产品".to_string(),
            is_active: true,
        });
        
        products.insert("prod_002".to_string(), Product {
            id: "prod_002".to_string(),
            name: "笔记本电脑".to_string(),
            price: 5999.0,
            stock: 20,
            category: "电子产品".to_string(),
            is_active: true,
        });
        
        Self {
            orders: std::sync::Mutex::new(HashMap::new()),
            products: std::sync::Mutex::new(products),
        }
    }
    
    /// 创建订单
    pub fn create_order(&self, user: &User, items: Vec<OrderItem>) -> Result<BusinessResult<Order>, BusinessError> {
        if items.is_empty() {
            return Err(BusinessError::ValidationError("订单不能为空".to_string()));
        }
        
        // 计算订单总金额
        let total_amount: f64 = items.iter().map(|item| item.total_price).sum();
        
        // 验证用户下单资格
        BusinessRuleEngine::validate_order_eligibility(user, total_amount)?;
        
        // 验证库存
        let products = self.products.lock().unwrap();
        BusinessRuleEngine::validate_stock_availability(&items, &products)?;
        
        // 计算折扣
        let discount = BusinessRuleEngine::calculate_discount(user, total_amount);
        let final_amount = total_amount * (1.0 - discount);
        
        // 检查用户余额
        if user.balance < final_amount {
            return Err(BusinessError::BusinessRuleViolation(
                format!("余额不足，需要: {:.2}, 当前余额: {:.2}", final_amount, user.balance)
            ));
        }
        
        // 创建订单
        let order_id = format!("order_{}", current_timestamp());
        let mut order = Order {
            id: order_id.clone(),
            user_id: user.id.clone(),
            items: items.clone(),
            total_amount: final_amount,
            status: OrderStatus::Pending,
            created_at: current_timestamp(),
            updated_at: current_timestamp(),
        };
        
        // 扣减库存
        drop(products); // 释放读锁
        let mut products = self.products.lock().unwrap();
        for item in &items {
            if let Some(product) = products.get_mut(&item.product_id) {
                product.stock -= item.quantity;
            }
        }
        drop(products);
        
        // 保存订单
        let mut orders = self.orders.lock().unwrap();
        orders.insert(order_id, order.clone());
        
        let mut result = BusinessResult::success_with_message(order, "订单创建成功".to_string());
        
        if discount > 0.0 {
            result = result.with_warning(format!("享受折扣: {:.1}%", discount * 100.0));
        }
        
        Ok(result)
    }
    
    /// 确认订单
    pub fn confirm_order(&self, order_id: &str, operator: &User) -> Result<BusinessResult<Order>, BusinessError> {
        BusinessRuleEngine::check_permission(operator, "confirm_order")?;
        
        let mut orders = self.orders.lock().unwrap();
        
        if let Some(order) = orders.get_mut(order_id) {
            if order.status != OrderStatus::Pending {
                return Err(BusinessError::BusinessRuleViolation(
                    format!("只能确认待处理的订单，当前状态: {:?}", order.status)
                ));
            }
            
            order.status = OrderStatus::Confirmed;
            order.updated_at = current_timestamp();
            
            Ok(BusinessResult::success_with_message(
                order.clone(),
                "订单确认成功".to_string()
            ))
        } else {
            Err(BusinessError::ResourceNotFound("订单不存在".to_string()))
        }
    }
    
    /// 取消订单
    pub fn cancel_order(&self, order_id: &str, user: &User, reason: String) -> Result<BusinessResult<Order>, BusinessError> {
        BusinessRuleEngine::check_permission(user, "cancel_order")?;
        
        let mut orders = self.orders.lock().unwrap();
        
        if let Some(order) = orders.get_mut(order_id) {
            // 只有订单所有者或管理员可以取消
            if order.user_id != user.id && user.role != UserRole::Admin && user.role != UserRole::Manager {
                return Err(BusinessError::InsufficientPermission("只能取消自己的订单".to_string()));
            }
            
            // 只能取消未发货的订单
            match order.status {
                OrderStatus::Pending | OrderStatus::Confirmed => {
                    order.status = OrderStatus::Cancelled;
                    order.updated_at = current_timestamp();
                    
                    // 恢复库存
                    let mut products = self.products.lock().unwrap();
                    for item in &order.items {
                        if let Some(product) = products.get_mut(&item.product_id) {
                            product.stock += item.quantity;
                        }
                    }
                    
                    Ok(BusinessResult::success_with_message(
                        order.clone(),
                        format!("订单取消成功，原因: {}", reason)
                    ))
                }
                _ => Err(BusinessError::BusinessRuleViolation(
                    "只能取消待处理或已确认的订单".to_string()
                )),
            }
        } else {
            Err(BusinessError::ResourceNotFound("订单不存在".to_string()))
        }
    }
    
    /// 获取用户订单列表
    pub fn get_user_orders(&self, user_id: &str) -> Result<Vec<Order>, BusinessError> {
        let orders = self.orders.lock().unwrap();
        let user_orders: Vec<Order> = orders
            .values()
            .filter(|order| order.user_id == user_id)
            .cloned()
            .collect();
        
        Ok(user_orders)
    }
    
    /// 获取订单详情
    pub fn get_order(&self, order_id: &str, user: &User) -> Result<Order, BusinessError> {
        let orders = self.orders.lock().unwrap();
        
        if let Some(order) = orders.get(order_id) {
            // 权限检查：只能查看自己的订单或管理员可以查看所有订单
            if order.user_id != user.id && user.role != UserRole::Admin && user.role != UserRole::Manager {
                return Err(BusinessError::InsufficientPermission("无权查看此订单".to_string()));
            }
            
            Ok(order.clone())
        } else {
            Err(BusinessError::ResourceNotFound("订单不存在".to_string()))
        }
    }
    
    /// 获取产品信息
    pub fn get_product(&self, product_id: &str) -> Result<Product, BusinessError> {
        let products = self.products.lock().unwrap();
        products.get(product_id)
            .cloned()
            .ok_or_else(|| BusinessError::ResourceNotFound("产品不存在".to_string()))
    }
    
    /// 获取所有产品
    pub fn get_all_products(&self) -> Vec<Product> {
        let products = self.products.lock().unwrap();
        products.values().filter(|p| p.is_active).cloned().collect()
    }
}

impl Default for OrderBusinessService {
    fn default() -> Self {
        Self::new()
    }
}

/// 业务流程协调器
pub struct BusinessProcessCoordinator {
    user_service: UserBusinessService,
    order_service: OrderBusinessService,
}

impl BusinessProcessCoordinator {
    pub fn new() -> Self {
        Self {
            user_service: UserBusinessService::new(),
            order_service: OrderBusinessService::new(),
        }
    }
    
    /// 完整的下单流程
    pub fn place_order_workflow(&self, user_id: &str, order_items: Vec<OrderItem>) -> Result<BusinessResult<Order>, BusinessError> {
        // 1. 获取用户信息
        let user = self.user_service.get_user(user_id)?;
        
        // 2. 创建订单
        let order_result = self.order_service.create_order(&user, order_items)?;
        
        if !order_result.success {
            return Ok(order_result);
        }
        
        let order = order_result.data.unwrap();
        
        // 3. 扣减用户余额（这里应该在事务中执行）
        self.user_service.recharge_balance(user_id, -order.total_amount)?;
        
        // 4. 发送通知（模拟）
        println!("发送订单确认邮件到: {}", user.email);
        
        Ok(BusinessResult::success_with_message(
            order,
            "订单处理完成".to_string()
        ).with_warning("已扣减账户余额".to_string()))
    }
    
    /// 完整的订单取消流程
    pub fn cancel_order_workflow(&self, order_id: &str, user_id: &str, reason: String) -> Result<BusinessResult<()>, BusinessError> {
        // 1. 获取用户信息
        let user = self.user_service.get_user(user_id)?;
        
        // 2. 获取订单信息
        let order = self.order_service.get_order(order_id, &user)?;
        
        // 3. 取消订单
        let cancel_result = self.order_service.cancel_order(order_id, &user, reason)?;
        
        if cancel_result.success {
            // 4. 退还余额
            self.user_service.recharge_balance(user_id, order.total_amount)?;
            
            // 5. 发送通知
            println!("发送订单取消通知到: {}", user.email);
            
            Ok(BusinessResult::success_with_message(
                (),
                "订单取消流程完成".to_string()
            ).with_warning("已退还账户余额".to_string()))
        } else {
            Ok(BusinessResult::failure(cancel_result.message))
        }
    }
}

impl Default for BusinessProcessCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

// 辅助函数
fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

/// 业务层模式演示
pub fn demo() {
    println!("=== Business Layer（业务层）模式演示 ===\n");
    
    let coordinator = BusinessProcessCoordinator::new();
    
    // 1. 用户注册和登录
    println!("1. 用户管理业务流程:");
    
    // 注册用户
    match coordinator.user_service.register_user(
        "zhang@example.com".to_string(),
        "张三".to_string(),
        "password123".to_string(),
    ) {
        Ok(result) => {
            println!("用户注册: {}", result.message);
            if let Some(user) = result.data {
                println!("注册用户: {} (ID: {})", user.name, user.id);
                
                // 充值余额
                match coordinator.user_service.recharge_balance(&user.id, 10000.0) {
                    Ok(balance_result) => println!("余额充值: {}", balance_result.message),
                    Err(e) => println!("充值失败: {}", e),
                }
                
                // 登录
                match coordinator.user_service.login_user(
                    "zhang@example.com".to_string(),
                    "password123".to_string(),
                ) {
                    Ok(login_result) => println!("用户登录: {}", login_result.message),
                    Err(e) => println!("登录失败: {}", e),
                }
            }
        }
        Err(e) => println!("注册失败: {}", e),
    }
    
    println!("{}", "=".repeat(50));
    
    // 2. 产品管理
    println!("2. 产品管理:");
    let products = coordinator.order_service.get_all_products();
    
    println!("可用产品:");
    for product in &products {
        println!("  {} - ¥{:.2} (库存: {})", product.name, product.price, product.stock);
    }
    
    println!("{}", "=".repeat(50));
    
    // 3. 订单业务流程
    println!("3. 订单业务流程:");
    
    // 获取注册的用户
    if let Ok(user) = coordinator.user_service.login_user(
        "zhang@example.com".to_string(),
        "password123".to_string(),
    ) {
        if let Some(user) = user.data {
            // 创建订单项
            let order_items = vec![
                OrderItem {
                    product_id: "prod_001".to_string(),
                    product_name: "智能手机".to_string(),
                    quantity: 1,
                    unit_price: 2999.0,
                    total_price: 2999.0,
                },
                OrderItem {
                    product_id: "prod_002".to_string(),
                    product_name: "笔记本电脑".to_string(),
                    quantity: 1,
                    unit_price: 5999.0,
                    total_price: 5999.0,
                },
            ];
            
            // 执行下单流程
            match coordinator.place_order_workflow(&user.id, order_items) {
                Ok(order_result) => {
                    println!("下单结果: {}", order_result.message);
                    for warning in &order_result.warnings {
                        println!("警告: {}", warning);
                    }
                    
                    if let Some(order) = order_result.data {
                        println!("订单ID: {}", order.id);
                        println!("订单金额: ¥{:.2}", order.total_amount);
                        println!("订单状态: {:?}", order.status);
                        
                        // 确认订单
                        match coordinator.order_service.confirm_order(&order.id, &user) {
                            Ok(confirm_result) => println!("订单确认: {}", confirm_result.message),
                            Err(e) => println!("确认失败: {}", e),
                        }
                        
                        // 取消订单演示
                        match coordinator.cancel_order_workflow(
                            &order.id,
                            &user.id,
                            "用户主动取消".to_string(),
                        ) {
                            Ok(cancel_result) => {
                                println!("取消结果: {}", cancel_result.message);
                                for warning in &cancel_result.warnings {
                                    println!("警告: {}", warning);
                                }
                            }
                            Err(e) => println!("取消失败: {}", e),
                        }
                    }
                }
                Err(e) => println!("下单失败: {}", e),
            }
            
            // 查看用户订单
            match coordinator.order_service.get_user_orders(&user.id) {
                Ok(orders) => {
                    println!("\n用户订单列表:");
                    for order in orders {
                        println!("  订单 {} - {:?} - ¥{:.2}", 
                               order.id, order.status, order.total_amount);
                    }
                }
                Err(e) => println!("获取订单列表失败: {}", e),
            }
        }
    }
    
    println!("{}", "=".repeat(50));
    
    // 4. 业务规则验证演示
    println!("4. 业务规则验证:");
    
    // 创建测试用户
    let test_user = User {
        id: "test_user".to_string(),
        email: "test@example.com".to_string(),
        name: "测试用户".to_string(),
        status: UserStatus::Active,
        role: UserRole::Guest,
        balance: 100.0,
        created_at: current_timestamp(),
        last_login: None,
    };
    
    // 测试订单金额限制
    match BusinessRuleEngine::validate_order_eligibility(&test_user, 1000.0) {
        Ok(_) => println!("订单验证通过"),
        Err(e) => println!("订单验证失败: {}", e),
    }
    
    // 测试权限检查
    match BusinessRuleEngine::check_permission(&test_user, "manage_products") {
        Ok(_) => println!("权限检查通过"),
        Err(e) => println!("权限检查失败: {}", e),
    }
    
    // 测试折扣计算
    let discount = BusinessRuleEngine::calculate_discount(&test_user, 1200.0);
    println!("计算折扣: {:.1}%", discount * 100.0);
    
    println!("{}", "=".repeat(50));
    
    // 5. 错误处理演示
    println!("5. 错误处理演示:");
    
    // 尝试注册重复邮箱
    match coordinator.user_service.register_user(
        "zhang@example.com".to_string(),
        "重复用户".to_string(),
        "password123".to_string(),
    ) {
        Ok(_) => println!("不应该成功"),
        Err(e) => println!("预期错误: {}", e),
    }
    
    // 尝试超额充值
    match coordinator.user_service.recharge_balance("test_user", 100000.0) {
        Ok(_) => println!("不应该成功"),
        Err(e) => println!("预期错误: {}", e),
    }
    
    println!("\n=== Business Layer模式演示完成 ===");
    
    // 输出模式总结
    println!("\n【Business Layer模式总结】");
    println!("核心职责:");
    println!("1. 业务规则实现：编码业务专家的知识");
    println!("2. 业务流程控制：协调多个业务操作");
    println!("3. 业务数据验证：确保业务数据的正确性");
    println!("4. 事务管理：控制业务操作的事务边界");
    println!("5. 权限控制：实现业务级别的访问控制");
    
    println!("\n设计原则:");
    println!("1. 业务逻辑集中化：避免业务逻辑散布在各层");
    println!("2. 技术无关性：不依赖于特定的技术实现");
    println!("3. 可测试性：业务逻辑易于单元测试");
    println!("4. 可维护性：业务变化时易于修改");
    
    println!("\n适用场景:");
    println!("1. 复杂的业务规则和计算");
    println!("2. 多步骤的业务流程");
    println!("3. 跨多个实体的业务操作");
    println!("4. 需要事务控制的业务场景");
    println!("5. 权限和安全控制");
} 