//! 金钱模式（Money）
//! 
//! 金钱模式是一种特殊的值对象，用于表示货币金额。它解决了浮点数精度问题，
//! 支持多币种，提供安全的货币运算，并确保不同币种之间不能直接运算。
//! 
//! 文件位置：/d%3A/workspace/RustLearn/RustDesignPattern/src/EnterpriseAppPattern/BasePatterns/money.rs

use std::fmt;
use std::ops::{Add, Sub, Mul, Div};
use std::cmp::{PartialEq, Eq, PartialOrd, Ord, Ordering};
use std::collections::HashMap;

// =================
// 货币类型
// =================

/// 货币类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Currency {
    USD,  // 美元
    EUR,  // 欧元
    CNY,  // 人民币
    JPY,  // 日元
    GBP,  // 英镑
    KRW,  // 韩元
    HKD,  // 港币
    SGD,  // 新加坡元
}

impl Currency {
    /// 获取货币代码
    pub fn code(&self) -> &'static str {
        match self {
            Currency::USD => "USD",
            Currency::EUR => "EUR",
            Currency::CNY => "CNY",
            Currency::JPY => "JPY",
            Currency::GBP => "GBP",
            Currency::KRW => "KRW",
            Currency::HKD => "HKD",
            Currency::SGD => "SGD",
        }
    }
    
    /// 获取货币符号
    pub fn symbol(&self) -> &'static str {
        match self {
            Currency::USD => "$",
            Currency::EUR => "€",
            Currency::CNY => "¥",
            Currency::JPY => "¥",
            Currency::GBP => "£",
            Currency::KRW => "₩",
            Currency::HKD => "HK$",
            Currency::SGD => "S$",
        }
    }
    
    /// 获取小数位数
    pub fn decimal_places(&self) -> u8 {
        match self {
            Currency::USD | Currency::EUR | Currency::CNY | 
            Currency::GBP | Currency::HKD | Currency::SGD => 2,
            Currency::JPY | Currency::KRW => 0,
        }
    }
    
    /// 获取货币名称
    pub fn name(&self) -> &'static str {
        match self {
            Currency::USD => "美元",
            Currency::EUR => "欧元",
            Currency::CNY => "人民币",
            Currency::JPY => "日元",
            Currency::GBP => "英镑",
            Currency::KRW => "韩元",
            Currency::HKD => "港币",
            Currency::SGD => "新加坡元",
        }
    }
    
    /// 从字符串创建货币类型
    pub fn from_code(code: &str) -> Option<Self> {
        match code.to_uppercase().as_str() {
            "USD" => Some(Currency::USD),
            "EUR" => Some(Currency::EUR),
            "CNY" => Some(Currency::CNY),
            "JPY" => Some(Currency::JPY),
            "GBP" => Some(Currency::GBP),
            "KRW" => Some(Currency::KRW),
            "HKD" => Some(Currency::HKD),
            "SGD" => Some(Currency::SGD),
            _ => None,
        }
    }
}

impl fmt::Display for Currency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.code())
    }
}

// =================
// 金钱值对象
// =================

/// 金钱值对象
/// 使用整数存储最小货币单位以避免浮点精度问题
#[derive(Debug, Clone, Copy)]
pub struct Money {
    /// 金额（以最小货币单位表示，如美分、分）
    amount: i64,
    /// 货币类型
    currency: Currency,
}

impl Money {
    /// 创建金钱对象（金额以主要单位表示，如元、美元）
    pub fn new(amount: f64, currency: Currency) -> Self {
        let decimal_places = currency.decimal_places();
        let multiplier = 10_i64.pow(decimal_places as u32);
        let amount_in_cents = (amount * multiplier as f64).round() as i64;
        
        Self {
            amount: amount_in_cents,
            currency,
        }
    }
    
    /// 创建金钱对象（金额以最小单位表示，如美分、分）
    pub fn from_cents(amount: i64, currency: Currency) -> Self {
        Self { amount, currency }
    }
    
    /// 创建零金额
    pub fn zero(currency: Currency) -> Self {
        Self {
            amount: 0,
            currency,
        }
    }
    
    /// 获取金额（主要单位）
    pub fn amount(&self) -> f64 {
        let decimal_places = self.currency.decimal_places();
        let divisor = 10_i64.pow(decimal_places as u32) as f64;
        self.amount as f64 / divisor
    }
    
