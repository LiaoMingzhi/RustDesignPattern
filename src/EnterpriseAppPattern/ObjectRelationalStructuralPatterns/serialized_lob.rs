//! # 序列化LOB模式（Serialized LOB Pattern）
//!
//! 序列化LOB（Large Object）模式用于保存复杂对象图的图形到数据库字段中。
//! 通过将整个对象或对象图序列化为单个大对象（如JSON、XML或二进制数据），
//! 可以简化对象关系映射的复杂性。
//!
//! ## 模式特点
//! - **简化映射**: 避免复杂的对象关系映射
//! - **原子操作**: 整个对象图作为单个单元读写
//! - **版本控制**: 便于对象的版本管理
//! - **性能优化**: 减少数据库查询次数
//!
//! ## 使用场景
//! - 复杂对象图需要完整保存时
//! - 对象结构经常变化时
//! - 读取频繁但修改较少时
//! - 需要保持对象版本历史时

use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::fmt::{self, Display, Formatter};
use std::error::Error;
use std::time::{SystemTime, UNIX_EPOCH};

/// 序列化LOB错误类型
#[derive(Debug)]
pub enum SerializedLobError {
    SerializationError(String),
    DeserializationError(String),
    DatabaseError(String),
    CompressionError(String),
    VersionMismatch(String),
    ValidationError(String),
}

impl Display for SerializedLobError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            SerializedLobError::SerializationError(msg) => write!(f, "序列化错误: {}", msg),
            SerializedLobError::DeserializationError(msg) => write!(f, "反序列化错误: {}", msg),
            SerializedLobError::DatabaseError(msg) => write!(f, "数据库错误: {}", msg),
            SerializedLobError::CompressionError(msg) => write!(f, "压缩错误: {}", msg),
            SerializedLobError::VersionMismatch(msg) => write!(f, "版本不匹配: {}", msg),
            SerializedLobError::ValidationError(msg) => write!(f, "验证错误: {}", msg),
        }
    }
}

impl Error for SerializedLobError {}

/// 序列化格式
#[derive(Debug, Clone, PartialEq)]
pub enum SerializationFormat {
    Json,
    Xml,
    Binary,
    MessagePack,
}

impl Display for SerializationFormat {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let format_str = match self {
            SerializationFormat::Json => "JSON",
            SerializationFormat::Xml => "XML",
            SerializationFormat::Binary => "Binary",
            SerializationFormat::MessagePack => "MessagePack",
        };
        write!(f, "{}", format_str)
    }
}

/// LOB元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LobMetadata {
    pub id: String,
    pub format: String,
    pub version: u32,
    pub created_at: u64,
    pub updated_at: u64,
    pub size: usize,
    pub checksum: String,
    pub compression: String,
}

impl LobMetadata {
    pub fn new(id: String, format: SerializationFormat) -> Self {
        let now = current_timestamp();
        Self {
            id,
            format: format.to_string(),
            version: 1,
            created_at: now,
            updated_at: now,
            size: 0,
            checksum: String::new(),
            compression: "none".to_string(),
        }
    }

    pub fn update_version(&mut self) {
        self.version += 1;
        self.updated_at = current_timestamp();
    }
}

/// 序列化LOB实体
#[derive(Debug, Clone)]
pub struct SerializedLob {
    pub metadata: LobMetadata,
    pub data: Vec<u8>,
    pub is_compressed: bool,
}

impl SerializedLob {
    /// 创建新的序列化LOB
    pub fn new(id: String, format: SerializationFormat) -> Self {
        Self {
            metadata: LobMetadata::new(id, format),
            data: Vec::new(),
            is_compressed: false,
        }
    }

