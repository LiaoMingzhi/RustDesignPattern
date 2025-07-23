//! 映射器模式（Mapper）
//! 
//! 映射器模式负责在对象和数据源（如数据库记录、XML、JSON等）之间进行转换。
//! 它将对象-关系映射的逻辑封装在专门的映射器类中，使领域对象保持纯净，
//! 不包含任何数据源特定的代码。
//! 
//! 文件位置：/d%3A/workspace/RustLearn/RustDesignPattern/src/EnterpriseAppPattern/BasePatterns/mapper.rs

use std::collections::HashMap;
use std::fmt;

// =================
// 通用映射器接口
// =================

/// 数据行接口 - 表示数据源中的一行数据
pub trait DataRow {
    fn get_string(&self, column: &str) -> Option<String>;
    fn get_i32(&self, column: &str) -> Option<i32>;
    fn get_i64(&self, column: &str) -> Option<i64>;
    fn get_f64(&self, column: &str) -> Option<f64>;
    fn get_bool(&self, column: &str) -> Option<bool>;
    fn set_string(&mut self, column: &str, value: String);
    fn set_i32(&mut self, column: &str, value: i32);
    fn set_i64(&mut self, column: &str, value: i64);
    fn set_f64(&mut self, column: &str, value: f64);
    fn set_bool(&mut self, column: &str, value: bool);
}

/// 数据记录的HashMap实现
#[derive(Debug, Clone)]
pub struct HashMapDataRow {
    data: HashMap<String, DataValue>,
}

/// 数据值枚举
#[derive(Debug, Clone)]
pub enum DataValue {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Null,
}

impl HashMapDataRow {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }
    
    pub fn from_map(data: HashMap<String, DataValue>) -> Self {
        Self { data }
    }
}

impl DataRow for HashMapDataRow {
    fn get_string(&self, column: &str) -> Option<String> {
        match self.data.get(column) {
            Some(DataValue::String(s)) => Some(s.clone()),
            Some(DataValue::Integer(i)) => Some(i.to_string()),
            Some(DataValue::Float(f)) => Some(f.to_string()),
            Some(DataValue::Boolean(b)) => Some(b.to_string()),
            _ => None,
        }
    }
    
    fn get_i32(&self, column: &str) -> Option<i32> {
        match self.data.get(column) {
            Some(DataValue::Integer(i)) => Some(*i as i32),
            Some(DataValue::String(s)) => s.parse().ok(),
            _ => None,
        }
    }
    
    fn get_i64(&self, column: &str) -> Option<i64> {
        match self.data.get(column) {
            Some(DataValue::Integer(i)) => Some(*i),
            Some(DataValue::String(s)) => s.parse().ok(),
            _ => None,
        }
    }
    
    fn get_f64(&self, column: &str) -> Option<f64> {
        match self.data.get(column) {
            Some(DataValue::Float(f)) => Some(*f),
            Some(DataValue::Integer(i)) => Some(*i as f64),
            Some(DataValue::String(s)) => s.parse().ok(),
            _ => None,
        }
    }
    
    fn get_bool(&self, column: &str) -> Option<bool> {
        match self.data.get(column) {
            Some(DataValue::Boolean(b)) => Some(*b),
            Some(DataValue::Integer(i)) => Some(*i != 0),
            Some(DataValue::String(s)) => match s.to_lowercase().as_str() {
                "true" | "1" | "yes" => Some(true),
                "false" | "0" | "no" => Some(false),
                _ => None,
            },
            _ => None,
        }
    }
    
    fn set_string(&mut self, column: &str, value: String) {
        self.data.insert(column.to_string(), DataValue::String(value));
    }
    
    fn set_i32(&mut self, column: &str, value: i32) {
        self.data.insert(column.to_string(), DataValue::Integer(value as i64));
    }
    
    fn set_i64(&mut self, column: &str, value: i64) {
        self.data.insert(column.to_string(), DataValue::Integer(value));
    }
    
    fn set_f64(&mut self, column: &str, value: f64) {
        self.data.insert(column.to_string(), DataValue::Float(value));
    }
    
    fn set_bool(&mut self, column: &str, value: bool) {
        self.data.insert(column.to_string(), DataValue::Boolean(value));
    }
}

/// 通用映射器接口
pub trait Mapper<T> {
    type Error;
    
