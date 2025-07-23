/*
 * 文件路径: /d%3A/workspace/RustLearn/RustDesignPattern/src/EnterpriseAppPattern/DistributionPatterns/data_transfer_object.rs
 * 
 * Data Transfer Object（数据传输对象）模式
 * 
 * 定义：
 * Data Transfer Object是一个数据容器对象，用于在不同的进程或网络边界之间传输数据。
 * 它通过减少方法调用的数量来提高分布式应用程序的性能。
 * 
 * 主要特点：
 * 1. 只包含数据，没有业务逻辑
 * 2. 序列化友好
 * 3. 扁平化数据结构
 * 4. 减少远程调用次数
 * 5. 版本兼容性考虑
 * 
 * 优势：
 * - 减少网络往返次数
 * - 提高分布式应用性能
 * - 数据传输优化
 * - 解耦客户端和服务端
 * - 版本演化支持
 * 
 * 适用场景：
 * - 分布式系统
 * - 跨网络的数据传输
 * - API设计
 * - 微服务架构
 */

use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Display, Formatter};

/// DTO错误类型
#[derive(Debug)]
pub enum DtoError {
    SerializationError(String),
    ValidationError(String),
    ConversionError(String),
}

impl Display for DtoError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DtoError::SerializationError(msg) => write!(f, "序列化错误: {}", msg),
            DtoError::ValidationError(msg) => write!(f, "验证错误: {}", msg),
            DtoError::ConversionError(msg) => write!(f, "转换错误: {}", msg),
        }
    }
}

impl Error for DtoError {}

/// 用户DTO（数据传输对象）
#[derive(Debug, Clone)]
pub struct UserDto {
    pub id: Option<u32>,
    pub username: String,
    pub email: String,
    pub full_name: String,
    pub phone: Option<String>,
    pub address: Option<AddressDto>,
    pub account_balance: f64,
    pub account_level: String,
    pub registration_date: String, // ISO 8601格式
    pub last_login_date: Option<String>,
    pub is_active: bool,
    pub preferences: HashMap<String, String>,
}

impl UserDto {
    pub fn new(username: String, email: String, full_name: String) -> Self {
        Self {
            id: None,
            username,
            email,
            full_name,
            phone: None,
            address: None,
            account_balance: 0.0,
            account_level: "Bronze".to_string(),
            registration_date: "2024-01-01T00:00:00Z".to_string(),
            last_login_date: None,
            is_active: true,
            preferences: HashMap::new(),
        }
    }
    
    /// 验证DTO数据
    pub fn validate(&self) -> Result<(), DtoError> {
        if self.username.trim().is_empty() {
            return Err(DtoError::ValidationError("用户名不能为空".to_string()));
        }
        
        if !self.email.contains('@') {
            return Err(DtoError::ValidationError("邮箱格式不正确".to_string()));
        }
        
        if self.full_name.trim().is_empty() {
            return Err(DtoError::ValidationError("姓名不能为空".to_string()));
        }
        
        if self.account_balance < 0.0 {
            return Err(DtoError::ValidationError("账户余额不能为负数".to_string()));
        }
        
        Ok(())
    }
    
    /// 序列化为JSON字符串（模拟）
    pub fn to_json(&self) -> Result<String, DtoError> {
        // 在真实应用中，这里会使用serde等序列化库
        let json = format!(
            r#"{{
    "id": {},
    "username": "{}",
    "email": "{}",
    "full_name": "{}",
    "phone": {},
    "address": {},
    "account_balance": {},
    "account_level": "{}",
    "registration_date": "{}",
    "last_login_date": {},
    "is_active": {},
    "preferences": {}
}}"#,
            match self.id { Some(id) => id.to_string(), None => "null".to_string() },
            self.username,
            self.email,
            self.full_name,
            match &self.phone { Some(p) => format!("\"{}\"", p), None => "null".to_string() },
            match &self.address { Some(a) => a.to_json()?, None => "null".to_string() },
            self.account_balance,
            self.account_level,
            self.registration_date,
            match &self.last_login_date { Some(d) => format!("\"{}\"", d), None => "null".to_string() },
            self.is_active,
            self.preferences_to_json()?
        );
        Ok(json)
    }
    
    fn preferences_to_json(&self) -> Result<String, DtoError> {
        let items: Vec<String> = self.preferences.iter()
            .map(|(k, v)| format!("\"{}\":\"{}\"", k, v))
            .collect();
        Ok(format!("{{{}}}", items.join(",")))
    }
    
    /// 创建简化版本（用于列表显示）
    pub fn to_summary(&self) -> UserSummaryDto {
        UserSummaryDto {
            id: self.id,
            username: self.username.clone(),
            email: self.email.clone(),
            full_name: self.full_name.clone(),
            account_level: self.account_level.clone(),
            is_active: self.is_active,
        }
    }
}

