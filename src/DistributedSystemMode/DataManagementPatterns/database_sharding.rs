/*
 * 文件路径: /d%3A/workspace/RustLearn/RustDesignPattern/src/DistributedSystemMode/DataManagementPatterns/database_sharding.rs
 * 
 * Database Sharding模式 (数据库分片)
 * 
 * 数据库分片是一种水平扩展数据库的技术，将大型数据库分割成多个较小的、
 * 更易管理的片段。每个分片都是一个独立的数据库，包含原始数据的一个子集。
 * 
 * 主要特点：
 * 1. 水平扩展 - 通过增加更多服务器来处理更多数据
 * 2. 负载分散 - 将数据和查询负载分散到多个节点
 * 3. 并行处理 - 多个分片可以并行处理查询
 * 4. 容错性 - 单个分片故障不会影响其他分片
 * 5. 性能提升 - 减少单个数据库的负载
 */

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::fmt;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

// =================
// 分片策略
// =================

/// 分片策略特质
pub trait ShardingStrategy: Send + Sync {
    fn get_shard_key(&self, key: &str) -> usize;
    fn get_shard_count(&self) -> usize;
}

/// 哈希分片策略
pub struct HashShardingStrategy {
    shard_count: usize,
}

impl HashShardingStrategy {
    pub fn new(shard_count: usize) -> Self {
        Self { shard_count }
    }
}

impl ShardingStrategy for HashShardingStrategy {
    fn get_shard_key(&self, key: &str) -> usize {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        (hasher.finish() as usize) % self.shard_count
    }
    
    fn get_shard_count(&self) -> usize {
        self.shard_count
    }
}

/// 范围分片策略
pub struct RangeShardingStrategy {
    ranges: Vec<(String, String)>, // (start, end) 范围
}

impl RangeShardingStrategy {
    pub fn new(ranges: Vec<(String, String)>) -> Self {
        Self { ranges }
    }
}

impl ShardingStrategy for RangeShardingStrategy {
    fn get_shard_key(&self, key: &str) -> usize {
        for (i, (start, end)) in self.ranges.iter().enumerate() {
            if key >= start && key < end {
                return i;
            }
        }
        0 // 默认返回第一个分片
    }
    
    fn get_shard_count(&self) -> usize {
        self.ranges.len()
    }
}

/// 目录分片策略（基于键的前缀）
pub struct DirectoryShardingStrategy {
    shard_count: usize,
}

impl DirectoryShardingStrategy {
    pub fn new(shard_count: usize) -> Self {
        Self { shard_count }
    }
}

impl ShardingStrategy for DirectoryShardingStrategy {
    fn get_shard_key(&self, key: &str) -> usize {
        if let Some(first_char) = key.chars().next() {
            (first_char as usize) % self.shard_count
        } else {
            0
        }
    }
    
    fn get_shard_count(&self) -> usize {
        self.shard_count
    }
}

// =================
// 分片错误处理
// =================

#[derive(Debug, Clone)]
pub enum ShardingError {
    ShardNotFound,
    AllShardsDown,
    ReplicationFailed,
    ConsistencyError,
    RebalancingInProgress,
}

impl fmt::Display for ShardingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ShardingError::ShardNotFound => write!(f, "分片未找到"),
            ShardingError::AllShardsDown => write!(f, "所有分片都不可用"),
            ShardingError::ReplicationFailed => write!(f, "数据复制失败"),
            ShardingError::ConsistencyError => write!(f, "数据一致性错误"),
            ShardingError::RebalancingInProgress => write!(f, "分片重平衡进行中"),
        }
    }
}

pub type ShardResult<T> = Result<T, ShardingError>;

// =================
// 分片数据实体
// =================

#[derive(Debug, Clone)]
pub struct ShardedUser {
    pub id: String,
    pub name: String,
    pub email: String,
    pub region: String,
    pub created_at: u64,
}

#[derive(Debug, Clone)]
pub struct ShardedOrder {
    pub id: String,
    pub user_id: String,
    pub product_id: String,
    pub amount: f64,
    pub status: String,
    pub created_at: u64,
}

// =================
// 单个分片实现
// =================

/// 单个数据分片
pub struct DataShard<T> {
    id: usize,
    name: String,
    data: Arc<RwLock<HashMap<String, T>>>,
    is_healthy: Arc<RwLock<bool>>,
    replica_count: usize,
}

