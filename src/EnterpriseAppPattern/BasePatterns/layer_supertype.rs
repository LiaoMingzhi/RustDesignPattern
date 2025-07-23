//! 层超类型模式 (Layer Supertype)
//! 
//! 为某个层的所有类型提供一个共同的超类型
//! 文件路径：/d%3A/workspace/RustLearn/RustDesignPattern/src/EnterpriseAppPattern/BasePatterns/layer_supertype.rs

use std::collections::HashMap;
use std::fmt;
use std::error::Error;

/// 领域对象基类（层超类型）
pub trait DomainObject: fmt::Debug + Clone {
    /// 获取对象ID
    fn get_id(&self) -> Option<u32>;
    
    /// 设置对象ID
    fn set_id(&mut self, id: u32);
    
    /// 验证对象
    fn validate(&self) -> Result<(), Box<dyn Error>>;
    
    /// 获取对象类型名称
    fn get_type_name(&self) -> &'static str;
    
    /// 获取创建时间
    fn get_created_at(&self) -> &str;
    
    /// 获取更新时间
    fn get_updated_at(&self) -> &str;
    
    /// 更新时间戳
    fn touch(&mut self);
    
    /// 检查对象是否为新对象（未保存）
    fn is_new(&self) -> bool {
        self.get_id().is_none()
    }
    
    /// 检查对象是否已保存
    fn is_persisted(&self) -> bool {
        self.get_id().is_some()
    }
}

/// 数据访问对象基类（层超类型）
pub trait DataAccessObject<T: DomainObject> {
    /// 根据ID查找对象
    fn find_by_id(&self, id: u32) -> Result<Option<T>, Box<dyn Error>>;
    
    /// 保存对象
    fn save(&mut self, entity: &mut T) -> Result<(), Box<dyn Error>>;
    
    /// 删除对象
    fn delete(&mut self, id: u32) -> Result<bool, Box<dyn Error>>;
    
    /// 获取所有对象
    fn find_all(&self) -> Result<Vec<T>, Box<dyn Error>>;
    
    /// 根据条件查找
    fn find_by_criteria(&self, criteria: &HashMap<String, String>) -> Result<Vec<T>, Box<dyn Error>>;
    
    /// 统计对象数量
    fn count(&self) -> Result<usize, Box<dyn Error>>;
    
    /// 检查对象是否存在
    fn exists(&self, id: u32) -> Result<bool, Box<dyn Error>> {
        Ok(self.find_by_id(id)?.is_some())
    }
}

/// 业务服务基类（层超类型）
pub trait BusinessService {
    /// 服务名称
    fn get_service_name(&self) -> &'static str;
    
    /// 验证业务规则
    fn validate_business_rules(&self, context: &BusinessContext) -> Result<(), BusinessError>;
    
    /// 记录审计日志
    fn audit_log(&self, action: &str, details: &str) {
        println!("[AUDIT] 服务: {}, 操作: {}, 详情: {}", 
                self.get_service_name(), action, details);
    }
    
    /// 开始事务
    fn begin_transaction(&self) -> TransactionContext {
        println!("[TRANSACTION] 开始事务 - 服务: {}", self.get_service_name());
        TransactionContext::new()
    }
    
    /// 提交事务
    fn commit_transaction(&self, _context: TransactionContext) {
        println!("[TRANSACTION] 提交事务 - 服务: {}", self.get_service_name());
    }
    
    /// 回滚事务
    fn rollback_transaction(&self, _context: TransactionContext) {
        println!("[TRANSACTION] 回滚事务 - 服务: {}", self.get_service_name());
    }
}

/// 业务上下文
#[derive(Debug, Clone)]
pub struct BusinessContext {
    pub user_id: Option<u32>,
    pub session_id: String,
    pub operation: String,
    pub data: HashMap<String, String>,
}

impl BusinessContext {
    pub fn new(session_id: String, operation: String) -> Self {
        Self {
            user_id: None,
            session_id,
            operation,
            data: HashMap::new(),
        }
    }
    
    pub fn with_user(mut self, user_id: u32) -> Self {
        self.user_id = Some(user_id);
        self
    }
    
    pub fn with_data(mut self, key: &str, value: &str) -> Self {
        self.data.insert(key.to_string(), value.to_string());
        self
    }
}

