/*
 * 文件路径: /d%3A/workspace/RustLearn/RustDesignPattern/src/ConcurrentMode/future_promise.rs
 * 
 * Future-Promise模式 (Future-Promise Pattern)
 * 
 * Future-Promise模式是一种异步编程模式，用于处理可能在未来完成的计算。
 * Future代表一个未来的值，Promise用于设置这个值。这种模式允许非阻塞的异步操作。
 * 
 * 主要特点：
 * 1. 异步计算 - 不阻塞当前线程等待结果
 * 2. 组合性 - 可以组合多个Future操作
 * 3. 错误处理 - 统一的错误传播机制
 * 4. 回调链 - 支持链式操作和转换
 * 5. 并发控制 - 可以并行执行多个异步操作
 */

use std::sync::{Arc, Mutex, Condvar};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};
use std::collections::HashMap;
use std::fmt;

// =================
// Future和Promise核心定义
// =================

/// Future状态
#[derive(Debug, Clone)]
pub enum FutureState<T, E> {
    Pending,
    Completed(T),
    Failed(E),
}

/// 异步计算错误
#[derive(Debug, Clone)]
pub enum AsyncError {
    Timeout,
    Cancelled,
    ComputationFailed(String),
    DependencyFailed,
    ResourceUnavailable,
}

impl fmt::Display for AsyncError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AsyncError::Timeout => write!(f, "操作超时"),
            AsyncError::Cancelled => write!(f, "操作被取消"),
            AsyncError::ComputationFailed(msg) => write!(f, "计算失败: {}", msg),
            AsyncError::DependencyFailed => write!(f, "依赖失败"),
            AsyncError::ResourceUnavailable => write!(f, "资源不可用"),
        }
    }
}

/// Future特质
pub trait Future<T, E>: Send + 'static {
    /// 检查Future是否完成
    fn is_ready(&self) -> bool;
    
    /// 非阻塞获取结果
    fn poll(&self) -> FutureState<T, E>;
    
    /// 阻塞等待结果
    fn wait(self) -> Result<T, E>;
    
    /// 带超时的等待
    fn wait_timeout(self, timeout: Duration) -> Result<T, E>;
    
    /// 添加完成回调
    fn on_complete<F>(self, callback: F) -> Self
    where
        F: FnOnce(Result<T, E>) + Send + 'static;
}

/// 简单Future实现
pub struct SimpleFuture<T: Clone + Send + 'static, E: Clone + Send + 'static> {
    state: Arc<Mutex<FutureState<T, E>>>,
    completed: Arc<Condvar>,
    callbacks: Arc<Mutex<Vec<Box<dyn FnOnce(Result<T, E>) + Send + 'static>>>>,
}

impl<T: Clone + Send + 'static, E: Clone + Send + 'static> SimpleFuture<T, E> {
    pub fn new() -> (Self, Promise<T, E>) {
        let state = Arc::new(Mutex::new(FutureState::Pending));
        let completed = Arc::new(Condvar::new());
        let callbacks = Arc::new(Mutex::new(Vec::new()));
        
        let future = SimpleFuture {
            state: Arc::clone(&state),
            completed: Arc::clone(&completed),
            callbacks: Arc::clone(&callbacks),
        };
        
        let promise = Promise {
            state,
            completed,
            callbacks,
        };
        
        (future, promise)
    }
    
    /// 创建已完成的Future
    pub fn completed(value: T) -> Self {
        let state = Arc::new(Mutex::new(FutureState::Completed(value)));
        let completed = Arc::new(Condvar::new());
        let callbacks = Arc::new(Mutex::new(Vec::new()));
        
        SimpleFuture {
            state,
            completed,
            callbacks,
        }
    }
    
    /// 创建已失败的Future
    pub fn failed(error: E) -> Self {
        let state = Arc::new(Mutex::new(FutureState::Failed(error)));
        let completed = Arc::new(Condvar::new());
        let callbacks = Arc::new(Mutex::new(Vec::new()));
        
        SimpleFuture {
            state,
            completed,
            callbacks,
        }
    }
    
    /// 映射Future的值
    pub fn map<U, F>(self, f: F) -> SimpleFuture<U, E>
    where
        U: Clone + Send + 'static,
        F: FnOnce(T) -> U + Send + 'static,
    {
        let (new_future, new_promise) = SimpleFuture::new();
        
        self.on_complete(move |result| {
            match result {
                Ok(value) => new_promise.complete(f(value)),
                Err(error) => new_promise.fail(error),
            }
        });
        
        new_future
    }
    
