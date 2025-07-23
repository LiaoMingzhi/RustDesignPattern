//! 结构型设计模式模块
//! 
//! 包含7种结构型设计模式的Rust实现：
//! - 适配器模式 (Adapter)
//! - 桥接模式 (Bridge)
//! - 组合模式 (Composite)
//! - 装饰器模式 (Decorator)
//! - 外观模式 (Facade)
//! - 享元模式 (Flyweight)
//! - 代理模式 (Proxy)

pub mod adapter;
pub mod bridge;
pub mod composite;
pub mod decorator;
pub mod facade;
pub mod flyweight;
pub mod proxy;

pub fn run_all_demos() {
    println!("=======================================");
    println!("      结构型设计模式演示");
    println!("=======================================");

    adapter::demo();
    bridge::demo();
    composite::demo();
    decorator::demo();
    facade::demo();
    flyweight::demo();
    proxy::demo();

    println!("=======================================");
    println!("      结构型模式演示完成");
    println!("=======================================\n");
} 