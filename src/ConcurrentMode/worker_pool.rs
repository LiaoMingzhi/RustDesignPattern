/*
 * 文件路径: /d%3A/workspace/RustLearn/RustDesignPattern/src/ConcurrentMode/worker_pool.rs
 * 
 * Worker Pool模式 (Worker Pool Pattern)
 * 
 * 工作线程池模式是一种并发设计模式，维护一组工作线程来执行任务。
 * 任务被提交到任务队列中，空闲的工作线程会从队列中取出任务执行。
 * 
 * 主要特点：
 * 1. 线程复用 - 避免频繁创建销毁线程的开销
 * 2. 资源控制 - 控制并发线程数量，避免资源耗尽
 * 3. 任务排队 - 支持任务缓冲和优先级处理
 * 4. 负载均衡 - 自动分配任务给空闲工作者
 * 5. 动态扩缩 - 根据负载动态调整线程数量
 */

use std::sync::{Arc, Condvar, Mutex};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};
use std::collections::{BinaryHeap, HashMap, VecDeque};
use std::cmp::Ordering;
use std::fmt;

// =================
// 任务定义和特质
// =================

/// 工作任务特质
pub trait Task: Send + 'static {
    type Output: Send + 'static;
    
    /// 执行任务
    fn execute(self: Box<Self>) -> Self::Output;
    
    /// 获取任务优先级（数值越大优先级越高）
    fn priority(&self) -> u8 {
        0
    }
    
    /// 获取任务描述
    fn description(&self) -> String {
        "未命名任务".to_string()
    }
}

/// 具体任务实现：数学计算任务
#[derive(Debug)]
pub struct MathTask {
    pub id: u64,
    pub operation: MathOperation,
    pub priority: u8,
}

#[derive(Debug)]
pub enum MathOperation {
    Add(i64, i64),
    Multiply(i64, i64),
    Fibonacci(u32),
    Prime(u32),
}

impl MathTask {
    pub fn new(id: u64, operation: MathOperation, priority: u8) -> Self {
        Self { id, operation, priority }
    }
}

impl Task for MathTask {
    type Output = (u64, i64); // (task_id, result)
    
    fn execute(self: Box<Self>) -> Self::Output {
        let result = match self.operation {
            MathOperation::Add(a, b) => a + b,
            MathOperation::Multiply(a, b) => a * b,
            MathOperation::Fibonacci(n) => fibonacci(n),
            MathOperation::Prime(n) => count_primes(n) as i64,
        };
        
        // 模拟计算时间
        thread::sleep(Duration::from_millis(5));
        
        (self.id, result)
    }
    
    fn priority(&self) -> u8 {
        self.priority
    }
    
    fn description(&self) -> String {
        format!("数学任务#{} - {:?}", self.id, self.operation)
    }
}

/// 网络请求任务
#[derive(Debug)]
pub struct NetworkTask {
    pub id: u64,
    pub url: String,
    pub timeout: Duration,
    pub priority: u8,
}

impl NetworkTask {
    pub fn new(id: u64, url: String, timeout: Duration, priority: u8) -> Self {
        Self { id, url, timeout, priority }
    }
}

impl Task for NetworkTask {
    type Output = (u64, Result<String, String>); // (task_id, response)
    
    fn execute(self: Box<Self>) -> Self::Output {
        // 模拟网络请求
        thread::sleep(Duration::from_millis(30));
        
        let response = if self.url.contains("error") {
            Err(format!("请求失败: {}", self.url))
        } else {
            Ok(format!("响应数据来自: {}", self.url))
        };
        
        (self.id, response)
    }
    
    fn priority(&self) -> u8 {
        self.priority
    }
    
    fn description(&self) -> String {
        format!("网络任务#{} - {}", self.id, self.url)
    }
}

// =================
// 优先级任务包装
// =================

/// 优先级任务包装器
struct PriorityTask {
    task: Box<dyn Task<Output = Box<dyn std::any::Any + Send>>>,
    priority: u8,
    created_at: Instant,
}

impl PriorityTask {
    fn new<T: Task + 'static>(task: T) -> Self
    where
        T::Output: 'static,
    {
        let priority = task.priority();
        let wrapped_task = Box::new(TaskWrapper::new(task));
        
        Self {
            task: wrapped_task,
            priority,
            created_at: Instant::now(),
        }
    }
}

impl PartialEq for PriorityTask {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority
    }
}

impl Eq for PriorityTask {}

