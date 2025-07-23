//! # 转换视图模式（Transform View Pattern）
//!
//! 转换视图模式通过转换来处理每个元素，将领域数据转换为HTML。
//! 与模板视图不同，转换视图是通过编程方式生成HTML，
//! 通常使用XSLT、函数式编程或其他转换技术。
//!
//! ## 模式特点
//! - **编程式生成**: 通过代码逻辑生成视图
//! - **数据驱动**: 基于数据结构动态生成界面
//! - **转换逻辑**: 明确的数据到视图的转换规则
//! - **灵活性高**: 可以处理复杂的视图逻辑
//!
//! ## 使用场景
//! - 需要复杂视图逻辑时
//! - 数据结构经常变化时
//! - 需要动态生成界面时
//! - 支持多种输出格式时

use std::collections::HashMap;
use std::fmt::{self, Display, Formatter};
use std::error::Error;

/// 转换视图错误类型
#[derive(Debug)]
pub enum TransformViewError {
    TransformationError(String),
    TemplateError(String),
    DataError(String),
    RenderError(String),
}

impl Display for TransformViewError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            TransformViewError::TransformationError(msg) => write!(f, "转换错误: {}", msg),
            TransformViewError::TemplateError(msg) => write!(f, "模板错误: {}", msg),
            TransformViewError::DataError(msg) => write!(f, "数据错误: {}", msg),
            TransformViewError::RenderError(msg) => write!(f, "渲染错误: {}", msg),
        }
    }
}

impl Error for TransformViewError {}

/// 输出格式枚举
#[derive(Debug, Clone, PartialEq)]
pub enum OutputFormat {
    Html,
    Json,
    Xml,
    Csv,
    Pdf,
}

impl Display for OutputFormat {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let format_str = match self {
            OutputFormat::Html => "HTML",
            OutputFormat::Json => "JSON",
            OutputFormat::Xml => "XML",
            OutputFormat::Csv => "CSV",
            OutputFormat::Pdf => "PDF",
        };
        write!(f, "{}", format_str)
    }
}

/// 数据模型trait
pub trait DataModel {
    fn get_data(&self) -> HashMap<String, String>;
    fn get_type(&self) -> String;
    fn validate(&self) -> Result<(), TransformViewError>;
}

/// 转换器trait
pub trait Transformer {
    fn transform(&self, data: &dyn DataModel, format: OutputFormat) -> Result<String, TransformViewError>;
    fn supports_format(&self, format: &OutputFormat) -> bool;
}

/// 客户数据模型
#[derive(Debug, Clone)]
pub struct Customer {
    pub id: String,
    pub name: String,
    pub email: String,
    pub phone: String,
    pub address: String,
    pub status: CustomerStatus,
    pub orders_count: i32,
    pub total_spent: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CustomerStatus {
    Active,
    Inactive,
    Premium,
    Suspended,
}

impl Customer {
    pub fn new(id: String, name: String, email: String) -> Self {
        Self {
            id,
            name,
            email,
            phone: String::new(),
            address: String::new(),
            status: CustomerStatus::Active,
            orders_count: 0,
            total_spent: 0.0,
        }
    }

    pub fn is_vip(&self) -> bool {
        self.status == CustomerStatus::Premium || self.total_spent > 10000.0
    }

    pub fn get_status_color(&self) -> &str {
        match self.status {
            CustomerStatus::Active => "green",
            CustomerStatus::Premium => "gold",
            CustomerStatus::Inactive => "gray",
            CustomerStatus::Suspended => "red",
        }
    }
}

impl DataModel for Customer {
    fn get_data(&self) -> HashMap<String, String> {
        let mut data = HashMap::new();
        data.insert("id".to_string(), self.id.clone());
        data.insert("name".to_string(), self.name.clone());
        data.insert("email".to_string(), self.email.clone());
        data.insert("phone".to_string(), self.phone.clone());
        data.insert("address".to_string(), self.address.clone());
        data.insert("status".to_string(), format!("{:?}", self.status));
        data.insert("status_color".to_string(), self.get_status_color().to_string());
        data.insert("orders_count".to_string(), self.orders_count.to_string());
        data.insert("total_spent".to_string(), format!("{:.2}", self.total_spent));
        data.insert("is_vip".to_string(), self.is_vip().to_string());
        data
    }

