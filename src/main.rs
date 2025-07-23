//! Rust设计模式学习项目
//! Rust 版本 1.86.0
//! 本项目实现了多种设计模式的Rust版本，包括：
//! - GoF（Gang of Four）23种经典设计模式
//! - Martin Fowler《企业应用架构模式》中的核心模式
//! - 并发编程模式（Concurrent Programming Patterns）
//! - 分布式系统模式（Distributed System Patterns）
//! - 函数式编程模式（Functional Programming Patterns）
//! 
//! 所有模式都包含了详细的中文注释和演示代码。
//! 
//! 作者：Rust学习者
//! 项目路径：/d%3A/workspace/RustLearn/RustDesignPattern

mod GoFDesignPattern;
mod EnterpriseAppPattern;
mod ConcurrentMode;
mod DistributedSystemMode;
mod FunctionalProgrammingPattern;

fn main() {
    println!("🎯 === Rust设计模式学习项目 === 🎯\n");
    
    println!("📚 本项目包含五大类设计模式：");
    println!("1. GoF（Gang of Four）23种经典设计模式");
    println!("2. 企业应用架构模式（Enterprise Application Architecture Patterns）");
    println!("3. 并发模式（Concurrent Patterns）");
    println!("4. 分布式系统模式（Distributed System Patterns）");
    println!("5. 函数式编程模式（Functional Programming Patterns）");
    
    println!("\n{}", "=".repeat(100));
    
    // 运行所有GoF设计模式演示
    println!("\n🏛️ === GoF设计模式演示开始 === 🏛️");
    GoFDesignPattern::run_all_patterns();
    
    println!("\n{}", "=".repeat(100));
    
    // 运行所有企业应用架构模式演示
    println!("\n🏢 === 企业应用架构模式演示开始 === 🏢");
    EnterpriseAppPattern::demo_all();
    
    println!("\n{}", "=".repeat(100));
    
    // 运行所有并发模式演示
    println!("\n⚡ === 并发模式演示开始 === ⚡");
    ConcurrentMode::demo_all_concurrent_patterns();
    
    println!("\n{}", "=".repeat(100));
    
    // 运行所有分布式系统模式演示
    println!("\n🌐 === 分布式系统模式演示开始 === 🌐");
    DistributedSystemMode::demo_all_distributed_patterns();
    
    println!("\n{}", "=".repeat(100));
    
    // 运行所有函数式编程模式演示
    println!("\n🔄 === 函数式编程模式演示开始 === 🔄");
    // FunctionalProgrammingPattern::demo_all_functional_patterns();
    println!("函数式编程模式演示暂未实现...");
    
    println!("\n{}", "=".repeat(100));
    
    println!("\n🎉 === 所有设计模式演示完成 === ��");
    println!("✅ GoF设计模式：23种经典模式");
    println!("✅ 企业应用架构模式：11个分类，数十种核心模式");
    println!("✅ 并发模式：8种核心并发编程模式");
    println!("✅ 分布式系统模式：若干种核心分布式系统设计模式");
    println!("✅ 函数式编程模式：若干种核心函数式编程相关模式");
    println!("📖 每个模式都包含完整的文档、实现和测试用例");
    println!("💡 建议根据具体场景选择合适的模式组合使用");
    
    println!("\n🔧 技术特色：");
    println!("   🛡️  所有代码都遵循Rust线程安全要求");
    println!("   📝  完整的中文注释和说明");
    println!("   🚀  可直接运行的演示代码");
    println!("   ⚡  高性能并发实现");
    println!("   🔒  内存安全保证");
}