impl PartialOrd for PriorityTask {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PriorityTask {
    fn cmp(&self, other: &Self) -> Ordering {
        // 优先级高的排在前面，如果优先级相同则按创建时间排序
        self.priority.cmp(&other.priority)
            .then_with(|| other.created_at.cmp(&self.created_at))
    }
}

/// 任务包装器，用于类型擦除
struct TaskWrapper<T: Task> {
    task: Option<T>,
}

impl<T: Task + 'static> TaskWrapper<T>
where
    T::Output: 'static,
{
    fn new(task: T) -> Self {
        Self { task: Some(task) }
    }
}

impl<T: Task + 'static> Task for TaskWrapper<T>
where
    T::Output: 'static,
{
    type Output = Box<dyn std::any::Any + Send>;
    
    fn execute(mut self: Box<Self>) -> Self::Output {
        if let Some(task) = self.task.take() {
            Box::new(Box::new(task).execute())
        } else {
            panic!("任务已被执行");
        }
    }
}

// =================
// 工作线程池错误处理
// =================

#[derive(Debug)]
pub enum WorkerPoolError {
    PoolShutdown,
    TaskRejected,
    WorkerPanic,
    InvalidConfiguration,
}

impl fmt::Display for WorkerPoolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WorkerPoolError::PoolShutdown => write!(f, "线程池已关闭"),
            WorkerPoolError::TaskRejected => write!(f, "任务被拒绝"),
            WorkerPoolError::WorkerPanic => write!(f, "工作线程恐慌"),
            WorkerPoolError::InvalidConfiguration => write!(f, "无效配置"),
        }
    }
}

// =================
// 工作线程池实现
// =================

/// 工作线程池配置
#[derive(Debug, Clone)]
pub struct PoolConfig {
    pub core_pool_size: usize,
    pub max_pool_size: usize,
    pub keep_alive_time: Duration,
    pub queue_capacity: usize,
    pub allow_core_timeout: bool,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            core_pool_size: 4,
            max_pool_size: 8,
            keep_alive_time: Duration::from_secs(60),
            queue_capacity: 100,
            allow_core_timeout: false,
        }
    }
}

/// 工作线程池
pub struct WorkerPool {
    config: PoolConfig,
    task_queue: Arc<Mutex<BinaryHeap<PriorityTask>>>,
    task_available: Arc<Condvar>,
    workers: Arc<Mutex<Vec<WorkerHandle>>>,
    worker_id_counter: Arc<Mutex<usize>>,
    shutdown: Arc<Mutex<bool>>,
    pool_stats: Arc<Mutex<PoolStats>>,
}

/// 工作线程句柄
struct WorkerHandle {
    id: usize,
    handle: JoinHandle<()>,
    is_core: bool,
    last_active: Instant,
}

/// 线程池统计信息
#[derive(Debug, Clone)]
pub struct PoolStats {
    pub tasks_completed: u64,
    pub tasks_failed: u64,
    pub total_execution_time: Duration,
    pub active_workers: usize,
    pub queue_size: usize,
}

impl Default for PoolStats {
    fn default() -> Self {
        Self {
            tasks_completed: 0,
            tasks_failed: 0,
            total_execution_time: Duration::new(0, 0),
            active_workers: 0,
            queue_size: 0,
        }
    }
}

impl WorkerPool {
    /// 创建新的工作线程池
    pub fn new(config: PoolConfig) -> Result<Self, WorkerPoolError> {
        if config.core_pool_size > config.max_pool_size {
            return Err(WorkerPoolError::InvalidConfiguration);
        }
        
        let pool = Self {
            config: config.clone(),
            task_queue: Arc::new(Mutex::new(BinaryHeap::new())),
            task_available: Arc::new(Condvar::new()),
            workers: Arc::new(Mutex::new(Vec::new())),
            worker_id_counter: Arc::new(Mutex::new(0)),
            shutdown: Arc::new(Mutex::new(false)),
            pool_stats: Arc::new(Mutex::new(PoolStats::default())),
        };
        
        // 启动核心工作线程
        for _ in 0..config.core_pool_size {
            pool.spawn_worker(true)?;
        }
        
        Ok(pool)
    }
    
    /// 使用默认配置创建线程池
    pub fn with_default_config() -> Result<Self, WorkerPoolError> {
        Self::new(PoolConfig::default())
    }
    