/// 事务上下文
#[derive(Debug)]
pub struct TransactionContext {
    pub transaction_id: String,
    pub start_time: String,
}

impl TransactionContext {
    pub fn new() -> Self {
        Self {
            transaction_id: format!("tx_{}", rand::random::<u32>()),
            start_time: "2024-01-01T10:00:00Z".to_string(), // 简化
        }
    }
}

/// 业务错误
#[derive(Debug)]
pub enum BusinessError {
    ValidationError(String),
    AuthorizationError(String),
    BusinessRuleViolation(String),
    DataIntegrityError(String),
}

impl fmt::Display for BusinessError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BusinessError::ValidationError(msg) => write!(f, "验证错误: {}", msg),
            BusinessError::AuthorizationError(msg) => write!(f, "授权错误: {}", msg),
            BusinessError::BusinessRuleViolation(msg) => write!(f, "业务规则违反: {}", msg),
            BusinessError::DataIntegrityError(msg) => write!(f, "数据完整性错误: {}", msg),
        }
    }
}

impl Error for BusinessError {}

// 具体实现示例

/// 产品领域对象
#[derive(Debug, Clone)]
pub struct Product {
    id: Option<u32>,
    name: String,
    price: f64,
    category: String,
    created_at: String,
    updated_at: String,
}

impl Product {
    pub fn new(name: String, price: f64, category: String) -> Self {
        let now = "2024-01-01T10:00:00Z".to_string(); // 简化
        Self {
            id: None,
            name,
            price,
            category,
            created_at: now.clone(),
            updated_at: now,
        }
    }
    
    pub fn get_name(&self) -> &str {
        &self.name
    }
    
    pub fn get_price(&self) -> f64 {
        self.price
    }
    
    pub fn set_price(&mut self, price: f64) {
        self.price = price;
        self.touch();
    }
    
    pub fn get_category(&self) -> &str {
        &self.category
    }
}

impl DomainObject for Product {
    fn get_id(&self) -> Option<u32> {
        self.id
    }
    
    fn set_id(&mut self, id: u32) {
        self.id = Some(id);
        self.touch();
    }
    
    fn validate(&self) -> Result<(), Box<dyn Error>> {
        if self.name.trim().is_empty() {
            return Err("产品名称不能为空".into());
        }
        if self.price < 0.0 {
            return Err("产品价格不能为负数".into());
        }
        if self.category.trim().is_empty() {
            return Err("产品分类不能为空".into());
        }
        Ok(())
    }
    
    fn get_type_name(&self) -> &'static str {
        "Product"
    }
    
    fn get_created_at(&self) -> &str {
        &self.created_at
    }
    
    fn get_updated_at(&self) -> &str {
        &self.updated_at
    }
    
    fn touch(&mut self) {
        self.updated_at = "2024-01-01T10:30:00Z".to_string(); // 简化
    }
}

/// 订单领域对象
#[derive(Debug, Clone)]
pub struct Order {
    id: Option<u32>,
    customer_id: u32,
    total_amount: f64,
    status: String,
    created_at: String,
    updated_at: String,
}

impl Order {
    pub fn new(customer_id: u32, total_amount: f64) -> Self {
        let now = "2024-01-01T10:00:00Z".to_string(); // 简化
        Self {
            id: None,
            customer_id,
            total_amount,
            status: "pending".to_string(),
            created_at: now.clone(),
            updated_at: now,
        }
    }
    
    pub fn get_customer_id(&self) -> u32 {
        self.customer_id
    }
    
    pub fn get_total_amount(&self) -> f64 {
        self.total_amount
    }
    
    pub fn get_status(&self) -> &str {
        &self.status
    }
    
    pub fn set_status(&mut self, status: String) {
        self.status = status;
        self.touch();
    }
}

impl DomainObject for Order {
    fn get_id(&self) -> Option<u32> {
        self.id
    }
    
    fn set_id(&mut self, id: u32) {
        self.id = Some(id);
        self.touch();
    }
    
    fn validate(&self) -> Result<(), Box<dyn Error>> {
        if self.customer_id == 0 {
            return Err("客户ID不能为0".into());
        }
        if self.total_amount < 0.0 {
            return Err("订单金额不能为负数".into());
        }
        if self.status.trim().is_empty() {
            return Err("订单状态不能为空".into());
        }
        Ok(())
    }
    