    fn get_type(&self) -> String {
        "Customer".to_string()
    }

    fn validate(&self) -> Result<(), TransformViewError> {
        if self.name.trim().is_empty() {
            return Err(TransformViewError::DataError("客户姓名不能为空".to_string()));
        }
        if self.email.trim().is_empty() {
            return Err(TransformViewError::DataError("客户邮箱不能为空".to_string()));
        }
        Ok(())
    }
}

/// 产品数据模型
#[derive(Debug, Clone)]
pub struct Product {
    pub id: String,
    pub name: String,
    pub description: String,
    pub price: f64,
    pub category: String,
    pub stock: i32,
    pub rating: f32,
    pub image_url: String,
}

impl Product {
    pub fn new(id: String, name: String, price: f64, category: String) -> Self {
        Self {
            id,
            name,
            description: String::new(),
            price,
            category,
            stock: 0,
            rating: 0.0,
            image_url: String::new(),
        }
    }

    pub fn is_in_stock(&self) -> bool {
        self.stock > 0
    }

    pub fn get_stock_status(&self) -> &str {
        if self.stock == 0 {
            "缺货"
        } else if self.stock < 10 {
            "库存不足"
        } else {
            "有库存"
        }
    }

    pub fn get_rating_stars(&self) -> String {
        let full_stars = self.rating.floor() as i32;
        let half_star = (self.rating - self.rating.floor()) >= 0.5;
        let empty_stars = 5 - full_stars - if half_star { 1 } else { 0 };

        let mut stars = "★".repeat(full_stars as usize);
        if half_star {
            stars.push('☆');
        }
        stars.push_str(&"☆".repeat(empty_stars as usize));
        stars
    }
}

impl DataModel for Product {
    fn get_data(&self) -> HashMap<String, String> {
        let mut data = HashMap::new();
        data.insert("id".to_string(), self.id.clone());
        data.insert("name".to_string(), self.name.clone());
        data.insert("description".to_string(), self.description.clone());
        data.insert("price".to_string(), format!("{:.2}", self.price));
        data.insert("category".to_string(), self.category.clone());
        data.insert("stock".to_string(), self.stock.to_string());
        data.insert("rating".to_string(), format!("{:.1}", self.rating));
        data.insert("rating_stars".to_string(), self.get_rating_stars());
        data.insert("image_url".to_string(), self.image_url.clone());
        data.insert("is_in_stock".to_string(), self.is_in_stock().to_string());
        data.insert("stock_status".to_string(), self.get_stock_status().to_string());
        data
    }

    fn get_type(&self) -> String {
        "Product".to_string()
    }

    fn validate(&self) -> Result<(), TransformViewError> {
        if self.name.trim().is_empty() {
            return Err(TransformViewError::DataError("产品名称不能为空".to_string()));
        }
        if self.price < 0.0 {
            return Err(TransformViewError::DataError("产品价格不能为负数".to_string()));
        }
        Ok(())
    }
}

/// 客户转换器
pub struct CustomerTransformer;

impl CustomerTransformer {
    pub fn new() -> Self {
        Self
    }