    /// 序列化对象到LOB
    pub fn serialize<T>(&mut self, object: &T, format: SerializationFormat) -> Result<(), SerializedLobError>
    where 
        T: Serialize,
    {
        let serialized_data = match format {
            SerializationFormat::Json => {
                serde_json::to_vec(object)
                    .map_err(|e| SerializedLobError::SerializationError(e.to_string()))?
            }
            SerializationFormat::Xml => {
                // 简化的XML序列化（实际应用中需要使用专门的XML库）
                let json_data = serde_json::to_string(object)
                    .map_err(|e| SerializedLobError::SerializationError(e.to_string()))?;
                format!("<xml>{}</xml>", json_data).into_bytes()
            }
            SerializationFormat::Binary => {
                // 简化的二进制序列化（实际应用中可能使用bincode等）
                serde_json::to_vec(object)
                    .map_err(|e| SerializedLobError::SerializationError(e.to_string()))?
            }
            SerializationFormat::MessagePack => {
                // 简化实现，实际需要使用rmp-serde等库
                serde_json::to_vec(object)
                    .map_err(|e| SerializedLobError::SerializationError(e.to_string()))?
            }
        };

        self.data = serialized_data;
        self.metadata.size = self.data.len();
        self.metadata.checksum = self.calculate_checksum();
        self.metadata.format = format.to_string();
        self.metadata.update_version();

        println!("✅ 对象序列化成功: {} bytes, 格式: {}", self.metadata.size, format);
        Ok(())
    }

    /// 从LOB反序列化对象
    pub fn deserialize<T>(&self) -> Result<T, SerializedLobError>
    where 
        T: for<'de> Deserialize<'de>,
    {
        if self.data.is_empty() {
            return Err(SerializedLobError::DeserializationError("LOB数据为空".to_string()));
        }

        // 验证校验和
        let current_checksum = self.calculate_checksum();
        if current_checksum != self.metadata.checksum {
            return Err(SerializedLobError::ValidationError("数据校验和不匹配".to_string()));
        }

        let format = match self.metadata.format.as_str() {
            "JSON" => SerializationFormat::Json,
            "XML" => SerializationFormat::Xml,
            "Binary" => SerializationFormat::Binary,
            "MessagePack" => SerializationFormat::MessagePack,
            _ => return Err(SerializedLobError::DeserializationError("未知序列化格式".to_string())),
        };

        let object = match format {
            SerializationFormat::Json => {
                serde_json::from_slice(&self.data)
                    .map_err(|e| SerializedLobError::DeserializationError(e.to_string()))?
            }
            SerializationFormat::Xml => {
                // 简化的XML反序列化
                let xml_str = String::from_utf8_lossy(&self.data);
                let json_content = xml_str.trim_start_matches("<xml>").trim_end_matches("</xml>");
                serde_json::from_str(json_content)
                    .map_err(|e| SerializedLobError::DeserializationError(e.to_string()))?
            }
            SerializationFormat::Binary => {
                serde_json::from_slice(&self.data)
                    .map_err(|e| SerializedLobError::DeserializationError(e.to_string()))?
            }
            SerializationFormat::MessagePack => {
                serde_json::from_slice(&self.data)
                    .map_err(|e| SerializedLobError::DeserializationError(e.to_string()))?
            }
        };

        println!("✅ 对象反序列化成功: 格式 {}", format);
        Ok(object)
    }

    /// 压缩LOB数据
    pub fn compress(&mut self) -> Result<(), SerializedLobError> {
        if self.is_compressed {
            return Ok(());
        }

        // 简化的压缩实现（实际应用中应使用专门的压缩库）
        let original_size = self.data.len();
        
        // 模拟压缩过程
        if original_size > 100 {
            // 对于大于100字节的数据进行"压缩"
            let compressed_data = format!("COMPRESSED[{}]:{}", original_size, 
                String::from_utf8_lossy(&self.data[..std::cmp::min(50, original_size)]));
            self.data = compressed_data.into_bytes();
            self.is_compressed = true;
            self.metadata.compression = "simple".to_string();
            
            let compression_ratio = (original_size as f64 - self.data.len() as f64) / original_size as f64 * 100.0;
            println!("🗜️ 数据压缩完成: {} -> {} bytes (压缩率: {:.1}%)", 
                     original_size, self.data.len(), compression_ratio);
        }

        self.metadata.size = self.data.len();
        self.metadata.checksum = self.calculate_checksum();
        Ok(())
    }