    /// 链接另一个Future
    pub fn and_then<U, F>(self, f: F) -> SimpleFuture<U, E>
    where
        U: Clone + Send + 'static,
        F: FnOnce(T) -> SimpleFuture<U, E> + Send + 'static,
    {
        let (new_future, new_promise) = SimpleFuture::new();
        
        self.on_complete(move |result| {
            match result {
                Ok(value) => {
                    let next_future = f(value);
                    next_future.on_complete(move |next_result| {
                        match next_result {
                            Ok(next_value) => new_promise.complete(next_value),
                            Err(next_error) => new_promise.fail(next_error),
                        }
                    });
                }
                Err(error) => new_promise.fail(error),
            }
        });
        
        new_future
    }
}

impl<T: Clone + Send + 'static, E: Clone + Send + 'static> Future<T, E> for SimpleFuture<T, E> {
    fn is_ready(&self) -> bool {
        !matches!(*self.state.lock().unwrap(), FutureState::Pending)
    }
    
    fn poll(&self) -> FutureState<T, E> {
        self.state.lock().unwrap().clone()
    }
    
    fn wait(self) -> Result<T, E> {
        let mut state = self.state.lock().unwrap();
        while matches!(*state, FutureState::Pending) {
            state = self.completed.wait(state).unwrap();
        }
        
        match state.clone() {
            FutureState::Completed(value) => Ok(value),
            FutureState::Failed(error) => Err(error),
            FutureState::Pending => unreachable!(),
        }
    }
    
    fn wait_timeout(self, timeout: Duration) -> Result<T, E> {
        let mut state = self.state.lock().unwrap();
        let start_time = Instant::now();
        
        while matches!(*state, FutureState::Pending) && start_time.elapsed() < timeout {
            let remaining = timeout - start_time.elapsed();
            if remaining.is_zero() {
                break;
            }
            
            let (new_state, wait_result) = self.completed.wait_timeout(state, remaining).unwrap();
            state = new_state;
            
            if wait_result.timed_out() {
                break;
            }
        }
        
        match state.clone() {
            FutureState::Completed(value) => Ok(value),
            FutureState::Failed(error) => Err(error),
            FutureState::Pending => {
                panic!("Future超时未完成");
            }
        }
    }
    
    fn on_complete<F>(self, callback: F) -> Self
    where
        F: FnOnce(Result<T, E>) + Send + 'static,
    {
        {
            let mut callbacks = self.callbacks.lock().unwrap();
            callbacks.push(Box::new(callback));
        }
        
        // 如果已经完成，立即执行回调
        if self.is_ready() {
            let state = self.state.lock().unwrap().clone();
            let mut callbacks = self.callbacks.lock().unwrap();
            
            let result = match state {
                FutureState::Completed(value) => Ok(value),
                FutureState::Failed(error) => Err(error),
                FutureState::Pending => unreachable!(),
            };
            
            if let Some(callback) = callbacks.pop() {
                drop(callbacks); // 释放锁
                callback(result);
            }
        }
        
        self
    }
}

/// Promise用于设置Future的值
pub struct Promise<T: Clone + Send + 'static, E: Clone + Send + 'static> {
    state: Arc<Mutex<FutureState<T, E>>>,
    completed: Arc<Condvar>,
    callbacks: Arc<Mutex<Vec<Box<dyn FnOnce(Result<T, E>) + Send + 'static>>>>,
}

impl<T: Clone + Send + 'static, E: Clone + Send + 'static> Promise<T, E> {
    /// 完成Promise并设置值
    pub fn complete(self, value: T) {
        {
            let mut state = self.state.lock().unwrap();
            if matches!(*state, FutureState::Pending) {
                *state = FutureState::Completed(value.clone());
            } else {
                return; // 已经完成或失败
            }
        }
        
        // 通知等待者
        self.completed.notify_all();
        
        // 执行回调
        let mut callbacks = self.callbacks.lock().unwrap();
        for callback in callbacks.drain(..) {
            callback(Ok(value.clone()));
        }
    }
    
    /// 使Promise失败
    pub fn fail(self, error: E) {
        {
            let mut state = self.state.lock().unwrap();
            if matches!(*state, FutureState::Pending) {
                *state = FutureState::Failed(error.clone());
            } else {
                return; // 已经完成或失败
            }
        }
        
        // 通知等待者
        self.completed.notify_all();
        
        // 执行回调
        let mut callbacks = self.callbacks.lock().unwrap();
        for callback in callbacks.drain(..) {
            callback(Err(error.clone()));
        }
    }
    
