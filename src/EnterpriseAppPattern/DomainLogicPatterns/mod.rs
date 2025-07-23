/*
 * 文件路径: /d%3A/workspace/RustLearn/RustDesignPattern/src/EnterpriseAppPattern/DomainLogicPatterns/mod.rs
 * 
 * Domain Logic Patterns（领域逻辑模式）模块
 * 
 * 该模块包含Martin Fowler《企业应用架构模式》中的领域逻辑模式：
 * 
 * 1. Transaction Script（事务脚本）
 * 2. Domain Model（领域模型）
 * 3. Table Module（表模块）
 * 4. Service Layer（服务层）
 * 
 * 这些模式主要解决如何组织和处理业务逻辑的问题，
 * 从简单的过程式方法到复杂的对象模型。
 */

pub mod transaction_script;
pub mod domain_model;
pub mod table_module;
pub mod service_layer;

/// 演示所有领域逻辑模式
pub fn demo_all() {
    println!("🏗️  === 企业应用架构模式 - 领域逻辑模式演示 ===\n");
    
    println!("📋 领域逻辑模式包括以下4种模式:");
    println!("1. Transaction Script（事务脚本）- 简单的过程式业务逻辑");
    println!("2. Domain Model（领域模型）- 面向对象的复杂业务逻辑");
    println!("3. Table Module（表模块）- 针对数据库表的业务逻辑组织");
    println!("4. Service Layer（服务层）- 应用程序边界和事务控制");
    
    println!("\n{}", "=".repeat(80));
    
    // 1. Transaction Script 演示
    println!("\n🚀 1. Transaction Script（事务脚本）模式演示");
    println!("适合：简单的业务逻辑，每个事务一个脚本方法");
    transaction_script::demo();
    
    println!("\n{}", "=".repeat(80));
    
    // 2. Domain Model 演示
    println!("\n🚀 2. Domain Model（领域模型）模式演示");
    println!("适合：复杂的业务逻辑，丰富的对象模型");
    domain_model::demo();
    
    println!("\n{}", "=".repeat(80));
    
    // 3. Table Module 演示
    println!("\n🚀 3. Table Module（表模块）模式演示");
    println!("适合：中等复杂度的业务逻辑，表导向的数据处理");
    table_module::demo();
    
    println!("\n{}", "=".repeat(80));
    
    // 4. Service Layer 演示
    println!("\n🚀 4. Service Layer（服务层）模式演示");
    println!("适合：复杂应用，需要清晰的应用边界和事务控制");
    service_layer::demo();
    
    println!("\n{}", "=".repeat(80));
    
    // 模式对比总结
    println!("\n📊 【领域逻辑模式对比】");
    println!("\n复杂度递增：");
    println!("Transaction Script < Table Module < Domain Model + Service Layer");
    
    println!("\n各模式特点对比：");
    println!("┌────────────────┬──────────────┬──────────────┬──────────────┬──────────────┐");
    println!("│     模式       │   复杂度     │   组织方式   │   状态管理   │   适用场景   │");
    println!("├────────────────┼──────────────┼──────────────┼──────────────┼──────────────┤");
    println!("│ Transaction    │     低       │   过程式     │   无状态     │   简单业务   │");
    println!("│ Script         │              │   脚本方法   │              │              │");
    println!("├────────────────┼──────────────┼──────────────┼──────────────┼──────────────┤");
    println!("│ Table Module   │     中       │   表导向     │   无状态     │   中等复杂   │");
    println!("│                │              │   模块类     │              │   表处理     │");
    println!("├────────────────┼──────────────┼──────────────┼──────────────┼──────────────┤");
    println!("│ Domain Model   │     高       │   对象模型   │   有状态     │   复杂业务   │");
    println!("│                │              │   领域对象   │              │   面向对象   │");
    println!("├────────────────┼──────────────┼──────────────┼──────────────┼──────────────┤");
    println!("│ Service Layer  │     高       │   服务接口   │   无状态     │   应用边界   │");
    println!("│                │              │   协调控制   │              │   事务控制   │");
    println!("└────────────────┴──────────────┴──────────────┴──────────────┴──────────────┘");
    
    println!("\n🎯 选择指导原则：");
    println!("1. 简单业务逻辑 → Transaction Script");
    println!("2. 表导向数据处理 → Table Module");
    println!("3. 复杂业务规则 → Domain Model");
    println!("4. 需要应用边界 → Service Layer");
    println!("5. 企业级应用 → Domain Model + Service Layer");
    
    println!("\n✨ 可以组合使用：");
    println!("• Service Layer + Domain Model：企业级复杂应用");
    println!("• Service Layer + Table Module：数据处理型应用");
    println!("• Service Layer + Transaction Script：简单服务化应用");
    
    println!("\n🏁 === 领域逻辑模式演示完成 ===");
} 