    /// 解压缩LOB数据
    pub fn decompress(&mut self) -> Result<(), SerializedLobError> {
        if !self.is_compressed {
            return Ok(());
        }

        // 简化的解压缩实现
        let compressed_str = String::from_utf8_lossy(&self.data);
        if compressed_str.starts_with("COMPRESSED[") {
            if let Some(end_bracket) = compressed_str.find(']') {
                if let Some(colon_pos) = compressed_str.find(':') {
                    let size_str = &compressed_str[11..end_bracket];
                    if let Ok(original_size) = size_str.parse::<usize>() {
                        let partial_data = &compressed_str[colon_pos + 1..];
                        // 模拟解压缩（实际应从完整压缩数据恢复）
                        let decompressed_data = format!("DECOMPRESSED_DATA[{}]:{}", original_size, partial_data);
                        self.data = decompressed_data.into_bytes();
                        self.is_compressed = false;
                        self.metadata.compression = "none".to_string();
                        
                        println!("🔓 数据解压缩完成: {} bytes", self.data.len());
                    }
                }
            }
        }

        self.metadata.size = self.data.len();
        self.metadata.checksum = self.calculate_checksum();
        Ok(())
    }

    /// 计算数据校验和
    fn calculate_checksum(&self) -> String {
        // 简化的校验和计算
        let sum: u32 = self.data.iter().map(|&b| b as u32).sum();
        format!("{:08x}", sum)
    }

    /// 获取数据大小
    pub fn size(&self) -> usize {
        self.data.len()
    }

    /// 验证数据完整性
    pub fn validate(&self) -> Result<(), SerializedLobError> {
        let current_checksum = self.calculate_checksum();
        if current_checksum != self.metadata.checksum {
            return Err(SerializedLobError::ValidationError("数据校验和验证失败".to_string()));
        }
        Ok(())
    }
}

impl Display for SerializedLob {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "SerializedLob[{}] - Format: {}, Version: {}, Size: {} bytes, Compressed: {}", 
               self.metadata.id, self.metadata.format, self.metadata.version, 
               self.metadata.size, self.is_compressed)
    }
}

