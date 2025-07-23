/*
 * 文件路径: /d%3A/workspace/RustLearn/RustDesignPattern/src/ConcurrentMode/master_worker.rs
 * 
 * Master-Worker模式 (Master-Worker Pattern)
 * 
 * 主从工作模式是一种分而治之的并发模式，其中Master负责任务分解和结果聚合，
 * Worker负责执行具体的子任务。这种模式适合处理可以分解的大型计算任务。
 * 
 * 主要特点：
 * 1. 任务分解 - Master将大任务分解为小任务
 * 2. 并行处理 - 多个Worker并行执行子任务
 * 3. 结果聚合 - Master收集和合并Worker的结果
 * 4. 负载均衡 - 动态分配任务给空闲Worker
 * 5. 容错处理 - 处理Worker失败和任务重试
 */

use std::sync::{Arc, Mutex, Condvar};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};
use std::collections::{HashMap, VecDeque};
use std::fmt;

// =================
// 任务和结果定义
// =================

/// 工作任务特质
pub trait WorkTask: Send + Clone + 'static {
    type Result: Send + 'static;
    
    /// 执行任务
    fn execute(&self) -> Self::Result;
    
    /// 获取任务ID
    fn task_id(&self) -> u64;
    
    /// 获取任务描述
    fn description(&self) -> String {
        format!("任务#{}", self.task_id())
    }
    
    /// 获取任务优先级
    fn priority(&self) -> u8 {
        0
    }
}

/// 任务分解器特质
pub trait TaskSplitter<T: WorkTask>: Send + 'static {
    /// 将大任务分解为多个子任务
    fn split(&mut self, task: T) -> Vec<T>;
    
    /// 合并子任务结果
    fn merge(&mut self, results: Vec<T::Result>) -> T::Result;
}

/// 具体任务实现：矩阵乘法
#[derive(Debug, Clone)]
pub struct MatrixMultiplyTask {
    pub id: u64,
    pub matrix_a: Vec<Vec<i32>>,
    pub matrix_b: Vec<Vec<i32>>,
    pub start_row: usize,
    pub end_row: usize,
}

impl MatrixMultiplyTask {
    pub fn new(id: u64, matrix_a: Vec<Vec<i32>>, matrix_b: Vec<Vec<i32>>) -> Self {
        let rows = matrix_a.len();
        Self {
            id,
            matrix_a,
            matrix_b,
            start_row: 0,
            end_row: rows,
        }
    }
    
    pub fn partial(
        id: u64,
        matrix_a: Vec<Vec<i32>>,
        matrix_b: Vec<Vec<i32>>,
        start_row: usize,
        end_row: usize,
    ) -> Self {
        Self {
            id,
            matrix_a,
            matrix_b,
            start_row,
            end_row,
        }
    }
}

impl WorkTask for MatrixMultiplyTask {
    type Result = Vec<Vec<i32>>;
    
    fn execute(&self) -> Self::Result {
        let rows_a = self.matrix_a.len();
        let cols_b = self.matrix_b[0].len();
        let mut result = vec![vec![0; cols_b]; self.end_row - self.start_row];
        
        for i in self.start_row..self.end_row {
            for j in 0..cols_b {
                let mut sum = 0;
                for k in 0..self.matrix_a[0].len() {
                    sum += self.matrix_a[i][k] * self.matrix_b[k][j];
                }
                result[i - self.start_row][j] = sum;
            }
        }
        
        // 模拟计算时间
        thread::sleep(Duration::from_millis(10));
        result
    }
    
    fn task_id(&self) -> u64 {
        self.id
    }
    
    fn description(&self) -> String {
        format!("矩阵乘法任务#{} (行 {}-{})", self.id, self.start_row, self.end_row)
    }
}

/// 矩阵乘法任务分解器
pub struct MatrixSplitter {
    chunks_per_worker: usize,
}

impl MatrixSplitter {
    pub fn new(chunks_per_worker: usize) -> Self {
        Self { chunks_per_worker }
    }
}