    /// 检查Promise是否已设置
    pub fn is_set(&self) -> bool {
        !matches!(*self.state.lock().unwrap(), FutureState::Pending)
    }
}

// =================
// Future组合器和工具
// =================

/// 组合多个Future，等待所有完成
pub fn join_all<T, E>(futures: Vec<SimpleFuture<T, E>>) -> SimpleFuture<Vec<T>, E>
where
    T: Clone + Send + 'static,
    E: Clone + Send + 'static,
{
    let (result_future, result_promise) = SimpleFuture::new();
    let futures_count = futures.len();
    
    if futures.is_empty() {
        result_promise.complete(Vec::new());
        return result_future;
    }
    
    let results = Arc::new(Mutex::new(vec![None; futures_count]));
    let completed_count = Arc::new(Mutex::new(0));
    let failed = Arc::new(Mutex::new(false));
    let result_promise = Arc::new(Mutex::new(Some(result_promise)));
    
    for (index, future) in futures.into_iter().enumerate() {
        let results = Arc::clone(&results);
        let completed_count = Arc::clone(&completed_count);
        let failed = Arc::clone(&failed);
        let result_promise = Arc::clone(&result_promise);
        
        future.on_complete(move |result| {
            match result {
                Ok(value) => {
                    {
                        let mut results = results.lock().unwrap();
                        results[index] = Some(value);
                    }
                    
                    let mut count = completed_count.lock().unwrap();
                    *count += 1;
                    
                    if *count == futures_count && !*failed.lock().unwrap() {
                        let results = results.lock().unwrap();
                        let final_results: Vec<T> = results.iter()
                            .map(|opt| opt.clone().unwrap())
                            .collect();
                        
                        if let Some(promise) = result_promise.lock().unwrap().take() {
                            promise.complete(final_results);
                        }
                    }
                }
                Err(error) => {
                    let mut is_failed = failed.lock().unwrap();
                    if !*is_failed {
                        *is_failed = true;
                        if let Some(promise) = result_promise.lock().unwrap().take() {
                            promise.fail(error);
                        }
                    }
                }
            }
        });
    }
    
    result_future
}

/// 选择第一个完成的Future
pub fn select_first<T, E>(futures: Vec<SimpleFuture<T, E>>) -> SimpleFuture<T, E>
where
    T: Clone + Send + 'static,
    E: Clone + Send + 'static,
{
    let (result_future, result_promise) = SimpleFuture::new();
    let completed = Arc::new(Mutex::new(false));
    let result_promise = Arc::new(Mutex::new(Some(result_promise)));
    
    for future in futures {
        let completed = Arc::clone(&completed);
        let result_promise = Arc::clone(&result_promise);
        
        future.on_complete(move |result| {
            let mut is_completed = completed.lock().unwrap();
            if !*is_completed {
                *is_completed = true;
                if let Some(promise) = result_promise.lock().unwrap().take() {
                    match result {
                        Ok(value) => promise.complete(value),
                        Err(error) => promise.fail(error),
                    }
                }
            }
        });
    }
    
    result_future
}

// =================
// 异步任务执行器
// =================

/// 异步任务执行器
pub struct AsyncExecutor {
    worker_handles: Vec<JoinHandle<()>>,
    task_sender: Sender<Box<dyn FnOnce() + Send + 'static>>,
    shutdown_signal: Arc<Mutex<bool>>,
}

impl AsyncExecutor {
    /// 创建新的异步执行器
    pub fn new(worker_count: usize) -> Self {
        let (task_sender, task_receiver) = mpsc::channel::<Box<dyn FnOnce() + Send + 'static>>();
        let task_receiver = Arc::new(Mutex::new(task_receiver));
        let shutdown_signal = Arc::new(Mutex::new(false));
        let mut worker_handles = Vec::new();
        
        for worker_id in 0..worker_count {
            let task_receiver = Arc::clone(&task_receiver);
            let shutdown_signal = Arc::clone(&shutdown_signal);
            
            let handle = thread::spawn(move || {
                println!("异步执行器工作线程 {} 启动", worker_id);
                
                loop {
                    if *shutdown_signal.lock().unwrap() {
                        break;
                    }
                    
                    let task = {
                        let receiver = task_receiver.lock().unwrap();
                        match receiver.recv_timeout(Duration::from_millis(100)) {
                            Ok(task) => task,
                            Err(_) => continue,
                        }
                    };
                    
                    // 执行任务
                    task();
                }
                
                println!("异步执行器工作线程 {} 停止", worker_id);
            });
            
            worker_handles.push(handle);
        }
        
        Self {
            worker_handles,
            task_sender,
            shutdown_signal,
        }
    }
    
