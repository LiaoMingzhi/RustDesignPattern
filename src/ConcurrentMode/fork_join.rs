/*
 * 文件路径: /d%3A/workspace/RustLearn/RustDesignPattern/src/ConcurrentMode/fork_join.rs
 * 
 * Fork-Join模式 (Fork-Join Pattern)
 * 
 * Fork-Join模式是一种分而治之的并行计算模式，将大任务递归地分解为小任务，
 * 并行执行后再合并结果。适用于可以分解的计算密集型任务。
 * 
 * 主要特点：
 * 1. 分而治之 - 递归地将大任务分解为小任务
 * 2. 并行执行 - 子任务可以并行执行
 * 3. 结果合并 - 将子任务结果合并为最终结果
 * 4. 工作窃取 - 空闲线程可以窃取其他线程的任务
 * 5. 动态负载均衡 - 自动平衡工作负载
 */

use std::sync::{Arc, Mutex, Condvar};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};
use std::collections::VecDeque;
use std::fmt;

// =================
// Fork-Join任务特质
// =================

/// Fork-Join任务特质
pub trait ForkJoinTask<T>: Send + 'static {
    /// 执行任务的计算部分
    fn compute(&self) -> T;
    
    /// 判断任务是否足够小，可以直接计算
    fn is_small_enough(&self) -> bool;
    
    /// 将任务分解为子任务
    fn fork(&self) -> Vec<Box<dyn ForkJoinTask<T>>>;
    
    /// 合并子任务的结果
    fn join(&self, results: Vec<T>) -> T;
    
    /// 获取任务描述
    fn description(&self) -> String {
        "Fork-Join任务".to_string()
    }
}

/// Fork-Join错误类型
#[derive(Debug)]
pub enum ForkJoinError {
    TaskFailed(String),
    PoolShutdown,
    Timeout,
    WorkerPanic,
}

impl fmt::Display for ForkJoinError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ForkJoinError::TaskFailed(msg) => write!(f, "任务失败: {}", msg),
            ForkJoinError::PoolShutdown => write!(f, "线程池已关闭"),
            ForkJoinError::Timeout => write!(f, "执行超时"),
            ForkJoinError::WorkerPanic => write!(f, "工作线程恐慌"),
        }
    }
}

// =================
// 具体任务实现
// =================

/// 归并排序任务
#[derive(Debug, Clone)]
pub struct MergeSortTask {
    data: Vec<i32>,
    threshold: usize, // 阈值，小于此值直接排序
}

impl MergeSortTask {
    pub fn new(data: Vec<i32>, threshold: usize) -> Self {
        Self { data, threshold }
    }
    
    fn merge(&self, left: Vec<i32>, right: Vec<i32>) -> Vec<i32> {
        let mut result = Vec::with_capacity(left.len() + right.len());
        let mut i = 0;
        let mut j = 0;
        
        while i < left.len() && j < right.len() {
            if left[i] <= right[j] {
                result.push(left[i]);
                i += 1;
            } else {
                result.push(right[j]);
                j += 1;
            }
        }
        
        result.extend_from_slice(&left[i..]);
        result.extend_from_slice(&right[j..]);
        result
    }
    
    fn sequential_sort(&self, mut data: Vec<i32>) -> Vec<i32> {
        data.sort_unstable();
        data
    }
}

impl ForkJoinTask<Vec<i32>> for MergeSortTask {
    fn compute(&self) -> Vec<i32> {
        self.sequential_sort(self.data.clone())
    }
    
    fn is_small_enough(&self) -> bool {
        self.data.len() <= self.threshold
    }
    
    fn fork(&self) -> Vec<Box<dyn ForkJoinTask<Vec<i32>>>> {
        let mid = self.data.len() / 2;
        let left_data = self.data[..mid].to_vec();
        let right_data = self.data[mid..].to_vec();
        
        vec![
            Box::new(MergeSortTask::new(left_data, self.threshold)),
            Box::new(MergeSortTask::new(right_data, self.threshold)),
        ]
    }
    
    fn join(&self, results: Vec<Vec<i32>>) -> Vec<i32> {
        if results.len() == 2 {
            self.merge(results[0].clone(), results[1].clone())
        } else {
            panic!("归并排序应该有两个子结果");
        }
    }
    
