/*
 * 文件路径: /d%3A/workspace/RustLearn/RustDesignPattern/src/ConcurrentMode/reactor_pattern.rs
 * 
 * Reactor模式 (Reactor Pattern)
 * 
 * Reactor模式是一种事件驱动的并发模式，用于处理并发的I/O操作。
 * 它通过事件循环监听多个事件源，当事件发生时分发给相应的处理器处理。
 * 
 * 主要特点：
 * 1. 事件驱动 - 基于事件的异步处理
 * 2. 单线程 - 使用单个事件循环线程
 * 3. 非阻塞 - 所有I/O操作都是非阻塞的
 * 4. 多路复用 - 同时监听多个事件源
 * 5. 事件分发 - 将事件分发给对应的处理器
 */

use std::sync::{Arc, Mutex};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use std::collections::{HashMap, VecDeque};
use std::fmt;

// =================
// 事件和处理器定义
// =================

/// 事件类型
#[derive(Debug, Clone)]
pub enum EventType {
    Read,
    Write,
    Timer,
    Custom(String),
}

/// 事件数据
#[derive(Debug, Clone)]
pub struct Event {
    pub id: u64,
    pub event_type: EventType,
    pub source_id: String,
    pub data: Vec<u8>,
    pub timestamp: Instant,
    pub metadata: HashMap<String, String>,
}

impl Event {
    pub fn new(id: u64, event_type: EventType, source_id: String) -> Self {
        Self {
            id,
            event_type,
            source_id,
            data: Vec::new(),
            timestamp: Instant::now(),
            metadata: HashMap::new(),
        }
    }
    
    pub fn with_data(mut self, data: Vec<u8>) -> Self {
        self.data = data;
        self
    }
    
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
    
    pub fn age(&self) -> Duration {
        self.timestamp.elapsed()
    }
}

/// 事件处理器特质
pub trait EventHandler: Send + 'static {
    /// 处理事件
    fn handle_event(&mut self, event: Event) -> Result<(), EventError>;
    
    /// 获取处理器名称
    fn name(&self) -> &str;
    
    /// 处理器初始化
    fn initialize(&mut self) -> Result<(), EventError> {
        Ok(())
    }
    
    /// 处理器清理
    fn cleanup(&mut self) -> Result<(), EventError> {
        Ok(())
    }
    
    /// 是否可以处理指定类型的事件
    fn can_handle(&self, event_type: &EventType) -> bool;
}

/// 事件处理错误
#[derive(Debug)]
pub enum EventError {
    HandlerNotFound,
    ProcessingFailed(String),
    InvalidEvent,
    Timeout,
    ResourceExhausted,
}

impl fmt::Display for EventError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EventError::HandlerNotFound => write!(f, "未找到事件处理器"),
            EventError::ProcessingFailed(msg) => write!(f, "事件处理失败: {}", msg),
            EventError::InvalidEvent => write!(f, "无效事件"),
            EventError::Timeout => write!(f, "处理超时"),
            EventError::ResourceExhausted => write!(f, "资源耗尽"),
        }
    }
}

// =================
// 具体事件处理器实现
// =================

/// 网络数据处理器
pub struct NetworkHandler {
    name: String,
    processed_count: u64,
    total_bytes: u64,
}

impl NetworkHandler {
    pub fn new(name: String) -> Self {
        Self {
            name,
            processed_count: 0,
            total_bytes: 0,
        }
    }
    
    pub fn get_stats(&self) -> (u64, u64) {
        (self.processed_count, self.total_bytes)
    }
}

impl EventHandler for NetworkHandler {
    fn handle_event(&mut self, event: Event) -> Result<(), EventError> {
        match event.event_type {
            EventType::Read => {
                self.processed_count += 1;
                self.total_bytes += event.data.len() as u64;
                
                let data_str = String::from_utf8_lossy(&event.data);
                println!("[{}] 处理读取事件 {} 来自 {}: {} 字节 - {}",
                         self.name, event.id, event.source_id, event.data.len(),
                         if data_str.len() > 50 { &data_str[..50] } else { &data_str });
                
                // 模拟处理时间
                thread::sleep(Duration::from_millis(1));
                Ok(())
            }
            EventType::Write => {
                self.processed_count += 1;
                println!("[{}] 处理写入事件 {} 到 {}: {} 字节",
                         self.name, event.id, event.source_id, event.data.len());
                
                // 模拟处理时间
                thread::sleep(Duration::from_millis(1));
                Ok(())
            }
            _ => Err(EventError::ProcessingFailed("不支持的事件类型".to_string())),
        }
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn can_handle(&self, event_type: &EventType) -> bool {
        matches!(event_type, EventType::Read | EventType::Write)
    }
}

/// 定时器处理器
pub struct TimerHandler {
    name: String,
    timer_count: u64,
}

impl TimerHandler {
    pub fn new(name: String) -> Self {
        Self {
            name,
            timer_count: 0,
        }
    }
    
