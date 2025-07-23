/*
 * 文件路径: /d%3A/workspace/RustLearn/RustDesignPattern/src/DistributedSystemMode/ServiceDiscoveryPatterns/service_registry.rs
 * 
 * Service Registry模式 (服务注册表)
 * 
 * 服务注册表是一个数据库，用于存储和管理微服务实例的网络位置信息。
 * 服务在启动时向注册表注册自己，在停止时注销自己。
 * 客户端通过注册表发现和调用服务。
 */

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct ServiceInstance {
    pub service_id: String,
    pub instance_id: String,
    pub host: String,
    pub port: u16,
    pub metadata: HashMap<String, String>,
    pub health_check_url: String,
    pub registered_at: Instant,
    pub last_heartbeat: Instant,
    pub status: ServiceStatus,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ServiceStatus {
    Starting,
    Up,
    Down,
    Unknown,
}

pub struct ServiceRegistry {
    services: Arc<RwLock<HashMap<String, Vec<ServiceInstance>>>>,
    heartbeat_timeout: Duration,
}

impl ServiceRegistry {
    pub fn new() -> Self {
        Self {
            services: Arc::new(RwLock::new(HashMap::new())),
            heartbeat_timeout: Duration::from_secs(30),
        }
    }
    
    pub fn register(&self, instance: ServiceInstance) -> Result<(), String> {
        let mut services = self.services.write().unwrap();
        let service_instances = services.entry(instance.service_id.clone()).or_insert_with(Vec::new);
        service_instances.push(instance);
        Ok(())
    }
    
    pub fn deregister(&self, service_id: &str, instance_id: &str) -> Result<(), String> {
        let mut services = self.services.write().unwrap();
        if let Some(instances) = services.get_mut(service_id) {
            instances.retain(|i| i.instance_id != instance_id);
            if instances.is_empty() {
                services.remove(service_id);
            }
        }
        Ok(())
    }
    
    pub fn discover(&self, service_id: &str) -> Vec<ServiceInstance> {
        let services = self.services.read().unwrap();
        services.get(service_id).cloned().unwrap_or_default()
    }
    
    pub fn heartbeat(&self, service_id: &str, instance_id: &str) -> Result<(), String> {
        let mut services = self.services.write().unwrap();
        if let Some(instances) = services.get_mut(service_id) {
            for instance in instances.iter_mut() {
                if instance.instance_id == instance_id {
                    instance.last_heartbeat = Instant::now();
                    instance.status = ServiceStatus::Up;
                    return Ok(());
                }
            }
        }
        Err("Service instance not found".to_string())
    }
}

/// Service Registry模式演示
pub fn demo_service_registry() {
    println!("=== Service Registry模式演示 ===\n");
    
    let registry = ServiceRegistry::new();
    
    // 注册服务实例
    let instance1 = ServiceInstance {
        service_id: "user-service".to_string(),
        instance_id: "user-service-1".to_string(),
        host: "192.168.1.10".to_string(),
        port: 8080,
        metadata: HashMap::new(),
        health_check_url: "/health".to_string(),
        registered_at: Instant::now(),
        last_heartbeat: Instant::now(),
        status: ServiceStatus::Up,
    };
    
    registry.register(instance1).unwrap();
    println!("注册用户服务实例");
    
    // 服务发现
    let instances = registry.discover("user-service");
    println!("发现服务实例数量: {}", instances.len());
    
    println!("\n【Service Registry模式特点】");
    println!("✓ 服务注册 - 服务实例向注册表注册网络位置");
    println!("✓ 服务发现 - 客户端通过注册表发现服务位置");
    println!("✓ 健康检查 - 监控服务实例的健康状态");
    println!("✓ 负载均衡 - 支持多个服务实例的负载分发");
} 