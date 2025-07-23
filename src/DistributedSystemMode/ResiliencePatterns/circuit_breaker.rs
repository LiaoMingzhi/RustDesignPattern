/*
 * 文件路径: /d%3A/workspace/RustLearn/RustDesignPattern/src/DistributedSystemMode/ResiliencePatterns/circuit_breaker.rs
 * 
 * Circuit Breaker模式 (熔断器)
 * 
 * 熔断器模式用于防止分布式系统中的级联故障。当检测到某个服务频繁失败时，
 * 熔断器会"跳闸"，暂时阻止对该服务的调用，避免浪费资源并给服务恢复时间。
 * 
 * 主要特点：
 * 1. 故障检测 - 监控服务调用的成功率和响应时间
 * 2. 快速失败 - 在服务不可用时立即返回错误，避免等待
 * 3. 自动恢复 - 定期尝试调用服务，检测服务是否已恢复
 * 4. 状态管理 - 管理关闭、打开、半开三种状态
 * 5. 指标收集 - 收集调用统计信息用于监控和决策
 */

use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};
use std::fmt;
use std::collections::VecDeque;

// =================
// 熔断器状态
// =================

/// 熔断器状态
#[derive(Debug, Clone, PartialEq, Copy)]
pub enum CircuitState {
    /// 关闭状态 - 正常处理请求
    Closed,
    /// 打开状态 - 拒绝所有请求
    Open,
    /// 半开状态 - 允许有限的请求用于测试服务是否恢复
    HalfOpen,
}

impl fmt::Display for CircuitState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CircuitState::Closed => write!(f, "关闭"),
            CircuitState::Open => write!(f, "打开"),
            CircuitState::HalfOpen => write!(f, "半开"),
        }
    }
}

// =================
// 熔断器配置
// =================

/// 熔断器配置
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// 失败阈值 - 触发熔断的失败次数
    pub failure_threshold: u32,
    /// 失败率阈值 - 触发熔断的失败率 (0.0 - 1.0)
    pub failure_rate_threshold: f64,
    /// 请求体积阈值 - 最小请求数量才开始计算失败率
    pub request_volume_threshold: u32,
    /// 超时时间 - 请求超时时间
    pub timeout: Duration,
    /// 恢复超时时间 - 熔断器打开后多久尝试恢复
    pub recovery_timeout: Duration,
    /// 半开状态最大尝试次数
    pub half_open_max_calls: u32,
    /// 统计窗口大小 - 保持多少个历史记录
    pub stats_window_size: usize,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            failure_rate_threshold: 0.5,
            request_volume_threshold: 20,
            timeout: Duration::from_secs(30),
            recovery_timeout: Duration::from_secs(60),
            half_open_max_calls: 3,
            stats_window_size: 100,
        }
    }
}

// =================
// 调用结果
// =================

/// 调用结果
#[derive(Debug, Clone)]
pub enum CallResult<T, E> {
    Success(T),
    Failure(E),
    Timeout,
}

/// 熔断器错误
#[derive(Debug, Clone)]
pub enum CircuitBreakerError<E> {
    /// 熔断器打开，调用被拒绝
    CircuitOpen,
    /// 调用超时
    CallTimeout,
    /// 底层服务错误
    ServiceError(E),
}

impl<E: fmt::Display> fmt::Display for CircuitBreakerError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CircuitBreakerError::CircuitOpen => write!(f, "熔断器打开，调用被拒绝"),
            CircuitBreakerError::CallTimeout => write!(f, "调用超时"),
            CircuitBreakerError::ServiceError(e) => write!(f, "服务错误: {}", e),
        }
    }
}

// =================
// 调用统计
// =================

/// 单次调用记录
#[derive(Debug, Clone)]
struct CallRecord {
    timestamp: Instant,
    success: bool,
    duration: Duration,
}

/// 熔断器统计信息
#[derive(Debug, Clone)]
pub struct CircuitBreakerStats {
    pub state: CircuitState,
    pub total_calls: u64,
    pub successful_calls: u64,
    pub failed_calls: u64,
    pub rejected_calls: u64,
    pub success_rate: f64,
    pub average_response_time: Duration,
    pub last_failure_time: Option<Instant>,
    pub last_success_time: Option<Instant>,
    pub state_changed_time: Instant,
}

