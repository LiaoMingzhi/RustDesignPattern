/*
 * 文件路径: /d%3A/workspace/RustLearn/RustDesignPattern/src/FunctionalProgrammingPattern/mod.rs
 * 
 * 函数式编程模式 (Functional Programming Patterns)
 * 
 * 这个模块包含了函数式编程相关的设计模式，展示如何在Rust中
 * 运用函数式编程的思想和技巧。
 */

// 导入所有函数式编程模式模块
pub mod higher_order_functions;
pub mod closures;
pub mod function_composition;
pub mod currying;
pub mod monad_pattern;
pub mod functor_pattern;
pub mod lazy_evaluation;
pub mod immutability_pattern;

// 重新导出演示函数
pub use higher_order_functions::demo_higher_order_functions;
pub use closures::demo_closures;
pub use function_composition::demo_function_composition;
pub use currying::demo_currying;
pub use monad_pattern::demo_monad_pattern;
pub use functor_pattern::demo_functor_pattern;
pub use lazy_evaluation::demo_lazy_evaluation;
pub use immutability_pattern::demo_immutability_pattern;

/// 演示所有函数式编程模式
pub fn demo_all_functional_patterns() {
    println!("=== 函数式编程模式演示合集 ===\n");
    
    // 1. 高阶函数模式
    demo_higher_order_functions();
    println!();
    
    // 2. 闭包模式
    demo_closures();
    println!();
    
    // 3. 函数组合模式
    demo_function_composition();
    println!();
    
    // 4. 柯里化模式
    demo_currying();
    println!();
    
    // 5. 单子模式
    demo_monad_pattern();
    println!();
    
    // 6. 函子模式
    demo_functor_pattern();
    println!();
    
    // 7. 惰性求值模式
    demo_lazy_evaluation();
    println!();
    
    // 8. 不变性模式
    demo_immutability_pattern();
    
    println!("\n=== 函数式编程模式演示完成 ===");
    println!("\n【函数式编程模式总结】");
    println!("✓ 高阶函数 - 函数作为一等公民，支持函数参数和返回值");
    println!("✓ 闭包 - 捕获环境变量的匿名函数");
    println!("✓ 函数组合 - 将简单函数组合成复杂操作");
    println!("✓ 柯里化 - 将多参数函数转换为单参数函数链");
    println!("✓ 单子 - 处理包装值的抽象模式");
    println!("✓ 函子 - 可映射的容器抽象");
    println!("✓ 惰性求值 - 按需计算，提高性能");
    println!("✓ 不变性 - 数据不可变，保证线程安全和可预测性");
} 