    fn description(&self) -> String {
        format!("归并排序任务 (大小: {})", self.data.len())
    }
}

/// 快速排序任务
#[derive(Debug, Clone)]
pub struct QuickSortTask {
    data: Vec<i32>,
    threshold: usize,
}

impl QuickSortTask {
    pub fn new(data: Vec<i32>, threshold: usize) -> Self {
        Self { data, threshold }
    }
    
    fn partition(&self, data: &mut Vec<i32>) -> usize {
        let pivot_index = data.len() - 1;
        let pivot = data[pivot_index];
        let mut i = 0;
        
        for j in 0..pivot_index {
            if data[j] < pivot {
                data.swap(i, j);
                i += 1;
            }
        }
        
        data.swap(i, pivot_index);
        i
    }
    
    fn sequential_sort(&self, mut data: Vec<i32>) -> Vec<i32> {
        if data.len() <= 1 {
            return data;
        }
        
        let pivot_index = self.partition(&mut data);
        let mut left = data[..pivot_index].to_vec();
        let pivot = data[pivot_index];
        let mut right = data[pivot_index + 1..].to_vec();
        
        left = self.sequential_sort(left);
        right = self.sequential_sort(right);
        
        let mut result = left;
        result.push(pivot);
        result.extend(right);
        result
    }
}

impl ForkJoinTask<Vec<i32>> for QuickSortTask {
    fn compute(&self) -> Vec<i32> {
        self.sequential_sort(self.data.clone())
    }
    
    fn is_small_enough(&self) -> bool {
        self.data.len() <= self.threshold
    }
    
    fn fork(&self) -> Vec<Box<dyn ForkJoinTask<Vec<i32>>>> {
        if self.data.len() <= 1 {
            return vec![Box::new(QuickSortTask::new(self.data.clone(), self.threshold))];
        }
        
        let mut data = self.data.clone();
        let pivot_index = self.partition(&mut data);
        
        let left_data = data[..pivot_index].to_vec();
        let right_data = data[pivot_index + 1..].to_vec();
        let pivot = data[pivot_index];
        
        let mut subtasks = Vec::new();
        if !left_data.is_empty() {
            subtasks.push(Box::new(QuickSortTask::new(left_data, self.threshold)) as Box<dyn ForkJoinTask<Vec<i32>>>);
        }
        if !right_data.is_empty() {
            subtasks.push(Box::new(QuickSortTask::new(right_data, self.threshold)) as Box<dyn ForkJoinTask<Vec<i32>>>);
        }
        
        // 如果没有子任务，返回包含pivot的任务
        if subtasks.is_empty() {
            subtasks.push(Box::new(QuickSortTask::new(vec![pivot], self.threshold)));
        }
        
        subtasks
    }
    
    fn join(&self, results: Vec<Vec<i32>>) -> Vec<i32> {
        let mut combined = Vec::new();
        for result in results {
            combined.extend(result);
        }
        combined
    }
    
    fn description(&self) -> String {
        format!("快速排序任务 (大小: {})", self.data.len())
    }
}

/// 矩阵乘法任务
#[derive(Debug, Clone)]
pub struct MatrixMultiplyTask {
    matrix_a: Vec<Vec<i32>>,
    matrix_b: Vec<Vec<i32>>,
    threshold: usize,
}

impl MatrixMultiplyTask {
    pub fn new(matrix_a: Vec<Vec<i32>>, matrix_b: Vec<Vec<i32>>, threshold: usize) -> Self {
        Self { matrix_a, matrix_b, threshold }
    }
    
    fn sequential_multiply(&self) -> Vec<Vec<i32>> {
        let rows_a = self.matrix_a.len();
        let cols_a = self.matrix_a[0].len();
        let cols_b = self.matrix_b[0].len();
        
        let mut result = vec![vec![0; cols_b]; rows_a];
        
        for i in 0..rows_a {
            for j in 0..cols_b {
                for k in 0..cols_a {
                    result[i][j] += self.matrix_a[i][k] * self.matrix_b[k][j];
                }
            }
        }
        
        result
    }
    
    fn split_matrix_a(&self) -> (Vec<Vec<i32>>, Vec<Vec<i32>>) {
        let mid = self.matrix_a.len() / 2;
        let top = self.matrix_a[..mid].to_vec();
        let bottom = self.matrix_a[mid..].to_vec();
        (top, bottom)
    }
}