    /// 获取金额（最小单位）
    pub fn amount_in_cents(&self) -> i64 {
        self.amount
    }
    
    /// 获取货币类型
    pub fn currency(&self) -> Currency {
        self.currency
    }
    
    /// 是否为零
    pub fn is_zero(&self) -> bool {
        self.amount == 0
    }
    
    /// 是否为正数
    pub fn is_positive(&self) -> bool {
        self.amount > 0
    }
    
    /// 是否为负数
    pub fn is_negative(&self) -> bool {
        self.amount < 0
    }
    
    /// 取绝对值
    pub fn abs(&self) -> Self {
        Self {
            amount: self.amount.abs(),
            currency: self.currency,
        }
    }
    
    /// 取负值
    pub fn negate(&self) -> Self {
        Self {
            amount: -self.amount,
            currency: self.currency,
        }
    }
    
    /// 验证货币类型是否匹配
    fn assert_same_currency(&self, other: &Money) -> Result<(), MoneyError> {
        if self.currency != other.currency {
            Err(MoneyError::CurrencyMismatch {
                left: self.currency,
                right: other.currency,
            })
        } else {
            Ok(())
        }
    }
    
    /// 安全加法
    pub fn add(&self, other: &Money) -> Result<Money, MoneyError> {
        self.assert_same_currency(other)?;
        
        match self.amount.checked_add(other.amount) {
            Some(result) => Ok(Money::from_cents(result, self.currency)),
            None => Err(MoneyError::Overflow),
        }
    }
    
    /// 安全减法
    pub fn subtract(&self, other: &Money) -> Result<Money, MoneyError> {
        self.assert_same_currency(other)?;
        
        match self.amount.checked_sub(other.amount) {
            Some(result) => Ok(Money::from_cents(result, self.currency)),
            None => Err(MoneyError::Overflow),
        }
    }
    
    /// 乘法（乘以数量）
    pub fn multiply(&self, factor: f64) -> Result<Money, MoneyError> {
        let result = (self.amount as f64 * factor).round() as i64;
        Ok(Money::from_cents(result, self.currency))
    }
    
    /// 除法（除以数量）
    pub fn divide(&self, divisor: f64) -> Result<Money, MoneyError> {
        if divisor == 0.0 {
            return Err(MoneyError::DivisionByZero);
        }
        
        let result = (self.amount as f64 / divisor).round() as i64;
        Ok(Money::from_cents(result, self.currency))
    }
    
    /// 分配金额（按比例分配，确保总和不变）
    pub fn allocate(&self, ratios: &[f64]) -> Result<Vec<Money>, MoneyError> {
        if ratios.is_empty() {
            return Err(MoneyError::InvalidRatio("比例数组不能为空".to_string()));
        }
        
        let total_ratio: f64 = ratios.iter().sum();
        if total_ratio == 0.0 {
            return Err(MoneyError::InvalidRatio("比例总和不能为零".to_string()));
        }
        
        let mut results = Vec::new();
        let mut remaining = self.amount;
        
        for (i, &ratio) in ratios.iter().enumerate() {
            if i == ratios.len() - 1 {
                // 最后一个分配剩余金额，确保总和正确
                results.push(Money::from_cents(remaining, self.currency));
            } else {
                let allocated = (self.amount as f64 * ratio / total_ratio).round() as i64;
                results.push(Money::from_cents(allocated, self.currency));
                remaining -= allocated;
            }
        }
        
        Ok(results)
    }
    
    /// 平均分配（尽可能均等，余数分配给前几个）
    pub fn distribute(&self, count: usize) -> Result<Vec<Money>, MoneyError> {
        if count == 0 {
            return Err(MoneyError::InvalidRatio("分配数量不能为零".to_string()));
        }
        
        let base_amount = self.amount / count as i64;
        let remainder = self.amount % count as i64;
        
        let mut results = Vec::with_capacity(count);
        for i in 0..count {
            let amount = if i < remainder as usize {
                base_amount + 1
            } else {
                base_amount
            };
            results.push(Money::from_cents(amount, self.currency));
        }
        
        Ok(results)
    }
    