    fn transform_to_html(&self, data: &HashMap<String, String>) -> String {
        let is_vip = data.get("is_vip").unwrap_or(&"false".to_string()) == "true";
        let vip_badge = if is_vip { " <span class='vip-badge'>VIP</span>" } else { "" };

        format!(
            r#"
<div class="customer-card" data-customer-id="{}">
    <div class="customer-header">
        <h3 class="customer-name">{}{}</h3>
        <span class="customer-status" style="color: {};">{}</span>
    </div>
    <div class="customer-info">
        <p><strong>邮箱:</strong> <a href="mailto:{}">{}</a></p>
        <p><strong>电话:</strong> {}</p>
        <p><strong>地址:</strong> {}</p>
    </div>
    <div class="customer-stats">
        <div class="stat">
            <span class="stat-label">订单数量</span>
            <span class="stat-value">{}</span>
        </div>
        <div class="stat">
            <span class="stat-label">总消费</span>
            <span class="stat-value">¥{}</span>
        </div>
    </div>
</div>
            "#,
            data.get("id").unwrap_or(&"".to_string()),
            data.get("name").unwrap_or(&"".to_string()),
            vip_badge,
            data.get("status_color").unwrap_or(&"black".to_string()),
            data.get("status").unwrap_or(&"".to_string()),
            data.get("email").unwrap_or(&"".to_string()),
            data.get("email").unwrap_or(&"".to_string()),
            data.get("phone").unwrap_or(&"".to_string()),
            data.get("address").unwrap_or(&"".to_string()),
            data.get("orders_count").unwrap_or(&"0".to_string()),
            data.get("total_spent").unwrap_or(&"0.00".to_string())
        )
    }

    fn transform_to_json(&self, data: &HashMap<String, String>) -> String {
        format!(
            r#"{{
    "id": "{}",
    "name": "{}",
    "email": "{}",
    "phone": "{}",
    "address": "{}",
    "status": "{}",
    "ordersCount": {},
    "totalSpent": {},
    "isVip": {}
}}"#,
            data.get("id").unwrap_or(&"".to_string()),
            data.get("name").unwrap_or(&"".to_string()),
            data.get("email").unwrap_or(&"".to_string()),
            data.get("phone").unwrap_or(&"".to_string()),
            data.get("address").unwrap_or(&"".to_string()),
            data.get("status").unwrap_or(&"".to_string()),
            data.get("orders_count").unwrap_or(&"0".to_string()),
            data.get("total_spent").unwrap_or(&"0.00".to_string()),
            data.get("is_vip").unwrap_or(&"false".to_string())
        )
    }

    fn transform_to_xml(&self, data: &HashMap<String, String>) -> String {
        format!(
            r#"<customer id="{}">
    <name>{}</name>
    <email>{}</email>
    <phone>{}</phone>
    <address>{}</address>
    <status>{}</status>
    <ordersCount>{}</ordersCount>
    <totalSpent>{}</totalSpent>
    <isVip>{}</isVip>
</customer>"#,
            data.get("id").unwrap_or(&"".to_string()),
            data.get("name").unwrap_or(&"".to_string()),
            data.get("email").unwrap_or(&"".to_string()),
            data.get("phone").unwrap_or(&"".to_string()),
            data.get("address").unwrap_or(&"".to_string()),
            data.get("status").unwrap_or(&"".to_string()),
            data.get("orders_count").unwrap_or(&"0".to_string()),
            data.get("total_spent").unwrap_or(&"0.00".to_string()),
            data.get("is_vip").unwrap_or(&"false".to_string())
        )
    }

    fn transform_to_csv(&self, data: &HashMap<String, String>) -> String {
        format!(
            "{},{},{},{},{},{},{},{}",
            data.get("id").unwrap_or(&"".to_string()),
            data.get("name").unwrap_or(&"".to_string()),
            data.get("email").unwrap_or(&"".to_string()),
            data.get("phone").unwrap_or(&"".to_string()),
            data.get("address").unwrap_or(&"".to_string()),
            data.get("status").unwrap_or(&"".to_string()),
            data.get("orders_count").unwrap_or(&"0".to_string()),
            data.get("total_spent").unwrap_or(&"0.00".to_string())
        )
    }
}

