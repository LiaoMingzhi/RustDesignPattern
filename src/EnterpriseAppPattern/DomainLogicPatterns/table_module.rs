/*
 * 文件路径: /d%3A/workspace/RustLearn/RustDesignPattern/src/EnterpriseAppPattern/DomainLogicPatterns/table_module.rs
 * 
 * Table Module（表模块）模式
 * 
 * 定义：
 * Table Module是一种组织领域逻辑的方式，其中一个类处理数据库表中所有行的业务逻辑。
 * 每个表模块包含针对该表的所有业务逻辑，但不保存状态信息，而是操作传入的数据。
 * 
 * 主要特点：
 * 1. 一个模块对应一个数据库表
 * 2. 包含该表的所有业务逻辑
 * 3. 无状态操作，通过参数传递数据
 * 4. 通常与Record Set配合使用
 * 5. 介于Transaction Script和Domain Model之间
 * 
 * 优势：
 * - 比Transaction Script更好的组织结构
 * - 比Domain Model更简单
 * - 易于理解和维护
 * - 适合面向表的操作
 * 
 * 适用场景：
 * - 中等复杂度的业务逻辑
 * - 数据主要按表组织的系统
 * - 需要复用业务逻辑的场景
 * - Record Set环境
 */

use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Display, Formatter};

/// 表模块错误类型
#[derive(Debug)]
pub enum TableModuleError {
    ValidationError(String),
    BusinessRuleViolation(String),
    DataNotFound(String),
    CalculationError(String),
}

impl Display for TableModuleError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TableModuleError::ValidationError(msg) => write!(f, "验证错误: {}", msg),
            TableModuleError::BusinessRuleViolation(msg) => write!(f, "业务规则违反: {}", msg),
            TableModuleError::DataNotFound(msg) => write!(f, "数据未找到: {}", msg),
            TableModuleError::CalculationError(msg) => write!(f, "计算错误: {}", msg),
        }
    }
}

impl Error for TableModuleError {}

/// 数据行类型 - 模拟数据库记录
pub type DataRow = HashMap<String, String>;

/// 数据集类型 - 模拟Record Set
pub type DataSet = Vec<DataRow>;

/// 辅助函数：从数据行获取字符串值
fn get_string_value(row: &DataRow, key: &str) -> Result<String, TableModuleError> {
    row.get(key)
        .cloned()
        .ok_or_else(|| TableModuleError::DataNotFound(format!("字段 {} 不存在", key)))
}

/// 辅助函数：从数据行获取数值
fn get_numeric_value(row: &DataRow, key: &str) -> Result<f64, TableModuleError> {
    let value = get_string_value(row, key)?;
    value.parse::<f64>()
        .map_err(|_| TableModuleError::ValidationError(format!("字段 {} 不是有效数值: {}", key, value)))
}

/// 辅助函数：从数据行获取整数值
fn get_integer_value(row: &DataRow, key: &str) -> Result<i32, TableModuleError> {
    let value = get_string_value(row, key)?;
    value.parse::<i32>()
        .map_err(|_| TableModuleError::ValidationError(format!("字段 {} 不是有效整数: {}", key, value)))
}

/// 辅助函数：创建数据行
fn create_row(data: Vec<(&str, &str)>) -> DataRow {
    data.into_iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}

/// 用户表模块
pub struct UserTableModule;

impl UserTableModule {
    /// 验证用户数据
    pub fn validate_user_data(user_data: &DataRow) -> Result<(), TableModuleError> {
        // 验证邮箱
        let email = get_string_value(user_data, "email")?;
        if email.is_empty() || !email.contains('@') {
            return Err(TableModuleError::ValidationError("邮箱格式不正确".to_string()));
        }
        
        // 验证姓名
        let name = get_string_value(user_data, "name")?;
        if name.trim().is_empty() {
            return Err(TableModuleError::ValidationError("姓名不能为空".to_string()));
        }
        
        // 验证年龄
        let age = get_integer_value(user_data, "age")?;
        if age < 0 || age > 150 {
            return Err(TableModuleError::ValidationError("年龄必须在0-150之间".to_string()));
        }
        
        // 验证余额
        let balance = get_numeric_value(user_data, "balance")?;
        if balance < 0.0 {
            return Err(TableModuleError::ValidationError("余额不能为负数".to_string()));
        }
        
        Ok(())
    }
    