impl<T: Clone + Send + Sync> DataShard<T> {
    pub fn new(id: usize, name: String, replica_count: usize) -> Self {
        Self {
            id,
            name,
            data: Arc::new(RwLock::new(HashMap::new())),
            is_healthy: Arc::new(RwLock::new(true)),
            replica_count,
        }
    }
    
    pub fn get_id(&self) -> usize {
        self.id
    }
    
    pub fn get_name(&self) -> &str {
        &self.name
    }
    
    pub fn is_healthy(&self) -> bool {
        *self.is_healthy.read().unwrap()
    }
    
    pub fn set_health(&self, healthy: bool) {
        *self.is_healthy.write().unwrap() = healthy;
    }
    
    pub fn get(&self, key: &str) -> Option<T> {
        if !self.is_healthy() {
            return None;
        }
        let data = self.data.read().unwrap();
        data.get(key).cloned()
    }
    
    pub fn put(&self, key: String, value: T) -> ShardResult<()> {
        if !self.is_healthy() {
            return Err(ShardingError::ShardNotFound);
        }
        
        let mut data = self.data.write().unwrap();
        data.insert(key, value);
        Ok(())
    }
    
    pub fn delete(&self, key: &str) -> ShardResult<bool> {
        if !self.is_healthy() {
            return Err(ShardingError::ShardNotFound);
        }
        
        let mut data = self.data.write().unwrap();
        Ok(data.remove(key).is_some())
    }
    
    pub fn get_all(&self) -> Vec<T> {
        if !self.is_healthy() {
            return Vec::new();
        }
        let data = self.data.read().unwrap();
        data.values().cloned().collect()
    }
    
    pub fn get_size(&self) -> usize {
        let data = self.data.read().unwrap();
        data.len()
    }
    
    pub fn get_stats(&self) -> ShardStats {
        let data = self.data.read().unwrap();
        ShardStats {
            shard_id: self.id,
            shard_name: self.name.clone(),
            record_count: data.len(),
            is_healthy: self.is_healthy(),
            replica_count: self.replica_count,
        }
    }
}

// =================
// 分片统计信息
// =================

#[derive(Debug, Clone)]
pub struct ShardStats {
    pub shard_id: usize,
    pub shard_name: String,
    pub record_count: usize,
    pub is_healthy: bool,
    pub replica_count: usize,
}

// =================
// 分片集群管理器
// =================

/// 分片集群管理器
pub struct ShardedCluster<T> {
    shards: Vec<Arc<DataShard<T>>>,
    strategy: Box<dyn ShardingStrategy>,
    replication_factor: usize,
}