    /// 将数据行映射为对象
    fn map_to_object(&self, row: &dyn DataRow) -> Result<T, Self::Error>;
    
    /// 将对象映射为数据行
    fn map_to_row(&self, object: &T) -> Result<Box<dyn DataRow>, Self::Error>;
    
    /// 更新数据行中的对象数据
    fn update_row(&self, object: &T, row: &mut dyn DataRow) -> Result<(), Self::Error>;
}

// =================
// 具体实体类
// =================

/// 用户实体
#[derive(Debug, Clone)]
pub struct User {
    pub id: Option<u32>,
    pub username: String,
    pub email: String,
    pub age: u32,
    pub is_active: bool,
    pub salary: f64,
}

impl User {
    pub fn new(username: String, email: String, age: u32, salary: f64) -> Self {
        Self {
            id: None,
            username,
            email,
            age,
            is_active: true,
            salary,
        }
    }
    
    pub fn with_id(id: u32, username: String, email: String, age: u32, salary: f64) -> Self {
        Self {
            id: Some(id),
            username,
            email,
            age,
            is_active: true,
            salary,
        }
    }
}

/// 产品实体
#[derive(Debug, Clone)]
pub struct Product {
    pub id: Option<u32>,
    pub name: String,
    pub description: String,
    pub price: f64,
    pub category_id: u32,
    pub in_stock: bool,
}

impl Product {
    pub fn new(name: String, description: String, price: f64, category_id: u32) -> Self {
        Self {
            id: None,
            name,
            description,
            price,
            category_id,
            in_stock: true,
        }
    }
}

/// 订单实体
#[derive(Debug, Clone)]
pub struct Order {
    pub id: Option<u32>,
    pub user_id: u32,
    pub total_amount: f64,
    pub status: OrderStatus,
    pub created_at: String,
}

#[derive(Debug, Clone)]
pub enum OrderStatus {
    Pending,
    Processing,
    Shipped,
    Delivered,
    Cancelled,
}

impl fmt::Display for OrderStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let status = match self {
            OrderStatus::Pending => "pending",
            OrderStatus::Processing => "processing",
            OrderStatus::Shipped => "shipped",
            OrderStatus::Delivered => "delivered",
            OrderStatus::Cancelled => "cancelled",
        };
        write!(f, "{}", status)
    }
}

impl OrderStatus {
    pub fn from_string(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "pending" => Some(OrderStatus::Pending),
            "processing" => Some(OrderStatus::Processing),
            "shipped" => Some(OrderStatus::Shipped),
            "delivered" => Some(OrderStatus::Delivered),
            "cancelled" => Some(OrderStatus::Cancelled),
            _ => None,
        }
    }
}

// =================
// 映射器错误
// =================

#[derive(Debug)]
pub enum MapperError {
    MissingField(String),
    InvalidValue(String),
    TypeConversion(String),
}

impl fmt::Display for MapperError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MapperError::MissingField(field) => write!(f, "缺少字段: {}", field),
            MapperError::InvalidValue(msg) => write!(f, "无效值: {}", msg),
            MapperError::TypeConversion(msg) => write!(f, "类型转换错误: {}", msg),
        }
    }
}

// =================
// 具体映射器实现
// =================

/// 用户映射器
pub struct UserMapper;

impl Mapper<User> for UserMapper {
    type Error = MapperError;
    
    fn map_to_object(&self, row: &dyn DataRow) -> Result<User, Self::Error> {
        let id = row.get_i32("id").map(|i| i as u32);
        
        let username = row.get_string("username")
            .ok_or_else(|| MapperError::MissingField("username".to_string()))?;
            
        let email = row.get_string("email")
            .ok_or_else(|| MapperError::MissingField("email".to_string()))?;
            
        let age = row.get_i32("age")
            .ok_or_else(|| MapperError::MissingField("age".to_string()))? as u32;
            
        let is_active = row.get_bool("is_active").unwrap_or(true);
        
        let salary = row.get_f64("salary")
            .ok_or_else(|| MapperError::MissingField("salary".to_string()))?;
        
        Ok(User {
            id,
            username,
            email,
            age,
            is_active,
            salary,
        })
    }
    
