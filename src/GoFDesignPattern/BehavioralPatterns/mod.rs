//! 行为型设计模式模块
//! 
//! 包含11种行为型设计模式的Rust实现：
//! - 责任链模式 (Chain of Responsibility)
//! - 命令模式 (Command)
//! - 解释器模式 (Interpreter)
//! - 迭代器模式 (Iterator)
//! - 中介者模式 (Mediator)
//! - 备忘录模式 (Memento)
//! - 观察者模式 (Observer)
//! - 状态模式 (State)
//! - 策略模式 (Strategy)
//! - 模板方法模式 (Template Method)
//! - 访问者模式 (Visitor)

pub mod chain_of_responsibility;
pub mod command;
pub mod interpreter;
pub mod iterator;
pub mod mediator;
pub mod memento;
pub mod observer;
pub mod state;
pub mod strategy;
pub mod template_method;
pub mod visitor;

pub fn run_all_demos() {
    println!("=======================================");
    println!("      行为型设计模式演示");
    println!("=======================================");

    chain_of_responsibility::demo();
    command::demo();
    interpreter::demo();
    iterator::demo();
    mediator::demo();
    memento::demo();
    observer::demo();
    state::demo();
    strategy::demo();
    template_method::demo();
    visitor::demo();

    println!("=======================================");
    println!("      行为型模式演示完成");
    println!("=======================================\n");
} 