/// 复杂业务对象示例
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CustomerProfile {
    pub id: String,
    pub personal_info: PersonalInfo,
    pub addresses: Vec<Address>,
    pub preferences: CustomerPreferences,
    pub purchase_history: Vec<PurchaseRecord>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PersonalInfo {
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub phone: String,
    pub birth_date: String,
    pub gender: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Address {
    pub address_type: String,
    pub street: String,
    pub city: String,
    pub state: String,
    pub country: String,
    pub postal_code: String,
    pub is_primary: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CustomerPreferences {
    pub language: String,
    pub currency: String,
    pub newsletter_subscription: bool,
    pub marketing_emails: bool,
    pub preferred_contact_method: String,
    pub interests: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PurchaseRecord {
    pub order_id: String,
    pub product_name: String,
    pub quantity: i32,
    pub unit_price: f64,
    pub purchase_date: String,
    pub category: String,
}

impl CustomerProfile {
    pub fn new(id: String, first_name: String, last_name: String, email: String) -> Self {
        Self {
            id,
            personal_info: PersonalInfo {
                first_name,
                last_name,
                email,
                phone: String::new(),
                birth_date: String::new(),
                gender: String::new(),
            },
            addresses: Vec::new(),
            preferences: CustomerPreferences {
                language: "zh-CN".to_string(),
                currency: "CNY".to_string(),
                newsletter_subscription: false,
                marketing_emails: false,
                preferred_contact_method: "email".to_string(),
                interests: Vec::new(),
            },
            purchase_history: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn add_address(&mut self, address: Address) {
        self.addresses.push(address);
    }

    pub fn add_purchase(&mut self, purchase: PurchaseRecord) {
        self.purchase_history.push(purchase);
    }

    pub fn update_preferences(&mut self, preferences: CustomerPreferences) {
        self.preferences = preferences;
    }
}

/// 序列化LOB仓储
pub struct SerializedLobRepository {
    storage: HashMap<String, SerializedLob>,
}

impl SerializedLobRepository {
    pub fn new() -> Self {
        Self {
            storage: HashMap::new(),
        }
    }

    /// 保存对象到LOB
    pub fn save<T>(&mut self, id: String, object: &T, format: SerializationFormat) -> Result<(), SerializedLobError>
    where 
        T: Serialize,
    {
        let mut lob = SerializedLob::new(id.clone(), format.clone());
        lob.serialize(object, format)?;
        
        // 如果数据较大，进行压缩
        if lob.size() > 1000 {
            lob.compress()?;
        }

        self.storage.insert(id, lob);
        println!("💾 对象保存到LOB仓储成功");
        Ok(())
    }

    /// 从LOB加载对象
    pub fn load<T>(&mut self, id: &str) -> Result<T, SerializedLobError>
    where 
        T: for<'de> Deserialize<'de>,
    {
        let lob = self.storage.get_mut(id)
            .ok_or_else(|| SerializedLobError::DatabaseError(format!("LOB {} 不存在", id)))?;

        // 验证数据完整性
        lob.validate()?;

        // 如果数据被压缩，先解压缩
        if lob.is_compressed {
            lob.decompress()?;
        }

        let object = lob.deserialize()?;
        println!("📂 从LOB仓储加载对象成功");
        Ok(object)
    }

    /// 更新LOB中的对象
    pub fn update<T>(&mut self, id: &str, object: &T) -> Result<(), SerializedLobError>
    where 
        T: Serialize,
    {
        let lob = self.storage.get_mut(id)
            .ok_or_else(|| SerializedLobError::DatabaseError(format!("LOB {} 不存在", id)))?;

        let format = match lob.metadata.format.as_str() {
            "JSON" => SerializationFormat::Json,
            "XML" => SerializationFormat::Xml,
            "Binary" => SerializationFormat::Binary,
            "MessagePack" => SerializationFormat::MessagePack,
            _ => SerializationFormat::Json,
        };

        lob.serialize(object, format)?;
        
        if lob.size() > 1000 {
            lob.compress()?;
        }

        println!("🔄 LOB对象更新成功");
        Ok(())
    }

    /// 删除LOB
    pub fn delete(&mut self, id: &str) -> Result<(), SerializedLobError> {
        self.storage.remove(id)
            .ok_or_else(|| SerializedLobError::DatabaseError(format!("LOB {} 不存在", id)))?;
        
        println!("🗑️ LOB删除成功: {}", id);
        Ok(())
    }

    /// 获取LOB元数据
    pub fn get_metadata(&self, id: &str) -> Result<LobMetadata, SerializedLobError> {
        let lob = self.storage.get(id)
            .ok_or_else(|| SerializedLobError::DatabaseError(format!("LOB {} 不存在", id)))?;
        
        Ok(lob.metadata.clone())
    }

    /// 列出所有LOB
    pub fn list_all(&self) -> Vec<String> {
        self.storage.keys().cloned().collect()
    }

    /// 获取存储统计信息
    pub fn get_statistics(&self) -> LobStatistics {
        let total_lobs = self.storage.len();
        let total_size: usize = self.storage.values().map(|lob| lob.size()).sum();
        let compressed_lobs = self.storage.values().filter(|lob| lob.is_compressed).count();

        let formats: HashMap<String, usize> = self.storage.values()
            .map(|lob| lob.metadata.format.clone())
            .fold(HashMap::new(), |mut acc, format| {
                *acc.entry(format).or_insert(0) += 1;
                acc
            });

        LobStatistics {
            total_lobs,
            total_size,
            compressed_lobs,
            formats,
        }
    }
}

/// LOB统计信息
#[derive(Debug)]
pub struct LobStatistics {
    pub total_lobs: usize,
    pub total_size: usize,
    pub compressed_lobs: usize,
    pub formats: HashMap<String, usize>,
}

impl Display for LobStatistics {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "LOB统计 - 总数: {}, 总大小: {} bytes, 压缩数: {}, 格式分布: {:?}", 
               self.total_lobs, self.total_size, self.compressed_lobs, self.formats)
    }
}

/// 获取当前时间戳
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

/// 演示序列化LOB模式
pub fn demo() {
    println!("=== 序列化LOB模式演示 ===\n");

    let mut repository = SerializedLobRepository::new();

    // 创建复杂的客户档案对象
    println!("1. 创建复杂客户档案对象");
    let mut customer_profile = CustomerProfile::new(
        "cust001".to_string(), 
        "张".to_string(), 
        "三".to_string(), 
        "zhang.san@example.com".to_string()
    );

    // 添加地址
    customer_profile.add_address(Address {
        address_type: "home".to_string(),
        street: "北京市朝阳区建国路88号".to_string(),
        city: "北京".to_string(),
        state: "北京".to_string(),
        country: "中国".to_string(),
        postal_code: "100022".to_string(),
        is_primary: true,
    });

    customer_profile.add_address(Address {
        address_type: "work".to_string(),
        street: "北京市海淀区中关村大街1号".to_string(),
        city: "北京".to_string(),
        state: "北京".to_string(),
        country: "中国".to_string(),
        postal_code: "100080".to_string(),
        is_primary: false,
    });

    // 添加购买记录
    customer_profile.add_purchase(PurchaseRecord {
        order_id: "order001".to_string(),
        product_name: "笔记本电脑".to_string(),
        quantity: 1,
        unit_price: 8999.0,
        purchase_date: "2024-01-15".to_string(),
        category: "电子产品".to_string(),
    });

    customer_profile.add_purchase(PurchaseRecord {
        order_id: "order002".to_string(),
        product_name: "无线鼠标".to_string(),
        quantity: 2,
        unit_price: 199.0,
        purchase_date: "2024-01-20".to_string(),
        category: "电子配件".to_string(),
    });

    // 更新偏好设置
    let mut preferences = customer_profile.preferences.clone();
    preferences.interests = vec!["科技".to_string(), "数码".to_string(), "编程".to_string()];
    preferences.newsletter_subscription = true;
    preferences.marketing_emails = true;
    customer_profile.update_preferences(preferences);

    println!("   客户档案创建完成: {} 地址, {} 购买记录", 
             customer_profile.addresses.len(), customer_profile.purchase_history.len());

    // 演示不同格式的序列化
    println!("\n2. 测试不同序列化格式");
    
    // JSON格式
    match repository.save("profile_json".to_string(), &customer_profile, SerializationFormat::Json) {
        Ok(_) => println!("   JSON格式保存成功"),
        Err(e) => println!("   JSON格式保存失败: {}", e),
    }

    // XML格式
    match repository.save("profile_xml".to_string(), &customer_profile, SerializationFormat::Xml) {
        Ok(_) => println!("   XML格式保存成功"),
        Err(e) => println!("   XML格式保存失败: {}", e),
    }

    // Binary格式
    match repository.save("profile_binary".to_string(), &customer_profile, SerializationFormat::Binary) {
        Ok(_) => println!("   Binary格式保存成功"),
        Err(e) => println!("   Binary格式保存失败: {}", e),
    }

    // 演示数据压缩
    println!("\n3. 测试数据压缩");
    let mut large_profile = customer_profile.clone();
    
    // 添加更多数据使其超过压缩阈值
    for i in 0..50 {
        large_profile.add_purchase(PurchaseRecord {
            order_id: format!("order{:03}", i + 3),
            product_name: format!("产品{}", i),
            quantity: 1,
            unit_price: 100.0 + i as f64,
            purchase_date: "2024-02-01".to_string(),
            category: "测试类别".to_string(),
        });
    }

    match repository.save("large_profile".to_string(), &large_profile, SerializationFormat::Json) {
        Ok(_) => println!("   大对象保存成功（自动压缩）"),
        Err(e) => println!("   大对象保存失败: {}", e),
    }

    // 演示数据加载和反序列化
    println!("\n4. 测试数据加载");
    match repository.load::<CustomerProfile>("profile_json") {
        Ok(loaded_profile) => {
            println!("   JSON格式加载成功");
            println!("   加载的客户: {} {}", 
                     loaded_profile.personal_info.first_name,
                     loaded_profile.personal_info.last_name);
            println!("   地址数量: {}", loaded_profile.addresses.len());
            println!("   购买记录数量: {}", loaded_profile.purchase_history.len());
            
            // 验证数据完整性
            if loaded_profile == customer_profile {
                println!("   ✅ 数据完整性验证通过");
            } else {
                println!("   ❌ 数据完整性验证失败");
            }
        }
        Err(e) => println!("   JSON格式加载失败: {}", e),
    }

    // 演示对象更新
    println!("\n5. 测试对象更新");
    let mut updated_profile = customer_profile.clone();
    updated_profile.personal_info.phone = "13800138000".to_string();
    updated_profile.metadata.insert("last_login".to_string(), "2024-01-25T10:30:00Z".to_string());
    updated_profile.metadata.insert("login_count".to_string(), "42".to_string());

    match repository.update("profile_json", &updated_profile) {
        Ok(_) => println!("   对象更新成功"),
        Err(e) => println!("   对象更新失败: {}", e),
    }

    // 获取元数据信息
    println!("\n6. 查看LOB元数据");
    for lob_id in repository.list_all() {
        if let Ok(metadata) = repository.get_metadata(&lob_id) {
            println!("   LOB[{}]: 格式={}, 版本={}, 大小={} bytes", 
                     lob_id, metadata.format, metadata.version, metadata.size);
        }
    }

    // 显示统计信息
    println!("\n7. 存储统计信息");
    let stats = repository.get_statistics();
    println!("   {}", stats);

    // 演示版本控制
    println!("\n8. 版本控制演示");
    if let Ok(mut metadata) = repository.get_metadata("profile_json") {
        println!("   当前版本: {}", metadata.version);
        println!("   创建时间: {}", metadata.created_at);
        println!("   更新时间: {}", metadata.updated_at);
    }

    println!("\n=== 序列化LOB模式演示完成 ===");

    println!("\n💡 序列化LOB模式的优势:");
    println!("1. 简化映射 - 避免复杂的对象关系映射");
    println!("2. 原子操作 - 整个对象图作为单个单元操作");
    println!("3. 版本控制 - 便于管理对象的版本历史");
    println!("4. 性能优化 - 减少数据库查询和连接次数");
    println!("5. 数据压缩 - 支持大对象的压缩存储");

    println!("\n⚠️ 使用注意事项:");
    println!("1. 查询限制 - 无法对序列化数据进行复杂查询");
    println!("2. 存储开销 - 可能产生数据冗余");
    println!("3. 版本兼容 - 需要考虑序列化格式的向后兼容性");
    println!("4. 并发控制 - 需要在应用层实现并发控制");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialized_lob_json() {
        let mut repository = SerializedLobRepository::new();
        let profile = CustomerProfile::new(
            "test001".to_string(),
            "测试".to_string(),
            "用户".to_string(),
            "test@example.com".to_string()
        );

        // 保存
        assert!(repository.save("test_profile".to_string(), &profile, SerializationFormat::Json).is_ok());

        // 加载
        let loaded_profile: Result<CustomerProfile, _> = repository.load("test_profile");
        assert!(loaded_profile.is_ok());
        assert_eq!(loaded_profile.unwrap().id, profile.id);
    }

    #[test]
    fn test_lob_compression() {
        let mut lob = SerializedLob::new("test".to_string(), SerializationFormat::Json);
        
        // 创建大数据
        let large_data = vec![65u8; 1500]; // 1500个'A'
        lob.data = large_data;
        lob.metadata.size = lob.data.len();
        lob.metadata.checksum = lob.calculate_checksum();

        // 压缩
        assert!(lob.compress().is_ok());
        assert!(lob.is_compressed);

        // 解压缩
        assert!(lob.decompress().is_ok());
        assert!(!lob.is_compressed);
    }

    #[test]
    fn test_lob_validation() {
        let mut lob = SerializedLob::new("test".to_string(), SerializationFormat::Json);
        lob.data = b"test data".to_vec();
        lob.metadata.size = lob.data.len();
        lob.metadata.checksum = lob.calculate_checksum();

        // 验证应该成功
        assert!(lob.validate().is_ok());

        // 修改数据后验证应该失败
        lob.data.push(b'!');
        assert!(lob.validate().is_err());
    }
} 