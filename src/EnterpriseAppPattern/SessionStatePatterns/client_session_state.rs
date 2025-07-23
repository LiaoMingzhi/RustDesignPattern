/*
 * 文件路径: /d%3A/workspace/RustLearn/RustDesignPattern/src/EnterpriseAppPattern/SessionStatePatterns/client_session_state.rs
 * 
 * Client Session State（客户端会话状态）模式
 * 
 * 定义：
 * 将会话状态存储在客户端，通常以表单隐藏字段、URL参数或客户端存储的形式。
 * 这种方式将状态管理的负担转移给客户端，减轻了服务器的内存压力。
 * 
 * 主要特点：
 * 1. 状态存储在客户端
 * 2. 每次请求都会传输状态数据
 * 3. 服务器无状态，易于扩展
 * 4. 需要考虑安全性和数据完整性
 * 5. 适合状态数据较少的场景
 * 
 * 适用场景：
 * - Web应用的表单多步骤提交
 * - 购物车状态管理
 * - 用户偏好设置
 * - 临时的交互状态
 */

use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Display, Formatter};

/// 客户端会话状态错误
#[derive(Debug)]
pub enum ClientSessionError {
    EncodingError(String),
    DecodingError(String),
    ValidationError(String),
    SecurityError(String),
}

impl Display for ClientSessionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ClientSessionError::EncodingError(msg) => write!(f, "编码错误: {}", msg),
            ClientSessionError::DecodingError(msg) => write!(f, "解码错误: {}", msg),
            ClientSessionError::ValidationError(msg) => write!(f, "验证错误: {}", msg),
            ClientSessionError::SecurityError(msg) => write!(f, "安全错误: {}", msg),
        }
    }
}

impl Error for ClientSessionError {}

/// 会话状态数据
#[derive(Debug, Clone, PartialEq)]
pub struct SessionData {
    data: HashMap<String, String>,
    timestamp: u64,
    checksum: String,
}

impl SessionData {
    /// 创建新的会话数据
    pub fn new() -> Self {
        let timestamp = current_timestamp();
        let mut session = Self {
            data: HashMap::new(),
            timestamp,
            checksum: String::new(),
        };
        session.update_checksum();
        session
    }
    
    /// 设置数据
    pub fn set(&mut self, key: &str, value: &str) {
        self.data.insert(key.to_string(), value.to_string());
        self.timestamp = current_timestamp();
        self.update_checksum();
    }
    
    /// 获取数据
    pub fn get(&self, key: &str) -> Option<&String> {
        self.data.get(key)
    }
    
    /// 删除数据
    pub fn remove(&mut self, key: &str) -> Option<String> {
        let result = self.data.remove(key);
        if result.is_some() {
            self.timestamp = current_timestamp();
            self.update_checksum();
        }
        result
    }
    
    /// 获取所有数据
    pub fn get_all(&self) -> &HashMap<String, String> {
        &self.data
    }
    
    /// 获取时间戳
    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }
    
    /// 检查是否过期
    pub fn is_expired(&self, timeout_seconds: u64) -> bool {
        let current = current_timestamp();
        current - self.timestamp > timeout_seconds
    }
    
    /// 更新校验和
    fn update_checksum(&mut self) {
        let mut content = String::new();
        let mut keys: Vec<_> = self.data.keys().collect();
        keys.sort(); // 确保顺序一致
        
        for key in keys {
            content.push_str(key);
            content.push_str(self.data.get(key).unwrap());
        }
        content.push_str(&self.timestamp.to_string());
        
        // 简化的校验和计算（实际应用中应使用更安全的哈希算法）
        self.checksum = format!("{:x}", (content.len() as u32) * 31 + content.chars().map(|c| c as u32).sum::<u32>());
    }
    
    /// 验证校验和
    pub fn validate_checksum(&self) -> bool {
        let mut temp_session = self.clone();
        let original_checksum = temp_session.checksum.clone();
        temp_session.update_checksum();
        temp_session.checksum == original_checksum
    }
}

impl Default for SessionData {
    fn default() -> Self {
        Self::new()
    }
}

/// 客户端状态编码器
pub struct ClientStateEncoder {
    secret_key: String,
}

