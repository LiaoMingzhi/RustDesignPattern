//! # 特殊情况模式（Special Case Pattern）
//!
//! 特殊情况模式通过创建专门的类来处理特殊情况，
//! 避免在客户端代码中进行大量的条件检查。
//! 这种模式最常见的应用是空对象模式（Null Object Pattern）。
//!
//! ## 模式特点
//! - **消除条件逻辑**: 减少客户端的if-else检查
//! - **多态行为**: 通过多态提供不同的行为
//! - **一致接口**: 特殊情况对象遵循相同的接口
//! - **简化客户端**: 客户端代码更加简洁
//!
//! ## 使用场景
//! - 处理空值或缺失数据
//! - 默认行为实现
//! - 异常情况的优雅处理
//! - 减少防御性编程代码

use std::collections::HashMap;
use std::fmt::{self, Display, Formatter};
use std::error::Error;

/// 特殊情况模式错误类型
#[derive(Debug)]
pub enum SpecialCaseError {
    InvalidOperation(String),
    ValidationError(String),
    NotSupported(String),
}

impl Display for SpecialCaseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            SpecialCaseError::InvalidOperation(msg) => write!(f, "无效操作: {}", msg),
            SpecialCaseError::ValidationError(msg) => write!(f, "验证错误: {}", msg),
            SpecialCaseError::NotSupported(msg) => write!(f, "不支持的操作: {}", msg),
        }
    }
}

impl Error for SpecialCaseError {}

/// 客户等级
#[derive(Debug, Clone, PartialEq)]
pub enum CustomerTier {
    Bronze,
    Silver,
    Gold,
    Platinum,
    Unknown,
}

/// 客户接口
pub trait Customer: Send + Sync {
    fn get_id(&self) -> &str;
    fn get_name(&self) -> &str;
    fn get_email(&self) -> &str;
    fn get_tier(&self) -> CustomerTier;
    fn get_discount_rate(&self) -> f64;
    fn get_credit_limit(&self) -> f64;
    fn can_purchase(&self, amount: f64) -> bool;
    fn apply_discount(&self, original_price: f64) -> f64;
    fn get_welcome_message(&self) -> String;
    fn is_special_case(&self) -> bool;
}

/// 常规客户实现
#[derive(Debug, Clone)]
pub struct RegularCustomer {
    pub id: String,
    pub name: String,
    pub email: String,
    pub tier: CustomerTier,
    pub credit_limit: f64,
    pub total_spent: f64,
}

impl RegularCustomer {
    pub fn new(id: String, name: String, email: String, tier: CustomerTier, credit_limit: f64) -> Self {
        Self {
            id,
            name,
            email,
            tier,
            credit_limit,
            total_spent: 0.0,
        }
    }

    pub fn add_purchase(&mut self, amount: f64) {
        self.total_spent += amount;
    }
}

impl Customer for RegularCustomer {
    fn get_id(&self) -> &str {
        &self.id
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn get_email(&self) -> &str {
        &self.email
    }

    fn get_tier(&self) -> CustomerTier {
        self.tier.clone()
    }

    fn get_discount_rate(&self) -> f64 {
        match self.tier {
            CustomerTier::Bronze => 0.02,      // 2%
            CustomerTier::Silver => 0.05,      // 5%
            CustomerTier::Gold => 0.10,        // 10%
            CustomerTier::Platinum => 0.15,    // 15%
            CustomerTier::Unknown => 0.0,
        }
    }

    fn get_credit_limit(&self) -> f64 {
        self.credit_limit
    }

    fn can_purchase(&self, amount: f64) -> bool {
        amount <= self.credit_limit
    }

    fn apply_discount(&self, original_price: f64) -> f64 {
        original_price * (1.0 - self.get_discount_rate())
    }

    fn get_welcome_message(&self) -> String {
        match self.tier {
            CustomerTier::Bronze => format!("欢迎回来，{}！感谢您的信任。", self.name),
            CustomerTier::Silver => format!("亲爱的{}，欢迎回来！您享有5%折扣。", self.name),
            CustomerTier::Gold => format!("尊贵的{}，欢迎回来！您享有10%折扣和优先服务。", self.name),
            CustomerTier::Platinum => format!("至尊{}，欢迎回来！您享有15%折扣和专属服务。", self.name),
            CustomerTier::Unknown => "欢迎访问我们的商店！".to_string(),
        }
    }

    fn is_special_case(&self) -> bool {
        false
    }
}

/// 空客户（特殊情况）- 处理未登录或无效客户
#[derive(Debug)]
pub struct NullCustomer;

impl Customer for NullCustomer {
    fn get_id(&self) -> &str {
        "GUEST"
    }