impl ForkJoinTask<Vec<Vec<i32>>> for MatrixMultiplyTask {
    fn compute(&self) -> Vec<Vec<i32>> {
        self.sequential_multiply()
    }
    
    fn is_small_enough(&self) -> bool {
        self.matrix_a.len() <= self.threshold
    }
    
    fn fork(&self) -> Vec<Box<dyn ForkJoinTask<Vec<Vec<i32>>>>> {
        let (top_a, bottom_a) = self.split_matrix_a();
        
        vec![
            Box::new(MatrixMultiplyTask::new(top_a, self.matrix_b.clone(), self.threshold)),
            Box::new(MatrixMultiplyTask::new(bottom_a, self.matrix_b.clone(), self.threshold)),
        ]
    }
    
    fn join(&self, results: Vec<Vec<Vec<i32>>>) -> Vec<Vec<i32>> {
        let mut combined = Vec::new();
        for result in results {
            combined.extend(result);
        }
        combined
    }
    
    fn description(&self) -> String {
        format!("矩阵乘法任务 ({}x{} * {}x{})",
                self.matrix_a.len(), self.matrix_a[0].len(),
                self.matrix_b.len(), self.matrix_b[0].len())
    }
}

/// 数组求和任务
#[derive(Debug, Clone)]
pub struct SumTask {
    data: Vec<i64>,
    threshold: usize,
}

impl SumTask {
    pub fn new(data: Vec<i64>, threshold: usize) -> Self {
        Self { data, threshold }
    }
}

impl ForkJoinTask<i64> for SumTask {
    fn compute(&self) -> i64 {
        self.data.iter().sum()
    }
    
    fn is_small_enough(&self) -> bool {
        self.data.len() <= self.threshold
    }
    
    fn fork(&self) -> Vec<Box<dyn ForkJoinTask<i64>>> {
        let mid = self.data.len() / 2;
        let left_data = self.data[..mid].to_vec();
        let right_data = self.data[mid..].to_vec();
        
        vec![
            Box::new(SumTask::new(left_data, self.threshold)),
            Box::new(SumTask::new(right_data, self.threshold)),
        ]
    }
    
    fn join(&self, results: Vec<i64>) -> i64 {
        results.iter().sum()
    }
    
    fn description(&self) -> String {
        format!("数组求和任务 (大小: {})", self.data.len())
    }
}

// =================
// Fork-Join线程池
// =================

/// Fork-Join线程池配置
#[derive(Debug, Clone)]
pub struct ForkJoinConfig {
    pub worker_count: usize,
    pub queue_capacity: usize,
    pub steal_threshold: usize,
    pub enable_work_stealing: bool,
}

impl Default for ForkJoinConfig {
    fn default() -> Self {
        let worker_count = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4);
        
        Self {
            worker_count,
            queue_capacity: 1000,
            steal_threshold: 10,
            enable_work_stealing: true,
        }
    }
}

/// 工作窃取队列
pub struct WorkStealingQueue<T> {
    queue: VecDeque<T>,
    steal_count: usize,
}

impl<T> WorkStealingQueue<T> {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
            steal_count: 0,
        }
    }
    
    pub fn push(&mut self, item: T) {
        self.queue.push_back(item);
    }
    
    pub fn pop(&mut self) -> Option<T> {
        self.queue.pop_back()
    }
    
    pub fn steal(&mut self) -> Option<T> {
        if let Some(item) = self.queue.pop_front() {
            self.steal_count += 1;
            Some(item)
        } else {
            None
        }
    }
    
    pub fn len(&self) -> usize {
        self.queue.len()
    }
    
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }
    
    pub fn steal_count(&self) -> usize {
        self.steal_count
    }
}

/// Fork-Join线程池
pub struct ForkJoinPool {
    config: ForkJoinConfig,
    worker_handles: Vec<JoinHandle<()>>,
    task_queues: Vec<Arc<Mutex<WorkStealingQueue<Box<dyn FnOnce() + Send>>>>>,
    result_sender: Sender<Box<dyn std::any::Any + Send>>,
    result_receiver: Receiver<Box<dyn std::any::Any + Send>>,
    shutdown_signal: Arc<Mutex<bool>>,
}

