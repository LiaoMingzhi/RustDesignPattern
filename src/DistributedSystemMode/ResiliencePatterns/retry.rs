/*
 * 文件路径: /d%3A/workspace/RustLearn/RustDesignPattern/src/DistributedSystemMode/ResiliencePatterns/retry.rs
 * 
 * Retry模式 (重试)
 * 
 * 重试模式用于处理瞬时故障，通过重复执行失败的操作来提高系统的可靠性。
 * 包含指数退避、抖动等策略来优化重试行为。
 */

use std::time::{Duration, Instant};
use std::fmt;

#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub base_delay: Duration,
    pub max_delay: Duration,
    pub multiplier: f64,
    pub jitter: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            multiplier: 2.0,
            jitter: true,
        }
    }
}

pub struct RetryExecutor {
    config: RetryConfig,
}

impl RetryExecutor {
    pub fn new(config: RetryConfig) -> Self {
        Self { config }
    }
    
    pub fn execute<T, E, F>(&self, mut operation: F) -> Result<T, E>
    where
        F: FnMut() -> Result<T, E>,
        E: fmt::Debug,
    {
        let mut attempt = 1;
        let mut delay = self.config.base_delay;
        
        loop {
            match operation() {
                Ok(result) => return Ok(result),
                Err(error) => {
                    if attempt >= self.config.max_attempts {
                        return Err(error);
                    }
                    
                    println!("重试第{}次失败，{}ms后重试", attempt, delay.as_millis());
                    std::thread::sleep(delay);
                    
                    // 计算下次延迟时间
                    delay = Duration::from_millis(
                        ((delay.as_millis() as f64) * self.config.multiplier) as u64
                    ).min(self.config.max_delay);
                    
                    // 添加抖动
                    if self.config.jitter {
                        let jitter_ms = (delay.as_millis() as f64 * 0.1) as u64;
                        delay += Duration::from_millis(jitter_ms);
                    }
                    
                    attempt += 1;
                }
            }
        }
    }
}

/// Retry模式演示
pub fn demo_retry() {
    println!("=== Retry模式演示 ===\n");
    
    let retry_executor = RetryExecutor::new(RetryConfig::default());
    
    let mut call_count = 0;
    let result = retry_executor.execute(|| {
        call_count += 1;
        if call_count < 3 {
            Err("服务暂时不可用")
        } else {
            Ok("操作成功")
        }
    });
    
    match result {
        Ok(msg) => println!("最终结果: {}", msg),
        Err(e) => println!("重试失败: {}", e),
    }
    
    println!("\n【Retry模式特点】");
    println!("✓ 瞬时故障处理 - 自动重试失败的操作");
    println!("✓ 指数退避 - 逐渐增加重试间隔时间");
    println!("✓ 抖动支持 - 避免雷群效应");
    println!("✓ 可配置策略 - 支持自定义重试参数");
} 