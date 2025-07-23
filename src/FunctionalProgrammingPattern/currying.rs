/*
 * 文件路径: /d%3A/workspace/RustLearn/RustDesignPattern/src/FunctionalProgrammingPattern/currying.rs
 * 
 * 柯里化模式 (Currying Pattern)
 * 
 * 柯里化是一种函数式编程技术，将接受多个参数的函数转换为一系列接受单个参数的函数。
 * 这种模式允许部分应用函数参数，创建特化的函数版本，提高代码的可重用性和灵活性。
 * 
 * 主要特点：
 * 1. 部分应用 - 可以固定函数的部分参数，创建新的特化函数
 * 2. 函数工厂 - 通过预设参数创建配置化的函数
 * 3. 组合性 - 柯里化的函数可以轻松组合和链式调用
 * 4. 延迟执行 - 参数可以分批提供，直到所有参数都提供才执行
 * 5. 类型安全 - 在编译时保证参数类型的正确性
 * 
 * 使用场景：
 * - 配置驱动的编程：创建带有预设配置的函数
 * - 事件处理：创建特定事件类型的处理器
 * - 数据验证：创建特定规则的验证器
 * - 函数式编程：支持高阶函数和函数组合
 * 
 * 实现说明：
 * - 使用Rust的闭包和trait约束确保类型安全
 * - 通过Box<dyn Fn>实现动态函数返回
 * - 使用Copy约束解决Rust的所有权问题
 * - 支持多种数据类型的柯里化操作
 * 
 * 注意事项：
 * - 柯里化可能导致性能开销，因为涉及闭包和装箱
 * - 在Rust中需要注意所有权和生命周期问题
 * - 对于简单操作，直接使用普通函数可能更高效
 */

/// 基础柯里化 - 将二元函数转换为柯里化形式
pub fn curry2<A, B, C>(f: impl Fn(A, B) -> C + Copy + 'static) -> impl Fn(A) -> Box<dyn Fn(B) -> C>
where
    A: Copy + 'static,
    B: 'static,
    C: 'static,
{
    move |a| {
        Box::new(move |b| f(a, b))
    }
}

/// 数学运算柯里化
pub struct MathCurry;

impl MathCurry {
    /// 柯里化加法
    pub fn add() -> impl Fn(i32) -> Box<dyn Fn(i32) -> i32> {
        |a| Box::new(move |b| a + b)
    }
    
    /// 柯里化乘法
    pub fn multiply() -> impl Fn(i32) -> Box<dyn Fn(i32) -> i32> {
        |a| Box::new(move |b| a * b)
    }
}

/// 字符串处理柯里化
pub struct StringCurry;

impl StringCurry {
    /// 柯里化字符串连接
    pub fn concat() -> impl Fn(String) -> Box<dyn Fn(String) -> String> {
        |a| Box::new(move |b| format!("{}{}", a, b))
    }
}

/// 比较操作柯里化
pub struct Comparison;

impl Comparison {
    pub fn greater_than<T>() -> impl Fn(T) -> Box<dyn Fn(T) -> bool>
    where
        T: PartialOrd + 'static,
    {
        |threshold| Box::new(move |value| value > threshold)
    }
}

/// 柯里化模式演示
pub fn demo_currying() {
    println!("=== 柯里化模式演示 ===");
    
    // 基础柯里化
    let add = |a: i32, b: i32| a + b;
    let curried_add = curry2(add);
    
    let add_5 = curried_add(5);
    println!("5 + 3 = {}", add_5(3));
    
    // 数学运算柯里化
    let add_func = MathCurry::add();
    let add_10 = add_func(10);
    println!("10 + 15 = {}", add_10(15));
    
    let multiply_func = MathCurry::multiply();
    let double = multiply_func(2);
    println!("双倍 8 = {}", double(8));
    
    // 字符串处理柯里化
    let concat_func = StringCurry::concat();
    let prefix_hello = concat_func("Hello, ".to_string());
    println!("问候: {}", prefix_hello("World!".to_string()));
    
    // 比较操作柯里化
    let gt_5 = Comparison::greater_than()(5);
    let numbers = vec![1, 5, 8, 3, 9, 2];
    let greater_than_5: Vec<i32> = numbers.into_iter().filter(|&x| gt_5(x)).collect();
    println!("大于5的数: {:?}", greater_than_5);
    
    println!("\n【柯里化模式特点】");
    println!("✓ 部分应用 - 可以固定部分参数创建特化函数");
    println!("✓ 函数重用 - 通过部分应用创建可重用的函数");
    println!("✓ 配置驱动 - 通过预设参数创建配置化的函数");
} 