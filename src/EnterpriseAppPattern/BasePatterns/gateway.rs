/*
 * 文件路径: /d%3A/workspace/RustLearn/RustDesignPattern/src/EnterpriseAppPattern/BasePatterns/gateway.rs
 * 
 * Gateway（入口）模式
 * 
 * 定义：
 * Gateway模式通过一个简单的API来封装对外部系统或资源的访问。
 * 它隐藏了复杂的API或网络通信的复杂性，为客户端提供一个简化的接口。
 * 
 * 主要特点：
 * 1. 简化复杂的外部API
 * 2. 提供统一的访问接口
 * 3. 隔离外部系统的变化
 * 4. 便于测试和模拟
 * 
 * 适用场景：
 * - 需要访问复杂的外部系统API时
 * - 需要对外部系统进行抽象和封装时
 * - 需要提供测试替身时
 * - 需要统一多个外部系统的访问方式时
 */

use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Display, Formatter};

/// Gateway错误类型
#[derive(Debug)]
pub enum GatewayError {
    NetworkError(String),
    AuthenticationError(String),
    DataNotFound(String),
    InvalidRequest(String),
}

impl Display for GatewayError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            GatewayError::NetworkError(msg) => write!(f, "网络错误: {}", msg),
            GatewayError::AuthenticationError(msg) => write!(f, "认证错误: {}", msg),
            GatewayError::DataNotFound(msg) => write!(f, "数据未找到: {}", msg),
            GatewayError::InvalidRequest(msg) => write!(f, "无效请求: {}", msg),
        }
    }
}

impl Error for GatewayError {}

/// 第三方支付响应
#[derive(Debug, Clone)]
pub struct PaymentResponse {
    pub transaction_id: String,
    pub status: String,
    pub amount: f64,
    pub message: String,
}

impl Display for PaymentResponse {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "支付响应 [ID: {}, 状态: {}, 金额: {:.2}, 消息: {}]", 
               self.transaction_id, self.status, self.amount, self.message)
    }
}

/// 第三方支付Gateway接口
/// 这个trait定义了支付Gateway的通用接口
pub trait PaymentGateway {
    /// 处理支付请求
    fn process_payment(&self, amount: f64, card_number: &str, description: &str) -> Result<PaymentResponse, GatewayError>;
    
    /// 查询支付状态
    fn query_payment_status(&self, transaction_id: &str) -> Result<PaymentResponse, GatewayError>;
    
    /// 退款
    fn refund_payment(&self, transaction_id: &str, amount: f64) -> Result<PaymentResponse, GatewayError>;
}

/// 支付宝Gateway实现
pub struct AlipayGateway {
    api_key: String,
    app_id: String,
    // 模拟的交易存储
    transactions: std::sync::Mutex<HashMap<String, PaymentResponse>>,
}

impl AlipayGateway {
    pub fn new(api_key: String, app_id: String) -> Self {
        Self {
            api_key,
            app_id,
            transactions: std::sync::Mutex::new(HashMap::new()),
        }
    }
    
    /// 模拟生成交易ID
    fn generate_transaction_id(&self) -> String {
        format!("alipay_{}", chrono::Utc::now().timestamp_millis())
    }
    
    /// 模拟API调用
    fn simulate_api_call(&self, endpoint: &str, _params: &HashMap<String, String>) -> Result<String, GatewayError> {
        // 模拟网络延迟和可能的错误
        if endpoint.contains("invalid") {
            return Err(GatewayError::InvalidRequest("无效的支付请求".to_string()));
        }
        
        Ok("success".to_string())
    }
}

impl PaymentGateway for AlipayGateway {
    fn process_payment(&self, amount: f64, card_number: &str, description: &str) -> Result<PaymentResponse, GatewayError> {
        // 验证输入参数
        if amount <= 0.0 {
            return Err(GatewayError::InvalidRequest("支付金额必须大于0".to_string()));
        }
        
        if card_number.is_empty() {
            return Err(GatewayError::InvalidRequest("卡号不能为空".to_string()));
        }
        
        // 构建API参数
        let mut params = HashMap::new();
        params.insert("app_id".to_string(), self.app_id.clone());
        params.insert("amount".to_string(), amount.to_string());
        params.insert("card_number".to_string(), card_number.to_string());
        params.insert("description".to_string(), description.to_string());
        
        // 调用支付宝API（模拟）
        match self.simulate_api_call("alipay/pay", &params) {
            Ok(_) => {
                let transaction_id = self.generate_transaction_id();
                let response = PaymentResponse {
                    transaction_id: transaction_id.clone(),
                    status: "success".to_string(),
                    amount,
                    message: format!("支付成功: {}", description),
                };
                
                // 存储交易记录
                if let Ok(mut transactions) = self.transactions.lock() {
                    transactions.insert(transaction_id.clone(), response.clone());
                }
                
                Ok(response)
            }
            Err(e) => Err(e),
        }
    }
    