    /// 提交异步任务
    pub fn submit<T, E, F>(&self, task: F) -> SimpleFuture<T, E>
    where
        T: Clone + Send + 'static,
        E: Clone + Send + 'static,
        F: FnOnce() -> Result<T, E> + Send + 'static,
    {
        let (future, promise) = SimpleFuture::new();
        
        let boxed_task = Box::new(move || {
            let result = task();
            match result {
                Ok(value) => promise.complete(value),
                Err(error) => promise.fail(error),
            }
        });
        
        if let Err(_) = self.task_sender.send(boxed_task) {
            // 如果发送失败，返回一个失败的Future
            // 这里我们不能假设E可以从AsyncError转换，所以直接panic或使用其他策略
            panic!("AsyncExecutor已关闭，无法提交任务");
        }
        
        future
    }
    
    /// 延迟执行任务
    pub fn delay<T, E, F>(&self, delay: Duration, task: F) -> SimpleFuture<T, E>
    where
        T: Clone + Send + 'static,
        E: Clone + Send + 'static,
        F: FnOnce() -> Result<T, E> + Send + 'static,
    {
        let (future, promise) = SimpleFuture::new();
        
        let boxed_task = Box::new(move || {
            thread::sleep(delay);
            let result = task();
            match result {
                Ok(value) => promise.complete(value),
                Err(error) => promise.fail(error),
            }
        });
        
        let _ = self.task_sender.send(boxed_task);
        future
    }
    
    /// 关闭执行器
    pub fn shutdown(self) {
        println!("开始关闭异步执行器...");
        
        *self.shutdown_signal.lock().unwrap() = true;
        
        for (i, handle) in self.worker_handles.into_iter().enumerate() {
            println!("等待工作线程 {} 完成...", i);
            let _ = handle.join();
        }
        
        println!("异步执行器已关闭");
    }
}

// =================
// 具体异步任务示例
// =================

/// 网络请求模拟
pub fn fetch_data(url: String, delay: Duration) -> SimpleFuture<String, AsyncError> {
    let (future, promise) = SimpleFuture::new();
    
    thread::spawn(move || {
        thread::sleep(delay);
        
        if url.contains("error") {
            promise.fail(AsyncError::ComputationFailed("网络错误".to_string()));
        } else {
            let response = format!("来自 {} 的响应数据", url);
            promise.complete(response);
        }
    });
    
    future
}

/// 数据库查询模拟
pub fn query_database(query: String, delay: Duration) -> SimpleFuture<Vec<String>, AsyncError> {
    let (future, promise) = SimpleFuture::new();
    
    thread::spawn(move || {
        thread::sleep(delay);
        
        if query.contains("invalid") {
            promise.fail(AsyncError::ComputationFailed("无效查询".to_string()));
        } else {
            let results = vec![
                format!("结果1 for {}", query),
                format!("结果2 for {}", query),
                format!("结果3 for {}", query),
            ];
            promise.complete(results);
        }
    });
    
    future
}

/// 计算密集型任务
pub fn compute_fibonacci(n: u32) -> SimpleFuture<u64, AsyncError> {
    let (future, promise) = SimpleFuture::new();
    
    thread::spawn(move || {
        if n > 50 {
            promise.fail(AsyncError::ComputationFailed("数值太大".to_string()));
            return;
        }
        
        let mut a = 0u64;
        let mut b = 1u64;
        
        for _ in 0..n {
            let temp = a + b;
            a = b;
            b = temp;
            
            // 模拟计算时间
            thread::sleep(Duration::from_millis(10));
        }
        
        promise.complete(a);
    });
    
    future
}

// =================
// 演示函数
// =================

