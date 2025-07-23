/*
 * 文件路径: /d%3A/workspace/RustLearn/RustDesignPattern/src/EnterpriseAppPattern/BasePatterns/value_object.rs
 * 
 * Value Object（值对象）模式
 * 
 * 定义：
 * 值对象是一个小而简单的对象，像金钱或日期范围那样，其相等性不是基于身份，而是基于值的相等性。
 * 值对象应该是不可变的，一旦创建就不能改变其状态。
 * 
 * 主要特点：
 * 1. 不可变性：一旦创建，状态不能被修改
 * 2. 值语义：相等性基于值而不是引用身份
 * 3. 没有身份：两个具有相同值的值对象被认为是相等的
 * 4. 替换性：可以用具有相同值的另一个实例来替换
 * 5. 副作用自由：操作不会产生副作用
 * 
 * 适用场景：
 * - 表示度量、数量或描述性的值时
 * - 需要确保数据完整性和不变性时
 * - 需要进行值比较而不是引用比较时
 * - 作为实体对象的属性时
 */

use std::fmt::{Display, Formatter};
use std::ops::{Add, Sub, Mul, Div};
use std::error::Error;

/// 值对象错误类型
#[derive(Debug)]
pub enum ValueObjectError {
    InvalidValue(String),
    ArithmeticError(String),
    ConversionError(String),
}

impl Display for ValueObjectError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ValueObjectError::InvalidValue(msg) => write!(f, "无效值: {}", msg),
            ValueObjectError::ArithmeticError(msg) => write!(f, "算术错误: {}", msg),
            ValueObjectError::ConversionError(msg) => write!(f, "转换错误: {}", msg),
        }
    }
}

impl Error for ValueObjectError {}

/// 货币枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Currency {
    CNY,  // 人民币
    USD,  // 美元
    EUR,  // 欧元
    JPY,  // 日元
    GBP,  // 英镑
}

impl Display for Currency {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Currency::CNY => write!(f, "CNY"),
            Currency::USD => write!(f, "USD"),
            Currency::EUR => write!(f, "EUR"),
            Currency::JPY => write!(f, "JPY"),
            Currency::GBP => write!(f, "GBP"),
        }
    }
}

impl Currency {
    /// 获取货币符号
    fn symbol(&self) -> &'static str {
        match self {
            Currency::CNY => "¥",
            Currency::USD => "$",
            Currency::EUR => "€",
            Currency::JPY => "¥",
            Currency::GBP => "£",
        }
    }
    
    /// 获取最小单位（小数位数）
    fn decimal_places(&self) -> u8 {
        match self {
            Currency::CNY => 2,
            Currency::USD => 2,
            Currency::EUR => 2,
            Currency::JPY => 0,  // 日元没有小数
            Currency::GBP => 2,
        }
    }
}

/// Money值对象 - 表示货币金额
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Money {
    amount: i64,     // 使用整数存储，避免浮点数精度问题（以最小单位存储）
    currency: Currency,
}

impl Money {
    /// 创建新的Money实例
    pub fn new(amount: f64, currency: Currency) -> Result<Self, ValueObjectError> {
        if amount < 0.0 {
            return Err(ValueObjectError::InvalidValue("金额不能为负数".to_string()));
        }
        
        // 转换为最小单位的整数存储
        let decimal_places = currency.decimal_places();
        let multiplier = 10_i64.pow(decimal_places as u32);
        let amount_in_cents = (amount * multiplier as f64).round() as i64;
        
        Ok(Self {
            amount: amount_in_cents,
            currency,
        })
    }
    
    /// 获取金额（转换为浮点数）
    pub fn amount(&self) -> f64 {
        let decimal_places = self.currency.decimal_places();
        let divisor = 10_f64.powi(decimal_places as i32);
        self.amount as f64 / divisor
    }
    
    /// 获取货币类型
    pub fn currency(&self) -> Currency {
        self.currency
    }
    
    /// 检查是否为零
    pub fn is_zero(&self) -> bool {
        self.amount == 0
    }
    
    /// 检查是否为正数
    pub fn is_positive(&self) -> bool {
        self.amount > 0
    }
    
    /// 分配金额（用于分摊费用等场景）
    pub fn allocate(&self, ratios: &[u32]) -> Result<Vec<Money>, ValueObjectError> {
        if ratios.is_empty() {
            return Err(ValueObjectError::InvalidValue("分配比例不能为空".to_string()));
        }
        
        let total_ratio: u32 = ratios.iter().sum();
        if total_ratio == 0 {
            return Err(ValueObjectError::InvalidValue("分配比例总和不能为零".to_string()));
        }
        
        let mut results = Vec::new();
        let mut remainder = self.amount;
        
        for (i, &ratio) in ratios.iter().enumerate() {
            let allocated = if i == ratios.len() - 1 {
                // 最后一个分配剩余的所有金额，避免精度问题
                remainder
            } else {
                (self.amount * ratio as i64) / total_ratio as i64
            };
            
            results.push(Money {
                amount: allocated,
                currency: self.currency,
            });
            remainder -= allocated;
        }
        
        Ok(results)
    }
    