    fn get_type_name(&self) -> &'static str {
        "Order"
    }
    
    fn get_created_at(&self) -> &str {
        &self.created_at
    }
    
    fn get_updated_at(&self) -> &str {
        &self.updated_at
    }
    
    fn touch(&mut self) {
        self.updated_at = "2024-01-01T10:30:00Z".to_string(); // 简化
    }
}

/// 产品数据访问对象
pub struct ProductDAO {
    products: HashMap<u32, Product>,
    next_id: u32,
}

impl ProductDAO {
    pub fn new() -> Self {
        Self {
            products: HashMap::new(),
            next_id: 1,
        }
    }
}

impl DataAccessObject<Product> for ProductDAO {
    fn find_by_id(&self, id: u32) -> Result<Option<Product>, Box<dyn Error>> {
        println!("查找产品 ID: {}", id);
        Ok(self.products.get(&id).cloned())
    }
    
    fn save(&mut self, product: &mut Product) -> Result<(), Box<dyn Error>> {
        product.validate()?;
        
        if product.is_new() {
            let id = self.next_id;
            self.next_id += 1;
            product.set_id(id);
            println!("创建新产品: {} (ID: {})", product.get_name(), id);
        } else {
            println!("更新产品: {} (ID: {:?})", product.get_name(), product.get_id());
        }
        
        self.products.insert(product.get_id().unwrap(), product.clone());
        Ok(())
    }
    
    fn delete(&mut self, id: u32) -> Result<bool, Box<dyn Error>> {
        match self.products.remove(&id) {
            Some(_) => {
                println!("删除产品 ID: {}", id);
                Ok(true)
            }
            None => Ok(false)
        }
    }
    
    fn find_all(&self) -> Result<Vec<Product>, Box<dyn Error>> {
        Ok(self.products.values().cloned().collect())
    }
    
    fn find_by_criteria(&self, criteria: &HashMap<String, String>) -> Result<Vec<Product>, Box<dyn Error>> {
        let mut results = Vec::new();
        
        for product in self.products.values() {
            let mut matches = true;
            
            if let Some(category) = criteria.get("category") {
                if product.get_category() != category {
                    matches = false;
                }
            }
            
            if let Some(min_price) = criteria.get("min_price") {
                if let Ok(price) = min_price.parse::<f64>() {
                    if product.get_price() < price {
                        matches = false;
                    }
                }
            }
            
            if matches {
                results.push(product.clone());
            }
        }
        
        Ok(results)
    }
    
    fn count(&self) -> Result<usize, Box<dyn Error>> {
        Ok(self.products.len())
    }
}

/// 产品服务
pub struct ProductService {
    dao: ProductDAO,
}

impl ProductService {
    pub fn new() -> Self {
        Self {
            dao: ProductDAO::new(),
        }
    }
    
    pub fn create_product(&mut self, name: String, price: f64, category: String, context: &BusinessContext) 
        -> Result<Product, Box<dyn Error>> {
        
        self.validate_business_rules(context)?;
        
        let mut product = Product::new(name, price, category);
        
        let tx = self.begin_transaction();
        
        match self.dao.save(&mut product) {
            Ok(_) => {
                self.commit_transaction(tx);
                self.audit_log("CREATE_PRODUCT", &format!("产品: {}", product.get_name()));
                Ok(product)
            }
            Err(e) => {
                self.rollback_transaction(tx);
                Err(e)
            }
        }
    }
    
    pub fn update_product_price(&mut self, id: u32, new_price: f64, context: &BusinessContext) 
        -> Result<(), Box<dyn Error>> {
        
        self.validate_business_rules(context)?;
        
        if let Some(mut product) = self.dao.find_by_id(id)? {
            let old_price = product.get_price();
            product.set_price(new_price);
            
            let tx = self.begin_transaction();
            
            match self.dao.save(&mut product) {
                Ok(_) => {
                    self.commit_transaction(tx);
                    self.audit_log("UPDATE_PRICE", 
                                  &format!("产品: {}, 价格: {} -> {}", 
                                          product.get_name(), old_price, new_price));
                    Ok(())
                }
                Err(e) => {
                    self.rollback_transaction(tx);
                    Err(e)
                }
            }
        } else {
            Err(format!("产品ID {} 不存在", id).into())
        }
    }
    