/// 地址DTO
#[derive(Debug, Clone)]
pub struct AddressDto {
    pub street: String,
    pub city: String,
    pub state: String,
    pub postal_code: String,
    pub country: String,
}

impl AddressDto {
    pub fn new(street: String, city: String, state: String, postal_code: String, country: String) -> Self {
        Self {
            street,
            city,
            state,
            postal_code,
            country,
        }
    }
    
    pub fn to_json(&self) -> Result<String, DtoError> {
        Ok(format!(
            r#"{{"street":"{}","city":"{}","state":"{}","postal_code":"{}","country":"{}"}}"#,
            self.street, self.city, self.state, self.postal_code, self.country
        ))
    }
    
    pub fn format_full_address(&self) -> String {
        format!("{}, {}, {} {}, {}", 
               self.street, self.city, self.state, self.postal_code, self.country)
    }
}

/// 用户摘要DTO（用于列表显示，减少数据传输）
#[derive(Debug, Clone)]
pub struct UserSummaryDto {
    pub id: Option<u32>,
    pub username: String,
    pub email: String,
    pub full_name: String,
    pub account_level: String,
    pub is_active: bool,
}

impl UserSummaryDto {
    pub fn to_json(&self) -> Result<String, DtoError> {
        Ok(format!(
            r#"{{"id":{},"username":"{}","email":"{}","full_name":"{}","account_level":"{}","is_active":{}}}"#,
            match self.id { Some(id) => id.to_string(), None => "null".to_string() },
            self.username,
            self.email,
            self.full_name,
            self.account_level,
            self.is_active
        ))
    }
}

/// 订单DTO
#[derive(Debug, Clone)]
pub struct OrderDto {
    pub id: Option<u32>,
    pub user_id: u32,
    pub order_number: String,
    pub order_date: String,
    pub status: String,
    pub total_amount: f64,
    pub currency: String,
    pub items: Vec<OrderItemDto>,
    pub shipping_address: AddressDto,
    pub billing_address: Option<AddressDto>,
    pub payment_method: String,
    pub notes: Option<String>,
}

impl OrderDto {
    pub fn new(user_id: u32, order_number: String, shipping_address: AddressDto) -> Self {
        Self {
            id: None,
            user_id,
            order_number,
            order_date: "2024-01-01T00:00:00Z".to_string(),
            status: "Pending".to_string(),
            total_amount: 0.0,
            currency: "USD".to_string(),
            items: Vec::new(),
            shipping_address,
            billing_address: None,
            payment_method: "Credit Card".to_string(),
            notes: None,
        }
    }
    
    pub fn add_item(&mut self, item: OrderItemDto) {
        self.total_amount += item.quantity as f64 * item.unit_price;
        self.items.push(item);
    }
    
    pub fn validate(&self) -> Result<(), DtoError> {
        if self.order_number.trim().is_empty() {
            return Err(DtoError::ValidationError("订单号不能为空".to_string()));
        }
        
        if self.items.is_empty() {
            return Err(DtoError::ValidationError("订单项不能为空".to_string()));
        }
        
        if self.total_amount <= 0.0 {
            return Err(DtoError::ValidationError("订单总额必须大于0".to_string()));
        }
        
        // 验证每个订单项
        for item in &self.items {
            item.validate()?;
        }
        
        Ok(())
    }
    