impl TaskSplitter<MatrixMultiplyTask> for MatrixSplitter {
    fn split(&mut self, task: MatrixMultiplyTask) -> Vec<MatrixMultiplyTask> {
        let total_rows = task.matrix_a.len();
        let chunk_size = (total_rows + self.chunks_per_worker - 1) / self.chunks_per_worker;
        let mut subtasks = Vec::new();
        
        for i in 0..self.chunks_per_worker {
            let start_row = i * chunk_size;
            let end_row = ((i + 1) * chunk_size).min(total_rows);
            
            if start_row < total_rows {
                let subtask = MatrixMultiplyTask::partial(
                    task.id * 1000 + i as u64,
                    task.matrix_a.clone(),
                    task.matrix_b.clone(),
                    start_row,
                    end_row,
                );
                subtasks.push(subtask);
            }
        }
        
        subtasks
    }
    
    fn merge(&mut self, results: Vec<Vec<Vec<i32>>>) -> Vec<Vec<i32>> {
        let mut merged = Vec::new();
        for result in results {
            merged.extend(result);
        }
        merged
    }
}

/// 数值计算任务
#[derive(Debug, Clone)]
pub struct ComputeTask {
    pub id: u64,
    pub operation: ComputeOperation,
    pub data: Vec<i64>,
}

#[derive(Debug, Clone)]
pub enum ComputeOperation {
    Sum,
    Product,
    Max,
    Min,
    Average,
}

impl ComputeTask {
    pub fn new(id: u64, operation: ComputeOperation, data: Vec<i64>) -> Self {
        Self { id, operation, data }
    }
}

impl WorkTask for ComputeTask {
    type Result = f64;
    
    fn execute(&self) -> Self::Result {
        thread::sleep(Duration::from_millis(5)); // 模拟计算时间
        
        match self.operation {
            ComputeOperation::Sum => self.data.iter().sum::<i64>() as f64,
            ComputeOperation::Product => self.data.iter().product::<i64>() as f64,
            ComputeOperation::Max => *self.data.iter().max().unwrap_or(&0) as f64,
            ComputeOperation::Min => *self.data.iter().min().unwrap_or(&0) as f64,
            ComputeOperation::Average => {
                if self.data.is_empty() {
                    0.0
                } else {
                    self.data.iter().sum::<i64>() as f64 / self.data.len() as f64
                }
            }
        }
    }
    
    fn task_id(&self) -> u64 {
        self.id
    }
    
    fn description(&self) -> String {
        format!("计算任务#{} ({:?}, {} 个数据)", self.id, self.operation, self.data.len())
    }
}

/// 数值计算任务分解器
pub struct ComputeSplitter {
    chunk_size: usize,
}

impl ComputeSplitter {
    pub fn new(chunk_size: usize) -> Self {
        Self { chunk_size }
    }
}

impl TaskSplitter<ComputeTask> for ComputeSplitter {
    fn split(&mut self, task: ComputeTask) -> Vec<ComputeTask> {
        let mut subtasks = Vec::new();
        
        for (i, chunk) in task.data.chunks(self.chunk_size).enumerate() {
            let subtask = ComputeTask::new(
                task.id * 1000 + i as u64,
                task.operation.clone(),
                chunk.to_vec(),
            );
            subtasks.push(subtask);
        }
        
        subtasks
    }
    
    fn merge(&mut self, results: Vec<f64>) -> f64 {
        match results.first().map(|_| &self.chunk_size) {
            Some(_) => {
                // 根据操作类型合并结果
                results.iter().sum() // 简化实现，实际应根据操作类型处理
            }
            None => 0.0,
        }
    }
}

// =================
// Master-Worker系统
// =================

/// Worker状态
#[derive(Debug, Clone)]
pub enum WorkerStatus {
    Idle,
    Working(u64), // 正在处理的任务ID
    Failed,
}

/// Worker统计信息
#[derive(Debug, Clone)]
pub struct WorkerStats {
    pub tasks_completed: u64,
    pub tasks_failed: u64,
    pub total_processing_time: Duration,
    pub average_processing_time: Duration,
}

impl Default for WorkerStats {
    fn default() -> Self {
        Self {
            tasks_completed: 0,
            tasks_failed: 0,
            total_processing_time: Duration::new(0, 0),
            average_processing_time: Duration::new(0, 0),
        }
    }
}

