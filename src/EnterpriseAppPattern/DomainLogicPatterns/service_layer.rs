/*
 * 文件路径: /d%3A/workspace/RustLearn/RustDesignPattern/src/EnterpriseAppPattern/DomainLogicPatterns/service_layer.rs
 * 
 * Service Layer（服务层）模式
 * 
 * 定义：
 * Service Layer定义应用程序的边界和一套可用的操作，封装业务逻辑并控制事务。
 * 它为客户端提供一个粗粒度的接口，协调应用程序在每个操作中的响应。
 * 
 * 主要特点：
 * 1. 定义应用程序的边界
 * 2. 封装业务逻辑
 * 3. 控制事务边界
 * 4. 提供粗粒度的操作
 * 5. 协调多个领域对象
 * 
 * 优势：
 * - 清晰的应用程序边界
 * - 事务控制集中化
 * - 减少客户端与领域层的耦合
 * - 提供统一的操作接口
 * - 支持多种客户端类型
 * 
 * 适用场景：
 * - 复杂的业务逻辑需要协调多个对象
 * - 需要清晰的事务边界
 * - 多种客户端访问同一业务逻辑
 * - 需要对外提供Web服务
 */

use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::rc::Rc;
use std::cell::RefCell;

/// 服务层错误类型
#[derive(Debug)]
pub enum ServiceError {
    ValidationError(String),
    BusinessError(String),
    NotFoundError(String),
    AuthorizationError(String),
    TransactionError(String),
    ExternalServiceError(String),
}

impl Display for ServiceError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ServiceError::ValidationError(msg) => write!(f, "验证错误: {}", msg),
            ServiceError::BusinessError(msg) => write!(f, "业务错误: {}", msg),
            ServiceError::NotFoundError(msg) => write!(f, "资源未找到: {}", msg),
            ServiceError::AuthorizationError(msg) => write!(f, "授权错误: {}", msg),
            ServiceError::TransactionError(msg) => write!(f, "事务错误: {}", msg),
            ServiceError::ExternalServiceError(msg) => write!(f, "外部服务错误: {}", msg),
        }
    }
}

impl Error for ServiceError {}

