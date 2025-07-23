/*
 * 文件路径: /d%3A/workspace/RustLearn/RustDesignPattern/src/DistributedSystemMode/ResiliencePatterns/mod.rs
 * 
 * 容错和弹性模式模块 (Resilience Patterns)
 */

pub mod circuit_breaker;
pub mod retry;

// 其他模式的存根实现
pub mod bulkhead {
    pub fn demo_bulkhead() {
        println!("=== Bulkhead模式演示 ===");
        println!("舱壁模式隔离资源池，防止级联故障");
    }
}

pub mod timeout {
    pub fn demo_timeout() {
        println!("=== Timeout模式演示 ===");
        println!("设置操作超时时间，避免无限等待");
    }
}

pub mod rate_limiting {
    pub fn demo_rate_limiting() {
        println!("=== Rate Limiting模式演示 ===");
        println!("限流模式控制请求速率，保护系统稳定");
    }
} 