    fn get_name(&self) -> &str {
        "游客"
    }

    fn get_email(&self) -> &str {
        "guest@example.com"
    }

    fn get_tier(&self) -> CustomerTier {
        CustomerTier::Unknown
    }

    fn get_discount_rate(&self) -> f64 {
        0.0 // 游客无折扣
    }

    fn get_credit_limit(&self) -> f64 {
        0.0 // 游客无信用额度
    }

    fn can_purchase(&self, _amount: f64) -> bool {
        false // 游客无法购买
    }

    fn apply_discount(&self, original_price: f64) -> f64 {
        original_price // 无折扣
    }

    fn get_welcome_message(&self) -> String {
        "欢迎访问！请登录以享受更多服务。".to_string()
    }

    fn is_special_case(&self) -> bool {
        true
    }
}

/// 封禁客户（特殊情况）- 处理被封禁的客户
#[derive(Debug)]
pub struct BannedCustomer {
    pub id: String,
    pub name: String,
    pub ban_reason: String,
}

impl BannedCustomer {
    pub fn new(id: String, name: String, ban_reason: String) -> Self {
        Self { id, name, ban_reason }
    }
}

impl Customer for BannedCustomer {
    fn get_id(&self) -> &str {
        &self.id
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn get_email(&self) -> &str {
        "banned@example.com"
    }

    fn get_tier(&self) -> CustomerTier {
        CustomerTier::Unknown
    }

    fn get_discount_rate(&self) -> f64 {
        0.0
    }

    fn get_credit_limit(&self) -> f64 {
        0.0
    }

    fn can_purchase(&self, _amount: f64) -> bool {
        false // 被封禁客户无法购买
    }

    fn apply_discount(&self, original_price: f64) -> f64 {
        original_price // 无折扣
    }

    fn get_welcome_message(&self) -> String {
        format!("抱歉，{}，您的账户已被暂停。原因: {}", self.name, self.ban_reason)
    }

    fn is_special_case(&self) -> bool {
        true
    }
}

/// 测试客户（特殊情况）- 处理测试环境中的客户
#[derive(Debug)]
pub struct TestCustomer {
    pub id: String,
}

impl TestCustomer {
    pub fn new(id: String) -> Self {
        Self { id }
    }
}

impl Customer for TestCustomer {
    fn get_id(&self) -> &str {
        &self.id
    }

    fn get_name(&self) -> &str {
        "测试用户"
    }

    fn get_email(&self) -> &str {
        "test@example.com"
    }

    fn get_tier(&self) -> CustomerTier {
        CustomerTier::Platinum // 测试用户给最高等级
    }

    fn get_discount_rate(&self) -> f64 {
        0.99 // 99%折扣用于测试
    }

    fn get_credit_limit(&self) -> f64 {
        1_000_000.0 // 高信用额度用于测试
    }

    fn can_purchase(&self, _amount: f64) -> bool {
        true // 测试用户可以购买任何金额
    }

    fn apply_discount(&self, original_price: f64) -> f64 {
        original_price * 0.01 // 仅付1%的价格
    }

    fn get_welcome_message(&self) -> String {
        "欢迎，测试用户！这是测试环境。".to_string()
    }