/// 统计收集器
struct StatsCollector {
    call_history: VecDeque<CallRecord>,
    window_size: usize,
    total_calls: u64,
    successful_calls: u64,
    failed_calls: u64,
    rejected_calls: u64,
    last_failure_time: Option<Instant>,
    last_success_time: Option<Instant>,
}

impl StatsCollector {
    fn new(window_size: usize) -> Self {
        Self {
            call_history: VecDeque::new(),
            window_size,
            total_calls: 0,
            successful_calls: 0,
            failed_calls: 0,
            rejected_calls: 0,
            last_failure_time: None,
            last_success_time: None,
        }
    }
    
    fn record_call(&mut self, success: bool, duration: Duration) {
        let record = CallRecord {
            timestamp: Instant::now(),
            success,
            duration,
        };
        
        // 添加新记录
        self.call_history.push_back(record);
        
        // 清理过期记录
        while self.call_history.len() > self.window_size {
            self.call_history.pop_front();
        }
        
        // 更新统计
        self.total_calls += 1;
        if success {
            self.successful_calls += 1;
            self.last_success_time = Some(Instant::now());
        } else {
            self.failed_calls += 1;
            self.last_failure_time = Some(Instant::now());
        }
    }
    
    fn record_rejection(&mut self) {
        self.rejected_calls += 1;
    }
    
    fn get_failure_rate(&self) -> f64 {
        if self.call_history.is_empty() {
            return 0.0;
        }
        
        let failed_count = self.call_history.iter().filter(|r| !r.success).count();
        failed_count as f64 / self.call_history.len() as f64
    }
    
    fn get_recent_call_count(&self) -> u32 {
        self.call_history.len() as u32
    }
    
    fn get_recent_failure_count(&self) -> u32 {
        self.call_history.iter().filter(|r| !r.success).count() as u32
    }
    
    fn get_average_response_time(&self) -> Duration {
        if self.call_history.is_empty() {
            return Duration::new(0, 0);
        }
        
        let total_duration: Duration = self.call_history.iter()
            .map(|r| r.duration)
            .sum();
        
        total_duration / self.call_history.len() as u32
    }
}

// =================
// 熔断器实现
// =================

/// 熔断器
pub struct CircuitBreaker {
    config: CircuitBreakerConfig,
    state: Arc<RwLock<CircuitState>>,
    stats: Arc<Mutex<StatsCollector>>,
    state_changed_time: Arc<Mutex<Instant>>,
    half_open_calls: Arc<Mutex<u32>>,
}