    /// 货币转换（简化实现，实际应该使用实时汇率）
    pub fn convert_to(&self, target_currency: Currency) -> Result<Money, ValueObjectError> {
        if self.currency == target_currency {
            return Ok(*self);
        }
        
        // 简化的汇率表（实际应该从外部服务获取）
        let rate = match (self.currency, target_currency) {
            (Currency::CNY, Currency::USD) => 0.14,
            (Currency::USD, Currency::CNY) => 7.2,
            (Currency::CNY, Currency::EUR) => 0.13,
            (Currency::EUR, Currency::CNY) => 7.8,
            (Currency::USD, Currency::EUR) => 0.85,
            (Currency::EUR, Currency::USD) => 1.18,
            _ => return Err(ValueObjectError::ConversionError(
                format!("不支持从{}到{}的转换", self.currency, target_currency)
            )),
        };
        
        let converted_amount = self.amount() * rate;
        Money::new(converted_amount, target_currency)
    }
}

impl Display for Money {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{:.2}", self.currency.symbol(), self.amount())
    }
}

impl Add for Money {
    type Output = Result<Money, ValueObjectError>;
    
    fn add(self, other: Money) -> Self::Output {
        if self.currency != other.currency {
            return Err(ValueObjectError::ArithmeticError(
                "不能直接相加不同货币的金额".to_string()
            ));
        }
        
        Ok(Money {
            amount: self.amount + other.amount,
            currency: self.currency,
        })
    }
}

impl Sub for Money {
    type Output = Result<Money, ValueObjectError>;
    
    fn sub(self, other: Money) -> Self::Output {
        if self.currency != other.currency {
            return Err(ValueObjectError::ArithmeticError(
                "不能直接相减不同货币的金额".to_string()
            ));
        }
        
        if self.amount < other.amount {
            return Err(ValueObjectError::ArithmeticError(
                "结果不能为负数".to_string()
            ));
        }
        
        Ok(Money {
            amount: self.amount - other.amount,
            currency: self.currency,
        })
    }
}

impl Mul<f64> for Money {
    type Output = Result<Money, ValueObjectError>;
    
    fn mul(self, multiplier: f64) -> Self::Output {
        if multiplier < 0.0 {
            return Err(ValueObjectError::ArithmeticError(
                "乘数不能为负数".to_string()
            ));
        }
        
        let new_amount = (self.amount as f64 * multiplier).round() as i64;
        Ok(Money {
            amount: new_amount,
            currency: self.currency,
        })
    }
}

/// DateRange值对象 - 表示日期范围
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DateRange {
    start_date: u32,  // 简化表示，使用天数（从某个基准日期开始）
    end_date: u32,
}

impl DateRange {
    /// 创建新的日期范围
    pub fn new(start_date: u32, end_date: u32) -> Result<Self, ValueObjectError> {
        if start_date > end_date {
            return Err(ValueObjectError::InvalidValue(
                "开始日期不能晚于结束日期".to_string()
            ));
        }
        
        Ok(Self { start_date, end_date })
    }
    
    /// 获取开始日期
    pub fn start_date(&self) -> u32 {
        self.start_date
    }
    
    /// 获取结束日期
    pub fn end_date(&self) -> u32 {
        self.end_date
    }
    
    /// 获取天数
    pub fn days(&self) -> u32 {
        self.end_date - self.start_date + 1
    }
    
    /// 检查是否包含指定日期
    pub fn contains(&self, date: u32) -> bool {
        date >= self.start_date && date <= self.end_date
    }
    
    /// 检查是否与另一个日期范围重叠
    pub fn overlaps_with(&self, other: &DateRange) -> bool {
        self.start_date <= other.end_date && self.end_date >= other.start_date
    }
    
    /// 获取与另一个日期范围的交集
    pub fn intersection(&self, other: &DateRange) -> Option<DateRange> {
        if !self.overlaps_with(other) {
            return None;
        }
        
        let start = self.start_date.max(other.start_date);
        let end = self.end_date.min(other.end_date);
        
        DateRange::new(start, end).ok()
    }
    