impl Transformer for CustomerTransformer {
    fn transform(&self, data: &dyn DataModel, format: OutputFormat) -> Result<String, TransformViewError> {
        if data.get_type() != "Customer" {
            return Err(TransformViewError::TransformationError("数据类型不匹配".to_string()));
        }

        data.validate()?;
        let data_map = data.get_data();

        let result = match format {
            OutputFormat::Html => self.transform_to_html(&data_map),
            OutputFormat::Json => self.transform_to_json(&data_map),
            OutputFormat::Xml => self.transform_to_xml(&data_map),
            OutputFormat::Csv => self.transform_to_csv(&data_map),
            OutputFormat::Pdf => {
                return Err(TransformViewError::TransformationError("PDF格式暂不支持".to_string()));
            }
        };

        Ok(result)
    }

    fn supports_format(&self, format: &OutputFormat) -> bool {
        matches!(format, OutputFormat::Html | OutputFormat::Json | OutputFormat::Xml | OutputFormat::Csv)
    }
}

/// 产品转换器
pub struct ProductTransformer;

impl ProductTransformer {
    pub fn new() -> Self {
        Self
    }

    fn transform_to_html(&self, data: &HashMap<String, String>) -> String {
        let is_in_stock = data.get("is_in_stock").unwrap_or(&"false".to_string()) == "true";
        let stock_class = if is_in_stock { "in-stock" } else { "out-of-stock" };

        format!(
            r#"
<div class="product-card {} " data-product-id="{}">
    <div class="product-image">
        <img src="{}" alt="{}" />
    </div>
    <div class="product-info">
        <h3 class="product-name">{}</h3>
        <p class="product-description">{}</p>
        <div class="product-rating">
            <span class="stars">{}</span>
            <span class="rating-value">({}/5)</span>
        </div>
        <div class="product-price">¥{}</div>
        <div class="product-category">分类: {}</div>
        <div class="product-stock">
            <span class="stock-status">{}</span>
            <span class="stock-count">库存: {}</span>
        </div>
    </div>
</div>
            "#,
            stock_class,
            data.get("id").unwrap_or(&"".to_string()),
            data.get("image_url").unwrap_or(&"/images/placeholder.jpg".to_string()),
            data.get("name").unwrap_or(&"".to_string()),
            data.get("name").unwrap_or(&"".to_string()),
            data.get("description").unwrap_or(&"".to_string()),
            data.get("rating_stars").unwrap_or(&"☆☆☆☆☆".to_string()),
            data.get("rating").unwrap_or(&"0.0".to_string()),
            data.get("price").unwrap_or(&"0.00".to_string()),
            data.get("category").unwrap_or(&"".to_string()),
            data.get("stock_status").unwrap_or(&"".to_string()),
            data.get("stock").unwrap_or(&"0".to_string())
        )
    }

    fn transform_to_json(&self, data: &HashMap<String, String>) -> String {
        format!(
            r#"{{
    "id": "{}",
    "name": "{}",
    "description": "{}",
    "price": {},
    "category": "{}",
    "stock": {},
    "rating": {},
    "imageUrl": "{}",
    "isInStock": {},
    "stockStatus": "{}"
}}"#,
            data.get("id").unwrap_or(&"".to_string()),
            data.get("name").unwrap_or(&"".to_string()),
            data.get("description").unwrap_or(&"".to_string()),
            data.get("price").unwrap_or(&"0.00".to_string()),
            data.get("category").unwrap_or(&"".to_string()),
            data.get("stock").unwrap_or(&"0".to_string()),
            data.get("rating").unwrap_or(&"0.0".to_string()),
            data.get("image_url").unwrap_or(&"".to_string()),
            data.get("is_in_stock").unwrap_or(&"false".to_string()),
            data.get("stock_status").unwrap_or(&"".to_string())
        )
    }
}