    fn map_to_row(&self, user: &User) -> Result<Box<dyn DataRow>, Self::Error> {
        let mut row = HashMapDataRow::new();
        
        if let Some(id) = user.id {
            row.set_i32("id", id as i32);
        }
        
        row.set_string("username", user.username.clone());
        row.set_string("email", user.email.clone());
        row.set_i32("age", user.age as i32);
        row.set_bool("is_active", user.is_active);
        row.set_f64("salary", user.salary);
        
        Ok(Box::new(row))
    }
    
    fn update_row(&self, user: &User, row: &mut dyn DataRow) -> Result<(), Self::Error> {
        if let Some(id) = user.id {
            row.set_i32("id", id as i32);
        }
        
        row.set_string("username", user.username.clone());
        row.set_string("email", user.email.clone());
        row.set_i32("age", user.age as i32);
        row.set_bool("is_active", user.is_active);
        row.set_f64("salary", user.salary);
        
        Ok(())
    }
}

/// 产品映射器
pub struct ProductMapper;

impl Mapper<Product> for ProductMapper {
    type Error = MapperError;
    
    fn map_to_object(&self, row: &dyn DataRow) -> Result<Product, Self::Error> {
        let id = row.get_i32("id").map(|i| i as u32);
        
        let name = row.get_string("name")
            .ok_or_else(|| MapperError::MissingField("name".to_string()))?;
            
        let description = row.get_string("description").unwrap_or_default();
        
        let price = row.get_f64("price")
            .ok_or_else(|| MapperError::MissingField("price".to_string()))?;
            
        let category_id = row.get_i32("category_id")
            .ok_or_else(|| MapperError::MissingField("category_id".to_string()))? as u32;
            
        let in_stock = row.get_bool("in_stock").unwrap_or(true);
        
        Ok(Product {
            id,
            name,
            description,
            price,
            category_id,
            in_stock,
        })
    }
    
    fn map_to_row(&self, product: &Product) -> Result<Box<dyn DataRow>, Self::Error> {
        let mut row = HashMapDataRow::new();
        
        if let Some(id) = product.id {
            row.set_i32("id", id as i32);
        }
        
        row.set_string("name", product.name.clone());
        row.set_string("description", product.description.clone());
        row.set_f64("price", product.price);
        row.set_i32("category_id", product.category_id as i32);
        row.set_bool("in_stock", product.in_stock);
        
        Ok(Box::new(row))
    }
    
    fn update_row(&self, product: &Product, row: &mut dyn DataRow) -> Result<(), Self::Error> {
        if let Some(id) = product.id {
            row.set_i32("id", id as i32);
        }
        
        row.set_string("name", product.name.clone());
        row.set_string("description", product.description.clone());
        row.set_f64("price", product.price);
        row.set_i32("category_id", product.category_id as i32);
        row.set_bool("in_stock", product.in_stock);
        
        Ok(())
    }
}

/// 订单映射器
pub struct OrderMapper;

impl Mapper<Order> for OrderMapper {
    type Error = MapperError;
    
    fn map_to_object(&self, row: &dyn DataRow) -> Result<Order, Self::Error> {
        let id = row.get_i32("id").map(|i| i as u32);
        
        let user_id = row.get_i32("user_id")
            .ok_or_else(|| MapperError::MissingField("user_id".to_string()))? as u32;
            
        let total_amount = row.get_f64("total_amount")
            .ok_or_else(|| MapperError::MissingField("total_amount".to_string()))?;
            
        let status_str = row.get_string("status")
            .ok_or_else(|| MapperError::MissingField("status".to_string()))?;
            
        let status = OrderStatus::from_string(&status_str)
            .ok_or_else(|| MapperError::InvalidValue(format!("无效的订单状态: {}", status_str)))?;
            
        let created_at = row.get_string("created_at").unwrap_or_else(|| "2024-01-01 00:00:00".to_string());
        
        Ok(Order {
            id,
            user_id,
            total_amount,
            status,
            created_at,
        })
    }
    
    fn map_to_row(&self, order: &Order) -> Result<Box<dyn DataRow>, Self::Error> {
        let mut row = HashMapDataRow::new();
        
        if let Some(id) = order.id {
            row.set_i32("id", id as i32);
        }
        
        row.set_i32("user_id", order.user_id as i32);
        row.set_f64("total_amount", order.total_amount);
        row.set_string("status", order.status.to_string());
        row.set_string("created_at", order.created_at.clone());
        
        Ok(Box::new(row))
    }
    