impl<T: Clone + Send + Sync + 'static> ShardedCluster<T> {
    pub fn new(shard_count: usize, strategy: Box<dyn ShardingStrategy>, replication_factor: usize) -> Self {
        let mut shards = Vec::new();
        for i in 0..shard_count {
            let shard_name = format!("shard_{}", i);
            shards.push(Arc::new(DataShard::new(i, shard_name, replication_factor)));
        }
        
        Self {
            shards,
            strategy,
            replication_factor,
        }
    }
    
    pub fn get(&self, key: &str) -> ShardResult<Option<T>> {
        let shard_key = self.strategy.get_shard_key(key);
        
        if let Some(shard) = self.shards.get(shard_key) {
            if shard.is_healthy() {
                Ok(shard.get(key))
            } else {
                // 尝试从副本读取
                self.try_read_from_replicas(key, shard_key)
            }
        } else {
            Err(ShardingError::ShardNotFound)
        }
    }
    
    pub fn put(&self, key: String, value: T) -> ShardResult<()> {
        let shard_key = self.strategy.get_shard_key(&key);
        
        if let Some(primary_shard) = self.shards.get(shard_key) {
            if primary_shard.is_healthy() {
                primary_shard.put(key.clone(), value.clone())?;
                
                // 复制到副本分片
                self.replicate_to_replicas(&key, &value, shard_key)?;
                
                Ok(())
            } else {
                Err(ShardingError::ShardNotFound)
            }
        } else {
            Err(ShardingError::ShardNotFound)
        }
    }
    
    pub fn delete(&self, key: &str) -> ShardResult<bool> {
        let shard_key = self.strategy.get_shard_key(key);
        
        if let Some(shard) = self.shards.get(shard_key) {
            if shard.is_healthy() {
                let result = shard.delete(key)?;
                
                // 从副本中删除
                self.delete_from_replicas(key, shard_key)?;
                
                Ok(result)
            } else {
                Err(ShardingError::ShardNotFound)
            }
        } else {
            Err(ShardingError::ShardNotFound)
        }
    }
    
    pub fn get_all(&self) -> Vec<T> {
        let mut all_data = Vec::new();
        for shard in &self.shards {
            if shard.is_healthy() {
                all_data.extend(shard.get_all());
            }
        }
        all_data
    }
    
    pub fn get_cluster_stats(&self) -> Vec<ShardStats> {
        self.shards.iter().map(|shard| shard.get_stats()).collect()
    }
    
    pub fn get_healthy_shard_count(&self) -> usize {
        self.shards.iter().filter(|shard| shard.is_healthy()).count()
    }
    
    pub fn set_shard_health(&self, shard_id: usize, healthy: bool) {
        if let Some(shard) = self.shards.get(shard_id) {
            shard.set_health(healthy);
        }
    }
    
    // 尝试从副本读取数据
    fn try_read_from_replicas(&self, key: &str, primary_shard_key: usize) -> ShardResult<Option<T>> {
        for i in 1..=self.replication_factor {
            let replica_key = (primary_shard_key + i) % self.shards.len();
            if let Some(replica_shard) = self.shards.get(replica_key) {
                if replica_shard.is_healthy() {
                    if let Some(value) = replica_shard.get(key) {
                        return Ok(Some(value));
                    }
                }
            }
        }
        Err(ShardingError::AllShardsDown)
    }
    
    // 复制数据到副本分片
    fn replicate_to_replicas(&self, key: &str, value: &T, primary_shard_key: usize) -> ShardResult<()> {
        let mut successful_replicas = 0;
        
        for i in 1..=self.replication_factor {
            let replica_key = (primary_shard_key + i) % self.shards.len();
            if let Some(replica_shard) = self.shards.get(replica_key) {
                if replica_shard.is_healthy() {
                    if replica_shard.put(key.to_string(), value.clone()).is_ok() {
                        successful_replicas += 1;
                    }
                }
            }
        }
        
        // 至少需要一半的副本写入成功
        if successful_replicas >= self.replication_factor / 2 {
            Ok(())
        } else {
            Err(ShardingError::ReplicationFailed)
        }
    }
    
    // 从副本中删除数据
    fn delete_from_replicas(&self, key: &str, primary_shard_key: usize) -> ShardResult<()> {
        for i in 1..=self.replication_factor {
            let replica_key = (primary_shard_key + i) % self.shards.len();
            if let Some(replica_shard) = self.shards.get(replica_key) {
                if replica_shard.is_healthy() {
                    let _ = replica_shard.delete(key); // 忽略删除错误
                }
            }
        }
        Ok(())
    }
}

// =================
// 演示函数
// =================