    pub fn to_json(&self) -> Result<String, DtoError> {
        let items_json: Result<Vec<String>, DtoError> = self.items.iter()
            .map(|item| item.to_json())
            .collect();
        
        let items_str = format!("[{}]", items_json?.join(","));
        
        Ok(format!(
            r#"{{"id":{},"user_id":{},"order_number":"{}","order_date":"{}","status":"{}","total_amount":{},"currency":"{}","items":{},"shipping_address":{},"billing_address":{},"payment_method":"{}","notes":{}}}"#,
            match self.id { Some(id) => id.to_string(), None => "null".to_string() },
            self.user_id,
            self.order_number,
            self.order_date,
            self.status,
            self.total_amount,
            self.currency,
            items_str,
            self.shipping_address.to_json()?,
            match &self.billing_address { Some(a) => a.to_json()?, None => "null".to_string() },
            self.payment_method,
            match &self.notes { Some(n) => format!("\"{}\"", n), None => "null".to_string() }
        ))
    }
    
    /// 创建订单摘要（用于列表显示）
    pub fn to_summary(&self) -> OrderSummaryDto {
        OrderSummaryDto {
            id: self.id,
            order_number: self.order_number.clone(),
            order_date: self.order_date.clone(),
            status: self.status.clone(),
            total_amount: self.total_amount,
            currency: self.currency.clone(),
            items_count: self.items.len(),
        }
    }
}

/// 订单项DTO
#[derive(Debug, Clone)]
pub struct OrderItemDto {
    pub product_id: u32,
    pub product_name: String,
    pub sku: String,
    pub quantity: u32,
    pub unit_price: f64,
    pub discount: f64,
    pub tax_amount: f64,
}

impl OrderItemDto {
    pub fn new(product_id: u32, product_name: String, sku: String, quantity: u32, unit_price: f64) -> Self {
        Self {
            product_id,
            product_name,
            sku,
            quantity,
            unit_price,
            discount: 0.0,
            tax_amount: 0.0,
        }
    }
    
    pub fn calculate_line_total(&self) -> f64 {
        (self.quantity as f64 * self.unit_price) - self.discount + self.tax_amount
    }
    
    pub fn validate(&self) -> Result<(), DtoError> {
        if self.product_name.trim().is_empty() {
            return Err(DtoError::ValidationError("商品名称不能为空".to_string()));
        }
        
        if self.quantity == 0 {
            return Err(DtoError::ValidationError("商品数量必须大于0".to_string()));
        }
        
        if self.unit_price <= 0.0 {
            return Err(DtoError::ValidationError("商品单价必须大于0".to_string()));
        }
        
        Ok(())
    }
    
    pub fn to_json(&self) -> Result<String, DtoError> {
        Ok(format!(
            r#"{{"product_id":{},"product_name":"{}","sku":"{}","quantity":{},"unit_price":{},"discount":{},"tax_amount":{}}}"#,
            self.product_id,
            self.product_name,
            self.sku,
            self.quantity,
            self.unit_price,
            self.discount,
            self.tax_amount
        ))
    }
}

/// 订单摘要DTO
#[derive(Debug, Clone)]
pub struct OrderSummaryDto {
    pub id: Option<u32>,
    pub order_number: String,
    pub order_date: String,
    pub status: String,
    pub total_amount: f64,
    pub currency: String,
    pub items_count: usize,
}

impl OrderSummaryDto {
    pub fn to_json(&self) -> Result<String, DtoError> {
        Ok(format!(
            r#"{{"id":{},"order_number":"{}","order_date":"{}","status":"{}","total_amount":{},"currency":"{}","items_count":{}}}"#,
            match self.id { Some(id) => id.to_string(), None => "null".to_string() },
            self.order_number,
            self.order_date,
            self.status,
            self.total_amount,
            self.currency,
            self.items_count
        ))
    }
}

/// API响应DTO
#[derive(Debug, Clone)]
pub struct ApiResponseDto<T> {
    pub success: bool,
    pub message: String,
    pub data: Option<T>,
    pub errors: Vec<String>,
    pub metadata: ResponseMetadata,
}