    pub fn get_timer_count(&self) -> u64 {
        self.timer_count
    }
}

impl EventHandler for TimerHandler {
    fn handle_event(&mut self, event: Event) -> Result<(), EventError> {
        if let EventType::Timer = event.event_type {
            self.timer_count += 1;
            println!("[{}] 处理定时器事件 {} 来自 {} (第 {} 次)",
                     self.name, event.id, event.source_id, self.timer_count);
            Ok(())
        } else {
            Err(EventError::ProcessingFailed("不是定时器事件".to_string()))
        }
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn can_handle(&self, event_type: &EventType) -> bool {
        matches!(event_type, EventType::Timer)
    }
}

/// 自定义事件处理器
pub struct CustomHandler {
    name: String,
    handled_types: Vec<String>,
    processed_count: u64,
}

impl CustomHandler {
    pub fn new(name: String, handled_types: Vec<String>) -> Self {
        Self {
            name,
            handled_types,
            processed_count: 0,
        }
    }
    
    pub fn get_processed_count(&self) -> u64 {
        self.processed_count
    }
}

impl EventHandler for CustomHandler {
    fn handle_event(&mut self, event: Event) -> Result<(), EventError> {
        match &event.event_type {
            EventType::Custom(event_name) => {
                if self.handled_types.contains(event_name) {
                    self.processed_count += 1;
                    println!("[{}] 处理自定义事件 {} ({}): {}",
                             self.name, event.id, event_name, 
                             String::from_utf8_lossy(&event.data));
                    
                    // 模拟复杂处理
                    thread::sleep(Duration::from_millis(5));
                    Ok(())
                } else {
                    Err(EventError::ProcessingFailed(
                        format!("不支持的自定义事件类型: {}", event_name)
                    ))
                }
            }
            _ => Err(EventError::ProcessingFailed("不是自定义事件".to_string())),
        }
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn can_handle(&self, event_type: &EventType) -> bool {
        if let EventType::Custom(event_name) = event_type {
            self.handled_types.contains(event_name)
        } else {
            false
        }
    }
}

// =================
// 事件源和生成器
// =================

/// 事件源特质
pub trait EventSource: Send + 'static {
    /// 生成事件
    fn generate_event(&mut self) -> Option<Event>;
    
    /// 获取事件源ID
    fn source_id(&self) -> &str;
    
    /// 是否还有更多事件
    fn has_more_events(&self) -> bool;
    
    /// 获取下一个事件的预期时间
    fn next_event_time(&self) -> Option<Instant> {
        None
    }
}

/// 网络连接模拟器
pub struct NetworkConnectionSimulator {
    source_id: String,
    event_counter: u64,
    max_events: u64,
    data_patterns: Vec<String>,
    last_event_time: Instant,
    event_interval: Duration,
}

impl NetworkConnectionSimulator {
    pub fn new(source_id: String, max_events: u64, event_interval: Duration) -> Self {
        Self {
            source_id,
            event_counter: 0,
            max_events,
            data_patterns: vec![
                "GET /api/users HTTP/1.1".to_string(),
                "POST /api/data HTTP/1.1".to_string(),
                "PUT /api/update HTTP/1.1".to_string(),
                "DELETE /api/item/123 HTTP/1.1".to_string(),
            ],
            last_event_time: Instant::now(),
            event_interval,
        }
    }
}

impl EventSource for NetworkConnectionSimulator {
    fn generate_event(&mut self) -> Option<Event> {
        if self.event_counter >= self.max_events {
            return None;
        }
        
        // 检查是否到时间生成事件
        if self.last_event_time.elapsed() < self.event_interval {
            return None;
        }
        
        self.event_counter += 1;
        self.last_event_time = Instant::now();
        
        let event_type = if self.event_counter % 3 == 0 {
            EventType::Write
        } else {
            EventType::Read
        };
        
        let data = self.data_patterns[self.event_counter as usize % self.data_patterns.len()]
            .as_bytes()
            .to_vec();
        
        Some(Event::new(self.event_counter, event_type, self.source_id.clone())
             .with_data(data)
             .with_metadata("connection_type".to_string(), "tcp".to_string()))
    }
    