impl ForkJoinPool {
    /// 创建新的Fork-Join线程池
    pub fn new(config: ForkJoinConfig) -> Self {
        let (result_sender, result_receiver) = mpsc::channel();
        let shutdown_signal = Arc::new(Mutex::new(false));
        let mut task_queues = Vec::new();
        let mut worker_handles = Vec::new();
        
        // 为每个工作线程创建队列
        for _ in 0..config.worker_count {
            task_queues.push(Arc::new(Mutex::new(WorkStealingQueue::new())));
        }
        
        // 启动工作线程
        for worker_id in 0..config.worker_count {
            let worker_queues = task_queues.clone();
            let shutdown = Arc::clone(&shutdown_signal);
            let steal_threshold = config.steal_threshold;
            let enable_stealing = config.enable_work_stealing;
            
            let handle = thread::spawn(move || {
                Self::worker_loop(
                    worker_id,
                    worker_queues,
                    shutdown,
                    steal_threshold,
                    enable_stealing,
                );
            });
            
            worker_handles.push(handle);
        }
        
        Self {
            config,
            worker_handles,
            task_queues,
            result_sender,
            result_receiver,
            shutdown_signal,
        }
    }
    
    /// 使用默认配置创建线程池
    pub fn with_default_config() -> Self {
        Self::new(ForkJoinConfig::default())
    }
    
    /// 工作线程主循环
    fn worker_loop(
        worker_id: usize,
        worker_queues: Vec<Arc<Mutex<WorkStealingQueue<Box<dyn FnOnce() + Send>>>>>,
        shutdown: Arc<Mutex<bool>>,
        steal_threshold: usize,
        enable_stealing: bool,
    ) {
        println!("Fork-Join工作线程 {} 启动", worker_id);
        
        let my_queue = &worker_queues[worker_id];
        let mut idle_count = 0;
        
        loop {
            if *shutdown.lock().unwrap() {
                break;
            }
            
            // 尝试从自己的队列获取任务
            let task = {
                let mut queue = my_queue.lock().unwrap();
                queue.pop()
            };
            
            if let Some(task) = task {
                idle_count = 0;
                task(); // 执行任务
            } else {
                idle_count += 1;
                
                // 如果启用工作窃取且空闲时间足够长，尝试窃取任务
                if enable_stealing && idle_count > steal_threshold {
                    let mut stolen = false;
                    
                    for (i, other_queue) in worker_queues.iter().enumerate() {
                        if i != worker_id {
                            let task = {
                                let mut queue = other_queue.lock().unwrap();
                                if queue.len() > 1 { // 只有当队列有多个任务时才窃取
                                    queue.steal()
                                } else {
                                    None
                                }
                            };
                            
                            if let Some(task) = task {
                                println!("工作线程 {} 从线程 {} 窃取任务", worker_id, i);
                                task();
                                stolen = true;
                                idle_count = 0;
                                break;
                            }
                        }
                    }
                    
                    if !stolen {
                        thread::sleep(Duration::from_millis(1)); // 短暂休息
                    }
                } else {
                    thread::sleep(Duration::from_millis(1));
                }
            }
        }
        
        println!("Fork-Join工作线程 {} 停止", worker_id);
    }
    
    /// 提交Fork-Join任务
    pub fn submit<T: Clone + Send + 'static>(&self, task: Box<dyn ForkJoinTask<T>>) -> Result<T, ForkJoinError> {
        if *self.shutdown_signal.lock().unwrap() {
            return Err(ForkJoinError::PoolShutdown);
        }
        
        let result_sender = self.result_sender.clone();
        
        // 选择负载最轻的队列
        let queue_index = self.find_lightest_queue();
        
        let wrapped_task = Box::new(move || {
            let result = Self::execute_recursive(task);
            let boxed_result = Box::new(result) as Box<dyn std::any::Any + Send>;
            let _ = result_sender.send(boxed_result);
        });
        
        {
            let mut queue = self.task_queues[queue_index].lock().unwrap();
            queue.push(wrapped_task);
        }
        