impl ClientStateEncoder {
    pub fn new(secret_key: String) -> Self {
        Self { secret_key }
    }
    
    /// 编码会话数据为客户端字符串
    pub fn encode(&self, session: &SessionData) -> Result<String, ClientSessionError> {
        // 验证数据完整性
        if !session.validate_checksum() {
            return Err(ClientSessionError::ValidationError("会话数据校验失败".to_string()));
        }
        
        // 序列化数据（简化实现）
        let mut encoded = String::new();
        encoded.push_str(&format!("ts:{};", session.timestamp));
        encoded.push_str(&format!("cs:{};", session.checksum));
        
        for (key, value) in &session.data {
            encoded.push_str(&format!("{}:{};", key, value));
        }
        
        // 简单的"加密"（实际应用中应使用真正的加密算法）
        let encrypted = self.simple_encrypt(&encoded)?;
        
        Ok(base64_encode(&encrypted))
    }
    
    /// 解码客户端字符串为会话数据
    pub fn decode(&self, encoded_data: &str) -> Result<SessionData, ClientSessionError> {
        // Base64解码
        let encrypted_data = base64_decode(encoded_data)
            .map_err(|e| ClientSessionError::DecodingError(format!("Base64解码失败: {}", e)))?;
        
        // 解密
        let decrypted = self.simple_decrypt(&encrypted_data)?;
        
        // 解析数据
        let mut session = SessionData::new();
        session.data.clear(); // 清空默认数据
        
        for part in decrypted.split(';') {
            if part.is_empty() {
                continue;
            }
            
            let kv: Vec<&str> = part.splitn(2, ':').collect();
            if kv.len() != 2 {
                continue;
            }
            
            let key = kv[0];
            let value = kv[1];
            
            match key {
                "ts" => {
                    session.timestamp = value.parse()
                        .map_err(|_| ClientSessionError::DecodingError("时间戳解析失败".to_string()))?;
                }
                "cs" => {
                    session.checksum = value.to_string();
                }
                _ => {
                    session.data.insert(key.to_string(), value.to_string());
                }
            }
        }
        
        // 验证校验和
        if !session.validate_checksum() {
            return Err(ClientSessionError::SecurityError("数据完整性验证失败".to_string()));
        }
        
        Ok(session)
    }
    
    /// 简单加密（实际应用中应使用AES等算法）
    fn simple_encrypt(&self, data: &str) -> Result<String, ClientSessionError> {
        let key_bytes = self.secret_key.as_bytes();
        let data_bytes = data.as_bytes();
        let mut result = Vec::new();
        
        for (i, &byte) in data_bytes.iter().enumerate() {
            let key_byte = key_bytes[i % key_bytes.len()];
            result.push(byte ^ key_byte);
        }
        
        Ok(result.iter().map(|b| format!("{:02x}", b)).collect())
    }
    
    /// 简单解密
    fn simple_decrypt(&self, encrypted: &str) -> Result<String, ClientSessionError> {
        if encrypted.len() % 2 != 0 {
            return Err(ClientSessionError::DecodingError("加密数据长度错误".to_string()));
        }
        
        let mut bytes = Vec::new();
        for i in (0..encrypted.len()).step_by(2) {
            let hex_byte = &encrypted[i..i+2];
            let byte = u8::from_str_radix(hex_byte, 16)
                .map_err(|_| ClientSessionError::DecodingError("十六进制解析失败".to_string()))?;
            bytes.push(byte);
        }
        
        let key_bytes = self.secret_key.as_bytes();
        let mut result = Vec::new();
        
        for (i, &byte) in bytes.iter().enumerate() {
            let key_byte = key_bytes[i % key_bytes.len()];
            result.push(byte ^ key_byte);
        }
        
        String::from_utf8(result)
            .map_err(|_| ClientSessionError::DecodingError("UTF-8解码失败".to_string()))
    }
}

/// HTTP表单会话状态管理器
pub struct FormSessionManager {
    encoder: ClientStateEncoder,
    timeout_seconds: u64,
}

impl FormSessionManager {
    pub fn new(secret_key: String, timeout_seconds: u64) -> Self {
        Self {
            encoder: ClientStateEncoder::new(secret_key),
            timeout_seconds,
        }
    }
    