impl Transformer for ProductTransformer {
    fn transform(&self, data: &dyn DataModel, format: OutputFormat) -> Result<String, TransformViewError> {
        if data.get_type() != "Product" {
            return Err(TransformViewError::TransformationError("数据类型不匹配".to_string()));
        }

        data.validate()?;
        let data_map = data.get_data();

        let result = match format {
            OutputFormat::Html => self.transform_to_html(&data_map),
            OutputFormat::Json => self.transform_to_json(&data_map),
            _ => {
                return Err(TransformViewError::TransformationError(format!("产品转换器不支持{}格式", format)));
            }
        };

        Ok(result)
    }

    fn supports_format(&self, format: &OutputFormat) -> bool {
        matches!(format, OutputFormat::Html | OutputFormat::Json)
    }
}

/// 转换视图引擎
pub struct TransformViewEngine {
    transformers: HashMap<String, Box<dyn Transformer>>,
}

impl TransformViewEngine {
    pub fn new() -> Self {
        let mut engine = Self {
            transformers: HashMap::new(),
        };

        // 注册默认转换器
        engine.register_transformer("Customer".to_string(), Box::new(CustomerTransformer::new()));
        engine.register_transformer("Product".to_string(), Box::new(ProductTransformer::new()));

        engine
    }

    /// 注册转换器
    pub fn register_transformer(&mut self, data_type: String, transformer: Box<dyn Transformer>) {
        println!("📝 注册转换器: {}", data_type);
        self.transformers.insert(data_type, transformer);
    }

    /// 渲染单个数据模型
    pub fn render(&self, data: &dyn DataModel, format: OutputFormat) -> Result<String, TransformViewError> {
        let data_type = data.get_type();
        
        let transformer = self.transformers.get(&data_type)
            .ok_or_else(|| TransformViewError::TransformationError(format!("未找到{}类型的转换器", data_type)))?;

        if !transformer.supports_format(&format) {
            return Err(TransformViewError::TransformationError(format!("转换器不支持{}格式", format)));
        }

        transformer.transform(data, format)
    }

    /// 渲染数据模型列表
    pub fn render_list(&self, data_list: &[&dyn DataModel], format: OutputFormat) -> Result<String, TransformViewError> {
        if data_list.is_empty() {
            return Ok(String::new());
        }

        let mut results = Vec::new();

        // 渲染每个数据项
        for data in data_list {
            let rendered = self.render(*data, format.clone())?;
            results.push(rendered);
        }

        // 根据格式组合结果
        let combined = match format {
            OutputFormat::Html => {
                format!("<div class=\"data-list\">\n{}\n</div>", results.join("\n"))
            }
            OutputFormat::Json => {
                format!("[\n{}\n]", results.join(",\n"))
            }
            OutputFormat::Xml => {
                format!("<items>\n{}\n</items>", results.join("\n"))
            }
            OutputFormat::Csv => {
                // 为CSV添加表头
                let header = match data_list[0].get_type().as_str() {
                    "Customer" => "ID,姓名,邮箱,电话,地址,状态,订单数,总消费\n",
                    "Product" => "ID,名称,描述,价格,分类,库存,评分,图片URL\n",
                    _ => "",
                };
                format!("{}{}", header, results.join("\n"))
            }
            OutputFormat::Pdf => {
                return Err(TransformViewError::TransformationError("PDF格式暂不支持".to_string()));
            }
        };

        Ok(combined)
    }

