/*
 * 文件路径: /d%3A/workspace/RustLearn/RustDesignPattern/src/EnterpriseAppPattern/ObjectRelationalBehavioralPatterns/mod.rs
 * 
 * Object-Relational Behavioral Patterns（对象-关系行为模式）模块
 * 
 * 该模块包含Martin Fowler《企业应用架构模式》中的对象-关系行为模式：
 * 
 * 1. Unit of Work（工作单元）
 * 2. Identity Map（身份映射）
 * 3. Lazy Load（延迟加载）
 * 
 * 这些模式主要解决对象与关系数据库交互中的行为问题。
 */

pub mod unit_of_work;
pub mod identity_map;
pub mod lazy_load;

/// 演示所有对象-关系行为模式
pub fn demo_all() {
    println!("🗄️  === 企业应用架构模式 - 对象-关系行为模式演示 ===\n");
    
    println!("📋 对象-关系行为模式包括以下模式:");
    println!("1. Unit of Work（工作单元）- 维护事务中的对象变更");
    println!("2. Identity Map（身份映射）- 确保对象唯一性");
    println!("3. Lazy Load（延迟加载）- 按需加载数据");
    
    println!("\n{}", "=".repeat(80));
    
    // 1. Unit of Work 演示
    println!("\n🚀 1. Unit of Work（工作单元）模式演示");
    println!("适合：复杂事务管理，批量数据库操作");
    unit_of_work::demo();
    
    println!("\n{}", "=".repeat(80));
    
    // 2. Identity Map 演示
    println!("\n🚀 2. Identity Map（身份映射）模式演示");
    println!("适合：确保对象唯一性，避免重复加载");
    identity_map::demo();
    
    println!("\n{}", "=".repeat(80));
    
    // 3. Lazy Load 演示
    println!("\n🚀 3. Lazy Load（延迟加载）模式演示");
    println!("适合：大对象加载，提高性能");
    lazy_load::demo();
    
    println!("\n{}", "=".repeat(80));
    
    // 模式总结
    println!("\n📊 【对象-关系行为模式总结】");
    println!("\n核心目标：优化对象与数据库的交互行为");
    
    println!("\n各模式特点：");
    println!("┌─────────────────┬──────────────┬──────────────────┬──────────────────┐");
    println!("│     模式        │   主要作用   │     解决问题     │     关键技术     │");
    println!("├─────────────────┼──────────────┼──────────────────┼──────────────────┤");
    println!("│ Unit of Work    │   事务管理   │   批量操作优化   │   变更追踪       │");
    println!("│                 │   状态追踪   │   数据一致性     │   依赖排序       │");
    println!("├─────────────────┼──────────────┼──────────────────┼──────────────────┤");
    println!("│ Identity Map    │   对象缓存   │   重复加载       │   对象标识       │");
    println!("│                 │   唯一性     │   内存一致性     │   引用管理       │");
    println!("├─────────────────┼──────────────┼──────────────────┼──────────────────┤");
    println!("│ Lazy Load       │   延迟加载   │   性能优化       │   代理对象       │");
    println!("│                 │   按需获取   │   内存使用       │   虚拟代理       │");
    println!("└─────────────────┴──────────────┴──────────────────┴──────────────────┘");
    
    println!("\n🎯 设计目标：");
    println!("1. 性能优化：减少数据库访问次数");
    println!("2. 内存管理：控制对象生命周期");
    println!("3. 一致性保证：维护数据完整性");
    println!("4. 事务控制：管理复杂的业务操作");
    println!("5. 并发安全：处理多用户访问");
    
    println!("\n✨ 组合使用：");
    println!("• Unit of Work + Identity Map：完整的会话管理");
    println!("• Identity Map + Lazy Load：智能对象缓存");
    println!("• 三者结合：高性能ORM实现");
    
    println!("\n🏁 === 对象-关系行为模式演示完成 ===");
} 