    /// 扩展日期范围
    pub fn extend(&self, days: u32) -> DateRange {
        DateRange {
            start_date: self.start_date,
            end_date: self.end_date + days,
        }
    }
}

impl Display for DateRange {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "日期范围: 第{}天 到 第{}天 ({} 天)", 
               self.start_date, self.end_date, self.days())
    }
}

/// EmailAddress值对象 - 表示电子邮件地址
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EmailAddress {
    email: String,
}

impl EmailAddress {
    /// 创建新的邮箱地址
    pub fn new(email: String) -> Result<Self, ValueObjectError> {
        if !Self::is_valid_email(&email) {
            return Err(ValueObjectError::InvalidValue(
                format!("无效的邮箱地址: {}", email)
            ));
        }
        
        Ok(Self {
            email: email.to_lowercase(), // 标准化为小写
        })
    }
    
    /// 验证邮箱格式
    fn is_valid_email(email: &str) -> bool {
        // 简化的邮箱验证
        email.contains('@') && 
        email.len() > 3 && 
        !email.starts_with('@') && 
        !email.ends_with('@') &&
        email.chars().all(|c| c.is_ascii())
    }
    
    /// 获取邮箱地址
    pub fn as_str(&self) -> &str {
        &self.email
    }
    
    /// 获取域名部分
    pub fn domain(&self) -> &str {
        self.email.split('@').nth(1).unwrap_or("")
    }
    
    /// 获取用户名部分
    pub fn username(&self) -> &str {
        self.email.split('@').next().unwrap_or("")
    }
    
    /// 检查是否为指定域名
    pub fn is_domain(&self, domain: &str) -> bool {
        self.domain().eq_ignore_ascii_case(domain)
    }
}

impl Display for EmailAddress {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.email)
    }
}

/// 产品规格值对象
#[derive(Debug, Clone, PartialEq)]
pub struct ProductSpecification {
    name: String,
    version: String,
    features: Vec<String>,
}

impl ProductSpecification {
    pub fn new(name: String, version: String, features: Vec<String>) -> Result<Self, ValueObjectError> {
        if name.trim().is_empty() {
            return Err(ValueObjectError::InvalidValue("产品名称不能为空".to_string()));
        }
        
        if version.trim().is_empty() {
            return Err(ValueObjectError::InvalidValue("版本号不能为空".to_string()));
        }
        
        Ok(Self { name, version, features })
    }
    
    pub fn name(&self) -> &str {
        &self.name
    }
    
    pub fn version(&self) -> &str {
        &self.version
    }
    
    pub fn features(&self) -> &[String] {
        &self.features
    }
    
    pub fn has_feature(&self, feature: &str) -> bool {
        self.features.iter().any(|f| f.eq_ignore_ascii_case(feature))
    }
    
    /// 创建升级版本
    pub fn upgrade(&self, new_version: String, additional_features: Vec<String>) -> Result<Self, ValueObjectError> {
        let mut new_features = self.features.clone();
        new_features.extend(additional_features);
        
        Self::new(self.name.clone(), new_version, new_features)
    }
}

impl Display for ProductSpecification {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} v{} [特性: {}]", 
               self.name, self.version, self.features.join(", "))
    }
}

