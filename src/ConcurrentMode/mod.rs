/*
 * 文件路径: /d%3A/workspace/RustLearn/RustDesignPattern/src/ConcurrentMode/mod.rs
 * 
 * 并发模式 (Concurrent Patterns)
 * 
 * 这个模块包含了各种并发编程模式的Rust实现，帮助开发者处理多线程和异步编程场景。
 * 所有实现都遵循Rust的线程安全要求，充分利用Rust的类型系统来防止数据竞争。
 * 
 * 包含的并发模式：
 * 1. Actor模式 - 基于消息传递的并发模型
 * 2. Producer-Consumer模式 - 生产者消费者模式
 * 3. Worker Pool模式 - 工作线程池模式
 * 4. Pipeline模式 - 流水线并发处理
 * 5. Master-Worker模式 - 主从工作模式
 * 6. Reactor模式 - 事件驱动的异步I/O模式
 * 7. Future-Promise模式 - 异步计算模式
 * 8. Fork-Join模式 - 分而治之的并行模式
 */

pub mod actor_pattern;
pub mod producer_consumer;
pub mod worker_pool;
pub mod pipeline_pattern;
pub mod master_worker;
pub mod reactor_pattern;
pub mod future_promise;
pub mod fork_join;

/// 演示所有并发模式
pub fn demo_all_concurrent_patterns() {
    println!("=== 并发模式演示合集 ===\n");
    
    // Actor模式演示
    println!("【1. Actor模式】");
    actor_pattern::demo_actor_pattern();
    println!("\n{}\n", "=".repeat(80));
    
    // Producer-Consumer模式演示
    println!("【2. Producer-Consumer模式】");
    producer_consumer::demo_producer_consumer();
    println!("\n{}\n", "=".repeat(80));
    
    // Pipeline模式演示
    println!("【3. Pipeline模式】");
    pipeline_pattern::demo_pipeline();
    println!("\n{}\n", "=".repeat(80));
    
    // Master-Worker模式演示
    println!("【4. Master-Worker模式】");
    master_worker::demo_master_worker();
    println!("\n{}\n", "=".repeat(80));
    
    // Reactor模式演示
    println!("【5. Reactor模式】");
    reactor_pattern::demo_reactor();
    println!("\n{}\n", "=".repeat(80));
    
    // Future-Promise模式演示
    println!("【6. Future-Promise模式】");
    future_promise::demo_future_promise();
    println!("\n{}\n", "=".repeat(80));
    
    // Fork-Join模式演示
    println!("【7. Fork-Join模式】");
    fork_join::demo_fork_join();

    // Worker Pool模式演示
    println!("【8. Worker Pool模式】");
    worker_pool::demo_worker_pool();
    println!("\n{}\n", "=".repeat(80));
    
    println!("\n=== 并发模式演示完成 ===");
} 