    fn source_id(&self) -> &str {
        &self.source_id
    }
    
    fn has_more_events(&self) -> bool {
        self.event_counter < self.max_events
    }
    
    fn next_event_time(&self) -> Option<Instant> {
        if self.has_more_events() {
            Some(self.last_event_time + self.event_interval)
        } else {
            None
        }
    }
}

/// 定时器事件源
pub struct TimerEventSource {
    source_id: String,
    interval: Duration,
    last_trigger: Instant,
    max_triggers: u64,
    trigger_count: u64,
}

impl TimerEventSource {
    pub fn new(source_id: String, interval: Duration, max_triggers: u64) -> Self {
        Self {
            source_id,
            interval,
            last_trigger: Instant::now(),
            max_triggers,
            trigger_count: 0,
        }
    }
}

impl EventSource for TimerEventSource {
    fn generate_event(&mut self) -> Option<Event> {
        if self.trigger_count >= self.max_triggers {
            return None;
        }
        
        if self.last_trigger.elapsed() >= self.interval {
            self.trigger_count += 1;
            self.last_trigger = Instant::now();
            
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis();
            
            Some(Event::new(self.trigger_count, EventType::Timer, self.source_id.clone())
                 .with_data(timestamp.to_string().as_bytes().to_vec())
                 .with_metadata("timer_type".to_string(), "periodic".to_string()))
        } else {
            None
        }
    }
    
    fn source_id(&self) -> &str {
        &self.source_id
    }
    
    fn has_more_events(&self) -> bool {
        self.trigger_count < self.max_triggers
    }
    
    fn next_event_time(&self) -> Option<Instant> {
        if self.has_more_events() {
            Some(self.last_trigger + self.interval)
        } else {
            None
        }
    }
}

// =================
// Reactor核心实现
// =================

/// Reactor配置
#[derive(Debug, Clone)]
pub struct ReactorConfig {
    pub max_events_per_loop: usize,
    pub loop_timeout: Duration,
    pub enable_statistics: bool,
    pub event_queue_size: usize,
}

impl Default for ReactorConfig {
    fn default() -> Self {
        Self {
            max_events_per_loop: 100,
            loop_timeout: Duration::from_millis(10),
            enable_statistics: true,
            event_queue_size: 1000,
        }
    }
}

/// Reactor统计信息
#[derive(Debug, Clone, Default)]
pub struct ReactorStats {
    pub events_processed: u64,
    pub events_failed: u64,
    pub loops_executed: u64,
    pub total_processing_time: Duration,
    pub average_event_processing_time: Duration,
}

/// Reactor事件循环
pub struct Reactor {
    config: ReactorConfig,
    handlers: HashMap<String, Box<dyn EventHandler>>,
    event_sources: Vec<Box<dyn EventSource>>,
    event_queue: VecDeque<Event>,
    stats: ReactorStats,
    running: bool,
    event_id_counter: u64,
}

impl Reactor {
    /// 创建新的Reactor
    pub fn new(config: ReactorConfig) -> Self {
        Self {
            config,
            handlers: HashMap::new(),
            event_sources: Vec::new(),
            event_queue: VecDeque::new(),
            stats: ReactorStats::default(),
            running: false,
            event_id_counter: 0,
        }
    }
    
    /// 使用默认配置创建Reactor
    pub fn with_default_config() -> Self {
        Self::new(ReactorConfig::default())
    }
    
    /// 注册事件处理器
    pub fn register_handler(&mut self, handler_id: String, handler: Box<dyn EventHandler>) {
        println!("注册事件处理器: {}", handler_id);
        self.handlers.insert(handler_id, handler);
    }
    
    /// 添加事件源
    pub fn add_event_source(&mut self, source: Box<dyn EventSource>) {
        println!("添加事件源: {}", source.source_id());
        self.event_sources.push(source);
    }
    
    /// 手动添加事件到队列
    pub fn submit_event(&mut self, mut event: Event) {
        if event.id == 0 {
            self.event_id_counter += 1;
            event.id = self.event_id_counter;
        }
        
        if self.event_queue.len() < self.config.event_queue_size {
            self.event_queue.push_back(event);
        } else {
            println!("警告: 事件队列已满，丢弃事件 {}", event.id);
        }
    }
    