    fn update_row(&self, order: &Order, row: &mut dyn DataRow) -> Result<(), Self::Error> {
        if let Some(id) = order.id {
            row.set_i32("id", id as i32);
        }
        
        row.set_i32("user_id", order.user_id as i32);
        row.set_f64("total_amount", order.total_amount);
        row.set_string("status", order.status.to_string());
        row.set_string("created_at", order.created_at.clone());
        
        Ok(())
    }
}

// =================
// 映射器注册表
// =================

/// 映射器注册表 - 管理所有映射器的中央注册表
pub struct MapperRegistry {
    user_mapper: UserMapper,
    product_mapper: ProductMapper,
    order_mapper: OrderMapper,
}

impl MapperRegistry {
    pub fn new() -> Self {
        Self {
            user_mapper: UserMapper,
            product_mapper: ProductMapper,
            order_mapper: OrderMapper,
        }
    }
    
    pub fn get_user_mapper(&self) -> &UserMapper {
        &self.user_mapper
    }
    
    pub fn get_product_mapper(&self) -> &ProductMapper {
        &self.product_mapper
    }
    
    pub fn get_order_mapper(&self) -> &OrderMapper {
        &self.order_mapper
    }
    
    /// 批量映射用户数据
    pub fn map_users(&self, rows: Vec<Box<dyn DataRow>>) -> Vec<Result<User, MapperError>> {
        rows.iter()
            .map(|row| self.user_mapper.map_to_object(row.as_ref()))
            .collect()
    }
    
    /// 批量映射产品数据
    pub fn map_products(&self, rows: Vec<Box<dyn DataRow>>) -> Vec<Result<Product, MapperError>> {
        rows.iter()
            .map(|row| self.product_mapper.map_to_object(row.as_ref()))
            .collect()
    }
}

// =================
// 复杂映射示例
// =================

/// 用户统计映射器 - 演示复杂对象映射
pub struct UserStatisticsMapper;

#[derive(Debug)]
pub struct UserStatistics {
    pub user_id: u32,
    pub username: String,
    pub total_orders: u32,
    pub total_spent: f64,
    pub average_order_value: f64,
    pub last_order_date: String,
}

impl Mapper<UserStatistics> for UserStatisticsMapper {
    type Error = MapperError;
    
    fn map_to_object(&self, row: &dyn DataRow) -> Result<UserStatistics, Self::Error> {
        let user_id = row.get_i32("user_id")
            .ok_or_else(|| MapperError::MissingField("user_id".to_string()))? as u32;
            
        let username = row.get_string("username")
            .ok_or_else(|| MapperError::MissingField("username".to_string()))?;
            
        let total_orders = row.get_i32("total_orders")
            .ok_or_else(|| MapperError::MissingField("total_orders".to_string()))? as u32;
            
        let total_spent = row.get_f64("total_spent")
            .ok_or_else(|| MapperError::MissingField("total_spent".to_string()))?;
            
        let average_order_value = if total_orders > 0 {
            total_spent / total_orders as f64
        } else {
            0.0
        };
        
        let last_order_date = row.get_string("last_order_date")
            .unwrap_or_else(|| "无订单".to_string());
        
        Ok(UserStatistics {
            user_id,
            username,
            total_orders,
            total_spent,
            average_order_value,
            last_order_date,
        })
    }
    
    fn map_to_row(&self, stats: &UserStatistics) -> Result<Box<dyn DataRow>, Self::Error> {
        let mut row = HashMapDataRow::new();
        
        row.set_i32("user_id", stats.user_id as i32);
        row.set_string("username", stats.username.clone());
        row.set_i32("total_orders", stats.total_orders as i32);
        row.set_f64("total_spent", stats.total_spent);
        row.set_f64("average_order_value", stats.average_order_value);
        row.set_string("last_order_date", stats.last_order_date.clone());
        
        Ok(Box::new(row))
    }
    
    fn update_row(&self, stats: &UserStatistics, row: &mut dyn DataRow) -> Result<(), Self::Error> {
        row.set_i32("user_id", stats.user_id as i32);
        row.set_string("username", stats.username.clone());
        row.set_i32("total_orders", stats.total_orders as i32);
        row.set_f64("total_spent", stats.total_spent);
        row.set_f64("average_order_value", stats.average_order_value);
        row.set_string("last_order_date", stats.last_order_date.clone());
        
        Ok(())
    }
}