/// Future-Promise模式演示
pub fn demo_future_promise() {
    println!("=== Future-Promise模式演示 ===\n");
    
    // 1. 基本Future使用
    println!("1. 基本Future使用:");
    {
        // 网络请求示例
        let future1 = fetch_data("https://api.example.com/users".to_string(), Duration::from_millis(100));
        let future2 = fetch_data("https://api.example.com/posts".to_string(), Duration::from_millis(150));
        let future3 = fetch_data("https://error.example.com/data".to_string(), Duration::from_millis(80));
        
        println!("开始异步网络请求...");
        
        // 等待结果
        match future1.wait_timeout(Duration::from_millis(200)) {
            Ok(data) => println!("请求1成功: {}", data),
            Err(e) => println!("请求1失败: {:?}", e),
        }
        
        match future2.wait_timeout(Duration::from_millis(300)) {
            Ok(data) => println!("请求2成功: {}", data),
            Err(e) => println!("请求2失败: {:?}", e),
        }
        
        match future3.wait_timeout(Duration::from_millis(200)) {
            Ok(data) => println!("请求3成功: {}", data),
            Err(e) => println!("请求3失败: {:?}", e),
        }
    }
    
    println!("\n{}\n", "=".repeat(50));
    
    // 2. Future组合操作
    println!("2. Future组合操作:");
    {
        // 创建多个计算任务
        let fib_futures = vec![
            compute_fibonacci(10),
            compute_fibonacci(15),
            compute_fibonacci(20),
            compute_fibonacci(25),
        ];
        
        println!("开始并行计算斐波那契数列...");
        
        // 等待所有任务完成
        let all_results = join_all(fib_futures);
        
        match all_results.wait_timeout(Duration::from_secs(5)) {
            Ok(results) => {
                println!("所有计算完成:");
                for (i, result) in results.iter().enumerate() {
                    println!("  斐波那契({}) = {}", (i + 1) * 5 + 5, result);
                }
            }
            Err(e) => println!("计算失败: {:?}", e),
        }
    }
    
    println!("\n{}\n", "=".repeat(50));
    
    // 3. Future链式操作
    println!("3. Future链式操作:");
    {
        let chained_future = fetch_data("https://api.example.com/user/123".to_string(), Duration::from_millis(50))
            .map(|response| {
                println!("处理响应: {}", response);
                response.len()
            })
            .and_then(|length| {
                println!("响应长度: {}", length);
                compute_fibonacci(length as u32 % 20 + 10)
            });
        
        match chained_future.wait_timeout(Duration::from_secs(3)) {
            Ok(result) => println!("链式操作结果: {}", result),
            Err(e) => println!("链式操作失败: {:?}", e),
        }
    }
    
    println!("\n{}\n", "=".repeat(50));
    
    // 4. 异步执行器演示
    println!("4. 异步执行器演示:");
    {
        let executor = AsyncExecutor::new(4);
        
        // 提交多个异步任务
        let mut task_futures = Vec::new();
        
        for i in 1..=8 {
            let future = executor.submit(move || -> Result<String, AsyncError> {
                thread::sleep(Duration::from_millis(50 + i * 10));
                Ok(format!("任务{}完成", i))
            });
            task_futures.push(future);
        }
        
        // 提交延迟任务
        let delayed_future = executor.delay(Duration::from_millis(200), || -> Result<String, AsyncError> {
            Ok("延迟任务完成".to_string())
        });
        task_futures.push(delayed_future);
        
        // 等待所有任务
        for (i, future) in task_futures.into_iter().enumerate() {
            match future.wait_timeout(Duration::from_secs(2)) {
                Ok(result) => println!("任务{}: {}", i + 1, result),
                Err(e) => println!("任务{}失败: {:?}", i + 1, e),
            }
        }
        
        executor.shutdown();
    }
    
    println!("\n{}\n", "=".repeat(50));
    
    // 5. 竞速操作演示
    println!("5. 竞速操作演示:");
    {
        // 创建多个数据源
        let sources = vec![
            fetch_data("https://fast-server.com/data".to_string(), Duration::from_millis(80)),
            fetch_data("https://medium-server.com/data".to_string(), Duration::from_millis(120)),
            fetch_data("https://slow-server.com/data".to_string(), Duration::from_millis(200)),
        ];
        
        println!("开始竞速获取数据...");
        
        // 等待最快的响应
        let fastest = select_first(sources);
        
        match fastest.wait_timeout(Duration::from_millis(300)) {
            Ok(data) => println!("最快响应: {}", data),
            Err(e) => println!("竞速失败: {:?}", e),
        }
    }
    
    println!("\n【Future-Promise模式特点】");
    println!("✓ 异步计算 - 不阻塞当前线程等待结果");
    println!("✓ 组合性 - 可以组合多个Future操作");
    println!("✓ 错误处理 - 统一的错误传播机制");
    println!("✓ 回调链 - 支持链式操作和转换");
    println!("✓ 并发控制 - 可以并行执行多个异步操作");
    println!("✓ 响应式 - 基于事件和回调的编程模型");
} 