    pub fn find_products_by_category(&self, category: &str) -> Result<Vec<Product>, Box<dyn Error>> {
        let mut criteria = HashMap::new();
        criteria.insert("category".to_string(), category.to_string());
        self.dao.find_by_criteria(&criteria)
    }
    
    pub fn get_product_statistics(&self) -> Result<ProductStatistics, Box<dyn Error>> {
        let products = self.dao.find_all()?;
        
        if products.is_empty() {
            return Ok(ProductStatistics::default());
        }
        
        let total_count = products.len();
        let total_value: f64 = products.iter().map(|p| p.get_price()).sum();
        let avg_price = total_value / total_count as f64;
        
        let mut category_counts = HashMap::new();
        for product in &products {
            *category_counts.entry(product.get_category().to_string()).or_insert(0) += 1;
        }
        
        Ok(ProductStatistics {
            total_count,
            total_value,
            average_price: avg_price,
            category_distribution: category_counts,
        })
    }
}

impl BusinessService for ProductService {
    fn get_service_name(&self) -> &'static str {
        "ProductService"
    }
    
    fn validate_business_rules(&self, context: &BusinessContext) -> Result<(), BusinessError> {
        // 检查用户权限
        if context.user_id.is_none() {
            return Err(BusinessError::AuthorizationError("用户未登录".to_string()));
        }
        
        // 检查操作权限
        match context.operation.as_str() {
            "CREATE_PRODUCT" | "UPDATE_PRICE" => {
                // 假设需要管理员权限
                if !context.data.get("role").map_or(false, |role| role == "admin") {
                    return Err(BusinessError::AuthorizationError("需要管理员权限".to_string()));
                }
            }
            _ => {}
        }
        
        Ok(())
    }
}

/// 产品统计信息
#[derive(Debug, Default)]
pub struct ProductStatistics {
    pub total_count: usize,
    pub total_value: f64,
    pub average_price: f64,
    pub category_distribution: HashMap<String, u32>,
}

impl fmt::Display for ProductStatistics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "=== 产品统计信息 ===")?;
        writeln!(f, "总产品数: {}", self.total_count)?;
        writeln!(f, "总价值: ¥{:.2}", self.total_value)?;
        writeln!(f, "平均价格: ¥{:.2}", self.average_price)?;
        
        writeln!(f, "\n分类分布:")?;
        for (category, count) in &self.category_distribution {
            writeln!(f, "  {}: {} 个产品", category, count)?;
        }
        
        Ok(())
    }
}

