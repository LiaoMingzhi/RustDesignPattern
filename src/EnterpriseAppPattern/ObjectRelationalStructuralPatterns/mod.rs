// 文件路径: /d%3A/workspace/RustLearn/RustDesignPattern/src/EnterpriseAppPattern/ObjectRelationalStructuralPatterns/mod.rs

//! # 对象-关系结构模式 (Object-Relational Structural Patterns)
//!
//! ## 模式分类
//! 本目录包含处理对象和关系数据库结构映射的模式：
//!
//! ### 1. 身份字段 (Identity Field)
//! - **文件**: `identity_field.rs`
//! - **描述**: 为每个对象保存一个数据库主键字段，维护对象与数据库记录的映射
//! - **优点**: 唯一标识对象，支持高效查找，便于实现缓存
//! - **适用**: 所有持久化对象，对象引用关系，身份映射
//!
//! ### 2. 外键映射 (Foreign Key Mapping)
//! - **文件**: `foreign_key_mapping.rs`
//! - **描述**: 通过外键在对象中维护对其他对象的引用关系
//! - **优点**: 维护引用完整性，支持对象导航，高效关联查询
//! - **适用**: 对象间有引用关系，需要保证数据一致性的场景
//!
//! ### 3. 关联表映射 (Association Table Mapping)
//! - **文件**: `association_table_mapping.rs`
//! - **描述**: 通过独立的关联表处理多对多关系
//! - **优点**: 处理复杂关系，支持关联属性，符合数据库设计规范
//! - **适用**: 多对多关系，需要在关联中存储额外信息的场景
//!
//! ### 4. 依赖映射 (Dependent Mapping)
//! - **文件**: `dependent_mapping.rs`
//! - **描述**: 处理依赖对象的映射，通常用于值对象和组合关系
//! - **优点**: 简化对象结构，减少表的数量，保持数据一致性
//! - **适用**: 值对象映射，组合关系，嵌套对象结构
//!
//! ### 5. 嵌入值 (Embedded Value)
//! - **文件**: `embedded_value.rs`
//! - **描述**: 将值对象的数据嵌入到拥有者对象的表中
//! - **优点**: 减少表连接，提高查询性能，简化数据模型
//! - **适用**: 值对象，地址、金额等复合值类型
//!
//! ### 6. 序列化LOB (Serialized LOB)
//! - **文件**: `serialized_lob.rs`
//! - **描述**: 将复杂对象图序列化为大对象存储
//! - **优点**: 简化映射，减少表数量，原子操作
//! - **适用**: 复杂对象图，文档存储，配置数据
//!
//! ### 7. 单表继承 (Single Table Inheritance)
//! - **文件**: `single_table_inheritance.rs`
//! - **描述**: 将继承层次的所有类映射到单个表
//! - **优点**: 简单查询，无需连接，多态查询高效
//! - **适用**: 继承层次简单，子类差异不大的场景
//!
//! ### 8. 类表继承 (Class Table Inheritance)
//! - **文件**: `class_table_inheritance.rs`
//! - **描述**: 为继承层次中的每个类创建独立的表
//! - **优点**: 数据规范化，无冗余，扩展性好
//! - **适用**: 继承层次复杂，子类差异较大的场景
//!
//! ### 9. 具体表继承 (Concrete Table Inheritance)
//! - **文件**: `concrete_table_inheritance.rs`
//! - **描述**: 为每个具体类创建一个表，包含所有继承的字段
//! - **优点**: 无需表连接，查询性能好，模式简单
//! - **适用**: 查询性能优先，很少多态查询，继承层次稳定

pub mod identity_field;
pub mod foreign_key_mapping;
pub mod association_table_mapping;
pub mod dependent_mapping;
pub mod embedded_value;
pub mod serialized_lob;
pub mod single_table_inheritance;
pub mod class_table_inheritance;
pub mod concrete_table_inheritance;

pub use identity_field::*;
pub use foreign_key_mapping::*;
pub use association_table_mapping::*;
pub use dependent_mapping::*;
pub use embedded_value::*;
pub use serialized_lob::*;
pub use single_table_inheritance::*;
pub use class_table_inheritance::*;
pub use concrete_table_inheritance::*;