impl<T> ApiResponseDto<T> {
    pub fn success(data: T, message: String) -> Self {
        Self {
            success: true,
            message,
            data: Some(data),
            errors: Vec::new(),
            metadata: ResponseMetadata::new(),
        }
    }
    
    pub fn error(message: String, errors: Vec<String>) -> Self {
        Self {
            success: false,
            message,
            data: None,
            errors,
            metadata: ResponseMetadata::new(),
        }
    }
    
    pub fn with_metadata(mut self, metadata: ResponseMetadata) -> Self {
        self.metadata = metadata;
        self
    }
}

/// 响应元数据
#[derive(Debug, Clone)]
pub struct ResponseMetadata {
    pub timestamp: String,
    pub request_id: String,
    pub version: String,
    pub server: String,
}

impl ResponseMetadata {
    pub fn new() -> Self {
        Self {
            timestamp: "2024-01-01T00:00:00Z".to_string(),
            request_id: "req-12345".to_string(),
            version: "v1.0".to_string(),
            server: "api-server-01".to_string(),
        }
    }
}

/// 分页DTO
#[derive(Debug, Clone)]
pub struct PagedResultDto<T> {
    pub data: Vec<T>,
    pub page_number: u32,
    pub page_size: u32,
    pub total_count: u32,
    pub total_pages: u32,
    pub has_previous: bool,
    pub has_next: bool,
}

impl<T> PagedResultDto<T> {
    pub fn new(data: Vec<T>, page_number: u32, page_size: u32, total_count: u32) -> Self {
        let total_pages = (total_count + page_size - 1) / page_size; // 向上取整
        let has_previous = page_number > 1;
        let has_next = page_number < total_pages;
        
        Self {
            data,
            page_number,
            page_size,
            total_count,
            total_pages,
            has_previous,
            has_next,
        }
    }
}

/// DTO映射器 - 领域对象与DTO之间的转换
pub struct DtoMapper;

impl DtoMapper {
    /// 从领域对象创建用户DTO
    pub fn user_to_dto(
        id: Option<u32>,
        username: String,
        email: String,
        full_name: String,
        balance: f64,
        level: String,
        is_active: bool
    ) -> UserDto {
        let mut dto = UserDto::new(username, email, full_name);
        dto.id = id;
        dto.account_balance = balance;
        dto.account_level = level;
        dto.is_active = is_active;
        dto
    }
    
    /// 批量转换用户列表为摘要DTO
    pub fn users_to_summary_list(users: Vec<UserDto>) -> Vec<UserSummaryDto> {
        users.into_iter().map(|u| u.to_summary()).collect()
    }
    
    /// 创建分页用户结果
    pub fn create_paged_users(
        users: Vec<UserSummaryDto>,
        page: u32,
        size: u32,
        total: u32
    ) -> PagedResultDto<UserSummaryDto> {
        PagedResultDto::new(users, page, size, total)
    }
}

/// DTO序列化器（模拟）
pub struct DtoSerializer;

impl DtoSerializer {
    /// 序列化为XML格式（模拟）
    pub fn to_xml<T: std::fmt::Debug>(data: &T, root_name: &str) -> Result<String, DtoError> {
        // 简化的XML序列化（实际应用中会使用专门的XML库）
        Ok(format!("<{}>{:?}</{}>", root_name, data, root_name))
    }
    
    /// 序列化为二进制格式（模拟）
    pub fn to_binary<T: std::fmt::Debug>(data: &T) -> Result<Vec<u8>, DtoError> {
        // 简化的二进制序列化（实际应用中会使用如bincode等库）
        let string_repr = format!("{:?}", data);
        Ok(string_repr.into_bytes())
    }
    
    /// 压缩序列化数据（模拟）
    pub fn compress_data(data: Vec<u8>) -> Result<Vec<u8>, DtoError> {
        // 模拟压缩（实际应用中会使用gzip、deflate等）
        println!("压缩数据: {} 字节 -> {} 字节", data.len(), data.len() / 2);
        Ok(data[..data.len()/2].to_vec()) // 模拟压缩效果
    }
}