/// 值对象演示
pub fn demo() {
    println!("=== Value Object（值对象）模式演示 ===\n");
    
    // 1. Money值对象演示
    println!("1. Money值对象演示:");
    
    // 创建货币对象
    let price1 = Money::new(100.50, Currency::CNY).unwrap();
    let price2 = Money::new(50.25, Currency::CNY).unwrap();
    let usd_price = Money::new(15.0, Currency::USD).unwrap();
    
    println!("商品1价格: {}", price1);
    println!("商品2价格: {}", price2);
    println!("美元价格: {}", usd_price);
    
    // 货币运算
    match price1 + price2 {
        Ok(total) => println!("总价: {}", total),
        Err(e) => println!("计算错误: {}", e),
    }
    
    match price1 - price2 {
        Ok(diff) => println!("差价: {}", diff),
        Err(e) => println!("计算错误: {}", e),
    }
    
    match price1 * 1.5 {
        Ok(discounted) => println!("1.5倍价格: {}", discounted),
        Err(e) => println!("计算错误: {}", e),
    }
    
    // 金额分配演示
    let total_amount = Money::new(100.0, Currency::CNY).unwrap();
    match total_amount.allocate(&[3, 2, 5]) {
        Ok(allocated) => {
            println!("按3:2:5分配{}:", total_amount);
            for (i, amount) in allocated.iter().enumerate() {
                println!("  部分{}: {}", i + 1, amount);
            }
        }
        Err(e) => println!("分配错误: {}", e),
    }
    
    // 货币转换演示
    let cny_amount = Money::new(72.0, Currency::CNY).unwrap();
    match cny_amount.convert_to(Currency::USD) {
        Ok(usd_amount) => println!("{} 转换为 {}", cny_amount, usd_amount),
        Err(e) => println!("转换错误: {}", e),
    }
    
    println!("{}", "=".repeat(50));
    
    // 2. DateRange值对象演示
    println!("2. DateRange值对象演示:");
    
    let range1 = DateRange::new(1, 10).unwrap();
    let range2 = DateRange::new(5, 15).unwrap();
    let range3 = DateRange::new(20, 25).unwrap();
    
    println!("范围1: {}", range1);
    println!("范围2: {}", range2);
    println!("范围3: {}", range3);
    
    println!("范围1是否包含第5天: {}", range1.contains(5));
    println!("范围1是否包含第15天: {}", range1.contains(15));
    
    println!("范围1与范围2是否重叠: {}", range1.overlaps_with(&range2));
    println!("范围1与范围3是否重叠: {}", range1.overlaps_with(&range3));
    
    if let Some(intersection) = range1.intersection(&range2) {
        println!("范围1与范围2的交集: {}", intersection);
    }
    
    let extended_range = range1.extend(5);
    println!("范围1扩展5天后: {}", extended_range);
    
    println!("{}", "=".repeat(50));
    
    // 3. EmailAddress值对象演示
    println!("3. EmailAddress值对象演示:");
    
    let email1 = EmailAddress::new("user@example.com".to_string()).unwrap();
    let email2 = EmailAddress::new("Admin@Company.COM".to_string()).unwrap();
    
    println!("邮箱1: {}", email1);
    println!("邮箱2: {}", email2);
    println!("邮箱1用户名: {}", email1.username());
    println!("邮箱1域名: {}", email1.domain());
    println!("邮箱2是否为company.com域: {}", email2.is_domain("company.com"));
    
    // 测试无效邮箱
    match EmailAddress::new("invalid-email".to_string()) {
        Ok(_) => println!("不应该成功"),
        Err(e) => println!("预期错误: {}", e),
    }
    
    println!("{}", "=".repeat(50));
    
    // 4. ProductSpecification值对象演示
    println!("4. 产品规格值对象演示:");
    
    let spec1 = ProductSpecification::new(
        "智能手机".to_string(),
        "1.0".to_string(),
        vec!["GPS".to_string(), "蓝牙".to_string(), "WiFi".to_string()],
    ).unwrap();
    
    println!("产品规格: {}", spec1);
    println!("是否支持GPS: {}", spec1.has_feature("GPS"));
    println!("是否支持NFC: {}", spec1.has_feature("NFC"));
    
    // 创建升级版本
    match spec1.upgrade("2.0".to_string(), vec!["NFC".to_string(), "5G".to_string()]) {
        Ok(upgraded_spec) => {
            println!("升级后规格: {}", upgraded_spec);
            println!("升级版是否支持NFC: {}", upgraded_spec.has_feature("NFC"));
        }
        Err(e) => println!("升级失败: {}", e),
    }
    
    println!("{}", "=".repeat(50));
    
    // 5. 值对象相等性演示
    println!("5. 值对象相等性演示:");
    
    let money_a = Money::new(100.0, Currency::CNY).unwrap();
    let money_b = Money::new(100.0, Currency::CNY).unwrap();
    let money_c = Money::new(100.0, Currency::USD).unwrap();
    
    println!("{}和{}相等: {}", money_a, money_b, money_a == money_b);
    println!("{}和{}相等: {}", money_a, money_c, money_a == money_c);
    
    let email_a = EmailAddress::new("test@example.com".to_string()).unwrap();
    let email_b = EmailAddress::new("TEST@EXAMPLE.COM".to_string()).unwrap();
    println!("{}和{}相等: {}", email_a, email_b, email_a == email_b);
    
    println!("\n=== Value Object模式演示完成 ===");
    
    // 输出模式总结
    println!("\n【Value Object模式总结】");
    println!("优点:");
    println!("1. 类型安全：防止原始类型被误用");
    println!("2. 不可变性：确保数据完整性");
    println!("3. 值语义：基于内容而不是身份进行比较");
    println!("4. 封装验证：在创建时确保数据有效性");
    println!("5. 表达力强：代码更具可读性和意图明确");
    
    println!("\n适用场景:");
    println!("1. 表示度量值（如货币、重量、距离）");
    println!("2. 表示复合值（如地址、日期范围）");
    println!("3. 需要确保数据完整性时");
    println!("4. 作为实体对象的属性时");
    println!("5. 需要进行值比较而不是引用比较时");
} 