/// 演示所有对象-关系结构模式
pub fn demo_all() {
    println!("=== 对象-关系结构模式总览 ===\n");
    
    println!("📋 模式对比表:");
    println!("┌─────────────────┬────────────────┬─────────────────┬──────────────────┐");
    println!("│ 模式           │ 适用场景       │ 优点            │ 缺点             │");
    println!("├─────────────────┼────────────────┼─────────────────┼──────────────────┤");
    println!("│ 身份字段       │ 所有持久化对象 │ 唯一标识，缓存  │ 额外存储开销     │");
    println!("│ 外键映射       │ 对象关系映射   │ 引用完整性      │ 查询复杂度       │");
    println!("│ 关联表映射     │ 多对多关系     │ 灵活，规范化    │ 额外表结构       │");
    println!("│ 依赖映射       │ 值对象映射     │ 简化结构        │ 生命周期耦合     │");
    println!("│ 嵌入值         │ 复合值类型     │ 性能好，简单    │ 数据冗余可能     │");
    println!("│ 序列化LOB      │ 复杂对象图     │ 简化映射        │ 查询限制         │");
    println!("│ 单表继承       │ 简单继承层次   │ 查询简单        │ 空值，表膨胀     │");
    println!("│ 类表继承       │ 复杂继承层次   │ 规范化，扩展性  │ 多表连接         │");
    println!("└─────────────────┴────────────────┴─────────────────┴──────────────────┘");
    
    println!("\n🎯 设计指南:");
    println!("  
    • 基础映射模式:");
    println!("    - 身份字段: 所有持久化对象的基础");
    println!("    - 外键映射: 处理对象间的引用关系");
    
    println!("\n  • 关系映射模式:");
    println!("    - 一对一: 外键 + 唯一约束 或 嵌入值");
    println!("    - 一对多: 外键映射");
    println!("    - 多对多: 关联表映射");
    
    println!("\n  • 继承映射策略:");
    println!("    - 简单继承: 单表继承");
    println!("    - 复杂继承: 类表继承");
    println!("    - 混合策略: 根据需要组合使用");
    
    println!("\n  • 复杂对象映射:");
    println!("    - 值对象: 嵌入值 或 依赖映射");
    println!("    - 对象图: 序列化LOB");
    
    println!("\n💡 选择标准:");
    
    println!("\n  1. 数据规范化 vs 查询性能:");
    println!("     - 高规范化: 类表继承, 关联表映射");
    println!("     - 高性能: 单表继承, 嵌入值, 序列化LOB");
    
    println!("\n  2. 查询复杂度 vs 存储效率:");
    println!("     - 简单查询: 单表继承, 嵌入值");
    println!("     - 存储效率: 类表继承, 依赖映射");
    
    println!("\n  3. 扩展性 vs 维护成本:");
    println!("     - 高扩展性: 类表继承, 关联表映射");
    println!("     - 低维护成本: 单表继承, 序列化LOB");
    
    println!("\n{}", "=".repeat(60));
    
    // 演示各种模式
    println!("\n🚀 开始演示各种对象-关系结构模式...\n");
    
    identity_field::demo();
    println!("\n{}", "-".repeat(60));
    
    foreign_key_mapping::demo();
    println!("\n{}", "-".repeat(60));
    
    association_table_mapping::demo();
    println!("\n{}", "-".repeat(60));
    
    dependent_mapping::demo();
    println!("\n{}", "-".repeat(60));
    
    embedded_value::demo();
    println!("\n{}", "-".repeat(60));
    
    serialized_lob::demo();
    println!("\n{}", "-".repeat(60));
    
    single_table_inheritance::demo();
    println!("\n{}", "-".repeat(60));
    
    class_table_inheritance::demo();
    println!("\n{}", "-".repeat(60));
    
    concrete_table_inheritance::demo();
    
    println!("\n=== 对象-关系结构模式演示全部完成 ===");
} 