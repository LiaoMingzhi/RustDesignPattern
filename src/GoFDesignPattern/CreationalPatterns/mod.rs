//! 创建型设计模式模块
//! 
//! 包含5种创建型设计模式的Rust实现：
//! - 抽象工厂模式 (Abstract Factory)
//! - 建造者模式 (Builder)
//! - 工厂方法模式 (Factory Method)
//! - 原型模式 (Prototype)
//! - 单例模式 (Singleton)

pub mod abstract_factory;
pub mod builder;
pub mod factory_method;
pub mod prototype;
pub mod singleton;

pub fn run_all_demos() {
    println!("=======================================");
    println!("      创建型设计模式演示");
    println!("=======================================");

    abstract_factory::demo();
    builder::demo();
    factory_method::demo();
    prototype::demo();
    singleton::demo();

    println!("=======================================");
    println!("      创建型模式演示完成");
    println!("=======================================\n");
} 