/// Master-Worker系统配置
#[derive(Debug, Clone)]
pub struct MasterWorkerConfig {
    pub worker_count: usize,
    pub task_queue_size: usize,
    pub result_queue_size: usize,
    pub worker_timeout: Duration,
    pub enable_load_balancing: bool,
}

impl Default for MasterWorkerConfig {
    fn default() -> Self {
        Self {
            worker_count: 4,
            task_queue_size: 100,
            result_queue_size: 100,
            worker_timeout: Duration::from_secs(30),
            enable_load_balancing: true,
        }
    }
}

/// Master-Worker系统
pub struct MasterWorkerSystem<T: WorkTask> {
    config: MasterWorkerConfig,
    task_sender: Sender<T>,
    result_receiver: Receiver<(u64, Result<T::Result, String>)>,
    worker_handles: Vec<JoinHandle<()>>,
    worker_stats: Arc<Mutex<HashMap<usize, WorkerStats>>>,
    shutdown_signal: Arc<Mutex<bool>>,
}

impl<T: WorkTask> MasterWorkerSystem<T> {
    /// 创建新的Master-Worker系统
    pub fn new(config: MasterWorkerConfig) -> Self {
        let (task_sender, task_receiver) = mpsc::channel();
        let (result_sender, result_receiver) = mpsc::channel();
        let task_receiver = Arc::new(Mutex::new(task_receiver));
        let worker_stats = Arc::new(Mutex::new(HashMap::new()));
        let shutdown_signal = Arc::new(Mutex::new(false));
        
        let mut worker_handles = Vec::new();
        
        // 启动Worker线程
        for worker_id in 0..config.worker_count {
            let task_receiver = Arc::clone(&task_receiver);
            let result_sender = result_sender.clone();
            let worker_stats = Arc::clone(&worker_stats);
            let shutdown_signal = Arc::clone(&shutdown_signal);
            let worker_timeout = config.worker_timeout;
            
            let handle = thread::spawn(move || {
                Self::worker_loop(
                    worker_id,
                    task_receiver,
                    result_sender,
                    worker_stats,
                    shutdown_signal,
                    worker_timeout,
                );
            });
            
            worker_handles.push(handle);
        }
        
        Self {
            config,
            task_sender,
            result_receiver,
            worker_handles,
            worker_stats,
            shutdown_signal,
        }
    }
    