impl CircuitBreaker {
    /// 创建新的熔断器
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            config: config.clone(),
            state: Arc::new(RwLock::new(CircuitState::Closed)),
            stats: Arc::new(Mutex::new(StatsCollector::new(config.stats_window_size))),
            state_changed_time: Arc::new(Mutex::new(Instant::now())),
            half_open_calls: Arc::new(Mutex::new(0)),
        }
    }
    
    /// 使用默认配置创建熔断器
    pub fn with_default_config() -> Self {
        Self::new(CircuitBreakerConfig::default())
    }
    
    /// 执行被保护的调用
    pub fn call<T, E, F>(&self, operation: F) -> Result<T, CircuitBreakerError<E>>
    where
        F: FnOnce() -> Result<T, E>,
    {
        // 检查是否允许调用
        if !self.allow_request() {
            let mut stats = self.stats.lock().unwrap();
            stats.record_rejection();
            return Err(CircuitBreakerError::CircuitOpen);
        }
        
        let start_time = Instant::now();
        
        // 执行调用
        let result = operation();
        let duration = start_time.elapsed();
        
        // 处理结果
        match result {
            Ok(value) => {
                self.on_success(duration);
                Ok(value)
            }
            Err(error) => {
                self.on_failure(duration);
                Err(CircuitBreakerError::ServiceError(error))
            }
        }
    }
    
    /// 执行异步调用（带超时）
    pub fn call_with_timeout<T, E, F>(&self, operation: F) -> Result<T, CircuitBreakerError<E>>
    where
        F: FnOnce() -> Result<T, E>,
    {
        if !self.allow_request() {
            let mut stats = self.stats.lock().unwrap();
            stats.record_rejection();
            return Err(CircuitBreakerError::CircuitOpen);
        }
        
        let start_time = Instant::now();
        
        // 简单的超时模拟 (实际实现中应该使用真正的异步超时机制)
        let result = operation();
        let duration = start_time.elapsed();
        
        if duration > self.config.timeout {
            self.on_failure(duration);
            return Err(CircuitBreakerError::CallTimeout);
        }
        
        match result {
            Ok(value) => {
                self.on_success(duration);
                Ok(value)
            }
            Err(error) => {
                self.on_failure(duration);
                Err(CircuitBreakerError::ServiceError(error))
            }
        }
    }
    
    /// 检查是否允许请求
    fn allow_request(&self) -> bool {
        let state = *self.state.read().unwrap();
        
        match state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                // 检查是否到了尝试恢复的时间
                let state_changed_time = *self.state_changed_time.lock().unwrap();
                if state_changed_time.elapsed() >= self.config.recovery_timeout {
                    self.transition_to_half_open();
                    true
                } else {
                    false
                }
            }
            CircuitState::HalfOpen => {
                // 半开状态下允许有限的请求
                let mut half_open_calls = self.half_open_calls.lock().unwrap();
                if *half_open_calls < self.config.half_open_max_calls {
                    *half_open_calls += 1;
                    true
                } else {
                    false
                }
            }
        }
    }
    
    /// 处理成功调用
    fn on_success(&self, duration: Duration) {
        let mut stats = self.stats.lock().unwrap();
        stats.record_call(true, duration);
        
        let state = *self.state.read().unwrap();
        if state == CircuitState::HalfOpen {
            // 半开状态下如果调用成功，转换到关闭状态
            self.transition_to_closed();
        }
    }
    
    /// 处理失败调用
    fn on_failure(&self, duration: Duration) {
        let mut stats = self.stats.lock().unwrap();
        stats.record_call(false, duration);
        
        let state = *self.state.read().unwrap();
        
        match state {
            CircuitState::Closed => {
                // 检查是否需要打开熔断器
                if self.should_trip(&stats) {
                    drop(stats); // 释放锁
                    self.transition_to_open();
                }
            }
            CircuitState::HalfOpen => {
                // 半开状态下如果调用失败，重新打开熔断器
                drop(stats); // 释放锁
                self.transition_to_open();
            }
            CircuitState::Open => {
                // 已经是打开状态，不需要额外操作
            }
        }
    }
    
    /// 检查是否应该触发熔断
    fn should_trip(&self, stats: &StatsCollector) -> bool {
        let recent_calls = stats.get_recent_call_count();
        let recent_failures = stats.get_recent_failure_count();
        let failure_rate = stats.get_failure_rate();
        
        // 请求量足够且失败率超过阈值
        (recent_calls >= self.config.request_volume_threshold &&
         failure_rate >= self.config.failure_rate_threshold) ||
        // 或者连续失败次数超过阈值
        recent_failures >= self.config.failure_threshold
    }
    
    /// 转换到关闭状态
    fn transition_to_closed(&self) {
        let mut state = self.state.write().unwrap();
        *state = CircuitState::Closed;
        
        let mut state_changed_time = self.state_changed_time.lock().unwrap();
        *state_changed_time = Instant::now();
        
        let mut half_open_calls = self.half_open_calls.lock().unwrap();
        *half_open_calls = 0;
        
        println!("熔断器状态变更: {} -> 关闭", state);
    }
    
    /// 转换到打开状态
    fn transition_to_open(&self) {
        let mut state = self.state.write().unwrap();
        *state = CircuitState::Open;
        
        let mut state_changed_time = self.state_changed_time.lock().unwrap();
        *state_changed_time = Instant::now();
        
        let mut half_open_calls = self.half_open_calls.lock().unwrap();
        *half_open_calls = 0;
        
        println!("熔断器状态变更: 打开");
    }
    
    /// 转换到半开状态
    fn transition_to_half_open(&self) {
        let mut state = self.state.write().unwrap();
        *state = CircuitState::HalfOpen;
        
        let mut state_changed_time = self.state_changed_time.lock().unwrap();
        *state_changed_time = Instant::now();
        
        let mut half_open_calls = self.half_open_calls.lock().unwrap();
        *half_open_calls = 0;
        
        println!("熔断器状态变更: 半开");
    }
    
    /// 获取当前状态
    pub fn get_state(&self) -> CircuitState {
        *self.state.read().unwrap()
    }
    
    /// 获取统计信息
    pub fn get_stats(&self) -> CircuitBreakerStats {
        let stats = self.stats.lock().unwrap();
        let state = *self.state.read().unwrap();
        let state_changed_time = *self.state_changed_time.lock().unwrap();
        
        CircuitBreakerStats {
            state,
            total_calls: stats.total_calls,
            successful_calls: stats.successful_calls,
            failed_calls: stats.failed_calls,
            rejected_calls: stats.rejected_calls,
            success_rate: if stats.total_calls > 0 {
                stats.successful_calls as f64 / stats.total_calls as f64
            } else {
                0.0
            },
            average_response_time: stats.get_average_response_time(),
            last_failure_time: stats.last_failure_time,
            last_success_time: stats.last_success_time,
            state_changed_time,
        }
    }
    
    /// 手动重置熔断器
    pub fn reset(&self) {
        let mut stats = self.stats.lock().unwrap();
        *stats = StatsCollector::new(self.config.stats_window_size);
        drop(stats);
        
        self.transition_to_closed();
        println!("熔断器已重置");
    }
}

