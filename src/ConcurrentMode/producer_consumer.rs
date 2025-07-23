/*
 * 文件路径: /d%3A/workspace/RustLearn/RustDesignPattern/src/ConcurrentMode/producer_consumer.rs
 * 
 * Producer-Consumer模式 (Producer-Consumer Pattern)
 * 
 * 生产者消费者模式是一种经典的并发设计模式，用于解决生产者和消费者之间的协调问题。
 * 生产者负责产生数据并放入缓冲区，消费者从缓冲区取出数据进行处理。
 * 
 * 主要特点：
 * 1. 解耦 - 生产者和消费者之间松耦合
 * 2. 缓冲 - 通过缓冲区平衡生产和消费速度
 * 3. 同步 - 协调多个生产者和消费者
 * 4. 背压控制 - 防止缓冲区溢出
 * 5. 流量控制 - 平衡系统负载
 */

use std::sync::{Arc, Condvar, Mutex};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};
use std::collections::VecDeque;
use std::fmt;

// =================
// 有界缓冲区实现
// =================

/// 有界缓冲区，支持多生产者多消费者
pub struct BoundedBuffer<T> {
    buffer: Mutex<VecDeque<T>>,
    not_full: Condvar,
    not_empty: Condvar,
    capacity: usize,
}

impl<T> BoundedBuffer<T> {
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: Mutex::new(VecDeque::with_capacity(capacity)),
            not_full: Condvar::new(),
            not_empty: Condvar::new(),
            capacity,
        }
    }
    
    /// 生产者放入数据（阻塞直到有空间）
    pub fn put(&self, item: T) -> Result<(), ProducerConsumerError> {
        let mut buffer = self.buffer.lock().unwrap();
        
        // 等待缓冲区不满
        while buffer.len() >= self.capacity {
            buffer = self.not_full.wait(buffer).unwrap();
        }
        
        let was_empty = buffer.is_empty();
        buffer.push_back(item);
        
        // 如果之前是空的，通知等待的消费者
        if was_empty {
            self.not_empty.notify_one();
        }
        
        Ok(())
    }
    
    /// 消费者取出数据（阻塞直到有数据）
    pub fn take(&self) -> Result<T, ProducerConsumerError> {
        let mut buffer = self.buffer.lock().unwrap();
        
        // 等待缓冲区不空
        while buffer.is_empty() {
            buffer = self.not_empty.wait(buffer).unwrap();
        }
        
        let was_full = buffer.len() >= self.capacity;
        let item = buffer.pop_front().unwrap();
        
        // 如果之前是满的，通知等待的生产者
        if was_full {
            self.not_full.notify_one();
        }
        
        Ok(item)
    }
    
    /// 获取当前缓冲区大小
    pub fn size(&self) -> usize {
        self.buffer.lock().unwrap().len()
    }
    
    /// 获取缓冲区容量
    pub fn capacity(&self) -> usize {
        self.capacity
    }
    
    /// 检查缓冲区是否为空
    pub fn is_empty(&self) -> bool {
        self.buffer.lock().unwrap().is_empty()
    }
    
    /// 检查缓冲区是否已满
    pub fn is_full(&self) -> bool {
        let buffer = self.buffer.lock().unwrap();
        buffer.len() >= self.capacity
    }
}

// =================
// 错误处理
// =================

#[derive(Debug)]
pub enum ProducerConsumerError {
    BufferFull,
    BufferEmpty,
    ProducerStopped,
    ConsumerStopped,
    Timeout,
}

impl fmt::Display for ProducerConsumerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProducerConsumerError::BufferFull => write!(f, "缓冲区已满"),
            ProducerConsumerError::BufferEmpty => write!(f, "缓冲区为空"),
            ProducerConsumerError::ProducerStopped => write!(f, "生产者已停止"),
            ProducerConsumerError::ConsumerStopped => write!(f, "消费者已停止"),
            ProducerConsumerError::Timeout => write!(f, "操作超时"),
        }
    }
}

// =================
// 生产者实现
// =================

/// 生产者特质
pub trait Producer<T>: Send + 'static {
    fn produce(&mut self) -> Option<T>;
    fn name(&self) -> &str;
}

/// 数字生产者
pub struct NumberProducer {
    name: String,
    current: i32,
    max: i32,
    delay: Duration,
}

impl NumberProducer {
    pub fn new(name: String, start: i32, max: i32, delay: Duration) -> Self {
        Self {
            name,
            current: start,
            max,
            delay,
        }
    }
}

impl Producer<i32> for NumberProducer {
    fn produce(&mut self) -> Option<i32> {
        if self.current <= self.max {
            let value = self.current;
            self.current += 1;
            thread::sleep(self.delay);
            Some(value)
        } else {
            None
        }
    }
    
    fn name(&self) -> &str {
        &self.name
    }
}

/// 任务生产者
#[derive(Debug, Clone)]
pub struct Task {
    pub id: u64,
    pub data: String,
    pub priority: u8,
    pub created_at: Instant,
}

impl Task {
    pub fn new(id: u64, data: String, priority: u8) -> Self {
        Self {
            id,
            data,
            priority,
            created_at: Instant::now(),
        }
    }
}