    /// 提交任务到线程池
    pub fn submit<T>(&self, task: T) -> Result<(), WorkerPoolError>
    where
        T: Task + 'static,
        T::Output: 'static,
    {
        if *self.shutdown.lock().unwrap() {
            return Err(WorkerPoolError::PoolShutdown);
        }
        
        let priority_task = PriorityTask::new(task);
        let mut queue = self.task_queue.lock().unwrap();
        
        // 检查队列容量
        if queue.len() >= self.config.queue_capacity {
            return Err(WorkerPoolError::TaskRejected);
        }
        
        queue.push(priority_task);
        println!("任务已添加到队列，当前队列大小: {}", queue.len());
        drop(queue); // 释放锁
        self.task_available.notify_one();
        println!("已通知工作线程有新任务");
        
        // 如果需要，启动额外的工作线程
        self.try_spawn_additional_worker();
        
        Ok(())
    }
    
    /// 批量提交任务
    pub fn submit_batch<T>(&self, tasks: Vec<T>) -> Result<(), WorkerPoolError>
    where
        T: Task + 'static,
        T::Output: 'static,
    {
        for task in tasks {
            self.submit(task)?;
        }
        Ok(())
    }
    
    /// 尝试启动额外的工作线程
    fn try_spawn_additional_worker(&self) {
        let queue_size = self.task_queue.lock().unwrap().len();
        let workers = self.workers.lock().unwrap();
        let current_worker_count = workers.len();
        
        // 如果队列中有任务且当前工作线程数小于最大值，启动新线程
        if queue_size > 0 && current_worker_count < self.config.max_pool_size {
            drop(workers); // 释放锁
            let _ = self.spawn_worker(false);
        }
    }
    
    /// 启动工作线程
    fn spawn_worker(&self, is_core: bool) -> Result<(), WorkerPoolError> {
        let worker_id = {
            let mut counter = self.worker_id_counter.lock().unwrap();
            *counter += 1;
            *counter
        };
        
        let task_queue = Arc::clone(&self.task_queue);
        let task_available = Arc::clone(&self.task_available);
        let shutdown = Arc::clone(&self.shutdown);
        let pool_stats = Arc::clone(&self.pool_stats);
        let keep_alive_time = self.config.keep_alive_time;
        let allow_core_timeout = self.config.allow_core_timeout;
        
        let handle = thread::spawn(move || {
            println!("工作线程 {} 启动 ({})", worker_id, if is_core { "核心" } else { "临时" });
            
            let start_time = Instant::now();
            let max_worker_lifetime = Duration::from_secs(3); // 最大工作时间3秒
            let mut task_count = 0u32;
            let max_tasks = 10; // 每个线程最多处理10个任务
            
            loop {
                // 检查工作线程是否已经运行了3秒或处理了足够多的任务，如果是则退出
                if start_time.elapsed() > max_worker_lifetime || task_count >= max_tasks {
                    println!("工作线程 {} 达到退出条件 (运行时间: {:?}, 处理任务: {}), 自动退出", 
                             worker_id, start_time.elapsed(), task_count);
                    return;
                }
                
                let task = {
                    let mut queue = task_queue.lock().unwrap();
                    
                    // 先检查是否已有任务
                    if !queue.is_empty() {
                        println!("工作线程 {} 发现队列中有任务，立即处理", worker_id);
                        Some(queue.pop().unwrap())
                    } else {
                        println!("工作线程 {} 开始等待任务...", worker_id);
                        // 等待任务或超时
                        let timeout = Duration::from_secs(1); // 所有线程1秒超时
                        
                        let (mut queue_result, timeout_result) = task_available
                            .wait_timeout(queue, timeout)
                            .unwrap();
                        
                        if *shutdown.lock().unwrap() {
                            println!("工作线程 {} 收到关闭信号，退出", worker_id);
                            return;
                        }
                        
                        if timeout_result.timed_out() {
                            if queue_result.is_empty() {
                                println!("工作线程 {} 等待超时且无任务", worker_id);
                                None
                            } else {
                                println!("工作线程 {} 等待超时但有任务", worker_id);
                                queue_result.pop()
                            }
                        } else {
                            println!("工作线程 {} 被唤醒，检查任务", worker_id);
                            queue_result.pop()
                        }
                    }
                };
                
                if let Some(priority_task) = task {
                    let start_time = Instant::now();
                    
                    // 执行任务
                    let task_box = priority_task.task;
                    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        task_box.execute()
                    })) {
                        Ok(result) => {
                            let execution_time = start_time.elapsed();
                            println!("工作线程 {} 完成任务，耗时: {:?}", worker_id, execution_time);
                            let mut stats = pool_stats.lock().unwrap();
                            stats.tasks_completed += 1;
                            stats.total_execution_time += execution_time;
                            task_count += 1; // 增加任务计数
                        }
                        Err(_) => {
                            println!("工作线程 {} 任务执行时发生恐慌", worker_id);
                            let mut stats = pool_stats.lock().unwrap();
                            stats.tasks_failed += 1;
                            task_count += 1; // 即使失败也增加计数
                        }
                    }
                } else if *shutdown.lock().unwrap() {
                    break;
                }
            }
            