        // 等待结果
        match self.result_receiver.recv() {
            Ok(boxed_result) => {
                if let Ok(result) = boxed_result.downcast::<T>() {
                    Ok(*result)
                } else {
                    Err(ForkJoinError::TaskFailed("类型转换失败".to_string()))
                }
            }
            Err(_) => Err(ForkJoinError::TaskFailed("接收结果失败".to_string())),
        }
    }
    
    /// 递归执行Fork-Join任务
    fn execute_recursive<T: Clone + Send + 'static>(task: Box<dyn ForkJoinTask<T>>) -> T {
        if task.is_small_enough() {
            task.compute()
        } else {
            let subtasks = task.fork();
            let mut results = Vec::new();
            
            // 递归处理子任务（这里简化为顺序执行，实际应该并行）
            for subtask in subtasks {
                let result = Self::execute_recursive(subtask);
                results.push(result);
            }
            
            task.join(results)
        }
    }
    
    /// 找到负载最轻的队列
    fn find_lightest_queue(&self) -> usize {
        let mut min_size = usize::MAX;
        let mut lightest_index = 0;
        
        for (i, queue) in self.task_queues.iter().enumerate() {
            let size = queue.lock().unwrap().len();
            if size < min_size {
                min_size = size;
                lightest_index = i;
            }
        }
        
        lightest_index
    }
    
    /// 获取队列统计信息
    pub fn get_queue_stats(&self) -> Vec<(usize, usize)> {
        self.task_queues.iter()
            .map(|queue| {
                let q = queue.lock().unwrap();
                (q.len(), q.steal_count())
            })
            .collect()
    }
    
    /// 关闭线程池
    pub fn shutdown(self) {
        println!("开始关闭Fork-Join线程池...");
        
        // 显示最终统计（在移动之前）
        let stats = self.get_queue_stats();
        println!("最终队列统计:");
        for (i, (queue_size, steal_count)) in stats.iter().enumerate() {
            println!("  队列 {}: 剩余任务 {}, 窃取次数 {}", i, queue_size, steal_count);
        }
        
        *self.shutdown_signal.lock().unwrap() = true;
        
        for (i, handle) in self.worker_handles.into_iter().enumerate() {
            println!("等待工作线程 {} 完成...", i);
            let _ = handle.join();
        }
        
        println!("Fork-Join线程池已关闭");
    }
}

// =================
// 演示函数
// =================