    /// 计算用户等级
    pub fn calculate_user_level(user_data: &DataRow) -> Result<String, TableModuleError> {
        let balance = get_numeric_value(user_data, "balance")?;
        let age = get_integer_value(user_data, "age")?;
        
        let level = if balance >= 100000.0 {
            "VIP"
        } else if balance >= 50000.0 {
            "金牌"
        } else if balance >= 10000.0 {
            "银牌"
        } else if age >= 18 {
            "普通"
        } else {
            "青少年"
        };
        
        Ok(level.to_string())
    }
    
    /// 计算用户信用额度
    pub fn calculate_credit_limit(user_data: &DataRow) -> Result<f64, TableModuleError> {
        let balance = get_numeric_value(user_data, "balance")?;
        let age = get_integer_value(user_data, "age")?;
        let level = Self::calculate_user_level(user_data)?;
        
        let base_credit = balance * 0.1; // 基础信用额度为余额的10%
        
        let age_factor = if age >= 25 {
            1.2
        } else if age >= 18 {
            1.0
        } else {
            0.5
        };
        
        let level_factor = match level.as_str() {
            "VIP" => 2.0,
            "金牌" => 1.5,
            "银牌" => 1.2,
            "普通" => 1.0,
            "青少年" => 0.3,
            _ => 1.0,
        };
        
        let credit_limit = base_credit * age_factor * level_factor;
        Ok(credit_limit.max(1000.0)) // 最低1000元信用额度
    }
    
    /// 批量处理用户升级
    pub fn batch_upgrade_users(users: &mut DataSet, upgrade_criteria_balance: f64) -> Result<Vec<String>, TableModuleError> {
        let mut upgraded_users = Vec::new();
        
        for user in users.iter_mut() {
            let balance = get_numeric_value(user, "balance")?;
            let current_level = Self::calculate_user_level(user)?;
            
            if balance >= upgrade_criteria_balance && current_level != "VIP" {
                let user_id = get_string_value(user, "id")?;
                let name = get_string_value(user, "name")?;
                
                // 升级奖励：增加余额
                let bonus = balance * 0.02; // 2%奖励
                let new_balance = balance + bonus;
                user.insert("balance".to_string(), new_balance.to_string());
                
                upgraded_users.push(format!("用户 {} ({}) 已升级，获得奖励: {:.2}", name, user_id, bonus));
            }
        }
        
        Ok(upgraded_users)
    }
    
    /// 计算用户统计信息
    pub fn calculate_user_statistics(users: &DataSet) -> Result<UserStatistics, TableModuleError> {
        if users.is_empty() {
            return Ok(UserStatistics::default());
        }
        
        let mut total_balance: f64 = 0.0;
        let mut total_age = 0i32;
        let mut level_counts = HashMap::new();
        let mut min_age = i32::MAX;
        let mut max_age = i32::MIN;
        let mut max_amount: f64 = 0.0;
        let mut min_balance: f64 = 999999.0;
        let mut active_count = 0;
        
        for user in users {
            let balance: f64 = get_numeric_value(user, "balance")?;
            let age = get_integer_value(user, "age")?;
            let level = Self::calculate_user_level(user)?;
            
            total_balance += balance;
            total_age += age;
            min_age = min_age.min(age);
            max_age = max_age.max(age);
            max_amount = max_amount.max(balance);
            min_balance = min_balance.min(balance);
            
            *level_counts.entry(level).or_insert(0) += 1;
            
            if balance > 0.0 {
                active_count += 1;
            }
        }
        
        let user_count = users.len();
        
        Ok(UserStatistics {
            total_users: user_count,
            average_balance: total_balance / user_count as f64,
            average_age: total_age as f64 / user_count as f64,
            total_balance,
            min_age,
            max_age,
            max_balance: max_amount,
            min_balance,
            level_distribution: level_counts,
            active_count,
        })
    }
}

/// 订单表模块
pub struct OrderTableModule;

