/*
 * 文件路径: /d%3A/workspace/RustLearn/RustDesignPattern/src/FunctionalProgrammingPattern/functor_pattern.rs
 * 
 * 函子模式 (Functor Pattern)
 * 
 * 函子是函数式编程中的一个基本概念，它是一个可以被映射的数据结构。
 * 函子提供了map操作，允许我们将函数应用到包装在容器中的值，而不需要
 * 手动处理容器的结构。这种抽象使得我们可以统一处理各种容器类型。
 * 
 * 主要特点：
 * 1. 结构保持 - 映射操作保持容器的结构不变，只改变其中的值
 * 2. 组合性 - 函子映射可以链式组合：fmap f . fmap g = fmap (f . g)
 * 3. 身份性 - 映射身份函数等于身份操作：fmap id = id
 * 4. 抽象化 - 提供统一的映射接口，无需关心具体的容器实现
 * 5. 类型安全 - 在编译时保证映射操作的类型正确性
 * 
 * 函子定律：
 * 1. 身份律：fmap id = id
 * 2. 组合律：fmap (f . g) = fmap f . fmap g
 * 
 * 使用场景：
 * - 数据转换：对容器中的数据进行统一变换
 * - 错误处理：在不失败的情况下转换可能失败的计算
 * - 异步编程：转换Future或Promise中的值
 * - 集合操作：对集合中的每个元素应用相同的操作
 * 
 * 实现说明：
 * - Identity函子：最简单的函子，直接包装一个值
 * - Pair函子：包装两个值的函子，演示部分映射
 * - 提供链式映射的示例
 * - 遵循Rust的类型系统和所有权规则
 * 
 * 注意事项：
 * - 函子是单子的基础，理解函子有助于理解单子
 * - 在Rust中需要考虑所有权转移问题
 * - 函子操作应该是纯函数，不应有副作用
 */

/// Identity函子
#[derive(Debug, Clone, PartialEq)]
pub struct Identity<T>(pub T);

impl<T> Identity<T> {
    pub fn new(value: T) -> Self {
        Identity(value)
    }
    
    pub fn get(self) -> T {
        self.0
    }
    
    pub fn map<U, F>(self, f: F) -> Identity<U>
    where
        F: FnOnce(T) -> U,
    {
        Identity(f(self.0))
    }
}

/// Pair函子
#[derive(Debug, Clone, PartialEq)]
pub struct Pair<A, B>(pub A, pub B);

impl<A, B> Pair<A, B> {
    pub fn new(a: A, b: B) -> Self {
        Pair(a, b)
    }
    
    pub fn map_second<C, F>(self, f: F) -> Pair<A, C>
    where
        F: FnOnce(B) -> C,
    {
        Pair(self.0, f(self.1))
    }
}

/// 函子模式演示
pub fn demo_functor_pattern() {
    println!("=== 函子模式演示 ===");
    
    // Identity函子演示
    let id_value = Identity::new(42);
    println!("原始值: {:?}", id_value);
    
    let mapped = id_value.map(|x| x * 2);
    println!("映射后 (*2): {:?}", mapped);
    
    let chained = Identity::new(10)
        .map(|x| x + 5)
        .map(|x| format!("结果: {}", x));
    println!("链式映射: {:?}", chained);
    
    // Pair函子演示
    let pair = Pair::new("键".to_string(), 42);
    println!("原始Pair: {:?}", pair);
    
    let mapped_pair = pair.map_second(|x| x * 2);
    println!("映射second: {:?}", mapped_pair);
    
    println!("\n【函子模式特点】");
    println!("✓ 结构保持 - 映射操作保持容器的结构不变");
    println!("✓ 组合性 - 函子映射可以链式组合");
    println!("✓ 抽象化 - 提供统一的映射接口");
} 