    /// 创建隐藏字段HTML
    pub fn create_hidden_field(&self, session: &SessionData) -> Result<String, ClientSessionError> {
        let encoded = self.encoder.encode(session)?;
        Ok(format!(r#"<input type="hidden" name="session_state" value="{}" />"#, encoded))
    }
    
    /// 从表单数据恢复会话
    pub fn restore_from_form(&self, form_data: &HashMap<String, String>) -> Result<SessionData, ClientSessionError> {
        let session_state = form_data.get("session_state")
            .ok_or_else(|| ClientSessionError::DecodingError("表单中缺少会话状态数据".to_string()))?;
        
        let session = self.encoder.decode(session_state)?;
        
        // 检查会话是否过期
        if session.is_expired(self.timeout_seconds) {
            return Err(ClientSessionError::ValidationError("会话已过期".to_string()));
        }
        
        Ok(session)
    }
}

/// URL参数会话状态管理器
pub struct UrlSessionManager {
    encoder: ClientStateEncoder,
}

impl UrlSessionManager {
    pub fn new(secret_key: String) -> Self {
        Self {
            encoder: ClientStateEncoder::new(secret_key),
        }
    }
    
    /// 将会话数据添加到URL
    pub fn add_to_url(&self, base_url: &str, session: &SessionData) -> Result<String, ClientSessionError> {
        let encoded = self.encoder.encode(session)?;
        let separator = if base_url.contains('?') { "&" } else { "?" };
        Ok(format!("{}{}session={}", base_url, separator, url_encode(&encoded)))
    }
    
    /// 从URL参数恢复会话
    pub fn restore_from_url(&self, url: &str) -> Result<Option<SessionData>, ClientSessionError> {
        if let Some(query_start) = url.find('?') {
            let query = &url[query_start + 1..];
            let params: HashMap<String, String> = query
                .split('&')
                .filter_map(|param| {
                    let parts: Vec<&str> = param.splitn(2, '=').collect();
                    if parts.len() == 2 {
                        Some((parts[0].to_string(), url_decode(parts[1])))
                    } else {
                        None
                    }
                })
                .collect();
            
            if let Some(session_data) = params.get("session") {
                let session = self.encoder.decode(session_data)?;
                return Ok(Some(session));
            }
        }
        
        Ok(None)
    }
}

/// 购物车示例（使用客户端会话状态）
#[derive(Debug, Clone)]
pub struct ShoppingCartItem {
    pub product_id: String,
    pub name: String,
    pub price: f64,
    pub quantity: u32,
}

impl Display for ShoppingCartItem {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} x{} (¥{:.2})", self.name, self.quantity, self.price)
    }
}

pub struct ShoppingCart {
    session_manager: FormSessionManager,
}

impl ShoppingCart {
    pub fn new(secret_key: String) -> Self {
        Self {
            session_manager: FormSessionManager::new(secret_key, 3600), // 1小时超时
        }
    }
    
    /// 添加商品到购物车
    pub fn add_item(&self, session: &mut SessionData, item: &ShoppingCartItem) {
        let item_key = format!("item_{}", item.product_id);
        let item_data = format!("{}|{}|{:.2}|{}", item.product_id, item.name, item.price, item.quantity);
        session.set(&item_key, &item_data);
    }
    
    /// 从购物车移除商品
    pub fn remove_item(&self, session: &mut SessionData, product_id: &str) {
        let item_key = format!("item_{}", product_id);
        session.remove(&item_key);
    }
    
    /// 获取购物车中的所有商品
    pub fn get_items(&self, session: &SessionData) -> Vec<ShoppingCartItem> {
        let mut items = Vec::new();
        
        for (key, value) in session.get_all() {
            if key.starts_with("item_") {
                if let Some(item) = self.parse_item_data(value) {
                    items.push(item);
                }
            }
        }
        
        items
    }
    
    /// 计算购物车总价
    pub fn calculate_total(&self, session: &SessionData) -> f64 {
        self.get_items(session)
            .iter()
            .map(|item| item.price * item.quantity as f64)
            .sum()
    }
    