impl OrderTableModule {
    /// 验证订单数据
    pub fn validate_order_data(order_data: &DataRow) -> Result<(), TableModuleError> {
        // 验证用户ID
        let user_id = get_string_value(order_data, "user_id")?;
        if user_id.is_empty() {
            return Err(TableModuleError::ValidationError("用户ID不能为空".to_string()));
        }
        
        // 验证订单金额
        let amount = get_numeric_value(order_data, "amount")?;
        if amount <= 0.0 {
            return Err(TableModuleError::ValidationError("订单金额必须大于0".to_string()));
        }
        
        // 验证商品数量
        let quantity = get_integer_value(order_data, "quantity")?;
        if quantity <= 0 {
            return Err(TableModuleError::ValidationError("商品数量必须大于0".to_string()));
        }
        
        Ok(())
    }
    
    /// 计算订单折扣
    pub fn calculate_order_discount(order_data: &DataRow, user_data: &DataRow) -> Result<f64, TableModuleError> {
        let amount = get_numeric_value(order_data, "amount")?;
        let quantity = get_integer_value(order_data, "quantity")?;
        let user_level = UserTableModule::calculate_user_level(user_data)?;
        
        let mut discount_rate: f64 = 0.0;
        
        // 基于用户等级的折扣
        discount_rate += match user_level.as_str() {
            "VIP" => 0.15,
            "金牌" => 0.10,
            "银牌" => 0.05,
            _ => 0.0,
        };
        
        // 基于订单金额的折扣
        if amount >= 10000.0 {
            discount_rate += 0.05;
        } else if amount >= 5000.0 {
            discount_rate += 0.03;
        } else if amount >= 1000.0 {
            discount_rate += 0.01;
        }
        
        // 基于商品数量的折扣
        if quantity >= 10 {
            discount_rate += 0.02;
        } else if quantity >= 5 {
            discount_rate += 0.01;
        }
        
        // 最大折扣不超过25%
        discount_rate = discount_rate.min(0.25);
        
        Ok(amount * discount_rate)
    }
    
    /// 计算订单最终金额
    pub fn calculate_final_amount(order_data: &DataRow, user_data: &DataRow) -> Result<f64, TableModuleError> {
        let original_amount = get_numeric_value(order_data, "amount")?;
        let discount = Self::calculate_order_discount(order_data, user_data)?;
        let final_amount = original_amount - discount;
        
        Ok(final_amount)
    }
    
    /// 批量计算订单统计
    pub fn calculate_order_statistics(orders: &DataSet) -> Result<OrderStatistics, TableModuleError> {
        if orders.is_empty() {
            return Ok(OrderStatistics::default());
        }
        
        let mut total_amount = 0.0;
        let mut total_quantity = 0i32;
        let mut max_amount: f64 = 0.0;
        let mut min_amount = f64::MAX;
        let mut user_order_counts = HashMap::new();
        
        for order in orders {
            let amount = get_numeric_value(order, "amount")?;
            let quantity = get_integer_value(order, "quantity")?;
            let user_id = get_string_value(order, "user_id")?;
            
            total_amount += amount;
            total_quantity += quantity;
            max_amount = max_amount.max(amount);
            min_amount = min_amount.min(amount);
            
            *user_order_counts.entry(user_id).or_insert(0) += 1;
        }
        
        let order_count = orders.len();
        
        Ok(OrderStatistics {
            total_orders: order_count,
            total_amount,
            average_amount: total_amount / order_count as f64,
            total_quantity,
            average_quantity: total_quantity as f64 / order_count as f64,
            max_amount,
            min_amount: if min_amount == f64::MAX { 0.0 } else { min_amount },
            unique_users: user_order_counts.len(),
            top_users: Self::get_top_users_by_order_count(user_order_counts),
        })
    }
    
    /// 获取订单数量最多的用户
    fn get_top_users_by_order_count(user_counts: HashMap<String, i32>) -> Vec<(String, i32)> {
        let mut sorted_users: Vec<(String, i32)> = user_counts.into_iter().collect();
        sorted_users.sort_by(|a, b| b.1.cmp(&a.1)); // 按订单数量降序排列
        sorted_users.into_iter().take(5).collect() // 取前5名
    }
    
