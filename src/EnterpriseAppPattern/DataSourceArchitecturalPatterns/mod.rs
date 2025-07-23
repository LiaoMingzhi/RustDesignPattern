//! 数据源架构模式模块 (Data Source Architectural Patterns)
//! 
//! 这些模式处理数据源层的架构设计。
//! 文件路径：/d%3A/workspace/RustLearn/RustDesignPattern/src/EnterpriseAppPattern/DataSourceArchitecturalPatterns/mod.rs

pub mod table_data_gateway;
pub mod row_data_gateway;
pub mod active_record;
pub mod data_mapper;

pub fn demo_all() {
    println!("=== 数据源架构模式演示 ===");
    
    println!("\n==========================================");
    table_data_gateway::demo();
    
    println!("\n==========================================");
    row_data_gateway::demo();
    
    println!("\n==========================================");
    active_record::demo();
    
    println!("\n==========================================");
    data_mapper::demo();
    
    println!("\n==========================================");
    println!("数据源架构模式总结:");
    println!("1. 表数据入口 (Table Data Gateway) - 一个类处理表的所有行");
    println!("2. 行数据入口 (Row Data Gateway) - 一个对象对应一行数据");
    println!("3. 活动记录 (Active Record) - 对象包含数据和行为");
    println!("4. 数据映射器 (Data Mapper) - 对象与数据库完全分离");
} 