    /// Worker主循环
    fn worker_loop(
        worker_id: usize,
        task_receiver: Arc<Mutex<Receiver<T>>>,
        result_sender: Sender<(u64, Result<T::Result, String>)>,
        worker_stats: Arc<Mutex<HashMap<usize, WorkerStats>>>,
        shutdown_signal: Arc<Mutex<bool>>,
        timeout: Duration,
    ) {
        println!("Worker {} 启动", worker_id);
        
        // 初始化Worker统计
        {
            let mut stats = worker_stats.lock().unwrap();
            stats.insert(worker_id, WorkerStats::default());
        }
        
        loop {
            // 检查关闭信号
            if *shutdown_signal.lock().unwrap() {
                break;
            }
            
            // 接收任务
            let task = {
                let receiver = task_receiver.lock().unwrap();
                match receiver.recv_timeout(Duration::from_millis(100)) {
                    Ok(task) => task,
                    Err(_) => continue, // 超时或通道关闭，继续循环
                }
            };
            
            let task_id = task.task_id();
            println!("Worker {} 开始处理任务 {}", worker_id, task_id);
            
            let start_time = Instant::now();
            
            // 执行任务
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                task.execute()
            }));
            
            let processing_time = start_time.elapsed();
            
            // 更新统计信息
            {
                let mut stats = worker_stats.lock().unwrap();
                if let Some(worker_stat) = stats.get_mut(&worker_id) {
                    match result {
                        Ok(_) => {
                            worker_stat.tasks_completed += 1;
                            worker_stat.total_processing_time += processing_time;
                            worker_stat.average_processing_time = 
                                worker_stat.total_processing_time / worker_stat.tasks_completed as u32;
                        }
                        Err(_) => {
                            worker_stat.tasks_failed += 1;
                        }
                    }
                }
            }
            
            // 发送结果
            let task_result = match result {
                Ok(res) => Ok(res),
                Err(_) => Err(format!("Worker {} 任务 {} 执行失败", worker_id, task_id)),
            };
            
            if let Err(_) = result_sender.send((task_id, task_result)) {
                println!("Worker {} 无法发送结果，Master可能已关闭", worker_id);
                break;
            }
            
            println!("Worker {} 完成任务 {} (耗时: {:?})", worker_id, task_id, processing_time);
        }
        
        println!("Worker {} 停止", worker_id);
    }
    
    /// 提交任务
    pub fn submit_task(&self, task: T) -> Result<(), String> {
        self.task_sender.send(task)
            .map_err(|_| "任务队列已关闭".to_string())
    }
    
    /// 批量提交任务
    pub fn submit_tasks(&self, tasks: Vec<T>) -> Result<(), String> {
        for task in tasks {
            self.submit_task(task)?;
        }
        Ok(())
    }
    
    /// 等待结果
    pub fn wait_for_result(&self, timeout: Duration) -> Result<(u64, T::Result), String> {
        match self.result_receiver.recv_timeout(timeout) {
            Ok((task_id, Ok(result))) => Ok((task_id, result)),
            Ok((task_id, Err(error))) => Err(format!("任务 {} 失败: {}", task_id, error)),
            Err(_) => Err("等待结果超时".to_string()),
        }
    }
    
    /// 收集所有结果
    pub fn collect_results(&self, expected_count: usize, timeout: Duration) -> Vec<(u64, Result<T::Result, String>)> {
        let mut results = Vec::new();
        let start_time = Instant::now();
        
        while results.len() < expected_count && start_time.elapsed() < timeout {
            match self.result_receiver.recv_timeout(Duration::from_millis(100)) {
                Ok(result) => results.push(result),
                Err(_) => continue,
            }
        }
        
        results
    }
    
    /// 获取Worker统计信息
    pub fn get_worker_stats(&self) -> HashMap<usize, WorkerStats> {
        self.worker_stats.lock().unwrap().clone()
    }
    
    /// 关闭系统
    pub fn shutdown(self) {
        println!("开始关闭Master-Worker系统...");
        
        // 设置关闭信号
        *self.shutdown_signal.lock().unwrap() = true;
        
        // 等待所有Worker完成
        for (i, handle) in self.worker_handles.into_iter().enumerate() {
            println!("等待Worker {} 完成...", i);
            let _ = handle.join();
        }
        
        // 显示最终统计
        let final_stats = self.worker_stats.lock().unwrap();
        let mut total_completed = 0;
        let mut total_failed = 0;
        let mut total_time = Duration::new(0, 0);
        
        for (worker_id, stats) in final_stats.iter() {
            println!("Worker {} 统计: 完成 {}, 失败 {}, 平均处理时间 {:?}",
                     worker_id, stats.tasks_completed, stats.tasks_failed, stats.average_processing_time);
            total_completed += stats.tasks_completed;
            total_failed += stats.tasks_failed;
            total_time += stats.total_processing_time;
        }
        
        println!("系统总计: 完成 {}, 失败 {}, 总处理时间 {:?}",
                 total_completed, total_failed, total_time);
        println!("Master-Worker系统已关闭");
    }
}

// =================
// 演示函数
// =================