pub struct TaskProducer {
    name: String,
    task_counter: u64,
    tasks_to_produce: usize,
    produced: usize,
}

impl TaskProducer {
    pub fn new(name: String, tasks_to_produce: usize) -> Self {
        Self {
            name,
            task_counter: 0,
            tasks_to_produce,
            produced: 0,
        }
    }
}

impl Producer<Task> for TaskProducer {
    fn produce(&mut self) -> Option<Task> {
        if self.produced < self.tasks_to_produce {
            self.task_counter += 1;
            self.produced += 1;
            
            let priority = (self.task_counter % 3 + 1) as u8;
            let task = Task::new(
                self.task_counter,
                format!("任务数据-{}", self.task_counter),
                priority,
            );
            
            thread::sleep(Duration::from_millis(50));
            Some(task)
        } else {
            None
        }
    }
    
    fn name(&self) -> &str {
        &self.name
    }
}

// =================
// 消费者实现
// =================

/// 消费者特质
pub trait Consumer<T>: Send + 'static {
    fn consume(&mut self, item: T) -> Result<(), String>;
    fn name(&self) -> &str;
}

/// 数字消费者
pub struct NumberConsumer {
    name: String,
    sum: i32,
    count: usize,
    delay: Duration,
}

impl NumberConsumer {
    pub fn new(name: String, delay: Duration) -> Self {
        Self {
            name,
            sum: 0,
            count: 0,
            delay,
        }
    }
    
    pub fn get_sum(&self) -> i32 {
        self.sum
    }
    
    pub fn get_count(&self) -> usize {
        self.count
    }
}

impl Consumer<i32> for NumberConsumer {
    fn consume(&mut self, item: i32) -> Result<(), String> {
        self.sum += item;
        self.count += 1;
        thread::sleep(self.delay);
        println!("[{}] 消费数字: {}, 当前总和: {}", self.name, item, self.sum);
        Ok(())
    }
    
    fn name(&self) -> &str {
        &self.name
    }
}

/// 任务消费者
pub struct TaskConsumer {
    name: String,
    processed_count: usize,
    total_processing_time: Duration,
}

impl TaskConsumer {
    pub fn new(name: String) -> Self {
        Self {
            name,
            processed_count: 0,
            total_processing_time: Duration::new(0, 0),
        }
    }
    
    pub fn get_stats(&self) -> (usize, Duration) {
        (self.processed_count, self.total_processing_time)
    }
}

impl Consumer<Task> for TaskConsumer {
    fn consume(&mut self, task: Task) -> Result<(), String> {
        let start_time = Instant::now();
        
        // 模拟处理时间（优先级越高处理越快）
        let processing_time = Duration::from_millis((400 - task.priority as u64 * 100).max(100));
        thread::sleep(processing_time);
        
        let elapsed = start_time.elapsed();
        self.processed_count += 1;
        self.total_processing_time += elapsed;
        
        let age = task.created_at.elapsed();
        println!("[{}] 处理任务 {} (优先级: {}, 等待时间: {:?})", 
                 self.name, task.id, task.priority, age);
        
        Ok(())
    }
    
    fn name(&self) -> &str {
        &self.name
    }
}

// =================
// 生产者消费者系统
// =================

/// 生产者消费者系统
pub struct ProducerConsumerSystem<T> {
    buffer: Arc<BoundedBuffer<T>>,
    producer_handles: Vec<JoinHandle<()>>,
    consumer_handles: Vec<JoinHandle<()>>,
    shutdown_signal: Arc<Mutex<bool>>,
}