    fn is_special_case(&self) -> bool {
        true
    }
}

/// 客户工厂 - 创建客户对象，包括特殊情况
pub struct CustomerFactory;

impl CustomerFactory {
    /// 根据客户ID创建客户对象
    pub fn create_customer(customer_id: Option<&str>) -> Box<dyn Customer> {
        match customer_id {
            None => Box::new(NullCustomer),
            Some(id) => {
                if id.is_empty() {
                    Box::new(NullCustomer)
                } else if id.starts_with("BANNED_") {
                    Box::new(BannedCustomer::new(
                        id.to_string(),
                        "被封禁用户".to_string(),
                        "违规操作".to_string(),
                    ))
                } else if id.starts_with("TEST_") {
                    Box::new(TestCustomer::new(id.to_string()))
                } else {
                    // 创建常规客户（简化的实现）
                    Box::new(RegularCustomer::new(
                        id.to_string(),
                        format!("客户{}", id),
                        format!("{}@example.com", id),
                        CustomerTier::Silver,
                        5000.0,
                    ))
                }
            }
        }
    }

    /// 从数据库加载客户（模拟）
    pub fn load_from_database(customer_id: &str) -> Box<dyn Customer> {
        // 模拟数据库查询失败或用户不存在的情况
        if customer_id == "NOT_FOUND" || customer_id == "ERROR" {
            return Box::new(NullCustomer);
        }

        Self::create_customer(Some(customer_id))
    }
}

/// 购物车项目
#[derive(Debug, Clone)]
pub struct CartItem {
    pub product_id: String,
    pub name: String,
    pub price: f64,
    pub quantity: i32,
}

impl CartItem {
    pub fn new(product_id: String, name: String, price: f64, quantity: i32) -> Self {
        Self {
            product_id,
            name,
            price,
            quantity,
        }
    }

    pub fn total_price(&self) -> f64 {
        self.price * self.quantity as f64
    }
}

/// 购物车服务 - 使用特殊情况模式的客户端
pub struct ShoppingCartService {
    pub items: Vec<CartItem>,
}

impl ShoppingCartService {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
        }
    }

    pub fn add_item(&mut self, item: CartItem) {
        self.items.push(item);
    }

    pub fn calculate_total(&self) -> f64 {
        self.items.iter().map(|item| item.total_price()).sum()
    }

    /// 结账处理 - 客户端不需要检查特殊情况
    pub fn checkout(&self, customer: &dyn Customer) -> Result<CheckoutResult, SpecialCaseError> {
        let total = self.calculate_total();
        
        if total == 0.0 {
            return Err(SpecialCaseError::ValidationError("购物车为空".to_string()));
        }

        // 应用折扣（不需要检查客户是否为空或特殊情况）
        let discounted_total = customer.apply_discount(total);
        
        // 检查购买能力（特殊情况客户自己处理逻辑）
        if !customer.can_purchase(discounted_total) {
            return Ok(CheckoutResult {
                success: false,
                total_amount: total,
                final_amount: discounted_total,
                discount_applied: total - discounted_total,
                message: "购买失败：余额不足或无购买权限".to_string(),
                customer_message: customer.get_welcome_message(),
            });
        }

        Ok(CheckoutResult {
            success: true,
            total_amount: total,
            final_amount: discounted_total,
            discount_applied: total - discounted_total,
            message: "购买成功！".to_string(),
            customer_message: customer.get_welcome_message(),
        })
    }

    /// 显示客户信息（演示统一处理）
    pub fn display_customer_info(&self, customer: &dyn Customer) {
        println!("🛍️  客户信息:");
        println!("   ID: {}", customer.get_id());
        println!("   姓名: {}", customer.get_name());
        println!("   邮箱: {}", customer.get_email());
        println!("   等级: {:?}", customer.get_tier());
        println!("   折扣率: {:.1}%", customer.get_discount_rate() * 100.0);
        println!("   信用额度: ¥{:.2}", customer.get_credit_limit());
        println!("   特殊情况: {}", if customer.is_special_case() { "是" } else { "否" });
        println!("   欢迎信息: {}", customer.get_welcome_message());
    }
}

