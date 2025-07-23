/*
 * 文件路径: /d%3A/workspace/RustLearn/RustDesignPattern/src/DistributedSystemMode/CommunicationPatterns/mod.rs
 * 
 * 通信模式模块 (Communication Patterns)
 */

pub mod api_gateway;

// 其他模式的存根实现
pub mod service_mesh {
    pub fn demo_service_mesh() {
        println!("=== Service Mesh模式演示 ===");
        println!("服务网格处理服务间通信的基础设施层");
    }
}

pub mod message_queue {
    pub fn demo_message_queue() {
        println!("=== Message Queue模式演示 ===");
        println!("异步消息传递，解耦生产者和消费者");
    }
}

pub mod publish_subscribe {
    pub fn demo_publish_subscribe() {
        println!("=== Publish-Subscribe模式演示 ===");
        println!("发布订阅模式支持一对多的消息传递");
    }
} 