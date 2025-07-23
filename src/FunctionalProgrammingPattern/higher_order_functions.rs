/*
 * 文件路径: /d%3A/workspace/RustLearn/RustDesignPattern/src/FunctionalProgrammingPattern/higher_order_functions.rs
 * 
 * 高阶函数模式 (Higher-Order Functions Pattern)
 * 
 * 高阶函数是接受函数作为参数或返回函数的函数。这种模式允许我们创建更加灵活和可重用的代码，
 * 通过将行为作为参数传递，我们可以将算法的框架与具体的实现分离。
 * 
 * 主要特点：
 * 1. 函数作为一等公民 - 函数可以像其他值一样传递和返回
 * 2. 行为参数化 - 将不同的行为作为参数传递给函数
 * 3. 代码复用 - 通过抽象通用模式来减少重复代码
 * 4. 函数组合 - 将简单函数组合成复杂的操作
 * 5. 延迟执行 - 可以延迟到需要时才执行特定操作
 */

use std::collections::HashMap;

// =================
// 基础高阶函数
// =================

/// 对集合中的每个元素应用函数
pub fn map<T, U, F>(collection: Vec<T>, function: F) -> Vec<U>
where
    F: Fn(T) -> U,
{
    collection.into_iter().map(function).collect()
}

/// 过滤集合中满足条件的元素
pub fn filter<T, F>(collection: Vec<T>, predicate: F) -> Vec<T>
where
    F: Fn(&T) -> bool,
{
    collection.into_iter().filter(predicate).collect()
}

/// 将集合减少为单个值
pub fn reduce<T, F>(collection: Vec<T>, initial: T, function: F) -> T
where
    F: Fn(T, T) -> T,
{
    collection.into_iter().fold(initial, function)
}

/// 查找集合中第一个满足条件的元素
pub fn find<T, F>(collection: Vec<T>, predicate: F) -> Option<T>
where
    F: Fn(&T) -> bool,
{
    collection.into_iter().find(predicate)
}

// =================
// 函数工厂模式
// =================

/// 创建一个乘法函数的工厂
pub fn create_multiplier(factor: i32) -> impl Fn(i32) -> i32 {
    move |x| x * factor
}

/// 创建一个范围检查器的工厂
pub fn create_range_checker(min: i32, max: i32) -> impl Fn(i32) -> bool {
    move |x| x >= min && x <= max
}

/// 创建一个格式化器的工厂
pub fn create_formatter(prefix: String, suffix: String) -> impl Fn(&str) -> String {
    move |text| format!("{}{}{}", prefix, text, suffix)
}

// =================
// 函数装饰器模式
// =================

/// 缓存装饰器 - 为函数添加缓存功能
pub struct MemoizedFunction<F, A, R>
where
    F: Fn(A) -> R,
    A: Clone + std::hash::Hash + Eq,
    R: Clone,
{
    function: F,
    cache: std::sync::RwLock<HashMap<A, R>>,
}

impl<F, A, R> MemoizedFunction<F, A, R>
where
    F: Fn(A) -> R,
    A: Clone + std::hash::Hash + Eq,
    R: Clone,
{
    pub fn new(function: F) -> Self {
        Self {
            function,
            cache: std::sync::RwLock::new(HashMap::new()),
        }
    }
    
    pub fn call(&self, arg: A) -> R {
        // 首先尝试从缓存中读取
        {
            let cache = self.cache.read().unwrap();
            if let Some(result) = cache.get(&arg) {
                return result.clone();
            }
        }
        
        // 如果缓存中没有，计算结果并缓存
        let result = (self.function)(arg.clone());
        {
            let mut cache = self.cache.write().unwrap();
            cache.insert(arg, result.clone());
        }
        result
    }
}

/// 计时装饰器 - 为函数添加执行时间测量
pub fn with_timing<F, R>(function: F) -> impl Fn() -> (R, std::time::Duration)
where
    F: Fn() -> R,
{
    move || {
        let start = std::time::Instant::now();
        let result = function();
        let duration = start.elapsed();
        (result, duration)
    }
}