/// 结账结果
#[derive(Debug)]
pub struct CheckoutResult {
    pub success: bool,
    pub total_amount: f64,
    pub final_amount: f64,
    pub discount_applied: f64,
    pub message: String,
    pub customer_message: String,
}

impl Display for CheckoutResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "结账结果:")?;
        writeln!(f, "  状态: {}", if self.success { "成功" } else { "失败" })?;
        writeln!(f, "  原价: ¥{:.2}", self.total_amount)?;
        writeln!(f, "  折扣: ¥{:.2}", self.discount_applied)?;
        writeln!(f, "  实付: ¥{:.2}", self.final_amount)?;
        writeln!(f, "  消息: {}", self.message)?;
        write!(f, "  客户消息: {}", self.customer_message)
    }
}

/// 客户报告服务 - 演示特殊情况处理
pub struct CustomerReportService;

impl CustomerReportService {
    /// 生成客户报告（无需特殊情况检查）
    pub fn generate_report(&self, customers: &[Box<dyn Customer>]) -> CustomerReport {
        let mut report = CustomerReport::new();
        
        for customer in customers {
            let customer_data = CustomerData {
                id: customer.get_id().to_string(),
                name: customer.get_name().to_string(),
                tier: customer.get_tier(),
                discount_rate: customer.get_discount_rate(),
                credit_limit: customer.get_credit_limit(),
                is_special_case: customer.is_special_case(),
            };
            
            report.add_customer(customer_data);
        }
        
        report
    }
}

/// 客户数据
#[derive(Debug, Clone)]
pub struct CustomerData {
    pub id: String,
    pub name: String,
    pub tier: CustomerTier,
    pub discount_rate: f64,
    pub credit_limit: f64,
    pub is_special_case: bool,
}

/// 客户报告
#[derive(Debug)]
pub struct CustomerReport {
    pub customers: Vec<CustomerData>,
    pub summary: ReportSummary,
}

#[derive(Debug)]
pub struct ReportSummary {
    pub total_customers: usize,
    pub regular_customers: usize,
    pub special_cases: usize,
    pub total_credit_limit: f64,
    pub average_discount_rate: f64,
}

impl CustomerReport {
    pub fn new() -> Self {
        Self {
            customers: Vec::new(),
            summary: ReportSummary {
                total_customers: 0,
                regular_customers: 0,
                special_cases: 0,
                total_credit_limit: 0.0,
                average_discount_rate: 0.0,
            },
        }
    }

    pub fn add_customer(&mut self, customer: CustomerData) {
        if customer.is_special_case {
            self.summary.special_cases += 1;
        } else {
            self.summary.regular_customers += 1;
        }
        
        self.summary.total_credit_limit += customer.credit_limit;
        self.customers.push(customer);
        self.update_summary();
    }

    fn update_summary(&mut self) {
        self.summary.total_customers = self.customers.len();
        
        if !self.customers.is_empty() {
            self.summary.average_discount_rate = self.customers.iter()
                .map(|c| c.discount_rate)
                .sum::<f64>() / self.customers.len() as f64;
        }
    }
}

impl Display for CustomerReport {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "📊 客户报告")?;
        writeln!(f, "====================")?;
        writeln!(f, "总客户数: {}", self.summary.total_customers)?;
        writeln!(f, "常规客户: {}", self.summary.regular_customers)?;
        writeln!(f, "特殊情况: {}", self.summary.special_cases)?;
        writeln!(f, "总信用额度: ¥{:.2}", self.summary.total_credit_limit)?;
        writeln!(f, "平均折扣率: {:.2}%", self.summary.average_discount_rate * 100.0)?;
        writeln!(f, "")?;
        writeln!(f, "客户详情:")?;
        
        for customer in &self.customers {
            writeln!(f, "  {} - {} ({:?}) - 特殊情况: {}", 
                     customer.id, customer.name, customer.tier, customer.is_special_case)?;
        }
        
        Ok(())
    }
}

