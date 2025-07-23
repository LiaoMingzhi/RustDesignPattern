/*
 * 文件路径: /d%3A/workspace/RustLearn/RustDesignPattern/src/FunctionalProgrammingPattern/function_composition.rs
 * 
 * 函数组合模式 (Function Composition Pattern)
 * 
 * 函数组合是将两个或多个函数组合成一个新函数的技术。在函数式编程中，这是一个核心概念，
 * 允许我们通过组合简单的函数来构建复杂的操作，遵循"组合优于继承"的原则。
 * 
 * 主要特点：
 * 1. 模块化 - 将复杂操作分解为简单函数的组合
 * 2. 可重用性 - 小函数可以在多个组合中复用
 * 3. 可读性 - 函数组合清晰表达了数据流和转换过程
 * 4. 数学性 - 遵循数学中的函数组合规则 (f ∘ g)(x) = f(g(x))
 * 5. 链式操作 - 支持流畅的链式调用接口
 * 
 * 使用场景：
 * - 数据处理管道：构建复杂的数据转换流程
 * - 业务逻辑组合：将简单的业务规则组合成复杂逻辑
 * - 函数式编程：实现高阶函数和函数式算法
 * - 装饰器模式：为函数添加横切关注点
 * 
 * 实现说明：
 * - 提供基础的compose函数用于函数组合
 * - 实现Pipe trait支持管道操作符
 * - 提供Combinator结构体支持链式组合
 * - 包含数学函数组合器的实用工具
 * 
 * 注意事项：
 * - 函数组合的顺序很重要，需要确保类型匹配
 * - 过度组合可能导致代码难以调试
 * - 在性能敏感的场景中要注意组合的开销
 */

/// 基础函数组合 - 组合两个函数
pub fn compose<A, B, C, F, G>(f: F, g: G) -> impl Fn(A) -> C
where
    F: Fn(A) -> B,
    G: Fn(B) -> C,
{
    move |x| g(f(x))
}

/// 管道操作符 - 从左到右的函数组合
pub trait Pipe<T> {
    fn pipe<U, F>(self, f: F) -> U
    where
        F: FnOnce(T) -> U;
}

impl<T> Pipe<T> for T {
    fn pipe<U, F>(self, f: F) -> U
    where
        F: FnOnce(T) -> U,
    {
        f(self)
    }
}

/// 组合器结构体 - 支持链式组合
pub struct Combinator<T> {
    value: T,
}

impl<T> Combinator<T> {
    pub fn new(value: T) -> Self {
        Self { value }
    }
    
    pub fn map<U, F>(self, f: F) -> Combinator<U>
    where
        F: FnOnce(T) -> U,
    {
        Combinator::new(f(self.value))
    }
    
    pub fn unwrap(self) -> T {
        self.value
    }
}

/// 数学函数组合器
pub struct MathComposer;

impl MathComposer {
    pub fn add(n: i32) -> impl Fn(i32) -> i32 {
        move |x| x + n
    }
    
    pub fn multiply(n: i32) -> impl Fn(i32) -> i32 {
        move |x| x * n
    }
    
    pub fn square() -> impl Fn(i32) -> i32 {
        |x| x * x
    }
}

/// 函数组合模式演示
pub fn demo_function_composition() {
    println!("=== 函数组合模式演示 ===");
    
    // 基础函数组合
    let add_one = |x: i32| x + 1;
    let square = |x: i32| x * x;
    
    let composed = compose(add_one, square);
    println!("(5 + 1)² = {}", composed(5));
    
    // 管道操作符
    let result = 10
        .pipe(|x| x + 5)
        .pipe(|x| x * 2)
        .pipe(|x| x - 3);
    println!("10 |> (+5) |> (*2) |> (-3) = {}", result);
    
    // 组合器链式操作
    let result = Combinator::new(42)
        .map(|x| x * 2)
        .map(|x| x + 10)
        .map(|x| format!("结果: {}", x))
        .unwrap();
    println!("{}", result);
    
    // 数学函数组合
    let add_10 = MathComposer::add(10);
    let multiply_2 = MathComposer::multiply(2);
    let square = MathComposer::square();
    
    let math_result = compose(compose(add_10, multiply_2), square)(5);
    println!("数学组合 (5): {}", math_result);
    
    println!("\n【函数组合模式特点】");
    println!("✓ 模块化 - 将复杂操作分解为简单函数的组合");
    println!("✓ 可重用性 - 小函数可以在多个组合中复用");
    println!("✓ 可读性 - 函数组合清晰表达了数据流");
} 