    /// 获取支持的格式列表
    pub fn get_supported_formats(&self, data_type: &str) -> Vec<OutputFormat> {
        if let Some(transformer) = self.transformers.get(data_type) {
            let all_formats = vec![
                OutputFormat::Html,
                OutputFormat::Json,
                OutputFormat::Xml,
                OutputFormat::Csv,
                OutputFormat::Pdf,
            ];
            
            all_formats.into_iter()
                .filter(|format| transformer.supports_format(format))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// 获取已注册的转换器列表
    pub fn get_registered_transformers(&self) -> Vec<String> {
        self.transformers.keys().cloned().collect()
    }
}

/// 演示转换视图模式
pub fn demo() {
    println!("=== 转换视图模式演示 ===\n");

    // 创建转换视图引擎
    let engine = TransformViewEngine::new();

    // 创建测试数据
    println!("1. 创建测试数据");
    let mut customer1 = Customer::new("cust001".to_string(), "张三".to_string(), "zhang@example.com".to_string());
    customer1.phone = "13800138000".to_string();
    customer1.address = "北京市朝阳区".to_string();
    customer1.status = CustomerStatus::Premium;
    customer1.orders_count = 15;
    customer1.total_spent = 25000.0;

    let mut customer2 = Customer::new("cust002".to_string(), "李四".to_string(), "li@example.com".to_string());
    customer2.phone = "13900139000".to_string();
    customer2.address = "上海市浦东区".to_string();
    customer2.status = CustomerStatus::Active;
    customer2.orders_count = 5;
    customer2.total_spent = 3500.0;

    let mut product1 = Product::new("prod001".to_string(), "苹果iPhone 15".to_string(), 8999.0, "手机".to_string());
    product1.description = "最新款苹果手机，搭载A17芯片".to_string();
    product1.stock = 50;
    product1.rating = 4.8;
    product1.image_url = "/images/iphone15.jpg".to_string();

    let mut product2 = Product::new("prod002".to_string(), "三星Galaxy S24".to_string(), 7999.0, "手机".to_string());
    product2.description = "三星旗舰手机，拍照功能强大".to_string();
    product2.stock = 0; // 缺货
    product2.rating = 4.5;
    product2.image_url = "/images/galaxy_s24.jpg".to_string();

    println!("   创建了 2 个客户和 2 个产品");

    // 演示单个对象的转换
    println!("\n2. 单个对象转换演示");
    
    // 客户转换为HTML
    println!("   客户转换为HTML:");
    match engine.render(&customer1, OutputFormat::Html) {
        Ok(html) => {
            println!("   {}", html);
        }
        Err(e) => println!("   转换失败: {}", e),
    }

    // 客户转换为JSON
    println!("\n   客户转换为JSON:");
    match engine.render(&customer1, OutputFormat::Json) {
        Ok(json) => {
            println!("   {}", json);
        }
        Err(e) => println!("   转换失败: {}", e),
    }

    // 产品转换为HTML
    println!("\n   产品转换为HTML:");
    match engine.render(&product1, OutputFormat::Html) {
        Ok(html) => {
            println!("   {}", html);
        }
        Err(e) => println!("   转换失败: {}", e),
    }

    // 演示列表转换
    println!("\n3. 列表转换演示");
    
    let customers: Vec<&dyn DataModel> = vec![&customer1, &customer2];
    let products: Vec<&dyn DataModel> = vec![&product1, &product2];

    // 客户列表转换为HTML
    println!("   客户列表转换为HTML:");
    match engine.render_list(&customers, OutputFormat::Html) {
        Ok(html) => {
            println!("   {}", html);
        }
        Err(e) => println!("   转换失败: {}", e),
    }

    // 客户列表转换为CSV
    println!("\n   客户列表转换为CSV:");
    match engine.render_list(&customers, OutputFormat::Csv) {
        Ok(csv) => {
            println!("   {}", csv);
        }
        Err(e) => println!("   转换失败: {}", e),
    }

    // 产品列表转换为JSON
    println!("\n   产品列表转换为JSON:");
    match engine.render_list(&products, OutputFormat::Json) {
        Ok(json) => {
            println!("   {}", json);
        }
        Err(e) => println!("   转换失败: {}", e),
    }

    // 演示格式支持查询
    println!("\n4. 格式支持查询");
    for transformer_type in engine.get_registered_transformers() {
        let supported_formats = engine.get_supported_formats(&transformer_type);
        println!("   {} 转换器支持的格式: {:?}", transformer_type, supported_formats);
    }

    // 演示错误处理
    println!("\n5. 错误处理演示");
    
    // 尝试不支持的格式
    match engine.render(&product1, OutputFormat::Pdf) {
        Ok(_) => println!("   PDF转换成功（不应该发生）"),
        Err(e) => println!("   ✅ 正确捕获错误: {}", e),
    }

    // 创建无效数据
    let mut invalid_customer = Customer::new("".to_string(), "".to_string(), "".to_string());
    match engine.render(&invalid_customer, OutputFormat::Html) {
        Ok(_) => println!("   无效数据转换成功（不应该发生）"),
        Err(e) => println!("   ✅ 正确捕获数据验证错误: {}", e),
    }

    // 演示不同输出格式的特点
    println!("\n6. 输出格式特点对比");
    println!("   📄 HTML格式: 用于网页显示，包含样式和结构");
    println!("   📊 JSON格式: 用于API交互，易于解析");
    println!("   📋 XML格式: 用于数据交换，结构化存储");
    println!("   📈 CSV格式: 用于数据导出，适合表格应用");

    println!("\n=== 转换视图模式演示完成 ===");

    println!("\n💡 转换视图模式的优势:");
    println!("1. 编程式控制 - 通过代码精确控制视图生成逻辑");
    println!("2. 多格式支持 - 同一数据可转换为多种输出格式");
    println!("3. 动态生成 - 可根据数据动态调整视图结构");
    println!("4. 逻辑清晰 - 转换逻辑明确，易于理解和维护");
    println!("5. 类型安全 - 编译时可检查转换逻辑错误");

    println!("\n⚠️ 设计考虑:");
    println!("1. 性能开销 - 编程式生成可能比模板渲染慢");
    println!("2. 代码维护 - 需要在代码中维护HTML等标记");
    println!("3. 设计者协作 - 设计师难以直接修改视图");
    println!("4. 复杂性管理 - 复杂视图逻辑可能导致代码臃肿");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_customer_data_model() {
        let customer = Customer::new("test001".to_string(), "测试用户".to_string(), "test@example.com".to_string());
        
        assert_eq!(customer.get_type(), "Customer");
        assert!(customer.validate().is_ok());
        
        let data = customer.get_data();
        assert_eq!(data.get("name").unwrap(), "测试用户");
        assert_eq!(data.get("email").unwrap(), "test@example.com");
    }

    #[test]
    fn test_customer_transformer() {
        let transformer = CustomerTransformer::new();
        let customer = Customer::new("test001".to_string(), "测试用户".to_string(), "test@example.com".to_string());
        
        // 测试HTML转换
        assert!(transformer.supports_format(&OutputFormat::Html));
        let html_result = transformer.transform(&customer, OutputFormat::Html);
        assert!(html_result.is_ok());
        assert!(html_result.unwrap().contains("测试用户"));
        
        // 测试JSON转换
        assert!(transformer.supports_format(&OutputFormat::Json));
        let json_result = transformer.transform(&customer, OutputFormat::Json);
        assert!(json_result.is_ok());
        assert!(json_result.unwrap().contains("测试用户"));
        
        // 测试不支持的格式
        assert!(!transformer.supports_format(&OutputFormat::Pdf));
    }

    #[test]
    fn test_transform_view_engine() {
        let engine = TransformViewEngine::new();
        let customer = Customer::new("test001".to_string(), "测试用户".to_string(), "test@example.com".to_string());
        
        // 测试单个对象渲染
        let result = engine.render(&customer, OutputFormat::Html);
        assert!(result.is_ok());
        
        // 测试列表渲染
        let customers: Vec<&dyn DataModel> = vec![&customer];
        let list_result = engine.render_list(&customers, OutputFormat::Json);
        assert!(list_result.is_ok());
        assert!(list_result.unwrap().starts_with("["));
    }

    #[test]
    fn test_data_validation() {
        let invalid_customer = Customer::new("".to_string(), "".to_string(), "".to_string());
        assert!(invalid_customer.validate().is_err());
        
        let valid_customer = Customer::new("test001".to_string(), "测试用户".to_string(), "test@example.com".to_string());
        assert!(valid_customer.validate().is_ok());
    }
} 