    /// 格式化为字符串
    pub fn format(&self) -> String {
        let symbol = self.currency.symbol();
        let amount = self.amount();
        
        match self.currency.decimal_places() {
            0 => format!("{}{:.0}", symbol, amount),
            2 => format!("{}{:.2}", symbol, amount),
            n => format!("{}{:.width$}", symbol, amount, width = n as usize),
        }
    }
    
    /// 格式化为带千分位分隔符的字符串
    pub fn format_with_separator(&self) -> String {
        let symbol = self.currency.symbol();
        let amount = self.amount();
        let decimal_places = self.currency.decimal_places();
        
        let formatted = if decimal_places == 0 {
            format!("{:.0}", amount)
        } else {
            format!("{:.width$}", amount, width = decimal_places as usize)
        };
        
        // 添加千分位分隔符
        let parts: Vec<&str> = formatted.split('.').collect();
        let integer_part = parts[0];
        let decimal_part = if parts.len() > 1 { parts[1] } else { "" };
        
        let mut result = String::new();
        let chars: Vec<char> = integer_part.chars().collect();
        for (i, &ch) in chars.iter().enumerate() {
            if i > 0 && (chars.len() - i) % 3 == 0 {
                result.push(',');
            }
            result.push(ch);
        }
        
        if !decimal_part.is_empty() {
            result.push('.');
            result.push_str(decimal_part);
        }
        
        format!("{}{}", symbol, result)
    }
}

// =================
// 错误类型
// =================

#[derive(Debug)]
pub enum MoneyError {
    CurrencyMismatch { left: Currency, right: Currency },
    Overflow,
    DivisionByZero,
    InvalidRatio(String),
}

impl fmt::Display for MoneyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MoneyError::CurrencyMismatch { left, right } => {
                write!(f, "货币类型不匹配: {} vs {}", left, right)
            }
            MoneyError::Overflow => write!(f, "数值溢出"),
            MoneyError::DivisionByZero => write!(f, "除零错误"),
            MoneyError::InvalidRatio(msg) => write!(f, "无效的比例: {}", msg),
        }
    }
}

// =================
// 运算符重载
// =================

impl PartialEq for Money {
    fn eq(&self, other: &Self) -> bool {
        self.currency == other.currency && self.amount == other.amount
    }
}

impl Eq for Money {}

impl PartialOrd for Money {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.currency != other.currency {
            None
        } else {
            Some(self.amount.cmp(&other.amount))
        }
    }
}

impl Ord for Money {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.currency != other.currency {
            panic!("无法比较不同币种的金额");
        }
        self.amount.cmp(&other.amount)
    }
}

impl Add for Money {
    type Output = Result<Money, MoneyError>;
    
    fn add(self, other: Money) -> Self::Output {
        self.add(other)
    }
}

impl Sub for Money {
    type Output = Result<Money, MoneyError>;
    
    fn sub(self, other: Money) -> Self::Output {
        self.subtract(&other)
    }
}

impl Mul<f64> for Money {
    type Output = Result<Money, MoneyError>;
    
    fn mul(self, factor: f64) -> Self::Output {
        self.multiply(factor)
    }
}

impl Div<f64> for Money {
    type Output = Result<Money, MoneyError>;
    
    fn div(self, divisor: f64) -> Self::Output {
        self.divide(divisor)
    }
}

impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format())
    }
}

// =================
// 货币转换器
// =================

/// 汇率信息
#[derive(Debug, Clone)]
pub struct ExchangeRate {
    from: Currency,
    to: Currency,
    rate: f64,
    timestamp: String,
}

impl ExchangeRate {
    pub fn new(from: Currency, to: Currency, rate: f64) -> Self {
        Self {
            from,
            to,
            rate,
            timestamp: "2024-01-01 00:00:00".to_string(),
        }
    }
    
    pub fn rate(&self) -> f64 {
        self.rate
    }
}

/// 货币转换器
pub struct CurrencyConverter {
    exchange_rates: HashMap<(Currency, Currency), ExchangeRate>,
}