/// 性能监控DTO
#[derive(Debug, Clone)]
pub struct PerformanceMetricsDto {
    pub serialization_time_ms: u64,
    pub data_size_bytes: usize,
    pub compression_ratio: f64,
    pub network_transfer_time_ms: u64,
}

impl PerformanceMetricsDto {
    pub fn new() -> Self {
        Self {
            serialization_time_ms: 0,
            data_size_bytes: 0,
            compression_ratio: 1.0,
            network_transfer_time_ms: 0,
        }
    }
    
    pub fn calculate_efficiency(&self) -> f64 {
        // 计算传输效率分数
        let base_score = 100.0;
        let size_penalty = (self.data_size_bytes as f64 / 1024.0) * 0.1; // 每KB扣0.1分
        let time_penalty = (self.serialization_time_ms + self.network_transfer_time_ms) as f64 * 0.01;
        let compression_bonus = (1.0 - self.compression_ratio) * 20.0; // 压缩率奖励
        
        (base_score - size_penalty - time_penalty + compression_bonus).max(0.0)
    }
}

/// Data Transfer Object模式演示
pub fn demo() {
    println!("=== Data Transfer Object（数据传输对象）模式演示 ===\n");
    
    // 1. 创建基本DTO
    println!("1. 创建基本DTO:");
    
    let mut user_dto = UserDto::new(
        "alice123".to_string(),
        "alice@example.com".to_string(),
        "Alice Johnson".to_string(),
    );
    
    user_dto.id = Some(1);
    user_dto.phone = Some("+1234567890".to_string());
    user_dto.account_balance = 1500.75;
    user_dto.account_level = "Gold".to_string();
    user_dto.preferences.insert("language".to_string(), "zh-CN".to_string());
    user_dto.preferences.insert("theme".to_string(), "dark".to_string());
    
    // 添加地址信息
    let address = AddressDto::new(
        "123 Main St".to_string(),
        "New York".to_string(),
        "NY".to_string(),
        "10001".to_string(),
        "USA".to_string(),
    );
    user_dto.address = Some(address);
    
    println!("用户DTO创建完成:");
    println!("  用户名: {}", user_dto.username);
    println!("  邮箱: {}", user_dto.email);
    println!("  姓名: {}", user_dto.full_name);
    println!("  余额: ${:.2}", user_dto.account_balance);
    println!("  等级: {}", user_dto.account_level);
    if let Some(addr) = &user_dto.address {
        println!("  地址: {}", addr.format_full_address());
    }
    
    println!("{}", "=".repeat(50));
    
    // 2. DTO验证
    println!("2. DTO验证:");
    
    match user_dto.validate() {
        Ok(_) => println!("✅ 用户DTO验证通过"),
        Err(e) => println!("❌ 用户DTO验证失败: {}", e),
    }
    
    // 创建无效DTO进行测试
    let invalid_user = UserDto::new(
        "".to_string(), // 空用户名
        "invalid-email".to_string(), // 无效邮箱
        "Test User".to_string(),
    );
    
    match invalid_user.validate() {
        Ok(_) => println!("⚠️ 无效DTO验证通过（不应该发生）"),
        Err(e) => println!("✅ 正确拒绝无效DTO: {}", e),
    }
    
    println!("{}", "=".repeat(50));
    
    // 3. DTO序列化
    println!("3. DTO序列化:");
    
    match user_dto.to_json() {
        Ok(json) => {
            println!("✅ JSON序列化成功:");
            println!("{}", json);
            
            // 计算数据大小
            let size = json.len();
            println!("\n📊 JSON数据大小: {} 字节", size);
        }
        Err(e) => println!("❌ JSON序列化失败: {}", e),
    }
    
    // XML序列化
    match DtoSerializer::to_xml(&user_dto, "user") {
        Ok(xml) => {
            println!("\n✅ XML序列化成功:");
            println!("{}", xml);
        }
        Err(e) => println!("❌ XML序列化失败: {}", e),
    }
    
    println!("{}", "=".repeat(50));
    
    // 4. 创建复杂订单DTO
    println!("4. 创建复杂订单DTO:");
    
    let shipping_address = AddressDto::new(
        "456 Oak Ave".to_string(),
        "Los Angeles".to_string(),
        "CA".to_string(),
        "90210".to_string(),
        "USA".to_string(),
    );
    
    let mut order_dto = OrderDto::new(
        1, // user_id
        "ORD-2024-001".to_string(),
        shipping_address,
    );
    
    // 添加订单项
    let item1 = OrderItemDto::new(
        101,
        "智能手机".to_string(),
        "SKU-PHONE-001".to_string(),
        1,
        999.99,
    );
    
    let mut item2 = OrderItemDto::new(
        102,
        "手机壳".to_string(),
        "SKU-CASE-001".to_string(),
        2,
        29.99,
    );
    item2.discount = 5.00; // 折扣
    item2.tax_amount = 2.40; // 税额
    
    order_dto.add_item(item1);
    order_dto.add_item(item2);
    order_dto.notes = Some("请小心包装".to_string());
    
    println!("订单DTO创建完成:");
    println!("  订单号: {}", order_dto.order_number);
    println!("  总金额: ${:.2}", order_dto.total_amount);
    println!("  商品数量: {}", order_dto.items.len());
    
    for (i, item) in order_dto.items.iter().enumerate() {
        println!("  商品{}: {} x {} = ${:.2}", 
               i + 1, item.product_name, item.quantity, item.calculate_line_total());
    }
    
    // 验证订单
    match order_dto.validate() {
        Ok(_) => println!("✅ 订单DTO验证通过"),
        Err(e) => println!("❌ 订单DTO验证失败: {}", e),
    }
    
    println!("{}", "=".repeat(50));
    
    // 5. DTO转换和优化
    println!("5. DTO转换和优化:");
    
    // 创建用户摘要（用于列表显示）
    let user_summary = user_dto.to_summary();
    println!("用户摘要DTO:");
    match user_summary.to_json() {
        Ok(json) => {
            let full_size = user_dto.to_json().unwrap().len();
            let summary_size = json.len();
            let reduction = ((full_size - summary_size) as f64 / full_size as f64) * 100.0;
            
            println!("  {}", json);
            println!("  📊 数据减少: {}% ({} -> {} 字节)", 
                   reduction as u32, full_size, summary_size);
        }
        Err(e) => println!("❌ 摘要序列化失败: {}", e),
    }
    
    // 创建订单摘要
    let order_summary = order_dto.to_summary();
    println!("\n订单摘要DTO:");
    match order_summary.to_json() {
        Ok(json) => {
            println!("  {}", json);
        }
        Err(e) => println!("❌ 订单摘要序列化失败: {}", e),
    }
    
    println!("{}", "=".repeat(50));
    
    // 6. 分页结果DTO
    println!("6. 分页结果DTO:");
    
    // 创建用户列表
    let users = vec![
        DtoMapper::user_to_dto(Some(1), "alice".to_string(), "alice@example.com".to_string(), 
                              "Alice Johnson".to_string(), 1500.0, "Gold".to_string(), true),
        DtoMapper::user_to_dto(Some(2), "bob".to_string(), "bob@example.com".to_string(), 
                              "Bob Smith".to_string(), 800.0, "Silver".to_string(), true),
        DtoMapper::user_to_dto(Some(3), "charlie".to_string(), "charlie@example.com".to_string(), 
                              "Charlie Brown".to_string(), 300.0, "Bronze".to_string(), false),
    ];
    
    let user_summaries = DtoMapper::users_to_summary_list(users);
    let paged_result = DtoMapper::create_paged_users(user_summaries, 1, 10, 25);
    
    println!("分页结果:");
    println!("  当前页: {}", paged_result.page_number);
    println!("  页面大小: {}", paged_result.page_size);
    println!("  总记录数: {}", paged_result.total_count);
    println!("  总页数: {}", paged_result.total_pages);
    println!("  有上一页: {}", paged_result.has_previous);
    println!("  有下一页: {}", paged_result.has_next);
    println!("  当前页数据: {} 条", paged_result.data.len());
    
    for user in &paged_result.data {
        println!("    - {} ({}) - {}", user.full_name, user.username, user.account_level);
    }
    
    println!("{}", "=".repeat(50));
    
    // 7. API响应包装
    println!("7. API响应包装:");
    
    let api_response = ApiResponseDto::success(
        paged_result,
        "用户列表获取成功".to_string(),
    ).with_metadata(ResponseMetadata::new());
    
    println!("API响应:");
    println!("  成功: {}", api_response.success);
    println!("  消息: {}", api_response.message);
    println!("  时间戳: {}", api_response.metadata.timestamp);
    println!("  请求ID: {}", api_response.metadata.request_id);
    println!("  版本: {}", api_response.metadata.version);
    
    // 错误响应示例
    let error_response: ApiResponseDto<()> = ApiResponseDto::error(
        "用户创建失败".to_string(),
        vec!["用户名已存在".to_string(), "邮箱格式不正确".to_string()],
    );
    
    println!("\n错误响应:");
    println!("  成功: {}", error_response.success);
    println!("  消息: {}", error_response.message);
    println!("  错误列表:");
    for error in &error_response.errors {
        println!("    - {}", error);
    }
    
    println!("{}", "=".repeat(50));
    
    // 8. 性能测试
    println!("8. 性能测试:");
    
    let mut metrics = PerformanceMetricsDto::new();
    
    // 模拟序列化性能测试
    let start_time = std::time::Instant::now();
    let _ = user_dto.to_json();
    metrics.serialization_time_ms = start_time.elapsed().as_millis() as u64;
    
    // 模拟数据大小和压缩
    if let Ok(json_data) = user_dto.to_json() {
        metrics.data_size_bytes = json_data.len();
        
        if let Ok(binary_data) = DtoSerializer::to_binary(&user_dto) {
            if let Ok(compressed_data) = DtoSerializer::compress_data(binary_data.clone()) {
                metrics.compression_ratio = compressed_data.len() as f64 / binary_data.len() as f64;
            }
        }
    }
    
    metrics.network_transfer_time_ms = 45; // 模拟网络传输时间
    
    println!("性能指标:");
    println!("  序列化时间: {} ms", metrics.serialization_time_ms);
    println!("  数据大小: {} 字节", metrics.data_size_bytes);
    println!("  压缩比率: {:.2}", metrics.compression_ratio);
    println!("  网络传输时间: {} ms", metrics.network_transfer_time_ms);
    println!("  传输效率分数: {:.1}", metrics.calculate_efficiency());
    
    println!("\n=== Data Transfer Object模式演示完成 ===");
    
    // 输出模式总结
    println!("\n【Data Transfer Object模式总结】");
    println!("核心特点:");
    println!("1. 数据容器：只包含数据，不包含业务逻辑");
    println!("2. 序列化友好：易于转换为JSON、XML等格式");
    println!("3. 扁平化结构：减少对象层次，提高传输效率");
    println!("4. 版本兼容：支持向前和向后兼容");
    println!("5. 数据验证：确保传输数据的完整性");
    
    println!("\n优势:");
    println!("1. 减少远程调用：一次传输多个数据");
    println!("2. 网络优化：可压缩、可缓存");
    println!("3. 解耦合：客户端和服务端独立演化");
    println!("4. 类型安全：编译时检查数据结构");
    println!("5. 多格式支持：JSON、XML、二进制等");
    
    println!("\n适用场景:");
    println!("1. 分布式系统间的数据传输");
    println!("2. Web API的请求和响应");
    println!("3. 微服务架构中的服务通信");
    println!("4. 移动应用与后端的数据交换");
    println!("5. 第三方集成和数据同步");
    
    println!("\n设计原则:");
    println!("1. 简单性：结构简单，易于理解和使用");
    println!("2. 完整性：包含客户端所需的所有数据");
    println!("3. 最小化：只包含必要的数据，避免冗余");
    println!("4. 稳定性：接口稳定，支持版本演化");
    println!("5. 可测试性：易于创建测试数据和验证");
}