    /// 生成购物车表单HTML
    pub fn generate_cart_form(&self, session: &SessionData) -> Result<String, ClientSessionError> {
        let hidden_field = self.session_manager.create_hidden_field(session)?;
        let items = self.get_items(session);
        let total = self.calculate_total(session);
        
        let mut html = String::new();
        html.push_str("<form method=\"post\" action=\"/checkout\">\n");
        html.push_str(&hidden_field);
        html.push_str("\n<h3>购物车</h3>\n<ul>\n");
        
        for item in items {
            html.push_str(&format!("<li>{}</li>\n", item));
        }
        
        html.push_str("</ul>\n");
        html.push_str(&format!("<p><strong>总计: ¥{:.2}</strong></p>\n", total));
        html.push_str("<button type=\"submit\">结算</button>\n");
        html.push_str("</form>");
        
        Ok(html)
    }
    
    /// 解析商品数据
    fn parse_item_data(&self, data: &str) -> Option<ShoppingCartItem> {
        let parts: Vec<&str> = data.split('|').collect();
        if parts.len() == 4 {
            Some(ShoppingCartItem {
                product_id: parts[0].to_string(),
                name: parts[1].to_string(),
                price: parts[2].parse().ok()?,
                quantity: parts[3].parse().ok()?,
            })
        } else {
            None
        }
    }
}

// 辅助函数
fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

fn base64_encode(data: &str) -> String {
    // 简化的Base64编码实现
    data.chars().map(|c| ((c as u8) + 1) as char).collect()
}

fn base64_decode(data: &str) -> Result<String, String> {
    // 简化的Base64解码实现
    let result: String = data.chars().map(|c| ((c as u8) - 1) as char).collect();
    Ok(result)
}

fn url_encode(data: &str) -> String {
    // 简化的URL编码
    data.replace(" ", "%20").replace("&", "%26").replace("=", "%3D")
}

fn url_decode(data: &str) -> String {
    // 简化的URL解码
    data.replace("%20", " ").replace("%26", "&").replace("%3D", "=")
}

