/*
 * 文件路径: /d%3A/workspace/RustLearn/RustDesignPattern/src/DistributedSystemMode/DataConsistencyPatterns/saga_pattern.rs
 * 
 * Saga Pattern模式 (Saga模式)
 * 
 * Saga模式是一种分布式事务管理模式，通过一系列本地事务来实现分布式事务。
 * 如果某个步骤失败，会执行补偿操作来回滚之前的操作。
 */

use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone)]
pub enum SagaStepResult {
    Success,
    Failure(String),
}

pub trait SagaStep: Send + Sync {
    fn execute(&self) -> SagaStepResult;
    fn compensate(&self) -> SagaStepResult;
    fn get_name(&self) -> &str;
}

pub struct OrderCreationStep {
    order_id: String,
}

impl OrderCreationStep {
    pub fn new(order_id: String) -> Self {
        Self { order_id }
    }
}

impl SagaStep for OrderCreationStep {
    fn execute(&self) -> SagaStepResult {
        println!("创建订单: {}", self.order_id);
        SagaStepResult::Success
    }
    
    fn compensate(&self) -> SagaStepResult {
        println!("取消订单: {}", self.order_id);
        SagaStepResult::Success
    }
    
    fn get_name(&self) -> &str {
        "OrderCreation"
    }
}

pub struct SagaOrchestrator {
    steps: Vec<Box<dyn SagaStep>>,
    executed_steps: Vec<usize>,
}

impl SagaOrchestrator {
    pub fn new() -> Self {
        Self {
            steps: Vec::new(),
            executed_steps: Vec::new(),
        }
    }
    
    pub fn add_step(&mut self, step: Box<dyn SagaStep>) {
        self.steps.push(step);
    }
    
    pub fn execute(&mut self) -> Result<(), String> {
        self.executed_steps.clear();
        
        for (index, step) in self.steps.iter().enumerate() {
            match step.execute() {
                SagaStepResult::Success => {
                    self.executed_steps.push(index);
                }
                SagaStepResult::Failure(error) => {
                    println!("步骤 {} 失败: {}, 开始回滚", step.get_name(), error);
                    self.compensate();
                    return Err(error);
                }
            }
        }
        
        Ok(())
    }
    
    fn compensate(&self) {
        for &index in self.executed_steps.iter().rev() {
            if let Some(step) = self.steps.get(index) {
                println!("补偿步骤: {}", step.get_name());
                let _ = step.compensate();
            }
        }
    }
}

/// Saga Pattern模式演示
pub fn demo_saga_pattern() {
    println!("=== Saga Pattern模式演示 ===\n");
    
    let mut saga = SagaOrchestrator::new();
    saga.add_step(Box::new(OrderCreationStep::new("order-123".to_string())));
    
    match saga.execute() {
        Ok(_) => println!("Saga执行成功"),
        Err(e) => println!("Saga执行失败: {}", e),
    }
    
    println!("\n【Saga Pattern模式特点】");
    println!("✓ 分布式事务 - 通过本地事务序列实现分布式事务");
    println!("✓ 补偿机制 - 失败时自动执行补偿操作");
    println!("✓ 最终一致性 - 保证系统最终达到一致状态");
    println!("✓ 容错处理 - 优雅处理部分失败场景");
} 