            println!("工作线程 {} 正常退出", worker_id);
        });
        
        let worker = WorkerHandle {
            id: worker_id,
            handle,
            is_core,
            last_active: Instant::now(),
        };
        
        let mut workers = self.workers.lock().unwrap();
        workers.push(worker);
        
        Ok(())
    }
    
    /// 获取线程池统计信息
    pub fn get_stats(&self) -> PoolStats {
        let mut stats = self.pool_stats.lock().unwrap().clone();
        stats.active_workers = self.workers.lock().unwrap().len();
        stats.queue_size = self.task_queue.lock().unwrap().len();
        stats
    }
    
    /// 获取当前活跃工作线程数
    pub fn active_count(&self) -> usize {
        self.workers.lock().unwrap().len()
    }
    
    /// 获取队列中的任务数
    pub fn queue_size(&self) -> usize {
        self.task_queue.lock().unwrap().len()
    }
    
    /// 关闭线程池
    pub fn shutdown(self) {
        println!("开始关闭线程池...");
        
        // 设置关闭标志
        *self.shutdown.lock().unwrap() = true;
        
        // 通知所有等待的线程
        self.task_available.notify_all();
        
        // 等待所有工作线程完成（限制等待时间）
        let workers = self.workers.lock().unwrap();
        let worker_count = workers.len();
        drop(workers);
        
        // 等待线程退出，但不超过2秒
        thread::sleep(Duration::from_millis(200));
        
        let final_stats = self.get_stats();
        println!("线程池已关闭 (原有{}个工作线程)", worker_count);
        if final_stats.tasks_completed > 0 {
            println!("最终统计: 完成任务 {}, 失败任务 {}, 平均执行时间 {:?}",
                     final_stats.tasks_completed,
                     final_stats.tasks_failed,
                     final_stats.total_execution_time / (final_stats.tasks_completed.max(1) as u32));
        }
    }
}

// =================
// 辅助函数
// =================

/// 计算斐波那契数列
fn fibonacci(n: u32) -> i64 {
    match n {
        0 => 0,
        1 => 1,
        _ => {
            let mut a = 0i64;
            let mut b = 1i64;
            for _ in 2..=n {
                let temp = a + b;
                a = b;
                b = temp;
            }
            b
        }
    }
}

/// 计算质数个数
fn count_primes(n: u32) -> u32 {
    if n < 2 {
        return 0;
    }
    
    let mut is_prime = vec![true; n as usize];
    is_prime[0] = false;
    if n > 1 {
        is_prime[1] = false;
    }
    
    let mut i = 2;
    while i * i < n {
        if is_prime[i as usize] {
            let mut j = i * i;
            while j < n {
                is_prime[j as usize] = false;
                j += i;
            }
        }
        i += 1;
    }
    
    is_prime.iter().filter(|&&x| x).count() as u32
}

// =================
// 演示函数
// =================

