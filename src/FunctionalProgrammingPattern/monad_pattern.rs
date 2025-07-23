/*
 * 文件路径: /d%3A/workspace/RustLearn/RustDesignPattern/src/FunctionalProgrammingPattern/monad_pattern.rs
 * 
 * 单子模式 (Monad Pattern)
 * 
 * 单子是函数式编程中的一个重要概念，它提供了一种结构化的方式来处理计算上下文
 * （如可能失败的计算、异步计算、状态计算等）。单子封装了值和相关的计算上下文，
 * 并提供了组合这些计算的统一接口。
 * 
 * 主要特点：
 * 1. 封装上下文 - 将值和计算上下文（如错误、状态）封装在一起
 * 2. 链式操作 - 通过bind/flat_map实现计算的链式组合
 * 3. 错误处理 - 优雅地处理可能失败的操作链
 * 4. 组合性 - 单子可以轻松组合和嵌套
 * 5. 纯函数式 - 避免副作用，保持计算的纯净性
 * 
 * 单子定律：
 * 1. 左单位元：return a >>= f ≡ f a
 * 2. 右单位元：m >>= return ≡ m  
 * 3. 结合律：(m >>= f) >>= g ≡ m >>= (\x -> f x >>= g)
 * 
 * 使用场景：
 * - 错误处理：避免嵌套的if-else或try-catch
 * - 异步编程：处理Future和Promise类型的操作
 * - 状态管理：管理有状态的计算
 * - 可选值处理：处理可能不存在的值
 * 
 * 实现说明：
 * - 实现Maybe单子用于处理可能不存在的值
 * - 提供map和flat_map操作支持函子和单子接口
 * - 包含实际的计算器示例展示链式操作
 * - 遵循Rust的类型系统和所有权规则
 * 
 * 注意事项：
 * - 单子的概念相对抽象，需要时间理解
 * - 过度使用可能导致代码难以理解
 * - 在Rust中需要注意生命周期和所有权问题
 */

/// Maybe单子 - 处理可能不存在的值
#[derive(Debug, Clone, PartialEq)]
pub enum Maybe<T> {
    Some(T),
    None,
}

impl<T> Maybe<T> {
    pub fn new(value: T) -> Self {
        Maybe::Some(value)
    }
    
    pub fn none() -> Self {
        Maybe::None
    }
    
    /// map操作 - 函子功能
    pub fn map<U, F>(self, f: F) -> Maybe<U>
    where
        F: FnOnce(T) -> U,
    {
        match self {
            Maybe::Some(value) => Maybe::Some(f(value)),
            Maybe::None => Maybe::None,
        }
    }
    
    /// flat_map操作 - 单子bind
    pub fn flat_map<U, F>(self, f: F) -> Maybe<U>
    where
        F: FnOnce(T) -> Maybe<U>,
    {
        match self {
            Maybe::Some(value) => f(value),
            Maybe::None => Maybe::None,
        }
    }
    
    pub fn unwrap_or(self, default: T) -> T {
        match self {
            Maybe::Some(value) => value,
            Maybe::None => default,
        }
    }
}

/// 计算器示例
pub struct Calculator;

impl Calculator {
    pub fn safe_divide(a: f64, b: f64) -> Maybe<f64> {
        if b != 0.0 {
            Maybe::Some(a / b)
        } else {
            Maybe::None
        }
    }
    
    pub fn safe_sqrt(x: f64) -> Maybe<f64> {
        if x >= 0.0 {
            Maybe::Some(x.sqrt())  
        } else {
            Maybe::None
        }
    }
    
    /// 链式计算示例
    pub fn divide_and_sqrt(a: f64, b: f64) -> Maybe<f64> {
        Self::safe_divide(a, b)
            .flat_map(|result| Self::safe_sqrt(result))
    }
}

/// 单子模式演示
pub fn demo_monad_pattern() {
    println!("=== 单子模式演示 ===");
    
    // Maybe单子演示
    let value1 = Maybe::new(42);
    let value2: Maybe<i32> = Maybe::none();
    
    println!("value1: {:?}", value1);
    println!("value2: {:?}", value2);
    
    // map操作
    let mapped1 = value1.clone().map(|x: i32| x * 2);
    let mapped2 = value2.clone().map(|x: i32| x * 2);
    
    println!("mapped1 (*2): {:?}", mapped1);
    println!("mapped2 (*2): {:?}", mapped2);
    
    // 计算器示例
    let calc1 = Calculator::divide_and_sqrt(16.0, 4.0);
    let calc2 = Calculator::divide_and_sqrt(16.0, 0.0);
    
    println!("sqrt(16/4) = {:?}", calc1);
    println!("sqrt(16/0) = {:?}", calc2);
    
    println!("\n【单子模式特点】");
    println!("✓ 链式操作 - 通过bind/flat_map实现操作链");
    println!("✓ 错误处理 - 优雅地处理可能失败的操作");
    println!("✓ 组合性 - 单子可以轻松组合和嵌套");
} 