/// 用户实体
#[derive(Debug, Clone)]
pub struct User {
    pub id: Option<u32>,
    pub username: String,
    pub email: String,
    pub balance: f64,
    pub status: UserStatus,
    pub level: UserLevel,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UserStatus {
    Active,
    Inactive,
    Suspended,
    Deleted,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UserLevel {
    Bronze,
    Silver,
    Gold,
    Platinum,
    VIP,
}

impl User {
    pub fn new(username: String, email: String) -> Self {
        Self {
            id: None,
            username,
            email,
            balance: 0.0,
            status: UserStatus::Active,
            level: UserLevel::Bronze,
        }
    }
    
    pub fn is_active(&self) -> bool {
        self.status == UserStatus::Active
    }
    
    pub fn can_transfer(&self, amount: f64) -> bool {
        self.is_active() && self.balance >= amount && amount > 0.0
    }
    
    pub fn update_level_by_balance(&mut self) {
        self.level = if self.balance >= 100000.0 {
            UserLevel::VIP
        } else if self.balance >= 50000.0 {
            UserLevel::Platinum
        } else if self.balance >= 20000.0 {
            UserLevel::Gold
        } else if self.balance >= 5000.0 {
            UserLevel::Silver
        } else {
            UserLevel::Bronze
        };
    }
}

/// 订单实体
#[derive(Debug, Clone)]
pub struct Order {
    pub id: Option<u32>,
    pub user_id: u32,
    pub amount: f64,
    pub status: OrderStatus,
    pub items: Vec<OrderItem>,
    pub created_at: String, // 简化为字符串
}

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

#[derive(Debug, Clone)]
pub struct OrderItem {
    pub product_id: u32,
    pub product_name: String,
    pub quantity: u32,
    pub unit_price: f64,
}

impl Order {
    pub fn new(user_id: u32, items: Vec<OrderItem>) -> Self {
        let amount = items.iter().map(|item| item.quantity as f64 * item.unit_price).sum();
        Self {
            id: None,
            user_id,
            amount,
            status: OrderStatus::Pending,
            items,
            created_at: "2024-01-01".to_string(), // 简化
        }
    }
    
    pub fn can_cancel(&self) -> bool {
        matches!(self.status, OrderStatus::Pending | OrderStatus::Confirmed)
    }
    
    pub fn total_amount(&self) -> f64 {
        self.items.iter().map(|item| item.quantity as f64 * item.unit_price).sum()
    }
}

/// 支付记录
#[derive(Debug, Clone)]
pub struct Payment {
    pub id: Option<u32>,
    pub order_id: u32,
    pub amount: f64,
    pub method: PaymentMethod,
    pub status: PaymentStatus,
}

#[derive(Debug, Clone)]
pub enum PaymentMethod {
    Balance,
    CreditCard,
    Alipay,
    WeChat,
    BankTransfer,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PaymentStatus {
    Pending,
    Completed,
    Failed,
    Refunded,
}

/// 通知消息
#[derive(Debug, Clone)]
pub struct Notification {
    pub user_id: u32,
    pub title: String,
    pub content: String,
    pub notification_type: NotificationType,
}

#[derive(Debug, Clone)]
pub enum NotificationType {
    OrderConfirmation,
    PaymentSuccess,
    Shipping,
    SystemAlert,
    Promotion,
}

/// 数据传输对象 (DTO)
#[derive(Debug)]
pub struct CreateUserRequest {
    pub username: String,
    pub email: String,
    pub initial_balance: Option<f64>,
}

#[derive(Debug)]
pub struct TransferRequest {
    pub from_user_id: u32,
    pub to_user_id: u32,
    pub amount: f64,
    pub description: Option<String>,
}

#[derive(Debug)]
pub struct CreateOrderRequest {
    pub user_id: u32,
    pub items: Vec<CreateOrderItem>,
}

#[derive(Debug)]
pub struct CreateOrderItem {
    pub product_id: u32,
    pub product_name: String,
    pub quantity: u32,
    pub unit_price: f64,
}

#[derive(Debug)]
pub struct ProcessPaymentRequest {
    pub order_id: u32,
    pub payment_method: PaymentMethod,
}

/// 服务响应
#[derive(Debug)]
pub struct ServiceResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: String,
    pub errors: Vec<String>,
}

impl<T> ServiceResponse<T> {
    pub fn success(data: T, message: String) -> Self {
        Self {
            success: true,
            data: Some(data),
            message,
            errors: Vec::new(),
        }
    }
    
    pub fn error(message: String, errors: Vec<String>) -> Self {
        Self {
            success: false,
            data: None,
            message,
            errors,
        }
    }
}

/// 仓储接口的简单实现（模拟数据库）
#[derive(Clone)]
pub struct MockRepository {
    users: Rc<RefCell<HashMap<u32, User>>>,
    orders: Rc<RefCell<HashMap<u32, Order>>>,
    payments: Rc<RefCell<HashMap<u32, Payment>>>,
    next_user_id: Rc<RefCell<u32>>,
    next_order_id: Rc<RefCell<u32>>,
    next_payment_id: Rc<RefCell<u32>>,
}

impl MockRepository {
    pub fn new() -> Self {
        Self {
            users: Rc::new(RefCell::new(HashMap::new())),
            orders: Rc::new(RefCell::new(HashMap::new())),
            payments: Rc::new(RefCell::new(HashMap::new())),
            next_user_id: Rc::new(RefCell::new(1)),
            next_order_id: Rc::new(RefCell::new(1)),
            next_payment_id: Rc::new(RefCell::new(1)),
        }
    }
    
    pub fn save_user(&self, mut user: User) -> User {
        if user.id.is_none() {
            let id = *self.next_user_id.borrow();
            user.id = Some(id);
            *self.next_user_id.borrow_mut() += 1;
        }
        
        let user_id = user.id.unwrap();
        self.users.borrow_mut().insert(user_id, user.clone());
        user
    }
    
    pub fn find_user(&self, id: u32) -> Option<User> {
        self.users.borrow().get(&id).cloned()
    }
    
    pub fn find_user_by_username(&self, username: &str) -> Option<User> {
        self.users.borrow().values()
            .find(|user| user.username == username)
            .cloned()
    }
    
    pub fn save_order(&self, mut order: Order) -> Order {
        if order.id.is_none() {
            let id = *self.next_order_id.borrow();
            order.id = Some(id);
            *self.next_order_id.borrow_mut() += 1;
        }
        
        let order_id = order.id.unwrap();
        self.orders.borrow_mut().insert(order_id, order.clone());
        order
    }
    
    pub fn find_order(&self, id: u32) -> Option<Order> {
        self.orders.borrow().get(&id).cloned()
    }
    
    pub fn find_orders_by_user(&self, user_id: u32) -> Vec<Order> {
        self.orders.borrow().values()
            .filter(|order| order.user_id == user_id)
            .cloned()
            .collect()
    }
    
    pub fn save_payment(&self, mut payment: Payment) -> Payment {
        if payment.id.is_none() {
            let id = *self.next_payment_id.borrow();
            payment.id = Some(id);
            *self.next_payment_id.borrow_mut() += 1;
        }
        
        let payment_id = payment.id.unwrap();
        self.payments.borrow_mut().insert(payment_id, payment.clone());
        payment
    }
    
    pub fn find_payment_by_order(&self, order_id: u32) -> Option<Payment> {
        self.payments.borrow().values()
            .find(|payment| payment.order_id == order_id)
            .cloned()
    }
}

/// 通知服务接口
pub trait NotificationService {
    fn send_notification(&self, notification: Notification) -> Result<(), ServiceError>;
    fn send_email(&self, to: &str, subject: &str, body: &str) -> Result<(), ServiceError>;
    fn send_sms(&self, phone: &str, message: &str) -> Result<(), ServiceError>;
}

/// 模拟通知服务
pub struct MockNotificationService;

impl NotificationService for MockNotificationService {
    fn send_notification(&self, notification: Notification) -> Result<(), ServiceError> {
        println!("发送通知给用户 {}: {} - {}", 
                notification.user_id, notification.title, notification.content);
        Ok(())
    }
    
    fn send_email(&self, to: &str, subject: &str, body: &str) -> Result<(), ServiceError> {
        println!("发送邮件到 {}: {} - {}", to, subject, body);
        Ok(())
    }
    
    fn send_sms(&self, phone: &str, message: &str) -> Result<(), ServiceError> {
        println!("发送短信到 {}: {}", phone, message);
        Ok(())
    }
}

/// 用户服务层
pub struct UserService {
    repository: MockRepository,
    notification_service: Box<dyn NotificationService>,
}

impl UserService {
    pub fn new(repository: MockRepository, notification_service: Box<dyn NotificationService>) -> Self {
        Self {
            repository,
            notification_service,
        }
    }
    
    /// 创建用户
    pub fn create_user(&self, request: CreateUserRequest) -> Result<ServiceResponse<User>, ServiceError> {
        // 验证输入
        if request.username.trim().is_empty() {
            return Ok(ServiceResponse::error(
                "创建用户失败".to_string(),
                vec!["用户名不能为空".to_string()]
            ));
        }
        
        if !request.email.contains('@') {
            return Ok(ServiceResponse::error(
                "创建用户失败".to_string(),
                vec!["邮箱格式不正确".to_string()]
            ));
        }
        
        // 检查用户名是否已存在
        if self.repository.find_user_by_username(&request.username).is_some() {
            return Ok(ServiceResponse::error(
                "创建用户失败".to_string(),
                vec!["用户名已存在".to_string()]
            ));
        }
        
        // 创建用户
        let mut user = User::new(request.username, request.email);
        
        if let Some(initial_balance) = request.initial_balance {
            if initial_balance < 0.0 {
                return Ok(ServiceResponse::error(
                    "创建用户失败".to_string(),
                    vec!["初始余额不能为负数".to_string()]
                ));
            }
            user.balance = initial_balance;
            user.update_level_by_balance();
        }
        
        // 保存用户
        let saved_user = self.repository.save_user(user);
        
        // 发送欢迎通知
        let notification = Notification {
            user_id: saved_user.id.unwrap(),
            title: "欢迎加入".to_string(),
            content: format!("欢迎 {} 加入我们的平台！", saved_user.username),
            notification_type: NotificationType::SystemAlert,
        };
        
        if let Err(e) = self.notification_service.send_notification(notification) {
            println!("警告：发送欢迎通知失败: {}", e);
        }
        
        Ok(ServiceResponse::success(
            saved_user,
            "用户创建成功".to_string()
        ))
    }
    
    /// 用户转账
    pub fn transfer_money(&self, request: TransferRequest) -> Result<ServiceResponse<String>, ServiceError> {
        // 开始事务（模拟）
        println!("开始转账事务");
        
        // 查找发送方和接收方
        let mut from_user = match self.repository.find_user(request.from_user_id) {
            Some(user) => user,
            None => return Ok(ServiceResponse::error(
                "转账失败".to_string(),
                vec!["发送方用户不存在".to_string()]
            )),
        };
        
        let mut to_user = match self.repository.find_user(request.to_user_id) {
            Some(user) => user,
            None => return Ok(ServiceResponse::error(
                "转账失败".to_string(),
                vec!["接收方用户不存在".to_string()]
            )),
        };
        
        // 业务验证
        if !from_user.can_transfer(request.amount) {
            return Ok(ServiceResponse::error(
                "转账失败".to_string(),
                vec!["余额不足或用户状态异常".to_string()]
            ));
        }
        
        if !to_user.is_active() {
            return Ok(ServiceResponse::error(
                "转账失败".to_string(),
                vec!["接收方用户状态异常".to_string()]
            ));
        }
        
        if request.amount <= 0.0 {
            return Ok(ServiceResponse::error(
                "转账失败".to_string(),
                vec!["转账金额必须大于0".to_string()]
            ));
        }
        
        // 执行转账
        from_user.balance -= request.amount;
        to_user.balance += request.amount;
        
        // 更新用户等级
        from_user.update_level_by_balance();
        to_user.update_level_by_balance();
        
        // 保存更改
        self.repository.save_user(from_user.clone());
        self.repository.save_user(to_user.clone());
        
        // 发送通知
        let from_notification = Notification {
            user_id: from_user.id.unwrap(),
            title: "转账成功".to_string(),
            content: format!("您已成功向 {} 转账 ¥{:.2}", to_user.username, request.amount),
            notification_type: NotificationType::SystemAlert,
        };
        
        let to_notification = Notification {
            user_id: to_user.id.unwrap(),
            title: "收款成功".to_string(),
            content: format!("您已收到来自 {} 的转账 ¥{:.2}", from_user.username, request.amount),
            notification_type: NotificationType::SystemAlert,
        };
        
        let _ = self.notification_service.send_notification(from_notification);
        let _ = self.notification_service.send_notification(to_notification);
        
        println!("转账事务完成");
        
        let description = request.description.unwrap_or_else(|| "无备注".to_string());
        Ok(ServiceResponse::success(
            format!("转账成功：{} -> {}，金额：¥{:.2}，备注：{}", 
                   from_user.username, to_user.username, request.amount, description),
            "转账操作完成".to_string()
        ))
    }
    
    /// 获取用户信息
    pub fn get_user_info(&self, user_id: u32) -> Result<ServiceResponse<User>, ServiceError> {
        match self.repository.find_user(user_id) {
            Some(user) => Ok(ServiceResponse::success(
                user,
                "获取用户信息成功".to_string()
            )),
            None => Ok(ServiceResponse::error(
                "获取用户信息失败".to_string(),
                vec!["用户不存在".to_string()]
            )),
        }
    }
    
    /// 更新用户余额
    pub fn update_balance(&self, user_id: u32, amount: f64) -> Result<ServiceResponse<User>, ServiceError> {
        let mut user = match self.repository.find_user(user_id) {
            Some(user) => user,
            None => return Ok(ServiceResponse::error(
                "更新余额失败".to_string(),
                vec!["用户不存在".to_string()]
            )),
        };
        
        if user.balance + amount < 0.0 {
            return Ok(ServiceResponse::error(
                "更新余额失败".to_string(),
                vec!["余额不足".to_string()]
            ));
        }
        
        user.balance += amount;
        user.update_level_by_balance();
        
        let updated_user = self.repository.save_user(user);
        
        Ok(ServiceResponse::success(
            updated_user,
            format!("余额更新成功，变动金额：¥{:.2}", amount)
        ))
    }
}

/// 订单服务层
pub struct OrderService {
    repository: MockRepository,
    user_service: UserService,
    notification_service: Box<dyn NotificationService>,
}

impl OrderService {
    pub fn new(
        repository: MockRepository, 
        user_service: UserService,
        notification_service: Box<dyn NotificationService>
    ) -> Self {
        Self {
            repository,
            user_service,
            notification_service,
        }
    }
    
    /// 创建订单
    pub fn create_order(&self, request: CreateOrderRequest) -> Result<ServiceResponse<Order>, ServiceError> {
        // 验证用户
        let user = match self.repository.find_user(request.user_id) {
            Some(user) => user,
            None => return Ok(ServiceResponse::error(
                "创建订单失败".to_string(),
                vec!["用户不存在".to_string()]
            )),
        };
        
        if !user.is_active() {
            return Ok(ServiceResponse::error(
                "创建订单失败".to_string(),
                vec!["用户状态异常".to_string()]
            ));
        }
        
        // 验证订单项
        if request.items.is_empty() {
            return Ok(ServiceResponse::error(
                "创建订单失败".to_string(),
                vec!["订单项不能为空".to_string()]
            ));
        }
        
        // 转换订单项
        let order_items: Vec<OrderItem> = request.items.into_iter()
            .map(|item| OrderItem {
                product_id: item.product_id,
                product_name: item.product_name,
                quantity: item.quantity,
                unit_price: item.unit_price,
            })
            .collect();
        
        // 创建订单
        let order = Order::new(request.user_id, order_items);
        let saved_order = self.repository.save_order(order);
        
        // 发送订单确认通知
        let notification = Notification {
            user_id: request.user_id,
            title: "订单创建成功".to_string(),
            content: format!("您的订单 #{} 已创建，金额：¥{:.2}", 
                           saved_order.id.unwrap(), saved_order.amount),
            notification_type: NotificationType::OrderConfirmation,
        };
        
        let _ = self.notification_service.send_notification(notification);
        
        Ok(ServiceResponse::success(
            saved_order,
            "订单创建成功".to_string()
        ))
    }
    
    /// 处理支付
    pub fn process_payment(&self, request: ProcessPaymentRequest) -> Result<ServiceResponse<Payment>, ServiceError> {
        // 查找订单
        let mut order = match self.repository.find_order(request.order_id) {
            Some(order) => order,
            None => return Ok(ServiceResponse::error(
                "支付处理失败".to_string(),
                vec!["订单不存在".to_string()]
            )),
        };
        
        // 检查订单状态
        if order.status != OrderStatus::Pending {
            return Ok(ServiceResponse::error(
                "支付处理失败".to_string(),
                vec!["订单状态不允许支付".to_string()]
            ));
        }
        
        // 根据支付方式处理
        let payment_result = match request.payment_method {
            PaymentMethod::Balance => {
                // 余额支付
                match self.user_service.update_balance(order.user_id, -order.amount) {
                    Ok(response) => {
                        if response.success {
                            PaymentStatus::Completed
                        } else {
                            return Ok(ServiceResponse::error(
                                "支付处理失败".to_string(),
                                response.errors
                            ));
                        }
                    }
                    Err(_) => PaymentStatus::Failed,
                }
            }
            _ => {
                // 其他支付方式（模拟成功）
                println!("模拟 {:?} 支付处理...", request.payment_method);
                PaymentStatus::Completed
            }
        };
        
        // 创建支付记录
        let payment = Payment {
            id: None,
            order_id: request.order_id,
            amount: order.amount,
            method: request.payment_method,
            status: payment_result,
        };
        
        let saved_payment = self.repository.save_payment(payment);
        
        // 更新订单状态
        if saved_payment.status == PaymentStatus::Completed {
            order.status = OrderStatus::Confirmed;
            let user_id = order.user_id; // 保存user_id，因为order将被移动
            self.repository.save_order(order);
            
            // 发送支付成功通知
            let notification = Notification {
                user_id: user_id,
                title: "支付成功".to_string(),
                content: format!("订单 #{} 支付成功，金额：¥{:.2}", 
                               request.order_id, saved_payment.amount),
                notification_type: NotificationType::PaymentSuccess,
            };
            
            let _ = self.notification_service.send_notification(notification);
        }
        
        Ok(ServiceResponse::success(
            saved_payment,
            "支付处理完成".to_string()
        ))
    }
    
    /// 取消订单
    pub fn cancel_order(&self, order_id: u32, user_id: u32) -> Result<ServiceResponse<String>, ServiceError> {
        let mut order = match self.repository.find_order(order_id) {
            Some(order) => order,
            None => return Ok(ServiceResponse::error(
                "取消订单失败".to_string(),
                vec!["订单不存在".to_string()]
            )),
        };
        
        // 验证用户权限
        if order.user_id != user_id {
            return Ok(ServiceResponse::error(
                "取消订单失败".to_string(),
                vec!["没有权限操作此订单".to_string()]
            ));
        }
        
        // 检查是否可以取消
        if !order.can_cancel() {
            return Ok(ServiceResponse::error(
                "取消订单失败".to_string(),
                vec!["订单状态不允许取消".to_string()]
            ));
        }
        
        // 如果已支付，需要退款
        if order.status == OrderStatus::Confirmed {
            if let Some(payment) = self.repository.find_payment_by_order(order_id) {
                if payment.status == PaymentStatus::Completed {
                    // 退款逻辑
                    if matches!(payment.method, PaymentMethod::Balance) {
                        let _ = self.user_service.update_balance(order.user_id, order.amount);
                    }
                    
                    // 更新支付状态为已退款（这里简化处理）
                    println!("订单 #{} 已退款 ¥{:.2}", order_id, payment.amount);
                }
            }
        }
        
        // 更新订单状态
        order.status = OrderStatus::Cancelled;
        self.repository.save_order(order);
        
        Ok(ServiceResponse::success(
            format!("订单 #{} 已成功取消", order_id),
            "订单取消成功".to_string()
        ))
    }
    
    /// 获取用户订单列表
    pub fn get_user_orders(&self, user_id: u32) -> Result<ServiceResponse<Vec<Order>>, ServiceError> {
        let orders = self.repository.find_orders_by_user(user_id);
        
        Ok(ServiceResponse::success(
            orders,
            "获取订单列表成功".to_string()
        ))
    }
}

/// Service Layer模式演示
pub fn demo() {
    println!("=== Service Layer（服务层）模式演示 ===\n");
    
    // 1. 初始化服务
    println!("1. 初始化服务层:");
    let repository = MockRepository::new();
    let notification_service = Box::new(MockNotificationService);
    let user_service = UserService::new(repository.clone(), notification_service);
    
    let notification_service2 = Box::new(MockNotificationService);
    let user_service_for_order = UserService::new(repository.clone(), Box::new(MockNotificationService));
    let order_service = OrderService::new(repository.clone(), user_service_for_order, notification_service2);
    
    println!("服务层初始化完成");
    
    println!("{}", "=".repeat(50));
    
    // 2. 用户服务演示
    println!("2. 用户服务操作:");
    
    // 创建用户
    let create_user1 = CreateUserRequest {
        username: "alice".to_string(),
        email: "alice@example.com".to_string(),
        initial_balance: Some(10000.0),
    };
    
    let create_user2 = CreateUserRequest {
        username: "bob".to_string(),
        email: "bob@example.com".to_string(),
        initial_balance: Some(5000.0),
    };
    
    let user1_response = user_service.create_user(create_user1).unwrap();
    let user2_response = user_service.create_user(create_user2).unwrap();
    
    if user1_response.success && user2_response.success {
        let user1 = user1_response.data.unwrap();
        let user2 = user2_response.data.unwrap();
        
        println!("用户1: {} (ID: {}, 余额: ¥{:.2}, 等级: {:?})", 
               user1.username, user1.id.unwrap(), user1.balance, user1.level);
        println!("用户2: {} (ID: {}, 余额: ¥{:.2}, 等级: {:?})", 
               user2.username, user2.id.unwrap(), user2.balance, user2.level);
        
        // 转账操作
        println!("\n转账操作:");
        let transfer_request = TransferRequest {
            from_user_id: user1.id.unwrap(),
            to_user_id: user2.id.unwrap(),
            amount: 2000.0,
            description: Some("测试转账".to_string()),
        };
        
        match user_service.transfer_money(transfer_request) {
            Ok(response) => {
                if response.success {
                    println!("✅ {}", response.data.unwrap());
                } else {
                    println!("❌ 转账失败: {:?}", response.errors);
                }
            }
            Err(e) => println!("❌ 转账错误: {}", e),
        }
        
        // 查看转账后的用户信息
        println!("\n转账后用户信息:");
        match user_service.get_user_info(user1.id.unwrap()) {
            Ok(response) => {
                if let Some(user) = response.data {
                    println!("用户1 - 余额: ¥{:.2}, 等级: {:?}", user.balance, user.level);
                }
            }
            Err(e) => println!("获取用户1信息失败: {}", e),
        }
        
        match user_service.get_user_info(user2.id.unwrap()) {
            Ok(response) => {
                if let Some(user) = response.data {
                    println!("用户2 - 余额: ¥{:.2}, 等级: {:?}", user.balance, user.level);
                }
            }
            Err(e) => println!("获取用户2信息失败: {}", e),
        }
        
        println!("{}", "=".repeat(50));
        
        // 3. 订单服务演示
        println!("3. 订单服务操作:");
        
        // 创建订单
        let create_order_request = CreateOrderRequest {
            user_id: user1.id.unwrap(),
            items: vec![
                CreateOrderItem {
                    product_id: 1,
                    product_name: "智能手机".to_string(),
                    quantity: 1,
                    unit_price: 2999.0,
                },
                CreateOrderItem {
                    product_id: 2,
                    product_name: "手机壳".to_string(),
                    quantity: 2,
                    unit_price: 49.0,
                },
            ],
        };
        
        match order_service.create_order(create_order_request) {
            Ok(response) => {
                if response.success {
                    let order = response.data.unwrap();
                    println!("✅ 订单创建成功:");
                    println!("   订单ID: {}", order.id.unwrap());
                    println!("   用户ID: {}", order.user_id);
                    println!("   订单金额: ¥{:.2}", order.amount);
                    println!("   订单状态: {:?}", order.status);
                    println!("   商品项目:");
                    for item in &order.items {
                        println!("     - {} x {} = ¥{:.2}", 
                               item.product_name, item.quantity, 
                               item.quantity as f64 * item.unit_price);
                    }
                    
                    // 处理支付
                    println!("\n处理支付:");
                    let payment_request = ProcessPaymentRequest {
                        order_id: order.id.unwrap(),
                        payment_method: PaymentMethod::Balance,
                    };
                    
                    match order_service.process_payment(payment_request) {
                        Ok(payment_response) => {
                            if payment_response.success {
                                let payment = payment_response.data.unwrap();
                                println!("✅ 支付成功:");
                                println!("   支付ID: {}", payment.id.unwrap());
                                println!("   支付金额: ¥{:.2}", payment.amount);
                                println!("   支付方式: {:?}", payment.method);
                                println!("   支付状态: {:?}", payment.status);
                            } else {
                                println!("❌ 支付失败: {:?}", payment_response.errors);
                            }
                        }
                        Err(e) => println!("❌ 支付错误: {}", e),
                    }
                    
                    // 获取用户订单列表
                    println!("\n用户订单列表:");
                    match order_service.get_user_orders(user1.id.unwrap()) {
                        Ok(orders_response) => {
                            if orders_response.success {
                                let orders = orders_response.data.unwrap();
                                println!("用户 {} 共有 {} 个订单:", user1.username, orders.len());
                                for order in orders {
                                    println!("  订单 #{}: 金额 ¥{:.2}, 状态 {:?}", 
                                           order.id.unwrap(), order.amount, order.status);
                                }
                            }
                        }
                        Err(e) => println!("获取订单列表失败: {}", e),
                    }
                    
                } else {
                    println!("❌ 订单创建失败: {:?}", response.errors);
                }
            }
            Err(e) => println!("❌ 订单创建错误: {}", e),
        }
    }
    
    println!("{}", "=".repeat(50));
    
    // 4. 错误处理演示
    println!("4. 错误处理演示:");
    
    // 尝试创建重复用户名的用户
    let duplicate_user = CreateUserRequest {
        username: "alice".to_string(),  // 重复用户名
        email: "alice2@example.com".to_string(),
        initial_balance: None,
    };
    
    match user_service.create_user(duplicate_user) {
        Ok(response) => {
            if !response.success {
                println!("✅ 正确拒绝重复用户名: {:?}", response.errors);
            }
        }
        Err(e) => println!("创建用户错误: {}", e),
    }
    
    // 尝试转账给不存在的用户
    let invalid_transfer = TransferRequest {
        from_user_id: 1,
        to_user_id: 999,  // 不存在的用户
        amount: 100.0,
        description: None,
    };
    
    match user_service.transfer_money(invalid_transfer) {
        Ok(response) => {
            if !response.success {
                println!("✅ 正确拒绝无效转账: {:?}", response.errors);
            }
        }
        Err(e) => println!("转账错误: {}", e),
    }
    
    // 尝试余额不足的转账
    let insufficient_transfer = TransferRequest {
        from_user_id: 2,
        to_user_id: 1,
        amount: 100000.0,  // 超过余额
        description: None,
    };
    
    match user_service.transfer_money(insufficient_transfer) {
        Ok(response) => {
            if !response.success {
                println!("✅ 正确拒绝余额不足转账: {:?}", response.errors);
            }
        }
        Err(e) => println!("转账错误: {}", e),
    }
    
    println!("\n=== Service Layer模式演示完成 ===");
    
    // 输出模式总结
    println!("\n【Service Layer模式总结】");
    println!("核心特点:");
    println!("1. 应用程序边界：定义清晰的服务接口");
    println!("2. 事务控制：管理业务操作的事务边界");
    println!("3. 业务逻辑封装：协调多个领域对象完成业务操作");
    println!("4. 粗粒度操作：提供高层次的业务操作接口");
    println!("5. 客户端隔离：减少客户端与领域层的直接耦合");
    
    println!("\n优势:");
    println!("1. 清晰的架构分层");
    println!("2. 统一的事务管理");
    println!("3. 业务逻辑复用");
    println!("4. 易于测试和维护");
    println!("5. 支持多种客户端类型");
    
    println!("\n适用场景:");
    println!("1. 复杂的业务逻辑需要协调多个对象");
    println!("2. 需要清晰的事务边界控制");
    println!("3. 多种客户端（Web、API、移动端）");
    println!("4. 需要对外提供服务接口");
    println!("5. 企业级应用开发");
} 