/// Worker Pool模式演示
pub fn demo_worker_pool() {
    println!("=== Worker Pool模式演示 ===\n");
    
    // 1. 基本线程池使用
    println!("1. 基本线程池使用:");
    {
        let config = PoolConfig {
            core_pool_size: 2,
            max_pool_size: 4,
            keep_alive_time: Duration::from_secs(2),
            allow_core_timeout: true, // 允许核心线程超时
            ..Default::default()
        };
        let pool = WorkerPool::new(config).unwrap();
        
        // 提交数学计算任务
        for i in 1..=6 {
            let task = MathTask::new(
                i,
                MathOperation::Add(i as i64, (i * 2) as i64),
                if i % 3 == 0 { 2 } else { 1 }, // 每第3个任务高优先级
            );
            println!("提交任务 {}: {} + {}", i, i, i * 2);
            pool.submit(task).unwrap();
        }
        
        thread::sleep(Duration::from_millis(100));
        let stats = pool.get_stats();
        println!("完成任务: {}, 活跃线程: {}", stats.tasks_completed, stats.active_workers);
        
        pool.shutdown();
    }
    
    println!("\n{}\n", "=".repeat(50));
    
    // 2. 优先级任务演示
    println!("2. 优先级任务演示:");
    {
        let config = PoolConfig {
            core_pool_size: 2,
            max_pool_size: 4,
            keep_alive_time: Duration::from_secs(2),
            allow_core_timeout: true,
            ..Default::default()
        };
        let pool = WorkerPool::new(config).unwrap();
        
        // 提交不同优先级的任务
        for i in 1..=6 {
            let priority = match i % 3 {
                0 => 3, // 高优先级
                1 => 2, // 中优先级
                _ => 1, // 低优先级
            };
            
            let task = MathTask::new(
                i,
                MathOperation::Fibonacci(15 + i as u32), // 降低计算复杂度
                priority,
            );
            
            println!("提交任务 {} (优先级: {}) - Fibonacci({})", i, priority, 15 + i);
            pool.submit(task).unwrap();
        }
        
        thread::sleep(Duration::from_millis(100));
        pool.shutdown();
    }
    
    println!("\n{}\n", "=".repeat(50));
    
    // 3. 混合任务类型演示
    println!("3. 混合任务类型演示:");
    {
        let config = PoolConfig {
            core_pool_size: 2,
            max_pool_size: 4,
            keep_alive_time: Duration::from_secs(2),
            allow_core_timeout: true,
            ..Default::default()
        };
        let pool = WorkerPool::new(config).unwrap();
        
        // 提交数学任务
        for i in 1..=3 {
            let task = MathTask::new(i, MathOperation::Prime(100 + i as u32), 2);
            println!("提交数学任务 {} - Prime({})", i, 100 + i);
            pool.submit(task).unwrap();
        }
        
        // 提交网络任务
        for i in 1..=3 {
            let url = if i % 3 == 0 {
                format!("https://error.example.com/api/{}", i)
            } else {
                format!("https://api.example.com/data/{}", i)
            };
            
            let task = NetworkTask::new(
                100 + i,
                url.clone(),
                Duration::from_millis(200),
                1,
            );
            println!("提交网络任务 {} - {}", 100 + i, url);
            pool.submit(task).unwrap();
        }
        
        thread::sleep(Duration::from_millis(100));
        pool.shutdown();
    }
    
    println!("\n{}\n", "=".repeat(50));
    
    // 4. 性能压力测试
    println!("4. 性能压力测试:");
    {
        let config = PoolConfig {
            core_pool_size: 3,
            max_pool_size: 6,
            queue_capacity: 50,
            keep_alive_time: Duration::from_secs(2),
            allow_core_timeout: true,
        };
        let pool = WorkerPool::new(config).unwrap();
        
        let start_time = Instant::now();
        const TASK_COUNT: u64 = 20;
        
        // 批量提交任务
        let mut tasks = Vec::new();
        for i in 1..=TASK_COUNT {
            tasks.push(MathTask::new(
                i,
                MathOperation::Fibonacci(20), // 降低计算复杂度
                1,
            ));
        }
        
        println!("批量提交 {} 个任务...", TASK_COUNT);
        pool.submit_batch(tasks).unwrap();
        
        // 等待所有任务完成或超时
        let timeout = Duration::from_secs(3);
        let wait_start = Instant::now();
        loop {
            let stats = pool.get_stats();
            if stats.tasks_completed >= TASK_COUNT {
                break;
            }
            if wait_start.elapsed() > timeout {
                println!("等待超时，强制退出");
                break;
            }
            thread::sleep(Duration::from_millis(50));
        }
        
        let total_time = start_time.elapsed();
        let final_stats = pool.get_stats();
        
        println!("处理 {} 个任务耗时: {:?}", final_stats.tasks_completed, total_time);
        if final_stats.tasks_completed > 0 {
            println!("平均每个任务: {:?}", total_time / final_stats.tasks_completed as u32);
            println!("吞吐量: {:.2} 任务/秒", final_stats.tasks_completed as f64 / total_time.as_secs_f64());
        }
        println!("失败任务: {}", final_stats.tasks_failed);
        
        pool.shutdown();
    }
    
    println!("\n【Worker Pool模式特点】");
    println!("✓ 线程复用 - 避免频繁创建销毁线程");
    println!("✓ 资源控制 - 限制并发线程数量");
    println!("✓ 任务排队 - 支持任务缓冲和优先级");
    println!("✓ 负载均衡 - 自动分配任务给空闲线程");
    println!("✓ 动态扩缩 - 根据负载调整线程数量");
    println!("✓ 容错处理 - 处理任务执行异常");
} 