/// 演示层超类型模式
pub fn demo() {
    println!("=== 层超类型模式演示 ===\n");
    
    let mut product_service = ProductService::new();
    
    println!("1. 使用领域对象层超类型:");
    
    // 创建业务上下文
    let context = BusinessContext::new("session_123".to_string(), "CREATE_PRODUCT".to_string())
        .with_user(1)
        .with_data("role", "admin");
    
    // 创建产品
    match product_service.create_product(
        "笔记本电脑".to_string(), 
        5999.0, 
        "电子产品".to_string(), 
        &context
    ) {
        Ok(product) => {
            println!("✅ 产品创建成功:");
            println!("   ID: {:?}", product.get_id());
            println!("   名称: {}", product.get_name());
            println!("   价格: ¥{:.2}", product.get_price());
            println!("   分类: {}", product.get_category());
            println!("   类型: {}", product.get_type_name());
            println!("   是否新对象: {}", product.is_new());
            println!("   是否已保存: {}", product.is_persisted());
        }
        Err(e) => println!("❌ 产品创建失败: {}", e),
    }
    
    println!("\n{}", "=".repeat(50));
    
    // 创建更多产品
    println!("2. 创建更多产品:");
    
    let products_data = vec![
        ("智能手机", 3999.0, "电子产品"),
        ("办公桌", 899.0, "家具"),
        ("咖啡机", 1299.0, "家电"),
        ("编程书籍", 89.0, "书籍"),
    ];
    
    for (name, price, category) in products_data {
        let context = BusinessContext::new("session_123".to_string(), "CREATE_PRODUCT".to_string())
            .with_user(1)
            .with_data("role", "admin");
            
        if let Ok(product) = product_service.create_product(
            name.to_string(), price, category.to_string(), &context
        ) {
            println!("✅ 创建产品: {} (¥{:.2})", product.get_name(), product.get_price());
        }
    }
    
    println!("\n{}", "=".repeat(50));
    
    // 使用数据访问对象层超类型
    println!("3. 使用数据访问对象层超类型:");
    
    // 按分类查找
    match product_service.find_products_by_category("电子产品") {
        Ok(products) => {
            println!("电子产品分类的产品:");
            for product in products {
                println!("  - {} (¥{:.2})", product.get_name(), product.get_price());
            }
        }
        Err(e) => println!("❌ 查找失败: {}", e),
    }
    
    println!("\n{}", "=".repeat(50));
    
    // 使用业务服务层超类型
    println!("4. 使用业务服务层超类型:");
    
    // 更新产品价格
    let update_context = BusinessContext::new("session_123".to_string(), "UPDATE_PRICE".to_string())
        .with_user(1)
        .with_data("role", "admin");
    
    match product_service.update_product_price(1, 5599.0, &update_context) {
        Ok(_) => println!("✅ 产品价格更新成功"),
        Err(e) => println!("❌ 价格更新失败: {}", e),
    }
    
    println!("\n{}", "=".repeat(50));
    
    // 权限验证演示
    println!("5. 权限验证演示:");
    
    // 尝试以普通用户身份创建产品
    let user_context = BusinessContext::new("session_456".to_string(), "CREATE_PRODUCT".to_string())
        .with_user(2)
        .with_data("role", "user");
    
    match product_service.create_product(
        "测试产品".to_string(), 
        100.0, 
        "测试".to_string(), 
        &user_context
    ) {
        Ok(_) => println!("✅ 普通用户创建产品成功"),
        Err(e) => println!("❌ 普通用户创建产品失败: {}", e),
    }
    
    println!("\n{}", "=".repeat(50));
    
    // 统计信息
    println!("6. 统计信息:");
    
    match product_service.get_product_statistics() {
        Ok(stats) => println!("{}", stats),
        Err(e) => println!("❌ 获取统计信息失败: {}", e),
    }
    
    println!("\n{}", "=".repeat(50));
    
    // 对象生命周期演示
    println!("7. 对象生命周期演示:");
    
    let mut new_product = Product::new("新产品".to_string(), 199.0, "测试".to_string());
    println!("新创建的产品:");
    println!("  是否新对象: {}", new_product.is_new());
    println!("  是否已保存: {}", new_product.is_persisted());
    println!("  创建时间: {}", new_product.get_created_at());
    
    // 设置ID（模拟保存）
    new_product.set_id(999);
    println!("\n设置ID后:");
    println!("  是否新对象: {}", new_product.is_new());
    println!("  是否已保存: {}", new_product.is_persisted());
    println!("  更新时间: {}", new_product.get_updated_at());
    
    // 验证对象
    match new_product.validate() {
        Ok(_) => println!("  ✅ 对象验证通过"),
        Err(e) => println!("  ❌ 对象验证失败: {}", e),
    }
    
    println!("\n{}", "=".repeat(50));
    
    println!("层超类型模式的特点:");
    println!("✅ 为同一层的所有类提供共同的行为");
    println!("✅ 减少代码重复，提高一致性");
    println!("✅ 集中处理通用功能（验证、审计、事务）");
    println!("✅ 便于维护和扩展");
    println!("✅ 强制执行层的约定和规范");
    
    println!("\n适用场景:");
    println!("• 多个类需要共同的行为");
    println!("• 需要强制执行层的规范");
    println!("• 希望减少代码重复");
    println!("• 需要集中处理横切关注点");
    println!("• 大型项目的架构标准化");
    
    println!("\n实现要点:");
    println!("• 识别层内的共同行为");
    println!("• 设计合适的抽象接口");
    println!("• 考虑trait vs 具体类的选择");
    println!("• 平衡灵活性和约束性");
    println!("• 提供默认实现减少重复代码");
} 