    fn query_payment_status(&self, transaction_id: &str) -> Result<PaymentResponse, GatewayError> {
        if let Ok(transactions) = self.transactions.lock() {
            if let Some(response) = transactions.get(transaction_id) {
                Ok(response.clone())
            } else {
                Err(GatewayError::DataNotFound(format!("交易ID {} 不存在", transaction_id)))
            }
        } else {
            Err(GatewayError::NetworkError("无法访问交易数据".to_string()))
        }
    }
    
    fn refund_payment(&self, transaction_id: &str, amount: f64) -> Result<PaymentResponse, GatewayError> {
        // 首先查询原交易
        let original_payment = self.query_payment_status(transaction_id)?;
        
        if amount > original_payment.amount {
            return Err(GatewayError::InvalidRequest("退款金额不能超过原支付金额".to_string()));
        }
        
        // 创建退款响应
        let refund_id = format!("refund_{}", chrono::Utc::now().timestamp_millis());
        let response = PaymentResponse {
            transaction_id: refund_id.clone(),
            status: "refunded".to_string(),
            amount,
            message: format!("退款成功，原交易ID: {}", transaction_id),
        };
        
        // 存储退款记录
        if let Ok(mut transactions) = self.transactions.lock() {
            transactions.insert(refund_id.clone(), response.clone());
        }
        
        Ok(response)
    }
}

/// 微信支付Gateway实现
pub struct WechatPayGateway {
    merchant_id: String,
    secret_key: String,
    transactions: std::sync::Mutex<HashMap<String, PaymentResponse>>,
}

impl WechatPayGateway {
    pub fn new(merchant_id: String, secret_key: String) -> Self {
        Self {
            merchant_id,
            secret_key,
            transactions: std::sync::Mutex::new(HashMap::new()),
        }
    }
    
    fn generate_transaction_id(&self) -> String {
        format!("wechat_{}", chrono::Utc::now().timestamp_millis())
    }
}

impl PaymentGateway for WechatPayGateway {
    fn process_payment(&self, amount: f64, card_number: &str, description: &str) -> Result<PaymentResponse, GatewayError> {
        // 验证商户信息
        if self.merchant_id.is_empty() {
            return Err(GatewayError::AuthenticationError("商户ID不能为空".to_string()));
        }
        
        let transaction_id = self.generate_transaction_id();
        let response = PaymentResponse {
            transaction_id: transaction_id.clone(),
            status: "success".to_string(),
            amount,
            message: format!("微信支付成功: {}", description),
        };
        
        // 存储交易记录
        if let Ok(mut transactions) = self.transactions.lock() {
            transactions.insert(transaction_id.clone(), response.clone());
        }
        
        Ok(response)
    }
    
    fn query_payment_status(&self, transaction_id: &str) -> Result<PaymentResponse, GatewayError> {
        if let Ok(transactions) = self.transactions.lock() {
            if let Some(response) = transactions.get(transaction_id) {
                Ok(response.clone())
            } else {
                Err(GatewayError::DataNotFound(format!("交易ID {} 不存在", transaction_id)))
            }
        } else {
            Err(GatewayError::NetworkError("无法访问交易数据".to_string()))
        }
    }
    
    fn refund_payment(&self, transaction_id: &str, amount: f64) -> Result<PaymentResponse, GatewayError> {
        let original_payment = self.query_payment_status(transaction_id)?;
        
        if amount > original_payment.amount {
            return Err(GatewayError::InvalidRequest("退款金额不能超过原支付金额".to_string()));
        }
        
        let refund_id = format!("wechat_refund_{}", chrono::Utc::now().timestamp_millis());
        let response = PaymentResponse {
            transaction_id: refund_id.clone(),
            status: "refunded".to_string(),
            amount,
            message: format!("微信退款成功，原交易ID: {}", transaction_id),
        };
        
        if let Ok(mut transactions) = self.transactions.lock() {
            transactions.insert(refund_id.clone(), response.clone());
        }
        
        Ok(response)
    }
}

/// 支付服务，使用Gateway模式
pub struct PaymentService {
    gateway: Box<dyn PaymentGateway + Send + Sync>,
}

impl PaymentService {
    pub fn new(gateway: Box<dyn PaymentGateway + Send + Sync>) -> Self {
        Self { gateway }
    }
    
    /// 统一的支付接口
    pub fn make_payment(&self, amount: f64, card_number: &str, description: &str) -> Result<PaymentResponse, GatewayError> {
        println!("正在处理支付请求...");
        let result = self.gateway.process_payment(amount, card_number, description);
        
        match &result {
            Ok(response) => println!("支付处理完成: {}", response),
            Err(e) => println!("支付处理失败: {}", e),
        }
        
        result
    }
    