/// Fork-Join模式演示
pub fn demo_fork_join() {
    println!("=== Fork-Join模式演示 ===\n");
    
    // 1. 归并排序演示
    println!("1. 归并排序演示:");
    {
        let pool = ForkJoinPool::with_default_config();
        
        // 创建测试数据
        let mut data: Vec<i32> = (1..=1000).rev().collect(); // 逆序数据
        data.extend_from_slice(&[5, 3, 8, 1, 9, 2, 7, 4, 6]);
        
        println!("原始数据前10个: {:?}", &data[..10]);
        println!("原始数据大小: {}", data.len());
        
        let task = Box::new(MergeSortTask::new(data.clone(), 50));
        
        let start_time = Instant::now();
        match pool.submit(task) {
            Ok(sorted_data) => {
                let elapsed = start_time.elapsed();
                println!("排序完成，耗时: {:?}", elapsed);
                println!("排序后前10个: {:?}", &sorted_data[..10]);
                println!("排序后后10个: {:?}", &sorted_data[sorted_data.len()-10..]);
                
                // 验证排序正确性
                let mut verification_data = data;
                verification_data.sort_unstable();
                let is_correct = sorted_data == verification_data;
                println!("排序正确性: {}", if is_correct { "正确" } else { "错误" });
            }
            Err(e) => println!("排序失败: {}", e),
        }
        
        pool.shutdown();
    }
    
    println!("\n{}\n", "=".repeat(50));
    
    // 2. 矩阵乘法演示
    println!("2. 矩阵乘法演示:");
    {
        let pool = ForkJoinPool::with_default_config();
        
        // 创建测试矩阵
        let size = 64;
        let matrix_a: Vec<Vec<i32>> = (0..size)
            .map(|i| (0..size).map(|j| (i + j) % 10).collect())
            .collect();
        
        let matrix_b: Vec<Vec<i32>> = (0..size)
            .map(|i| (0..size).map(|j| (i * j) % 10).collect())
            .collect();
        
        println!("矩阵大小: {}x{}", size, size);
        
        let task = Box::new(MatrixMultiplyTask::new(matrix_a, matrix_b, 8));
        
        let start_time = Instant::now();
        match pool.submit(task) {
            Ok(result_matrix) => {
                let elapsed = start_time.elapsed();
                println!("矩阵乘法完成，耗时: {:?}", elapsed);
                println!("结果矩阵大小: {}x{}", result_matrix.len(), result_matrix[0].len());
                println!("结果矩阵左上角3x3:");
                for i in 0..3 {
                    println!("  {:?}", &result_matrix[i][..3]);
                }
            }
            Err(e) => println!("矩阵乘法失败: {}", e),
        }
        
        pool.shutdown();
    }
    
    println!("\n{}\n", "=".repeat(50));
    
    // 3. 大数组求和演示
    println!("3. 大数组求和演示:");
    {
        let config = ForkJoinConfig {
            worker_count: 6,
            enable_work_stealing: true,
            ..Default::default()
        };
        let pool = ForkJoinPool::new(config);
        
        // 创建大数组
        const ARRAY_SIZE: usize = 10_000_000;
        let data: Vec<i64> = (1..=ARRAY_SIZE as i64).collect();
        println!("数组大小: {}", ARRAY_SIZE);
        
        let task = Box::new(SumTask::new(data.clone(), 10000));
        
        let start_time = Instant::now();
        match pool.submit(task) {
            Ok(sum) => {
                let elapsed = start_time.elapsed();
                println!("并行求和完成，耗时: {:?}", elapsed);
                println!("并行计算结果: {}", sum);
                
                // 验证结果
                let expected_sum = (ARRAY_SIZE as i64 * (ARRAY_SIZE as i64 + 1)) / 2;
                println!("期望结果: {}", expected_sum);
                println!("结果正确性: {}", if sum == expected_sum { "正确" } else { "错误" });
                
                // 性能对比
                let seq_start = Instant::now();
                let seq_sum: i64 = data.iter().sum();
                let seq_elapsed = seq_start.elapsed();
                println!("顺序求和耗时: {:?}", seq_elapsed);
                println!("加速比: {:.2}x", seq_elapsed.as_secs_f64() / elapsed.as_secs_f64());
            }
            Err(e) => println!("求和失败: {}", e),
        }
        
        // 显示工作窃取统计
        let stats = pool.get_queue_stats();
        println!("工作窃取统计:");
        for (i, (queue_size, steal_count)) in stats.iter().enumerate() {
            println!("  线程 {}: 剩余任务 {}, 窃取次数 {}", i, queue_size, steal_count);
        }
        
        pool.shutdown();
    }
    
    println!("\n{}\n", "=".repeat(50));
    
    // 4. 快速排序演示
    println!("4. 快速排序演示:");
    {
        let pool = ForkJoinPool::with_default_config();
        
        // 创建随机数据
        let mut data: Vec<i32> = Vec::new();
        for i in 0..5000 {
            data.push((i * 17 + 37) % 10000); // 伪随机数据
        }
        
        println!("原始数据大小: {}", data.len());
        println!("原始数据前10个: {:?}", &data[..10]);
        
        let task = Box::new(QuickSortTask::new(data.clone(), 100));
        
        let start_time = Instant::now();
        match pool.submit(task) {
            Ok(sorted_data) => {
                let elapsed = start_time.elapsed();
                println!("快速排序完成，耗时: {:?}", elapsed);
                println!("排序后前10个: {:?}", &sorted_data[..10]);
                
                // 验证排序
                let is_sorted = sorted_data.windows(2).all(|w| w[0] <= w[1]);
                println!("排序正确性: {}", if is_sorted { "正确" } else { "错误" });
            }
            Err(e) => println!("快速排序失败: {}", e),
        }
        
        pool.shutdown();
    }
    
    println!("\n【Fork-Join模式特点】");
    println!("✓ 分而治之 - 递归地将大任务分解为小任务");
    println!("✓ 并行执行 - 子任务可以并行执行");
    println!("✓ 结果合并 - 将子任务结果合并为最终结果");
    println!("✓ 工作窃取 - 空闲线程可以窃取其他线程的任务");
    println!("✓ 动态负载均衡 - 自动平衡工作负载");
    println!("✓ 高效并行 - 充分利用多核处理器性能");
} 