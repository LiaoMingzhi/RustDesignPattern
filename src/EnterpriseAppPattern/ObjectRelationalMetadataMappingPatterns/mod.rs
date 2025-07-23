// 文件路径: /d%3A/workspace/RustLearn/RustDesignPattern/src/EnterpriseAppPattern/ObjectRelationalMetadataMappingPatterns/mod.rs

//! # 对象-关系元数据映射模式 (Object-Relational Metadata Mapping Patterns)
//!
//! ## 模式分类
//! 本目录包含处理对象-关系映射元数据管理的模式：
//!
//! ### 1. 元数据映射 (Metadata Mapping)
//! - **文件**: `metadata_mapping.rs`
//! - **描述**: 通过配置文件或注解定义对象和数据库表之间的映射关系
//! - **优点**: 映射与代码分离，支持运行时配置，便于维护
//! - **适用**: 复杂映射关系，多数据库支持，遗留系统集成
//!
//! ### 2. 查询对象 (Query Object)
//! - **文件**: `query_object.rs`
//! - **描述**: 将查询逻辑封装在对象中，提供面向对象的查询构建方式
//! - **优点**: 类型安全，复杂查询重用，流畅API
//! - **适用**: 复杂查询，动态查询条件，查询逻辑重用
//!
//! ### 3. 仓储 (Repository)
//! - **文件**: `repository.rs`
//! - **描述**: 提供集合式的数据访问接口，封装数据访问逻辑
//! - **优点**: 集中数据访问，可测试性好，支持多数据源
//! - **适用**: 领域驱动设计，复杂查询，需要抽象数据层

pub mod metadata_mapping;
pub mod query_object;
pub mod repository;

pub use metadata_mapping::*;
pub use query_object::*;
pub use repository::*;

/// 演示所有对象-关系元数据映射模式
pub fn demo_all() {
    println!("=== 对象-关系元数据映射模式总览 ===\n");
    
    println!("📋 模式对比表:");
    println!("┌──────────────────┬────────────────────┬────────────────────┬────────────────────┐");
    println!("│ 特性             │ 元数据映射         │ 查询对象           │ 仓储模式           │");
    println!("├──────────────────┼────────────────────┼────────────────────┼────────────────────┤");
    println!("│ 主要职责         │ 映射配置管理       │ 查询逻辑封装       │ 数据访问抽象       │");
    println!("│ 配置方式         │ 外部配置文件       │ 代码构建           │ 接口定义           │");
    println!("│ 类型安全         │ 中等               │ 高                 │ 高                 │");
    println!("│ 复用性           │ 高                 │ 高                 │ 高                 │");
    println!("│ 测试友好         │ 中等               │ 好                 │ 很好               │");
    println!("│ 学习曲线         │ 中等               │ 低                 │ 低                 │");
    println!("│ 适用场景         │ 配置驱动的系统     │ 复杂查询逻辑       │ 领域驱动设计       │");
    println!("└──────────────────┴────────────────────┴────────────────────┴────────────────────┘");
    
    println!("\n🎯 选择指南:");
    println!("• 元数据映射 适用于:");
    println!("  - 需要支持多种数据库的应用");
    println!("  - 映射关系复杂且经常变化");
    println!("  - 配置驱动的系统架构");
    println!("  - 与遗留数据库系统集成");
    
    println!("\n• 查询对象 适用于:");
    println!("  - 有复杂查询逻辑的系统");
    println!("  - 需要动态构建查询条件");
    println!("  - 希望查询逻辑可重用");
    println!("  - 类型安全的查询构建");
    
    println!("\n• 仓储模式 适用于:");
    println!("  - 领域驱动设计项目");
    println!("  - 需要高度可测试的代码");
    println!("  - 支持多种数据源");
    println!("  - 集中化数据访问逻辑");
    
    println!("\n💡 组合使用建议:");
    println!("  这三种模式通常可以组合使用：");
    println!("  - 元数据映射 定义对象-表映射关系");
    println!("  - 查询对象 构建复杂查询逻辑");
    println!("  - 仓储模式 提供统一的数据访问接口");
    
    println!("\n{}", "=".repeat(60));
    
    // 演示元数据映射
    metadata_mapping::demo();
    
    println!("\n{}", "=".repeat(60));
    
    // 演示查询对象
    query_object::demo();
    
    println!("\n{}", "=".repeat(60));
    
    // 演示仓储模式
    repository::demo();
    
    println!("\n=== 对象-关系元数据映射模式演示全部完成 ===");
} 