/// 映射器模式演示
pub fn demo_mapper_pattern() {
    println!("=== 映射器（Mapper）模式演示 ===\n");
    
    let registry = MapperRegistry::new();
    
    println!("1. 用户映射器演示:");
    
    // 创建用户对象
    let user = User::new(
        "张三".to_string(),
        "zhangsan@example.com".to_string(),
        30,
        8000.0
    );
    
    // 对象映射为数据行
    let user_mapper = registry.get_user_mapper();
    let row = user_mapper.map_to_row(&user).unwrap();
    
    println!("对象转换为数据行:");
    println!("  用户名: {:?}", row.get_string("username"));
    println!("  邮箱: {:?}", row.get_string("email"));
    println!("  年龄: {:?}", row.get_i32("age"));
    println!("  薪资: {:?}", row.get_f64("salary"));
    
    // 数据行映射为对象
    let restored_user = user_mapper.map_to_object(row.as_ref()).unwrap();
    println!("数据行转换为对象: {:?}\n", restored_user);
    
    println!("2. 产品映射器演示:");
    
    let product = Product::new(
        "iPhone 15".to_string(),
        "最新款苹果手机".to_string(),
        7999.0,
        1
    );
    
    let product_mapper = registry.get_product_mapper();
    let product_row = product_mapper.map_to_row(&product).unwrap();
    
    println!("产品对象: {:?}", product);
    
    let restored_product = product_mapper.map_to_object(product_row.as_ref()).unwrap();
    println!("恢复的产品: {:?}\n", restored_product);
    
    println!("3. 订单映射器演示:");
    
    let order = Order {
        id: Some(1),
        user_id: 1,
        total_amount: 15998.0,
        status: OrderStatus::Processing,
        created_at: "2024-01-15 10:30:00".to_string(),
    };
    
    let order_mapper = registry.get_order_mapper();
    let order_row = order_mapper.map_to_row(&order).unwrap();
    
    println!("订单状态: {:?}", order_row.get_string("status"));
    
    let restored_order = order_mapper.map_to_object(order_row.as_ref()).unwrap();
    println!("恢复的订单: {:?}\n", restored_order);
    
    println!("4. 复杂对象映射演示:");
    
    // 模拟从数据库查询得到的统计数据
    let mut stats_data = HashMap::new();
    stats_data.insert("user_id".to_string(), DataValue::Integer(1));
    stats_data.insert("username".to_string(), DataValue::String("张三".to_string()));
    stats_data.insert("total_orders".to_string(), DataValue::Integer(5));
    stats_data.insert("total_spent".to_string(), DataValue::Float(25000.0));
    stats_data.insert("last_order_date".to_string(), DataValue::String("2024-01-15".to_string()));
    
    let stats_row = HashMapDataRow::from_map(stats_data);
    let stats_mapper = UserStatisticsMapper;
    
    let user_stats = stats_mapper.map_to_object(&stats_row).unwrap();
    println!("用户统计信息:");
    println!("  用户ID: {}", user_stats.user_id);
    println!("  用户名: {}", user_stats.username);
    println!("  订单总数: {}", user_stats.total_orders);
    println!("  消费总额: {:.2}", user_stats.total_spent);
    println!("  平均订单金额: {:.2}", user_stats.average_order_value);
    println!("  最后订单日期: {}\n", user_stats.last_order_date);
    
    println!("5. 错误处理演示:");
    
    // 创建缺少必需字段的数据行
    let mut invalid_data = HashMap::new();
    invalid_data.insert("username".to_string(), DataValue::String("无效用户".to_string()));
    // 缺少email字段
    
    let invalid_row = HashMapDataRow::from_map(invalid_data);
    
    match user_mapper.map_to_object(&invalid_row) {
        Ok(user) => println!("映射成功: {:?}", user),
        Err(error) => println!("映射失败: {}", error),
    }
    
    println!("\n=== 映射器模式特点 ===");
    println!("✓ 分离关注点 - 对象保持纯净，映射逻辑独立");
    println!("✓ 类型安全 - 强类型映射，编译时检查");
    println!("✓ 可重用性 - 映射器可以在多个场景中重用");
    println!("✓ 扩展性 - 易于添加新的映射规则和数据类型");
    println!("✓ 错误处理 - 完善的错误处理机制");
    println!("✓ 复杂映射 - 支持复杂对象和聚合映射");
}