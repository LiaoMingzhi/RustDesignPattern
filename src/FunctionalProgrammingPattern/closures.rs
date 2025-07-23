// 文件路径: /d%3A/workspace/RustLearn/RustDesignPattern/src/FunctionalProgrammingPattern/closures.rs

/*
 * 闭包模式 (Closures Pattern)
 * 
 * 闭包是可以捕获其定义环境中变量的匿名函数。在Rust中，闭包提供了一种优雅的方式
 * 来处理需要保持状态或访问外部变量的函数式编程场景。
 */

use std::rc::Rc;
use std::cell::RefCell;
use std::sync::{Arc, Mutex};

/// 基础闭包示例 - 捕获外部变量
pub fn create_counter() -> impl FnMut() -> i32 {
    let mut count = 0;
    move || {
        count += 1;
        count
    }
}

/// 配置闭包 - 捕获配置参数
pub fn create_validator(min_length: usize, max_length: usize) -> impl Fn(&str) -> bool {
    move |input| input.len() >= min_length && input.len() <= max_length
}

/// 状态机闭包
pub struct StateMachine {
    state: Arc<Mutex<String>>,
}

impl StateMachine {
    pub fn new(initial_state: &str) -> Self {
        Self {
            state: Arc::new(Mutex::new(initial_state.to_string())),
        }
    }
    
    pub fn create_transition(&self, from: &str, to: &str) -> impl Fn() -> bool {
        let state = Arc::clone(&self.state);
        let from = from.to_string();
        let to = to.to_string();
        
        move || {
            let mut current_state = state.lock().unwrap();
            if *current_state == from {
                *current_state = to.clone();
                true
            } else {
                false
            }
        }
    }
    
    pub fn get_state(&self) -> String {
        self.state.lock().unwrap().clone()
    }
}

/// 事件监听器工厂
pub fn create_event_listener<F>(callback: F) -> impl Fn(&str) 
where
    F: Fn(&str) + Clone + 'static,
{
    move |event| {
        callback(event);
    }
}

/// 累加器闭包
pub fn create_accumulator(initial: i32) -> impl FnMut(i32) -> i32 {
    let mut sum = initial;
    move |value| {
        sum += value;
        sum
    }
}

/// 记忆化斐波那契函数
pub fn create_fibonacci_memo() -> impl Fn(u32) -> u64 {
    let cache = Rc::new(RefCell::new(std::collections::HashMap::new()));
    
    move |n| {
        if let Some(&result) = cache.borrow().get(&n) {
            return result;
        }
        
        let result = match n {
            0 => 0,
            1 => 1,
            _ => {
                // 由于递归调用的复杂性，这里简化实现
                let mut a = 0u64;
                let mut b = 1u64;
                for _ in 2..=n {
                    let temp = a + b;
                    a = b;
                    b = temp;
                }
                b
            }
        };
        
        cache.borrow_mut().insert(n, result);
        result
    }
}

/// 函数式管道处理器
pub struct Pipeline<T> {
    value: T,
}

impl<T> Pipeline<T> {
    pub fn new(value: T) -> Self {
        Self { value }
    }
    
    pub fn then<U, F>(self, func: F) -> Pipeline<U>
    where
        F: FnOnce(T) -> U,
    {
        Pipeline::new(func(self.value))
    }
    
    pub fn apply<F>(self, func: F) -> Pipeline<T>
    where
        F: FnOnce(&T),
    {
        func(&self.value);
        self
    }
    
    pub fn unwrap(self) -> T {
        self.value
    }
}

/// 延迟执行闭包
pub struct LazyValue<T, F>
where
    F: FnOnce() -> T,
{
    func: Option<F>,
    value: Option<T>,
}

impl<T, F> LazyValue<T, F>
where
    F: FnOnce() -> T,
{
    pub fn new(func: F) -> Self {
        Self {
            func: Some(func),
            value: None,
        }
    }
    
    pub fn get(&mut self) -> &T {
        if self.value.is_none() {
            let func = self.func.take().unwrap();
            self.value = Some(func());
        }
        self.value.as_ref().unwrap()
    }
}

/// 条件执行闭包
pub fn create_conditional_executor<T>(
    condition: bool,
) -> impl Fn(Box<dyn Fn() -> T>, Box<dyn Fn() -> T>) -> T {
    move |if_true, if_false| {
        if condition {
            if_true()
        } else {
            if_false()
        }
    }
}

/// 批处理闭包
pub fn create_batch_processor<T>(
    batch_size: usize,
) -> impl FnMut(T, Box<dyn Fn(&[T])>) -> bool {
    let mut batch = Vec::new();
    
    move |item, processor| {
        batch.push(item);
        if batch.len() >= batch_size {
            processor(&batch);
            batch.clear();
            true  // 批次已处理
        } else {
            false // 批次未满
        }
    }
}

/// 闭包模式演示
pub fn demo_closures() {
    println!("=== 闭包模式演示 ===");
    
    // 1. 计数器闭包
    println!("1. 计数器闭包:");
    let mut counter = create_counter();
    println!("计数: {}", counter());
    println!("计数: {}", counter());
    println!("计数: {}", counter());
    
    // 2. 验证器闭包
    println!("\n2. 验证器闭包:");
    let validator = create_validator(3, 10);
    println!("'hello' 验证结果: {}", validator("hello"));
    println!("'hi' 验证结果: {}", validator("hi"));
    println!("'very long string' 验证结果: {}", validator("very long string"));
    
    // 3. 状态机闭包
    println!("\n3. 状态机闭包:");
    let state_machine = StateMachine::new("idle");
    println!("初始状态: {}", state_machine.get_state());
    
    let start_transition = state_machine.create_transition("idle", "running");
    let stop_transition = state_machine.create_transition("running", "stopped");
    
    println!("转换到running: {}", start_transition());
    println!("当前状态: {}", state_machine.get_state());
    
    println!("转换到stopped: {}", stop_transition());
    println!("当前状态: {}", state_machine.get_state());
    
    // 4. 累加器闭包
    println!("\n4. 累加器闭包:");
    let mut accumulator = create_accumulator(0);
    println!("累加5: {}", accumulator(5));
    println!("累加3: {}", accumulator(3));
    println!("累加-2: {}", accumulator(-2));
    
    // 5. 记忆化斐波那契
    println!("\n5. 记忆化斐波那契:");
    let fib = create_fibonacci_memo();
    println!("fib(10) = {}", fib(10));
    println!("fib(15) = {}", fib(15));
    println!("fib(20) = {}", fib(20));
    
    // 6. 管道处理
    println!("\n6. 管道处理:");
    let result = Pipeline::new(10)
        .then(|x| x * 2)
        .apply(|x| println!("中间值: {}", x))
        .then(|x| x + 5)
        .then(|x| format!("结果: {}", x))
        .unwrap();
    println!("{}", result);
    
    // 7. 延迟计算
    println!("\n7. 延迟计算:");
    let mut lazy_value = LazyValue::new(|| {
        println!("执行昂贵计算...");
        std::thread::sleep(std::time::Duration::from_millis(100));
        42
    });
    
    println!("延迟值尚未计算");
    println!("获取值: {}", lazy_value.get());
    println!("再次获取值: {}", lazy_value.get()); // 不会重新计算
    
    // 8. 事件监听器
    println!("\n8. 事件监听器:");
    let listener = create_event_listener(|event| {
        println!("收到事件: {}", event);
    });
    
    listener("用户登录");
    listener("文件上传");
    listener("数据更新");
} 