    /// 生成订单报告
    pub fn generate_order_report(orders: &DataSet, users: &DataSet) -> Result<String, TableModuleError> {
        let order_stats = Self::calculate_order_statistics(orders)?;
        let user_stats = UserTableModule::calculate_user_statistics(users)?;
        
        let mut report = String::new();
        report.push_str("=== 订单分析报告 ===\n\n");
        
        report.push_str(&format!("订单统计:\n"));
        report.push_str(&format!("- 总订单数: {}\n", order_stats.total_orders));
        report.push_str(&format!("- 订单总金额: ¥{:.2}\n", order_stats.total_amount));
        report.push_str(&format!("- 平均订单金额: ¥{:.2}\n", order_stats.average_amount));
        report.push_str(&format!("- 最大订单金额: ¥{:.2}\n", order_stats.max_amount));
        report.push_str(&format!("- 总商品数量: {}\n", order_stats.total_quantity));
        report.push_str(&format!("- 涉及用户数: {}\n\n", order_stats.unique_users));
        
        report.push_str(&format!("用户统计:\n"));
        report.push_str(&format!("- 总用户数: {}\n", user_stats.total_users));
        report.push_str(&format!("- 用户总余额: ¥{:.2}\n", user_stats.total_balance));
        report.push_str(&format!("- 平均用户余额: ¥{:.2}\n", user_stats.average_balance));
        report.push_str(&format!("- 平均用户年龄: {:.1}岁\n\n", user_stats.average_age));
        
        report.push_str("活跃用户排行榜:\n");
        for (i, (user_id, count)) in order_stats.top_users.iter().enumerate() {
            report.push_str(&format!("{}. 用户{}: {}个订单\n", i + 1, user_id, count));
        }
        
        Ok(report)
    }
}

/// 产品表模块
pub struct ProductTableModule;

impl ProductTableModule {
    /// 验证产品数据
    pub fn validate_product_data(product_data: &DataRow) -> Result<(), TableModuleError> {
        let name = get_string_value(product_data, "name")?;
        if name.trim().is_empty() {
            return Err(TableModuleError::ValidationError("产品名称不能为空".to_string()));
        }
        
        let price = get_numeric_value(product_data, "price")?;
        if price <= 0.0 {
            return Err(TableModuleError::ValidationError("产品价格必须大于0".to_string()));
        }
        
        let stock = get_integer_value(product_data, "stock")?;
        if stock < 0 {
            return Err(TableModuleError::ValidationError("库存不能为负数".to_string()));
        }
        
        Ok(())
    }
    
    /// 计算产品价值
    pub fn calculate_product_value(products: &DataSet) -> Result<f64, TableModuleError> {
        let mut total_value = 0.0;
        
        for product in products {
            let price = get_numeric_value(product, "price")?;
            let stock = get_integer_value(product, "stock")?;
            total_value += price * stock as f64;
        }
        
        Ok(total_value)
    }
    
    /// 检查低库存产品
    pub fn check_low_stock_products(products: &DataSet, threshold: i32) -> Result<Vec<String>, TableModuleError> {
        let mut low_stock_products = Vec::new();
        
        for product in products {
            let name = get_string_value(product, "name")?;
            let stock = get_integer_value(product, "stock")?;
            
            if stock <= threshold {
                low_stock_products.push(format!("{} (库存: {})", name, stock));
            }
        }
        
        Ok(low_stock_products)
    }
}

/// 用户统计信息
#[derive(Debug, Default)]
pub struct UserStatistics {
    pub total_users: usize,
    pub average_balance: f64,
    pub average_age: f64,
    pub total_balance: f64,
    pub min_age: i32,
    pub max_age: i32,
    pub max_balance: f64,
    pub min_balance: f64,
    pub level_distribution: HashMap<String, i32>,
    pub active_count: usize,
}

impl Display for UserStatistics {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "用户统计: {} 人, 平均余额: ¥{:.2}, 平均年龄: {:.1}岁", 
               self.total_users, self.average_balance, self.average_age)
    }
}

/// 订单统计信息
#[derive(Debug, Default)]
pub struct OrderStatistics {
    pub total_orders: usize,
    pub total_amount: f64,
    pub average_amount: f64,
    pub total_quantity: i32,
    pub average_quantity: f64,
    pub max_amount: f64,
    pub min_amount: f64,
    pub unique_users: usize,
    pub top_users: Vec<(String, i32)>,
}

