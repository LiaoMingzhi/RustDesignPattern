/*
 * 文件路径: /d%3A/workspace/RustLearn/RustDesignPattern/src/DistributedSystemMode/LoadBalancingPatterns/load_balancer.rs
 * 
 * Load Balancer模式 (负载均衡器)
 * 
 * 负载均衡器将传入的请求分发到多个后端服务实例，
 * 以提高系统的可用性、性能和可扩展性。
 */

use std::sync::{Arc, Mutex};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Server {
    pub id: String,
    pub host: String,
    pub port: u16,
    pub weight: u32,
    pub active_connections: u32,
    pub is_healthy: bool,
}

pub trait LoadBalancingStrategy: Send + Sync {
    fn select_server(&self, servers: &[Server]) -> Option<usize>;
}

pub struct RoundRobinStrategy {
    current_index: Arc<Mutex<usize>>,
}

impl RoundRobinStrategy {
    pub fn new() -> Self {
        Self {
            current_index: Arc::new(Mutex::new(0)),
        }
    }
}

impl LoadBalancingStrategy for RoundRobinStrategy {
    fn select_server(&self, servers: &[Server]) -> Option<usize> {
        let healthy_servers: Vec<(usize, &Server)> = servers
            .iter()
            .enumerate()
            .filter(|(_, server)| server.is_healthy)
            .collect();
            
        if healthy_servers.is_empty() {
            return None;
        }
        
        let mut index = self.current_index.lock().unwrap();
        *index = (*index + 1) % healthy_servers.len();
        Some(healthy_servers[*index].0)
    }
}

pub struct LoadBalancer {
    servers: Vec<Server>,
    strategy: Box<dyn LoadBalancingStrategy>,
}

impl LoadBalancer {
    pub fn new(strategy: Box<dyn LoadBalancingStrategy>) -> Self {
        Self {
            servers: Vec::new(),
            strategy,
        }
    }
    
    pub fn add_server(&mut self, server: Server) {
        self.servers.push(server);
    }
    
    pub fn get_server(&self) -> Option<&Server> {
        let index = self.strategy.select_server(&self.servers)?;
        self.servers.get(index)
    }
    
    pub fn set_server_health(&mut self, server_id: &str, healthy: bool) {
        for server in &mut self.servers {
            if server.id == server_id {
                server.is_healthy = healthy;
                break;
            }
        }
    }
}

/// Load Balancer模式演示
pub fn demo_load_balancer() {
    println!("=== Load Balancer模式演示 ===\n");
    
    let mut load_balancer = LoadBalancer::new(Box::new(RoundRobinStrategy::new()));
    
    // 添加服务器
    load_balancer.add_server(Server {
        id: "server1".to_string(),
        host: "192.168.1.10".to_string(),
        port: 8080,
        weight: 1,
        active_connections: 0,
        is_healthy: true,
    });
    
    load_balancer.add_server(Server {
        id: "server2".to_string(),
        host: "192.168.1.11".to_string(),
        port: 8080,
        weight: 1,
        active_connections: 0,
        is_healthy: true,
    });
    
    // 模拟请求分发
    for i in 1..=6 {
        if let Some(server) = load_balancer.get_server() {
            println!("请求{} -> 服务器: {}:{}", i, server.host, server.port);
        }
    }
    
    println!("\n【Load Balancer模式特点】");
    println!("✓ 请求分发 - 将请求分发到多个后端服务");
    println!("✓ 健康检查 - 只向健康的服务器发送请求");
    println!("✓ 多种策略 - 支持轮询、加权、最少连接等算法");
    println!("✓ 故障转移 - 自动处理服务器故障");
} 