// =================
// 模拟服务
// =================

/// 模拟的不稳定服务
pub struct UnstableService {
    name: String,
    failure_rate: f64,
    response_time: Duration,
    current_failure_count: Arc<Mutex<u32>>,
}

impl UnstableService {
    pub fn new(name: String, failure_rate: f64, response_time: Duration) -> Self {
        Self {
            name,
            failure_rate,
            response_time,
            current_failure_count: Arc::new(Mutex::new(0)),
        }
    }
    
    pub fn call(&self, data: &str) -> Result<String, String> {
        // 模拟处理时间
        std::thread::sleep(self.response_time);
        
        // 模拟失败率
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let random = (now % 100) as f64 / 100.0;
        
        if random < self.failure_rate {
            let mut count = self.current_failure_count.lock().unwrap();
            *count += 1;
            Err(format!("{}服务失败 (失败次数: {})", self.name, *count))
        } else {
            Ok(format!("{}服务处理: {}", self.name, data))
        }
    }
    
    pub fn set_failure_rate(&self, rate: f64) {
        // 在实际实现中，这里会修改failure_rate字段
        // 为了简化，这里只打印信息
        println!("{}服务失败率调整为: {:.2}", self.name, rate);
    }
}

// =================
// 演示函数
// =================

/// Circuit Breaker模式演示
pub fn demo_circuit_breaker() {
    println!("=== Circuit Breaker模式演示 ===\n");
    
    // 创建配置
    let config = CircuitBreakerConfig {
        failure_threshold: 3,
        failure_rate_threshold: 0.6,
        request_volume_threshold: 5,
        timeout: Duration::from_millis(1000),
        recovery_timeout: Duration::from_secs(2),
        half_open_max_calls: 2,
        stats_window_size: 10,
    };
    
    // 创建熔断器
    let circuit_breaker = CircuitBreaker::new(config);
    println!("创建熔断器，配置:");
    println!("  失败阈值: 3次");
    println!("  失败率阈值: 60%");
    println!("  最小请求量: 5次");
    println!("  恢复超时: 2秒");
    
    // 创建不稳定服务
    let service = UnstableService::new("用户服务".to_string(), 0.7, Duration::from_millis(100));
    
    // 1. 正常调用阶段
    println!("\n1. 正常调用阶段 (高失败率):");
    for i in 1..=10 {
        let result = circuit_breaker.call(|| service.call(&format!("请求{}", i)));
        
        match result {
            Ok(response) => println!("  调用{}成功: {}", i, response),
            Err(CircuitBreakerError::ServiceError(e)) => println!("  调用{}失败: {}", i, e),
            Err(CircuitBreakerError::CircuitOpen) => println!("  调用{}被拒绝: 熔断器打开", i),
            Err(CircuitBreakerError::CallTimeout) => println!("  调用{}超时", i),
        }
        
        // 短暂延迟
        std::thread::sleep(Duration::from_millis(100));
    }
    
    // 显示统计信息
    println!("\n当前熔断器状态:");
    let stats = circuit_breaker.get_stats();
    println!("  状态: {}", stats.state);
    println!("  总调用次数: {}", stats.total_calls);
    println!("  成功次数: {}", stats.successful_calls);
    println!("  失败次数: {}", stats.failed_calls);
    println!("  被拒绝次数: {}", stats.rejected_calls);
    println!("  成功率: {:.2}%", stats.success_rate * 100.0);
    
    // 2. 熔断器打开期间
    println!("\n2. 熔断器打开期间:");
    for i in 11..=15 {
        let result = circuit_breaker.call(|| service.call(&format!("请求{}", i)));
        
        match result {
            Ok(response) => println!("  调用{}成功: {}", i, response),
            Err(CircuitBreakerError::ServiceError(e)) => println!("  调用{}失败: {}", i, e),
            Err(CircuitBreakerError::CircuitOpen) => println!("  调用{}被拒绝: 熔断器打开", i),
            Err(CircuitBreakerError::CallTimeout) => println!("  调用{}超时", i),
        }
        
        std::thread::sleep(Duration::from_millis(100));
    }
    
    // 3. 等待自动恢复
    println!("\n3. 等待自动恢复 (2秒)...");
    std::thread::sleep(Duration::from_secs(3));
    
    // 模拟服务恢复（降低失败率）
    println!("模拟服务恢复，降低失败率...");
    service.set_failure_rate(0.1);
    
    // 4. 半开状态测试
    println!("\n4. 半开状态测试:");
    for i in 16..=20 {
        let result = circuit_breaker.call(|| {
            // 模拟低失败率
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let random = (now % 100) as f64 / 100.0;
            
            std::thread::sleep(Duration::from_millis(100));
            
            if random < 0.1 { // 10% 失败率
                Err(format!("服务偶尔失败"))
            } else {
                Ok(format!("用户服务处理: 请求{}", i))
            }
        });
        
        match result {
            Ok(response) => println!("  调用{}成功: {}", i, response),
            Err(CircuitBreakerError::ServiceError(e)) => println!("  调用{}失败: {}", i, e),
            Err(CircuitBreakerError::CircuitOpen) => println!("  调用{}被拒绝: 熔断器打开", i),
            Err(CircuitBreakerError::CallTimeout) => println!("  调用{}超时", i),
        }
        
        std::thread::sleep(Duration::from_millis(200));
    }
    
    // 5. 最终统计
    println!("\n5. 最终统计:");
    let final_stats = circuit_breaker.get_stats();
    println!("  最终状态: {}", final_stats.state);
    println!("  总调用次数: {}", final_stats.total_calls);
    println!("  成功次数: {}", final_stats.successful_calls);
    println!("  失败次数: {}", final_stats.failed_calls);
    println!("  被拒绝次数: {}", final_stats.rejected_calls);
    println!("  成功率: {:.2}%", final_stats.success_rate * 100.0);
    println!("  平均响应时间: {}ms", final_stats.average_response_time.as_millis());
    
    if let Some(last_failure) = final_stats.last_failure_time {
        println!("  最后失败时间: {}秒前", last_failure.elapsed().as_secs());
    }
    
    if let Some(last_success) = final_stats.last_success_time {
        println!("  最后成功时间: {}秒前", last_success.elapsed().as_secs());
    }
    
    println!("\n【Circuit Breaker模式特点】");
    println!("✓ 故障检测 - 监控服务调用的成功率和响应时间");
    println!("✓ 快速失败 - 在服务不可用时立即返回错误，避免等待");
    println!("✓ 自动恢复 - 定期尝试调用服务，检测服务是否已恢复");
    println!("✓ 状态管理 - 管理关闭、打开、半开三种状态");
    println!("✓ 指标收集 - 收集调用统计信息用于监控和决策");
} 