    /// 查询支付状态
    pub fn check_payment_status(&self, transaction_id: &str) -> Result<PaymentResponse, GatewayError> {
        self.gateway.query_payment_status(transaction_id)
    }
    
    /// 处理退款
    pub fn process_refund(&self, transaction_id: &str, amount: f64) -> Result<PaymentResponse, GatewayError> {
        println!("正在处理退款请求...");
        let result = self.gateway.refund_payment(transaction_id, amount);
        
        match &result {
            Ok(response) => println!("退款处理完成: {}", response),
            Err(e) => println!("退款处理失败: {}", e),
        }
        
        result
    }
}

/// Gateway模式演示
pub fn demo() {
    println!("=== Gateway（入口）模式演示 ===\n");
    
    // 1. 使用支付宝Gateway
    println!("1. 支付宝Gateway演示:");
    let alipay_gateway = AlipayGateway::new(
        "your_alipay_api_key".to_string(),
        "your_app_id".to_string(),
    );
    let alipay_service = PaymentService::new(Box::new(alipay_gateway));
    
    // 处理支付
    match alipay_service.make_payment(100.0, "1234567890", "购买商品") {
        Ok(response) => {
            println!("支付成功！交易ID: {}\n", response.transaction_id);
            
            // 查询支付状态
            match alipay_service.check_payment_status(&response.transaction_id) {
                Ok(status) => println!("支付状态查询: {}\n", status),
                Err(e) => println!("状态查询失败: {}\n", e),
            }
            
            // 处理退款
            match alipay_service.process_refund(&response.transaction_id, 50.0) {
                Ok(refund) => println!("退款成功: {}\n", refund),
                Err(e) => println!("退款失败: {}\n", e),
            }
        }
        Err(e) => println!("支付失败: {}\n", e),
    }
    
    println!("{}", "=".repeat(50));
    
    // 2. 使用微信支付Gateway
    println!("2. 微信支付Gateway演示:");
    let wechat_gateway = WechatPayGateway::new(
        "your_merchant_id".to_string(),
        "your_secret_key".to_string(),
    );
    let wechat_service = PaymentService::new(Box::new(wechat_gateway));
    
    match wechat_service.make_payment(200.0, "0987654321", "充值余额") {
        Ok(response) => {
            println!("微信支付成功！交易ID: {}\n", response.transaction_id);
            
            match wechat_service.check_payment_status(&response.transaction_id) {
                Ok(status) => println!("微信支付状态: {}\n", status),
                Err(e) => println!("状态查询失败: {}\n", e),
            }
        }
        Err(e) => println!("微信支付失败: {}\n", e),
    }
    
    println!("{}", "=".repeat(50));
    
    // 3. 演示错误处理
    println!("3. 错误处理演示:");
    let alipay_gateway = AlipayGateway::new(
        "test_key".to_string(),
        "test_app".to_string(),
    );
    let error_service = PaymentService::new(Box::new(alipay_gateway));
    
    // 无效金额
    match error_service.make_payment(-10.0, "1234", "测试") {
        Ok(_) => println!("不应该成功"),
        Err(e) => println!("预期错误: {}", e),
    }
    
    // 空卡号
    match error_service.make_payment(100.0, "", "测试") {
        Ok(_) => println!("不应该成功"),
        Err(e) => println!("预期错误: {}", e),
    }
    
    // 查询不存在的交易
    match error_service.check_payment_status("non_existent_id") {
        Ok(_) => println!("不应该成功"),
        Err(e) => println!("预期错误: {}", e),
    }
    
    println!("\n=== Gateway模式演示完成 ===");
    
    // 输出模式总结
    println!("\n【Gateway模式总结】");
    println!("优点:");
    println!("1. 封装复杂性：隐藏外部API的复杂性");
    println!("2. 统一接口：为不同的外部系统提供统一的访问方式");
    println!("3. 易于测试：可以轻松创建Mock实现进行测试");
    println!("4. 错误处理：统一的错误处理机制");
    println!("5. 可替换性：可以轻松切换不同的外部服务提供商");
    
    println!("\n适用场景:");
    println!("1. 需要访问第三方API或外部服务时");
    println!("2. 需要为复杂的外部系统提供简化接口时");
    println!("3. 需要隔离外部系统变化的影响时");
    println!("4. 需要提供统一的访问模式时");
}

// 引入chrono用于时间戳生成（需要在Cargo.toml中添加依赖）
mod chrono {
    pub struct Utc;
    
    impl Utc {
        pub fn now() -> DateTime {
            DateTime
        }
    }
    
    pub struct DateTime;
    
    impl DateTime {
        pub fn timestamp_millis(&self) -> u128 {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis()
        }
    }
} 