impl<T: Send + 'static> ProducerConsumerSystem<T> {
    pub fn new(buffer_capacity: usize) -> Self {
        Self {
            buffer: Arc::new(BoundedBuffer::new(buffer_capacity)),
            producer_handles: Vec::new(),
            consumer_handles: Vec::new(),
            shutdown_signal: Arc::new(Mutex::new(false)),
        }
    }
    
    /// 启动生产者
    pub fn start_producer<P>(&mut self, mut producer: P) 
    where
        P: Producer<T> + 'static,
    {
        let buffer = Arc::clone(&self.buffer);
        let shutdown = Arc::clone(&self.shutdown_signal);
        let producer_name = producer.name().to_string();
        
        let handle = thread::spawn(move || {
            println!("生产者 {} 启动", producer_name);
            let mut produced_count = 0;
            
            loop {
                // 检查关闭信号
                if *shutdown.lock().unwrap() {
                    break;
                }
                
                match producer.produce() {
                    Some(item) => {
                        match buffer.put(item) {
                            Ok(_) => {
                                produced_count += 1;
                            }
                            Err(e) => {
                                println!("生产者 {} 错误: {}", producer_name, e);
                                break;
                            }
                        }
                    }
                    None => {
                        println!("生产者 {} 完成，共生产 {} 项", producer_name, produced_count);
                        break;
                    }
                }
            }
        });
        
        self.producer_handles.push(handle);
    }
    
    /// 启动消费者
    pub fn start_consumer<C>(&mut self, mut consumer: C)
    where
        C: Consumer<T> + 'static,
    {
        let buffer = Arc::clone(&self.buffer);
        let shutdown = Arc::clone(&self.shutdown_signal);
        let consumer_name = consumer.name().to_string();
        
        let handle = thread::spawn(move || {
            println!("消费者 {} 启动", consumer_name);
            let mut consumed_count = 0;
            
            loop {
                // 检查关闭信号和缓冲区状态
                if *shutdown.lock().unwrap() && buffer.is_empty() {
                    break;
                }
                
                match buffer.take() {
                    Ok(item) => {
                        match consumer.consume(item) {
                            Ok(_) => {
                                consumed_count += 1;
                            }
                            Err(e) => {
                                println!("消费者 {} 处理错误: {}", consumer_name, e);
                            }
                        }
                    }
                    Err(ProducerConsumerError::BufferEmpty) => {
                        // 如果缓冲区为空，稍等一下
                        thread::sleep(Duration::from_millis(10));
                    }
                    Err(e) => {
                        println!("消费者 {} 错误: {}", consumer_name, e);
                        break;
                    }
                }
            }
            
            println!("消费者 {} 停止，共消费 {} 项", consumer_name, consumed_count);
        });
        
        self.consumer_handles.push(handle);
    }
    
    /// 等待所有生产者完成
    pub fn wait_producers(&mut self) {
        for handle in self.producer_handles.drain(..) {
            let _ = handle.join();
        }
    }
    
    /// 关闭系统
    pub fn shutdown(mut self) {
        // 等待生产者完成
        self.wait_producers();
        
        // 发送关闭信号
        *self.shutdown_signal.lock().unwrap() = true;
        
        // 等待消费者完成
        for handle in self.consumer_handles {
            let _ = handle.join();
        }
        
        println!("生产者消费者系统已关闭");
    }
    
    /// 获取缓冲区状态
    pub fn get_buffer_status(&self) -> (usize, usize) {
        (self.buffer.size(), self.buffer.capacity())
    }
}

// =================
// 高性能环形缓冲区
// =================

/// 无锁环形缓冲区（单生产者单消费者）
pub struct RingBuffer<T> {
    buffer: Vec<T>,
    capacity: usize,
    read_pos: Arc<Mutex<usize>>,
    write_pos: Arc<Mutex<usize>>,
}

impl<T: Default + Clone> RingBuffer<T> {
    pub fn new(capacity: usize) -> Self {
        let mut buffer = Vec::with_capacity(capacity);
        buffer.resize(capacity, T::default());
        
        Self {
            buffer,
            capacity,
            read_pos: Arc::new(Mutex::new(0)),
            write_pos: Arc::new(Mutex::new(0)),
        }
    }
    
    /// 生产者写入数据
    pub fn write(&mut self, item: T) -> Result<(), ProducerConsumerError> {
        let mut write_pos = self.write_pos.lock().unwrap();
        let read_pos = *self.read_pos.lock().unwrap();
        
        let next_write = (*write_pos + 1) % self.capacity;
        
        // 检查是否会覆盖未读数据
        if next_write == read_pos {
            return Err(ProducerConsumerError::BufferFull);
        }
        
        unsafe {
            let ptr = self.buffer.as_mut_ptr().add(*write_pos);
            std::ptr::write(ptr, item);
        }
        
        *write_pos = next_write;
        Ok(())
    }
    
    /// 消费者读取数据
    pub fn read(&self) -> Result<T, ProducerConsumerError> {
        let mut read_pos = self.read_pos.lock().unwrap();
        let write_pos = *self.write_pos.lock().unwrap();
        
        // 检查是否有数据可读
        if *read_pos == write_pos {
            return Err(ProducerConsumerError::BufferEmpty);
        }
        
        let item = unsafe {
            let ptr = self.buffer.as_ptr().add(*read_pos);
            std::ptr::read(ptr)
        };
        
        *read_pos = (*read_pos + 1) % self.capacity;
        Ok(item)
    }
}

// =================
// 演示函数
// =================

/// Producer-Consumer模式演示
pub fn demo_producer_consumer() {
    println!("=== Producer-Consumer模式演示 ===\n");
    
    let buffer = Arc::new(BoundedBuffer::new(5));
    
    // 生产者线程
    let producer_buffer = Arc::clone(&buffer);
    let producer = thread::spawn(move || {
        for i in 1..=10 {
            producer_buffer.put(i);
            println!("生产: {}", i);
            thread::sleep(Duration::from_millis(100));
        }
        println!("生产者完成");
    });
    
    // 消费者线程
    let consumer_buffer = Arc::clone(&buffer);
    let consumer = thread::spawn(move || {
        for _ in 1..=10 {
            let item = consumer_buffer.take();
            println!("消费: {:?}", item);
            thread::sleep(Duration::from_millis(150));
        }
        println!("消费者完成");
    });
    
    producer.join().unwrap();
    consumer.join().unwrap();
    
    println!("\n【Producer-Consumer模式特点】");
    println!("✓ 解耦 - 生产者和消费者独立工作");
    println!("✓ 缓冲 - 平衡生产和消费速度差异");
    println!("✓ 并发 - 支持多生产者多消费者");
} 