/// Database Sharding模式演示
pub fn demo_database_sharding() {
    println!("=== Database Sharding模式演示 ===\n");
    
    // 1. 哈希分片策略演示
    println!("1. 哈希分片策略演示:");
    let hash_strategy = Box::new(HashShardingStrategy::new(4));
    let user_cluster = ShardedCluster::new(4, hash_strategy, 2);
    
    // 插入用户数据
    let users = vec![
        ShardedUser {
            id: "user_001".to_string(),
            name: "张三".to_string(),
            email: "zhangsan@example.com".to_string(),
            region: "北京".to_string(),
            created_at: 1672531200,
        },
        ShardedUser {
            id: "user_002".to_string(),
            name: "李四".to_string(),
            email: "lisi@example.com".to_string(),
            region: "上海".to_string(),
            created_at: 1672531300,
        },
        ShardedUser {
            id: "user_003".to_string(),
            name: "王五".to_string(),
            email: "wangwu@example.com".to_string(),
            region: "广州".to_string(),
            created_at: 1672531400,
        },
        ShardedUser {
            id: "user_004".to_string(),
            name: "赵六".to_string(),
            email: "zhaoliu@example.com".to_string(),
            region: "深圳".to_string(),
            created_at: 1672531500,
        },
    ];
    
    for user in users {
        let key = user.id.clone();
        match user_cluster.put(key.clone(), user.clone()) {
            Ok(_) => println!("插入用户: {} 到分片", key),
            Err(e) => println!("插入失败: {}", e),
        }
    }
    
    // 查询数据
    println!("\n查询用户数据:");
    match user_cluster.get("user_001") {
        Ok(Some(user)) => println!("找到用户: {} - {}", user.name, user.email),
        Ok(None) => println!("用户不存在"),
        Err(e) => println!("查询失败: {}", e),
    }
    
    // 2. 范围分片策略演示
    println!("\n2. 范围分片策略演示:");
    let ranges = vec![
        ("order_0000".to_string(), "order_2500".to_string()),
        ("order_2500".to_string(), "order_5000".to_string()),
        ("order_5000".to_string(), "order_7500".to_string()),
        ("order_7500".to_string(), "order_9999".to_string()),
    ];
    let range_strategy = Box::new(RangeShardingStrategy::new(ranges));
    let order_cluster = ShardedCluster::new(4, range_strategy, 1);
    
    // 插入订单数据
    let orders = vec![
        ShardedOrder {
            id: "order_0100".to_string(),
            user_id: "user_001".to_string(),
            product_id: "product_001".to_string(),
            amount: 299.99,
            status: "pending".to_string(),
            created_at: 1672531600,
        },
        ShardedOrder {
            id: "order_3000".to_string(),
            user_id: "user_002".to_string(),
            product_id: "product_002".to_string(),
            amount: 599.99,
            status: "confirmed".to_string(),
            created_at: 1672531700,
        },
        ShardedOrder {
            id: "order_6000".to_string(),
            user_id: "user_003".to_string(),
            product_id: "product_003".to_string(),
            amount: 899.99,
            status: "shipped".to_string(),
            created_at: 1672531800,
        },
        ShardedOrder {
            id: "order_8000".to_string(),
            user_id: "user_004".to_string(),
            product_id: "product_004".to_string(),
            amount: 1299.99,
            status: "delivered".to_string(),
            created_at: 1672531900,
        },
    ];
    
    for order in orders {
        let key = order.id.clone();
        match order_cluster.put(key.clone(), order) {
            Ok(_) => println!("插入订单: {} 到范围分片", key),
            Err(e) => println!("插入失败: {}", e),
        }
    }
    
    // 3. 集群统计信息
    println!("\n3. 集群统计信息:");
    println!("用户集群统计:");
    let user_stats = user_cluster.get_cluster_stats();
    for stats in user_stats {
        println!("  分片 {}: {} 条记录, 健康状态: {}", 
                 stats.shard_name, stats.record_count, 
                 if stats.is_healthy { "健康" } else { "故障" });
    }
    
    println!("订单集群统计:");
    let order_stats = order_cluster.get_cluster_stats();
    for stats in order_stats {
        println!("  分片 {}: {} 条记录, 健康状态: {}", 
                 stats.shard_name, stats.record_count, 
                 if stats.is_healthy { "健康" } else { "故障" });
    }
    
    // 4. 容错性演示
    println!("\n4. 容错性演示:");
    println!("模拟分片故障...");
    user_cluster.set_shard_health(0, false);
    
    match user_cluster.get("user_001") {
        Ok(Some(user)) => println!("从副本读取用户: {} - {}", user.name, user.email),
        Ok(None) => println!("用户不存在"),
        Err(e) => println!("读取失败: {}", e),
    }
    
    println!("恢复分片健康状态...");
    user_cluster.set_shard_health(0, true);
    
    // 5. 性能统计
    println!("\n5. 性能统计:");
    println!("用户集群:");
    println!("  总分片数: {}", user_cluster.shards.len());
    println!("  健康分片数: {}", user_cluster.get_healthy_shard_count());
    println!("  总记录数: {}", user_cluster.get_all().len());
    
    println!("订单集群:");
    println!("  总分片数: {}", order_cluster.shards.len());
    println!("  健康分片数: {}", order_cluster.get_healthy_shard_count());
    println!("  总记录数: {}", order_cluster.get_all().len());
    
    println!("\n【Database Sharding模式特点】");
    println!("✓ 水平扩展 - 通过增加更多服务器来处理更多数据");
    println!("✓ 负载分散 - 将数据和查询负载分散到多个节点");
    println!("✓ 并行处理 - 多个分片可以并行处理查询");
    println!("✓ 容错性 - 单个分片故障不会影响其他分片");
    println!("✓ 性能提升 - 减少单个数据库的负载");
} 