    /// 启动事件循环
    pub fn run(&mut self) -> Result<(), EventError> {
        println!("启动Reactor事件循环...");
        self.running = true;
        
        // 初始化所有处理器
        for (handler_id, handler) in &mut self.handlers {
            if let Err(e) = handler.initialize() {
                println!("处理器 {} 初始化失败: {}", handler_id, e);
                return Err(e);
            }
        }
        
        let start_time = Instant::now();
        
        while self.running && (!self.event_sources.is_empty() || !self.event_queue.is_empty()) {
            let loop_start = Instant::now();
            
            // 1. 从事件源收集新事件
            self.collect_events_from_sources();
            
            // 2. 处理事件队列中的事件
            let processed_count = self.process_events();
            
            // 3. 更新统计信息
            self.stats.loops_executed += 1;
            if processed_count == 0 {
                // 如果没有事件处理，稍微休息一下
                thread::sleep(self.config.loop_timeout);
            }
            
            let loop_time = loop_start.elapsed();
            self.stats.total_processing_time += loop_time;
            
            // 移除已耗尽的事件源
            self.event_sources.retain(|source| source.has_more_events());
        }
        
        // 清理所有处理器
        for (handler_id, handler) in &mut self.handlers {
            if let Err(e) = handler.cleanup() {
                println!("处理器 {} 清理失败: {}", handler_id, e);
            }
        }
        
        let total_time = start_time.elapsed();
        println!("Reactor事件循环结束，总耗时: {:?}", total_time);
        
        // 计算平均处理时间
        if self.stats.events_processed > 0 {
            self.stats.average_event_processing_time = 
                self.stats.total_processing_time / self.stats.events_processed as u32;
        }
        
        Ok(())
    }
    
    /// 从事件源收集事件
    fn collect_events_from_sources(&mut self) {
        for source in &mut self.event_sources {
            while let Some(event) = source.generate_event() {
                if self.event_queue.len() < self.config.event_queue_size {
                    self.event_queue.push_back(event);
                } else {
                    break; // 队列已满
                }
            }
        }
    }
    
    /// 处理事件队列中的事件
    fn process_events(&mut self) -> usize {
        let mut processed_count = 0;
        let max_events = self.config.max_events_per_loop.min(self.event_queue.len());
        
        for _ in 0..max_events {
            if let Some(event) = self.event_queue.pop_front() {
                let event_start = Instant::now();
                
                // 寻找合适的处理器
                let mut handled = false;
                for (handler_id, handler) in &mut self.handlers {
                    if handler.can_handle(&event.event_type) {
                        match handler.handle_event(event.clone()) {
                            Ok(_) => {
                                self.stats.events_processed += 1;
                                handled = true;
                                break;
                            }
                            Err(e) => {
                                println!("处理器 {} 处理事件 {} 失败: {}", handler_id, event.id, e);
                                self.stats.events_failed += 1;
                            }
                        }
                    }
                }
                
                if !handled {
                    println!("警告: 事件 {} ({:?}) 没有找到合适的处理器", event.id, event.event_type);
                    self.stats.events_failed += 1;
                }
                
                processed_count += 1;
            }
        }
        
        processed_count
    }
    
    /// 停止事件循环
    pub fn stop(&mut self) {
        println!("请求停止Reactor事件循环");
        self.running = false;
    }
    
    /// 获取统计信息
    pub fn get_stats(&self) -> &ReactorStats {
        &self.stats
    }
    
    /// 获取当前队列大小
    pub fn queue_size(&self) -> usize {
        self.event_queue.len()
    }
    
    /// 获取处理器数量
    pub fn handler_count(&self) -> usize {
        self.handlers.len()
    }
    