/// 客户端会话状态模式演示
pub fn demo() {
    println!("=== Client Session State（客户端会话状态）模式演示 ===\n");
    
    // 1. 基本会话数据操作
    println!("1. 基本会话数据操作:");
    let mut session = SessionData::new();
    
    session.set("user_id", "12345");
    session.set("username", "张三");
    session.set("step", "2");
    
    println!("用户ID: {}", session.get("user_id").unwrap());
    println!("用户名: {}", session.get("username").unwrap());
    println!("当前步骤: {}", session.get("step").unwrap());
    println!("会话时间戳: {}", session.timestamp());
    println!("校验和验证: {}", session.validate_checksum());
    
    println!("{}", "=".repeat(50));
    
    // 2. 会话数据编码/解码
    println!("2. 会话数据编码/解码:");
    let encoder = ClientStateEncoder::new("my_secret_key_123".to_string());
    
    match encoder.encode(&session) {
        Ok(encoded) => {
            println!("编码后的会话数据: {}", encoded);
            
            match encoder.decode(&encoded) {
                Ok(decoded_session) => {
                    println!("解码成功！");
                    println!("解码后用户ID: {}", decoded_session.get("user_id").unwrap());
                    println!("解码后用户名: {}", decoded_session.get("username").unwrap());
                    println!("数据完整性验证: {}", decoded_session.validate_checksum());
                }
                Err(e) => println!("解码失败: {}", e),
            }
        }
        Err(e) => println!("编码失败: {}", e),
    }
    
    println!("{}", "=".repeat(50));
    
    // 3. 表单会话状态管理
    println!("3. 表单会话状态管理:");
    let form_manager = FormSessionManager::new("form_secret_123".to_string(), 3600);
    
    let mut form_session = SessionData::new();
    form_session.set("form_step", "1");
    form_session.set("form_data", "用户输入的数据");
    
    match form_manager.create_hidden_field(&form_session) {
        Ok(hidden_field) => {
            println!("生成的隐藏字段HTML:");
            println!("{}", hidden_field);
            
            // 模拟从表单恢复会话
            let mut form_data = HashMap::new();
            // 这里应该从隐藏字段中提取值
            if let Ok(encoded) = encoder.encode(&form_session) {
                form_data.insert("session_state".to_string(), encoded);
                
                match form_manager.restore_from_form(&form_data) {
                    Ok(restored_session) => {
                        println!("从表单恢复会话成功！");
                        println!("表单步骤: {}", restored_session.get("form_step").unwrap());
                        println!("表单数据: {}", restored_session.get("form_data").unwrap());
                    }
                    Err(e) => println!("从表单恢复会话失败: {}", e),
                }
            }
        }
        Err(e) => println!("创建隐藏字段失败: {}", e),
    }
    
    println!("{}", "=".repeat(50));
    
    // 4. URL会话状态管理
    println!("4. URL会话状态管理:");
    let url_manager = UrlSessionManager::new("url_secret_456".to_string());
    
    let mut url_session = SessionData::new();
    url_session.set("page", "product_detail");
    url_session.set("category", "electronics");
    
    match url_manager.add_to_url("https://example.com/products", &url_session) {
        Ok(url_with_session) => {
            println!("带会话的URL: {}", url_with_session);
            
            match url_manager.restore_from_url(&url_with_session) {
                Ok(Some(restored_session)) => {
                    println!("从URL恢复会话成功！");
                    println!("页面: {}", restored_session.get("page").unwrap());
                    println!("分类: {}", restored_session.get("category").unwrap());
                }
                Ok(None) => println!("URL中没有会话数据"),
                Err(e) => println!("从URL恢复会话失败: {}", e),
            }
        }
        Err(e) => println!("添加会话到URL失败: {}", e),
    }
    
    println!("{}", "=".repeat(50));
    
    // 5. 购物车示例
    println!("5. 购物车示例（客户端状态）:");
    let shopping_cart = ShoppingCart::new("cart_secret_789".to_string());
    let mut cart_session = SessionData::new();
    
    // 添加商品到购物车
    let item1 = ShoppingCartItem {
        product_id: "prod_001".to_string(),
        name: "智能手机".to_string(),
        price: 2999.00,
        quantity: 1,
    };
    
    let item2 = ShoppingCartItem {
        product_id: "prod_002".to_string(),
        name: "无线耳机".to_string(),
        price: 299.00,
        quantity: 2,
    };
    
    shopping_cart.add_item(&mut cart_session, &item1);
    shopping_cart.add_item(&mut cart_session, &item2);
    
    println!("购物车商品:");
    for item in shopping_cart.get_items(&cart_session) {
        println!("  {}", item);
    }
    
    println!("购物车总价: ¥{:.2}", shopping_cart.calculate_total(&cart_session));
    
    // 生成购物车表单
    match shopping_cart.generate_cart_form(&cart_session) {
        Ok(form_html) => {
            println!("\n生成的购物车表单HTML:");
            println!("{}", form_html);
        }
        Err(e) => println!("生成表单失败: {}", e),
    }
    
    // 从购物车移除商品
    shopping_cart.remove_item(&mut cart_session, "prod_001");
    println!("\n移除智能手机后的购物车总价: ¥{:.2}", 
             shopping_cart.calculate_total(&cart_session));
    
    println!("\n=== Client Session State模式演示完成 ===");
    
    // 输出模式总结
    println!("\n【Client Session State模式总结】");
    println!("优点:");
    println!("1. 服务器无状态：易于扩展和负载均衡");
    println!("2. 减少服务器内存使用");
    println!("3. 不依赖服务器会话存储");
    println!("4. 支持浏览器前进/后退");
    
    println!("\n缺点:");
    println!("1. 增加网络传输量");
    println!("2. 存在安全风险（需要加密和签名）");
    println!("3. 有大小限制（表单字段、URL长度）");
    println!("4. 客户端可能被篡改");
    
    println!("\n适用场景:");
    println!("1. 多步骤表单提交");
    println!("2. 购物车状态管理");
    println!("3. 用户偏好设置");
    println!("4. 临时的交互状态");
    println!("5. 无状态服务架构");
}

/// 客户端会话状态模式演示（包装函数）
pub fn demo_client_session_state() {
    demo();
} 