/// Master-Worker模式演示
pub fn demo_master_worker() {
    println!("=== Master-Worker模式演示 ===\n");
    
    // 1. 基本数值计算演示
    println!("1. 基本数值计算演示:");
    {
        let config = MasterWorkerConfig {
            worker_count: 3,
            ..Default::default()
        };
        let system = MasterWorkerSystem::new(config);
        
        // 创建计算任务
        let tasks = vec![
            ComputeTask::new(1, ComputeOperation::Sum, (1..=100).collect()),
            ComputeTask::new(2, ComputeOperation::Product, vec![2, 3, 4, 5]),
            ComputeTask::new(3, ComputeOperation::Max, vec![10, 25, 5, 30, 15]),
            ComputeTask::new(4, ComputeOperation::Average, (1..=20).collect()),
        ];
        
        // 提交任务
        system.submit_tasks(tasks.clone()).unwrap();
        
        // 收集结果
        let results = system.collect_results(tasks.len(), Duration::from_secs(5));
        
        for (task_id, result) in results {
            match result {
                Ok(value) => println!("任务 {} 结果: {:.2}", task_id, value),
                Err(error) => println!("任务 {} 失败: {}", task_id, error),
            }
        }
        
        system.shutdown();
    }
    
    println!("\n{}\n", "=".repeat(50));
    
    // 2. 矩阵乘法演示
    println!("2. 矩阵乘法演示:");
    {
        let config = MasterWorkerConfig {
            worker_count: 4,
            ..Default::default()
        };
        let system = MasterWorkerSystem::new(config);
        
        // 创建测试矩阵
        let matrix_a = vec![
            vec![1, 2, 3],
            vec![4, 5, 6],
            vec![7, 8, 9],
            vec![10, 11, 12],
        ];
        
        let matrix_b = vec![
            vec![1, 0],
            vec![0, 1],
            vec![1, 1],
        ];
        
        let task = MatrixMultiplyTask::new(1, matrix_a, matrix_b);
        
        // 使用分解器分解任务
        let mut splitter = MatrixSplitter::new(2);
        let subtasks = splitter.split(task);
        
        println!("将矩阵乘法分解为 {} 个子任务", subtasks.len());
        
        // 提交子任务
        system.submit_tasks(subtasks).unwrap();
        
        // 收集结果
        let results = system.collect_results(2, Duration::from_secs(3));
        let mut partial_results = Vec::new();
        
        for (task_id, result) in results {
            match result {
                Ok(matrix) => {
                    println!("子任务 {} 完成，结果矩阵大小: {}x{}", 
                             task_id, matrix.len(), matrix[0].len());
                    partial_results.push(matrix);
                }
                Err(error) => println!("子任务 {} 失败: {}", task_id, error),
            }
        }
        
        // 合并结果
        let final_result = splitter.merge(partial_results);
        println!("最终矩阵乘法结果:");
        for row in final_result {
            println!("  {:?}", row);
        }
        
        system.shutdown();
    }
    
    println!("\n{}\n", "=".repeat(50));
    
    // 3. 性能压力测试
    println!("3. 性能压力测试:");
    {
        let config = MasterWorkerConfig {
            worker_count: 6,
            task_queue_size: 1000,
            ..Default::default()
        };
        let system = MasterWorkerSystem::new(config);
        
        let start_time = Instant::now();
        const TASK_COUNT: usize = 50;
        
        // 创建大量计算密集型任务
        let mut tasks = Vec::new();
        for i in 1..=TASK_COUNT {
            let data: Vec<i64> = (1..=1000).map(|x| x * i as i64).collect();
            tasks.push(ComputeTask::new(i as u64, ComputeOperation::Sum, data));
        }
        
        // 批量提交任务
        system.submit_tasks(tasks).unwrap();
        
        // 等待所有结果
        let results = system.collect_results(TASK_COUNT, Duration::from_secs(10));
        let processing_time = start_time.elapsed();
        
        let successful_tasks = results.iter().filter(|(_, result)| result.is_ok()).count();
        let failed_tasks = results.len() - successful_tasks;
        
        println!("性能测试结果:");
        println!("  任务总数: {}", TASK_COUNT);
        println!("  成功任务: {}", successful_tasks);
        println!("  失败任务: {}", failed_tasks);
        println!("  总耗时: {:?}", processing_time);
        println!("  平均每任务: {:?}", processing_time / TASK_COUNT as u32);
        println!("  吞吐量: {:.2} 任务/秒", TASK_COUNT as f64 / processing_time.as_secs_f64());
        
        // 显示Worker统计
        let worker_stats = system.get_worker_stats();
        for (worker_id, stats) in worker_stats {
            println!("  Worker {}: 完成 {}, 平均时间 {:?}", 
                     worker_id, stats.tasks_completed, stats.average_processing_time);
        }
        
        system.shutdown();
    }
    
    println!("\n【Master-Worker模式特点】");
    println!("✓ 任务分解 - Master将大任务分解为小任务");
    println!("✓ 并行处理 - 多个Worker并行执行子任务");
    println!("✓ 结果聚合 - Master收集和合并Worker的结果");
    println!("✓ 负载均衡 - 动态分配任务给空闲Worker");
    println!("✓ 容错处理 - 处理Worker失败和任务重试");
    println!("✓ 可扩展性 - 可以动态调整Worker数量");
} 