impl Display for OrderStatistics {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "订单统计: {} 个订单, 总金额: ¥{:.2}, 平均金额: ¥{:.2}", 
               self.total_orders, self.total_amount, self.average_amount)
    }
}

/// Table Module模式演示
pub fn demo() {
    println!("=== Table Module（表模块）模式演示 ===\n");
    
    // 1. 创建示例数据
    println!("1. 创建示例数据:");
    let mut users = vec![
        create_row(vec![
            ("id", "1"), ("name", "张三"), ("email", "zhang@example.com"),
            ("age", "28"), ("balance", "15000.50")
        ]),
        create_row(vec![
            ("id", "2"), ("name", "李四"), ("email", "li@example.com"),
            ("age", "35"), ("balance", "85000.00")
        ]),
        create_row(vec![
            ("id", "3"), ("name", "王五"), ("email", "wang@example.com"),
            ("age", "22"), ("balance", "3500.75")
        ]),
        create_row(vec![
            ("id", "4"), ("name", "赵六"), ("email", "zhao@example.com"),
            ("age", "45"), ("balance", "120000.00")
        ]),
    ];
    
    let orders = vec![
        create_row(vec![
            ("id", "1"), ("user_id", "1"), ("amount", "2500.00"), ("quantity", "5")
        ]),
        create_row(vec![
            ("id", "2"), ("user_id", "2"), ("amount", "8500.00"), ("quantity", "12")
        ]),
        create_row(vec![
            ("id", "3"), ("user_id", "1"), ("amount", "1200.00"), ("quantity", "3")
        ]),
        create_row(vec![
            ("id", "4"), ("user_id", "3"), ("amount", "600.00"), ("quantity", "2")
        ]),
        create_row(vec![
            ("id", "5"), ("user_id", "4"), ("amount", "15000.00"), ("quantity", "20")
        ]),
    ];
    
    let products = vec![
        create_row(vec![
            ("id", "1"), ("name", "智能手机"), ("price", "2999.00"), ("stock", "25")
        ]),
        create_row(vec![
            ("id", "2"), ("name", "笔记本电脑"), ("price", "5999.00"), ("stock", "8")
        ]),
        create_row(vec![
            ("id", "3"), ("name", "无线耳机"), ("price", "299.00"), ("stock", "3")
        ]),
    ];
    
    println!("已创建 {} 个用户, {} 个订单, {} 个产品", users.len(), orders.len(), products.len());
    
    println!("{}", "=".repeat(50));
    
    // 2. 用户表模块演示
    println!("2. 用户表模块操作:");
    
    // 验证用户数据
    for (i, user) in users.iter().enumerate() {
        match UserTableModule::validate_user_data(user) {
            Ok(_) => {
                let name = user.get("name").unwrap();
                let level = UserTableModule::calculate_user_level(user).unwrap();
                let credit_limit = UserTableModule::calculate_credit_limit(user).unwrap();
                println!("用户 {} - 等级: {}, 信用额度: ¥{:.2}", name, level, credit_limit);
            }
            Err(e) => println!("用户 {} 验证失败: {}", i + 1, e),
        }
    }
    
    // 批量升级用户
    println!("\n批量升级用户 (余额≥80000):");
    match UserTableModule::batch_upgrade_users(&mut users, 80000.0) {
        Ok(upgrades) => {
            for upgrade in upgrades {
                println!("  {}", upgrade);
            }
        }
        Err(e) => println!("升级失败: {}", e),
    }
    
    // 用户统计
    match UserTableModule::calculate_user_statistics(&users) {
        Ok(stats) => {
            println!("\n用户统计信息:");
            println!("  {}", stats);
            println!("  年龄范围: {} - {}岁", stats.min_age, stats.max_age);
            println!("  最高余额: ¥{:.2}", stats.max_balance);
            println!("  等级分布:");
            for (level, count) in &stats.level_distribution {
                println!("    {}: {} 人", level, count);
            }
        }
        Err(e) => println!("统计失败: {}", e),
    }
    
    println!("{}", "=".repeat(50));
    
    // 3. 订单表模块演示
    println!("3. 订单表模块操作:");
    
    // 计算订单折扣和最终金额
    for order in &orders {
        match order.get("user_id").and_then(|uid| {
            users.iter().find(|u| u.get("id") == Some(uid))
        }) {
            Some(user) => {
                let order_id = order.get("id").unwrap();
                let original_amount = get_numeric_value(order, "amount").unwrap();
                
                match OrderTableModule::calculate_order_discount(order, user) {
                    Ok(discount) => {
                        let final_amount = original_amount - discount;
                        let user_name = user.get("name").unwrap();
                        println!("订单 {} (用户: {}) - 原价: ¥{:.2}, 折扣: ¥{:.2}, 实付: ¥{:.2}", 
                               order_id, user_name, original_amount, discount, final_amount);
                    }
                    Err(e) => println!("订单 {} 计算失败: {}", order_id, e),
                }
            }
            None => println!("订单 {} 找不到对应用户", order.get("id").unwrap()),
        }
    }
    
    // 订单统计
    match OrderTableModule::calculate_order_statistics(&orders) {
        Ok(stats) => {
            println!("\n订单统计信息:");
            println!("  {}", stats);
            println!("  订单金额范围: ¥{:.2} - ¥{:.2}", stats.min_amount, stats.max_amount);
            println!("  活跃用户数: {}", stats.unique_users);
        }
        Err(e) => println!("订单统计失败: {}", e),
    }
    
    println!("{}", "=".repeat(50));
    
    // 4. 产品表模块演示
    println!("4. 产品表模块操作:");
    
    // 产品价值计算
    match ProductTableModule::calculate_product_value(&products) {
        Ok(total_value) => println!("库存总价值: ¥{:.2}", total_value),
        Err(e) => println!("价值计算失败: {}", e),
    }
    
    // 低库存检查
    match ProductTableModule::check_low_stock_products(&products, 10) {
        Ok(low_stock) => {
            if low_stock.is_empty() {
                println!("没有低库存产品");
            } else {
                println!("低库存产品 (≤10):");
                for product in low_stock {
                    println!("  {}", product);
                }
            }
        }
        Err(e) => println!("库存检查失败: {}", e),
    }
    
    println!("{}", "=".repeat(50));
    
    // 5. 综合报告生成
    println!("5. 生成综合报告:");
    match OrderTableModule::generate_order_report(&orders, &users) {
        Ok(report) => println!("{}", report),
        Err(e) => println!("报告生成失败: {}", e),
    }
    
    println!("{}", "=".repeat(50));
    
    // 6. 数据验证演示
    println!("6. 数据验证演示:");
    
    // 无效用户数据
    let invalid_user = create_row(vec![
        ("id", "999"), ("name", ""), ("email", "invalid"),
        ("age", "-5"), ("balance", "-1000")
    ]);
    
    match UserTableModule::validate_user_data(&invalid_user) {
        Ok(_) => println!("无效用户验证通过（不应该发生）"),
        Err(e) => println!("无效用户验证失败（预期）: {}", e),
    }
    
    // 无效订单数据
    let invalid_order = create_row(vec![
        ("id", "999"), ("user_id", ""), ("amount", "-100"), ("quantity", "0")
    ]);
    
    match OrderTableModule::validate_order_data(&invalid_order) {
        Ok(_) => println!("无效订单验证通过（不应该发生）"),
        Err(e) => println!("无效订单验证失败（预期）: {}", e),
    }
    
    println!("\n=== Table Module模式演示完成 ===");
    
    // 输出模式总结
    println!("\n【Table Module模式总结】");
    println!("核心特点:");
    println!("1. 表导向组织：每个表一个模块类");
    println!("2. 无状态操作：不保存数据状态，通过参数传递");
    println!("3. 业务逻辑集中：针对表的所有业务逻辑都在模块中");
    println!("4. Record Set友好：适合与数据集配合使用");
    
    println!("\n优势:");
    println!("1. 比Transaction Script更好的组织结构");
    println!("2. 比Domain Model更简单直接");
    println!("3. 易于理解和实现");
    println!("4. 适合数据库导向的设计");
    println!("5. 支持批量操作");
    
    println!("\n适用场景:");
    println!("1. 中等复杂度的业务逻辑");
    println!("2. 表结构相对稳定的系统");
    println!("3. 需要大量数据处理的应用");
    println!("4. Record Set或DataSet环境");
    println!("5. 报表和分析系统");
}