/// 重试装饰器 - 为函数添加重试机制
pub fn with_retry<F, R, E>(function: F, max_attempts: u32) -> impl Fn() -> Result<R, E>
where
    F: Fn() -> Result<R, E>,
{
    move || {
        let mut attempts = 0;
        loop {
            attempts += 1;
            match function() {
                Ok(result) => return Ok(result),
                Err(error) => {
                    if attempts >= max_attempts {
                        return Err(error);
                    }
                    // 在实际应用中，这里可能会有延迟
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
            }
        }
    }
}

// =================
// 策略模式与高阶函数
// =================

/// 排序策略枚举
pub enum SortStrategy {
    Ascending,
    Descending,
    ByLength,
    Custom(Box<dyn Fn(&str, &str) -> std::cmp::Ordering>),
}

/// 使用策略模式进行排序
pub fn sort_strings(mut strings: Vec<String>, strategy: SortStrategy) -> Vec<String> {
    match strategy {
        SortStrategy::Ascending => {
            strings.sort();
        }
        SortStrategy::Descending => {
            strings.sort_by(|a, b| b.cmp(a));
        }
        SortStrategy::ByLength => {
            strings.sort_by(|a, b| a.len().cmp(&b.len()));
        }
        SortStrategy::Custom(comparator) => {
            strings.sort_by(|a, b| comparator(a, b));
        }
    }
    strings
}

// =================
// 数据处理管道
// =================

/// 数据处理管道
pub struct DataPipeline<T> {
    data: Vec<T>,
}

impl<T> DataPipeline<T> {
    pub fn new(data: Vec<T>) -> Self {
        Self { data }
    }
    
    /// 映射操作
    pub fn map<U, F>(self, function: F) -> DataPipeline<U>
    where
        F: Fn(T) -> U,
    {
        DataPipeline {
            data: self.data.into_iter().map(function).collect(),
        }
    }
    
    /// 过滤操作
    pub fn filter<F>(self, predicate: F) -> DataPipeline<T>
    where
        F: Fn(&T) -> bool,
    {
        DataPipeline {
            data: self.data.into_iter().filter(predicate).collect(),
        }
    }
    
    /// 获取结果
    pub fn collect(self) -> Vec<T> {
        self.data
    }
    
    /// 聚合操作
    pub fn reduce<F>(self, initial: T, function: F) -> T
    where
        F: Fn(T, T) -> T,
    {
        self.data.into_iter().fold(initial, function)
    }
}

// =================
// 条件执行高阶函数
// =================

/// 条件执行函数
pub fn conditional_execute<T, F1, F2>(
    condition: bool,
    if_true: F1,
    if_false: F2,
) -> impl Fn() -> T
where
    F1: Fn() -> T,
    F2: Fn() -> T,
{
    move || {
        if condition {
            if_true()
        } else {
            if_false()
        }
    }
}

/// 惰性条件执行
pub fn lazy_conditional<T, F1, F2, P>(
    predicate: P,
    if_true: F1,
    if_false: F2,
) -> impl Fn() -> T
where
    P: Fn() -> bool,
    F1: Fn() -> T,
    F2: Fn() -> T,
{
    move || {
        if predicate() {
            if_true()
        } else {
            if_false()
        }
    }
}

// =================
// 事件处理系统
// =================

/// 事件类型
#[derive(Debug, Clone)]
pub enum Event {
    Click { x: i32, y: i32 },
    KeyPress { key: char },
    Timer { elapsed: std::time::Duration },
}

/// 事件处理器
pub type EventHandler = Box<dyn Fn(Event) + Send + Sync>;

/// 事件分发器
pub struct EventDispatcher {
    handlers: Vec<EventHandler>,
}

impl EventDispatcher {
    pub fn new() -> Self {
        Self {
            handlers: Vec::new(),
        }
    }
    
    /// 添加事件处理器
    pub fn add_handler<F>(&mut self, handler: F)
    where
        F: Fn(Event) + Send + Sync + 'static,
    {
        self.handlers.push(Box::new(handler));
    }
    
    /// 分发事件
    pub fn dispatch(&self, event: Event) {
        for handler in &self.handlers {
            handler(event.clone());
        }
    }
}

// =================
// 演示函数
// =================

/// 高阶函数模式演示
pub fn demo_higher_order_functions() {
    println!("=== 高阶函数模式演示 ===");
    
    // 基础高阶函数演示
    let numbers = vec![1, 2, 3, 4, 5];
    let doubled = map(numbers.clone(), |x| x * 2);
    println!("原数组: {:?}", numbers);
    println!("翻倍后: {:?}", doubled);
    
    let evens = filter(numbers.clone(), |&x| x % 2 == 0);
    println!("偶数: {:?}", evens);
    
    // 函数工厂演示
    let multiply_by_3 = create_multiplier(3);
    println!("5 * 3 = {}", multiply_by_3(5));
    
    // 缓存装饰器演示
    let expensive_function = |x: i32| {
        println!("执行昂贵计算: {}", x);
        x * x
    };
    
    let memoized = MemoizedFunction::new(expensive_function);
    println!("第一次调用: {}", memoized.call(5));
    println!("第二次调用: {}", memoized.call(5)); // 从缓存获取
} 