/// 演示特殊情况模式
pub fn demo() {
    println!("=== 特殊情况模式演示 ===\n");

    // 创建购物车服务
    let mut cart = ShoppingCartService::new();
    cart.add_item(CartItem::new("P001".to_string(), "笔记本电脑".to_string(), 5999.0, 1));
    cart.add_item(CartItem::new("P002".to_string(), "无线鼠标".to_string(), 299.0, 2));
    
    println!("🛒 购物车总额: ¥{:.2}\n", cart.calculate_total());

    println!("1. 演示不同类型的客户（包括特殊情况）");
    
    // 创建不同类型的客户
    let customers = vec![
        ("常规客户", CustomerFactory::create_customer(Some("CUST001"))),
        ("游客（空客户）", CustomerFactory::create_customer(None)),
        ("被封禁客户", CustomerFactory::create_customer(Some("BANNED_001"))),
        ("测试客户", CustomerFactory::create_customer(Some("TEST_001"))),
        ("数据库查询失败", CustomerFactory::load_from_database("NOT_FOUND")),
    ];

    for (label, customer) in &customers {
        println!("\n   📋 {}", label);
        cart.display_customer_info(customer.as_ref());
        
        // 尝试结账（客户端代码无需条件检查）
        match cart.checkout(customer.as_ref()) {
            Ok(result) => {
                println!("   💳 结账结果:");
                for line in result.to_string().lines() {
                    println!("      {}", line);
                }
            }
            Err(e) => println!("   ❌ 结账错误: {}", e),
        }
        
        println!("   {}", "-".repeat(50));
    }

    // 演示特殊情况在报告生成中的处理
    println!("\n2. 生成客户报告（自动处理所有情况）");
    
    let report_service = CustomerReportService;
    let report = report_service.generate_report(&customers.into_iter().map(|(_, c)| c).collect::<Vec<_>>());
    
    println!("{}", report);

    // 演示处理空值和异常情况的优雅性
    println!("\n3. 演示客户端代码的简洁性");
    
    let test_scenarios = vec![
        ("存在的客户", Some("REGULAR_CUSTOMER")),
        ("不存在的客户", None),
        ("空字符串", Some("")),
        ("被封禁客户", Some("BANNED_USER")),
        ("测试用户", Some("TEST_USER")),
    ];

    for (scenario, customer_id) in test_scenarios {
        let customer = CustomerFactory::create_customer(customer_id);
        
        println!("\n   🎯 场景: {}", scenario);
        println!("     客户ID: {:?}", customer_id);
        println!("     欢迎信息: {}", customer.get_welcome_message());
        println!("     可以购买 ¥100: {}", customer.can_purchase(100.0));
        println!("     ¥100 打折后: ¥{:.2}", customer.apply_discount(100.0));
    }

    // 演示特殊情况模式的多态性
    println!("\n4. 演示多态行为");
    
    let special_customers: Vec<Box<dyn Customer>> = vec![
        Box::new(NullCustomer),
        Box::new(BannedCustomer::new("B001".to_string(), "违规用户".to_string(), "恶意刷单".to_string())),
        Box::new(TestCustomer::new("T001".to_string())),
        Box::new(RegularCustomer::new("R001".to_string(), "正常用户".to_string(), "user@example.com".to_string(), CustomerTier::Gold, 10000.0)),
    ];

    for customer in &special_customers {
        println!("   📄 {} ({}): {}", 
                 customer.get_name(),
                 if customer.is_special_case() { "特殊情况" } else { "常规客户" },
                 customer.get_welcome_message());
    }

    println!("\n=== 特殊情况模式演示完成 ===");

    println!("\n💡 特殊情况模式的优势:");
    println!("1. 消除条件逻辑 - 客户端无需检查null或特殊状态");
    println!("2. 多态行为 - 通过多态提供不同的处理逻辑");
    println!("3. 一致接口 - 所有对象都遵循相同的接口");
    println!("4. 简化客户端 - 客户端代码更加简洁清晰");

    println!("\n⚠️ 设计考虑:");
    println!("1. 接口设计 - 需要设计好通用的接口");
    println!("2. 特殊情况识别 - 正确识别需要特殊处理的情况");
    println!("3. 行为定义 - 特殊情况的行为要合理且一致");
    println!("4. 测试覆盖 - 确保所有特殊情况都有适当的测试");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_null_customer() {
        let customer = NullCustomer;
        assert_eq!(customer.get_id(), "GUEST");
        assert_eq!(customer.get_name(), "游客");
        assert_eq!(customer.get_discount_rate(), 0.0);
        assert!(!customer.can_purchase(100.0));
        assert!(customer.is_special_case());
    }

    #[test]
    fn test_regular_customer() {
        let customer = RegularCustomer::new(
            "TEST001".to_string(),
            "测试客户".to_string(),
            "test@example.com".to_string(),
            CustomerTier::Gold,
            5000.0,
        );
        
        assert_eq!(customer.get_id(), "TEST001");
        assert_eq!(customer.get_discount_rate(), 0.10);
        assert!(customer.can_purchase(3000.0));
        assert!(!customer.is_special_case());
    }

    #[test]
    fn test_banned_customer() {
        let customer = BannedCustomer::new(
            "BANNED001".to_string(),
            "被封禁用户".to_string(),
            "违规操作".to_string(),
        );
        
        assert!(!customer.can_purchase(1.0));
        assert!(customer.is_special_case());
        assert!(customer.get_welcome_message().contains("暂停"));
    }

    #[test]
    fn test_customer_factory() {
        let null_customer = CustomerFactory::create_customer(None);
        assert!(null_customer.is_special_case());
        
        let banned_customer = CustomerFactory::create_customer(Some("BANNED_001"));
        assert!(banned_customer.is_special_case());
        
        let test_customer = CustomerFactory::create_customer(Some("TEST_001"));
        assert!(test_customer.is_special_case());
        
        let regular_customer = CustomerFactory::create_customer(Some("REGULAR_001"));
        assert!(!regular_customer.is_special_case());
    }

    #[test]
    fn test_shopping_cart_checkout() {
        let mut cart = ShoppingCartService::new();
        cart.add_item(CartItem::new("P001".to_string(), "产品".to_string(), 100.0, 1));
        
        // 测试常规客户
        let regular_customer = RegularCustomer::new(
            "R001".to_string(),
            "客户".to_string(),
            "test@example.com".to_string(),
            CustomerTier::Silver,
            1000.0,
        );
        
        let result = cart.checkout(&regular_customer).unwrap();
        assert!(result.success);
        assert_eq!(result.final_amount, 95.0); // 5% 折扣
        
        // 测试空客户
        let null_customer = NullCustomer;
        let result = cart.checkout(&null_customer).unwrap();
        assert!(!result.success);
    }

    #[test]
    fn test_customer_report() {
        let customers: Vec<Box<dyn Customer>> = vec![
            Box::new(RegularCustomer::new("R001".to_string(), "客户1".to_string(), "r1@example.com".to_string(), CustomerTier::Gold, 5000.0)),
            Box::new(NullCustomer),
            Box::new(TestCustomer::new("T001".to_string())),
        ];
        
        let report_service = CustomerReportService;
        let report = report_service.generate_report(&customers);
        
        assert_eq!(report.summary.total_customers, 3);
        assert_eq!(report.summary.regular_customers, 1);
        assert_eq!(report.summary.special_cases, 2);
    }
} 