impl CurrencyConverter {
    pub fn new() -> Self {
        let mut converter = Self {
            exchange_rates: HashMap::new(),
        };
        
        // 初始化一些示例汇率
        converter.add_rate(ExchangeRate::new(Currency::USD, Currency::CNY, 7.2));
        converter.add_rate(ExchangeRate::new(Currency::CNY, Currency::USD, 1.0 / 7.2));
        converter.add_rate(ExchangeRate::new(Currency::EUR, Currency::CNY, 7.8));
        converter.add_rate(ExchangeRate::new(Currency::CNY, Currency::EUR, 1.0 / 7.8));
        converter.add_rate(ExchangeRate::new(Currency::USD, Currency::EUR, 0.85));
        converter.add_rate(ExchangeRate::new(Currency::EUR, Currency::USD, 1.0 / 0.85));
        converter.add_rate(ExchangeRate::new(Currency::GBP, Currency::CNY, 9.1));
        converter.add_rate(ExchangeRate::new(Currency::CNY, Currency::GBP, 1.0 / 9.1));
        
        converter
    }
    
    pub fn add_rate(&mut self, rate: ExchangeRate) {
        let key = (rate.from, rate.to);
        self.exchange_rates.insert(key, rate);
    }
    
    pub fn get_rate(&self, from: Currency, to: Currency) -> Option<&ExchangeRate> {
        self.exchange_rates.get(&(from, to))
    }
    
    pub fn convert(&self, money: &Money, to_currency: Currency) -> Result<Money, MoneyError> {
        if money.currency == to_currency {
            return Ok(*money);
        }
        
        if let Some(rate) = self.get_rate(money.currency, to_currency) {
            let converted_amount = money.amount() * rate.rate();
            Ok(Money::new(converted_amount, to_currency))
        } else {
            Err(MoneyError::InvalidRatio(format!(
                "无法找到从 {} 到 {} 的汇率",
                money.currency,
                to_currency
            )))
        }
    }
    
    pub fn list_rates(&self) -> Vec<&ExchangeRate> {
        self.exchange_rates.values().collect()
    }
}

// =================
// 金钱集合操作
// =================

/// 金钱集合 - 支持多币种金额的集合操作
pub struct MoneyBag {
    amounts: HashMap<Currency, Money>,
}

impl MoneyBag {
    pub fn new() -> Self {
        Self {
            amounts: HashMap::new(),
        }
    }
    
    pub fn add(&mut self, money: Money) {
        let currency = money.currency();
        match self.amounts.get(&currency) {
            Some(existing) => {
                if let Ok(sum) = existing.add(&money) {
                    self.amounts.insert(currency, sum);
                }
            }
            None => {
                self.amounts.insert(currency, money);
            }
        }
    }
    
    pub fn subtract(&mut self, money: Money) {
        let currency = money.currency();
        match self.amounts.get(&currency) {
            Some(existing) => {
                if let Ok(diff) = existing.subtract(&money) {
                    if diff.is_zero() {
                        self.amounts.remove(&currency);
                    } else {
                        self.amounts.insert(currency, diff);
                    }
                }
            }
            None => {
                self.amounts.insert(currency, money.negate());
            }
        }
    }
    
    pub fn get(&self, currency: Currency) -> Option<&Money> {
        self.amounts.get(&currency)
    }
    
    pub fn currencies(&self) -> Vec<Currency> {
        self.amounts.keys().cloned().collect()
    }
    
    pub fn is_empty(&self) -> bool {
        self.amounts.is_empty()
    }
    
    pub fn convert_to(&self, currency: Currency, converter: &CurrencyConverter) -> Result<Money, MoneyError> {
        let mut total = Money::zero(currency);
        
        for money in self.amounts.values() {
            let converted = converter.convert(money, currency)?;
            total = total.add(converted)?;
        }
        
        Ok(total)
    }
}

impl fmt::Display for MoneyBag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.amounts.is_empty() {
            return write!(f, "空钱袋");
        }
        
        let amounts: Vec<String> = self.amounts.values()
            .map(|money| money.format())
            .collect();
        
        write!(f, "[{}]", amounts.join(", "))
    }
}