    /// 获取事件源数量
    pub fn source_count(&self) -> usize {
        self.event_sources.len()
    }
}

// =================
// 演示函数
// =================

/// Reactor模式演示
pub fn demo_reactor() {
    println!("=== Reactor模式演示 ===\n");
    
    // 1. 基本事件处理演示
    println!("1. 基本事件处理演示:");
    {
        let mut reactor = Reactor::with_default_config();
        
        // 注册事件处理器
        reactor.register_handler("network".to_string(), 
                                Box::new(NetworkHandler::new("网络处理器".to_string())));
        reactor.register_handler("timer".to_string(), 
                                Box::new(TimerHandler::new("定时器处理器".to_string())));
        
        // 添加事件源
        reactor.add_event_source(Box::new(NetworkConnectionSimulator::new(
            "连接1".to_string(), 
            10, 
            Duration::from_millis(50)
        )));
        
        reactor.add_event_source(Box::new(TimerEventSource::new(
            "定时器1".to_string(),
            Duration::from_millis(100),
            5
        )));
        
        // 手动添加一些事件
        reactor.submit_event(Event::new(1000, EventType::Read, "手动事件".to_string())
                            .with_data("手动生成的数据".as_bytes().to_vec()));
        
        // 运行Reactor
        let _ = reactor.run();
        
        // 显示统计信息
        let stats = reactor.get_stats();
        println!("处理统计:");
        println!("  事件总数: {}", stats.events_processed);
        println!("  失败事件: {}", stats.events_failed);
        println!("  循环次数: {}", stats.loops_executed);
        println!("  平均处理时间: {:?}", stats.average_event_processing_time);
    }
    
    println!("\n{}\n", "=".repeat(50));
    
    // 2. 多处理器协作演示
    println!("2. 多处理器协作演示:");
    {
        let config = ReactorConfig {
            max_events_per_loop: 50,
            loop_timeout: Duration::from_millis(5),
            ..Default::default()
        };
        let mut reactor = Reactor::new(config);
        
        // 注册多个网络处理器
        reactor.register_handler("network1".to_string(),
                                Box::new(NetworkHandler::new("网络处理器1".to_string())));
        reactor.register_handler("network2".to_string(),
                                Box::new(NetworkHandler::new("网络处理器2".to_string())));
        
        // 注册自定义事件处理器
        reactor.register_handler("custom".to_string(),
                                Box::new(CustomHandler::new("自定义处理器".to_string(),
                                vec!["user_action".to_string(), "system_event".to_string()])));
        
        // 添加多个网络连接源
        for i in 1..=3 {
            reactor.add_event_source(Box::new(NetworkConnectionSimulator::new(
                format!("连接{}", i),
                8,
                Duration::from_millis(30 + i * 10)
            )));
        }
        
        // 手动添加自定义事件
        for i in 1..=5 {
            reactor.submit_event(Event::new(
                2000 + i,
                EventType::Custom("user_action".to_string()),
                "用户界面".to_string()
            ).with_data(format!("用户操作{}", i).as_bytes().to_vec()));
        }
        
        // 运行Reactor
        let _ = reactor.run();
        
        let stats = reactor.get_stats();
        println!("多处理器协作统计:");
        println!("  处理器数量: {}", reactor.handler_count());
        println!("  事件源数量: {}", reactor.source_count());
        println!("  处理事件: {}", stats.events_processed);
        println!("  失败事件: {}", stats.events_failed);
        println!("  处理效率: {:.2}%", 
                 (stats.events_processed as f64 / (stats.events_processed + stats.events_failed) as f64) * 100.0);
    }
    
    println!("\n{}\n", "=".repeat(50));
    
    // 3. 高负载性能测试
    println!("3. 高负载性能测试:");
    {
        let config = ReactorConfig {
            max_events_per_loop: 200,
            loop_timeout: Duration::from_millis(1),
            event_queue_size: 2000,
            ..Default::default()
        };
        let mut reactor = Reactor::new(config);
        
        // 注册高性能处理器
        reactor.register_handler("fast_network".to_string(),
                                Box::new(NetworkHandler::new("快速网络处理器".to_string())));
        reactor.register_handler("batch_timer".to_string(),
                                Box::new(TimerHandler::new("批量定时器".to_string())));
        
        // 添加高频事件源
        for i in 1..=5 {
            reactor.add_event_source(Box::new(NetworkConnectionSimulator::new(
                format!("高频连接{}", i),
                50,
                Duration::from_millis(2)
            )));
        }
        
        reactor.add_event_source(Box::new(TimerEventSource::new(
            "高频定时器".to_string(),
            Duration::from_millis(5),
            30
        )));
        
        let start_time = Instant::now();
        let _ = reactor.run();
        let total_time = start_time.elapsed();
        
        let stats = reactor.get_stats();
        println!("高负载性能测试结果:");
        println!("  总耗时: {:?}", total_time);
        println!("  处理事件: {}", stats.events_processed);
        println!("  失败事件: {}", stats.events_failed);
        println!("  循环次数: {}", stats.loops_executed);
        println!("  事件吞吐量: {:.2} 事件/秒", 
                 stats.events_processed as f64 / total_time.as_secs_f64());
        println!("  平均循环时间: {:?}", 
                 stats.total_processing_time / stats.loops_executed as u32);
    }
    
    println!("\n【Reactor模式特点】");
    println!("✓ 事件驱动 - 基于事件的异步处理");
    println!("✓ 单线程 - 使用单个事件循环线程");
    println!("✓ 非阻塞 - 所有I/O操作都是非阻塞的");
    println!("✓ 多路复用 - 同时监听多个事件源");
    println!("✓ 事件分发 - 将事件分发给对应的处理器");
    println!("✓ 高效率 - 避免线程切换开销");
} 