/// 金钱模式演示
pub fn demo_money_pattern() {
    println!("=== 金钱（Money）模式演示 ===\n");
    
    println!("1. 基本金钱操作:");
    
    let usd100 = Money::new(100.0, Currency::USD);
    let usd50 = Money::new(50.0, Currency::USD);
    let cny500 = Money::new(500.0, Currency::CNY);
    
    println!("USD $100: {}", usd100);
    println!("USD $50: {}", usd50);
    println!("CNY ¥500: {}", cny500);
    
    // 相同币种运算
    if let Ok(sum) = usd100.add(usd50) {
        println!("$100 + $50 = {}", sum);
    }
    
    if let Ok(diff) = usd100.subtract(&usd50) {
        println!("$100 - $50 = {}", diff);
    }
    
    if let Ok(product) = usd100.multiply(1.5) {
        println!("$100 × 1.5 = {}", product);
    }
    
    if let Ok(quotient) = usd100.divide(4.0) {
        println!("$100 ÷ 4 = {}", quotient);
    }
    
    println!();
    
    println!("2. 货币类型安全:");
    
    // 不同币种运算会报错
    match usd100.add(cny500) {
        Ok(result) => println!("运算成功: {}", result),
        Err(error) => println!("运算失败: {}", error),
    }
    
    println!();
    
    println!("3. 精确计算演示:");
    
    let money1 = Money::new(0.1, Currency::USD);
    let money2 = Money::new(0.2, Currency::USD);
    
    if let Ok(sum) = money1.add(money2) {
        println!("$0.1 + $0.2 = {} (精确计算)", sum);
        println!("原始金额 (分): {} + {} = {}", 
                 money1.amount_in_cents(), 
                 money2.amount_in_cents(),
                 sum.amount_in_cents());
    }
    
    println!();
    
    println!("4. 金额分配:");
    
    let total = Money::new(100.0, Currency::USD);
    
    // 按比例分配
    let ratios = [0.3, 0.5, 0.2];
    if let Ok(allocated) = total.allocate(&ratios) {
        println!("$100 按比例 [30%, 50%, 20%] 分配:");
        for (i, amount) in allocated.iter().enumerate() {
            println!("  第{}份: {}", i + 1, amount);
        }
        
        // 验证总和
        let mut sum = Money::zero(Currency::USD);
        for amount in &allocated {
            sum = sum.add(*amount).unwrap();
        }
        println!("  总和验证: {}", sum);
    }
    
    // 平均分配
    if let Ok(distributed) = total.distribute(3) {
        println!("$100 平均分配给3个人:");
        for (i, amount) in distributed.iter().enumerate() {
            println!("  第{}人: {}", i + 1, amount);
        }
    }
    
    println!();
    
    println!("5. 货币转换:");
    
    let converter = CurrencyConverter::new();
    
    let usd_amount = Money::new(100.0, Currency::USD);
    
    if let Ok(cny_amount) = converter.convert(&usd_amount, Currency::CNY) {
        println!("$100 转换为人民币: {}", cny_amount);
    }
    
    if let Ok(eur_amount) = converter.convert(&usd_amount, Currency::EUR) {
        println!("$100 转换为欧元: {}", eur_amount);
    }
    
    println!();
    
    println!("6. 多币种钱袋:");
    
    let mut money_bag = MoneyBag::new();
    money_bag.add(Money::new(100.0, Currency::USD));
    money_bag.add(Money::new(500.0, Currency::CNY));
    money_bag.add(Money::new(75.0, Currency::EUR));
    money_bag.add(Money::new(50.0, Currency::USD)); // 同币种累加
    
    println!("钱袋内容: {}", money_bag);
    
    // 转换为单一币种
    if let Ok(total_usd) = money_bag.convert_to(Currency::USD, &converter) {
        println!("钱袋总价值 (USD): {}", total_usd);
    }
    
    if let Ok(total_cny) = money_bag.convert_to(Currency::CNY, &converter) {
        println!("钱袋总价值 (CNY): {}", total_cny);
    }
    
    println!();
    
    println!("7. 格式化输出:");
    
    let amounts = [
        Money::new(1234567.89, Currency::USD),
        Money::new(9876543.21, Currency::CNY),
        Money::new(1000000.0, Currency::JPY),
        Money::new(12345.67, Currency::EUR),
    ];
    
    for amount in &amounts {
        println!("  基本格式: {}", amount.format());
        println!("  千分位格式: {}", amount.format_with_separator());
        println!();
    }
    
    println!("=== 金钱模式特点 ===");
    println!("✓ 精确计算 - 使用整数避免浮点精度问题");
    println!("✓ 类型安全 - 不同币种无法直接运算");
    println!("✓ 不可变性 - 值对象特性，操作返回新对象");
    println!("✓ 多币种支持 - 完整的货币类型系统");
    println!("✓ 丰富操作 - 分配、转换、格式化等功能");
    println!("✓ 溢出保护 - 安全的数学运算");
} 