/*
 * 文件路径: /d%3A/workspace/RustLearn/RustDesignPattern/src/DistributedSystemMode/ObservabilityPatterns/distributed_tracing.rs
 * 
 * Distributed Tracing模式 (分布式追踪)
 * 
 * 分布式追踪用于跟踪请求在微服务架构中的完整路径，
 * 帮助理解系统行为、性能分析和问题诊断。
 */

use std::collections::HashMap;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct TraceContext {
    pub trace_id: String,
    pub span_id: String,
    pub parent_span_id: Option<String>,
    pub baggage: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct Span {
    pub trace_id: String,
    pub span_id: String,
    pub parent_span_id: Option<String>,
    pub operation_name: String,
    pub start_time: Instant,
    pub finish_time: Option<Instant>,
    pub tags: HashMap<String, String>,
    pub logs: Vec<String>,
}

impl Span {
    pub fn new(trace_id: String, span_id: String, operation_name: String) -> Self {
        Self {
            trace_id,
            span_id,
            parent_span_id: None,
            operation_name,
            start_time: Instant::now(),
            finish_time: None,
            tags: HashMap::new(),
            logs: Vec::new(),
        }
    }
    
    pub fn set_tag(&mut self, key: String, value: String) {
        self.tags.insert(key, value);
    }
    
    pub fn log(&mut self, message: String) {
        self.logs.push(format!("[{}] {}", self.start_time.elapsed().as_millis(), message));
    }
    
    pub fn finish(&mut self) {
        self.finish_time = Some(Instant::now());
    }
    
    pub fn duration(&self) -> Option<Duration> {
        self.finish_time.map(|finish| finish.duration_since(self.start_time))
    }
}

pub struct Tracer {
    service_name: String,
}

impl Tracer {
    pub fn new(service_name: String) -> Self {
        Self { service_name }
    }
    
    pub fn start_span(&self, operation_name: String) -> Span {
        let trace_id = format!("trace_{}", uuid::generate());
        let span_id = format!("span_{}", uuid::generate());
        
        let mut span = Span::new(trace_id, span_id, operation_name);
        span.set_tag("service.name".to_string(), self.service_name.clone());
        span
    }
    
    pub fn start_child_span(&self, parent: &Span, operation_name: String) -> Span {
        let span_id = format!("span_{}", uuid::generate());
        
        let mut span = Span::new(parent.trace_id.clone(), span_id, operation_name);
        span.parent_span_id = Some(parent.span_id.clone());
        span.set_tag("service.name".to_string(), self.service_name.clone());
        span
    }
}

mod uuid {
    use std::time::{SystemTime, UNIX_EPOCH};
    
    pub fn generate() -> String {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        format!("{:x}", now.as_nanos())
    }
}

/// Distributed Tracing模式演示
pub fn demo_distributed_tracing() {
    println!("=== Distributed Tracing模式演示 ===\n");
    
    let tracer = Tracer::new("order-service".to_string());
    
    // 创建根Span
    let mut root_span = tracer.start_span("process_order".to_string());
    root_span.set_tag("order.id".to_string(), "order-123".to_string());
    root_span.log("开始处理订单".to_string());
    
    // 创建子Span
    let mut child_span = tracer.start_child_span(&root_span, "validate_payment".to_string());
    child_span.log("验证支付信息".to_string());
    std::thread::sleep(Duration::from_millis(50));
    child_span.finish();
    
    root_span.log("订单处理完成".to_string());
    root_span.finish();
    
    println!("追踪ID: {}", root_span.trace_id);
    println!("根Span持续时间: {:?}", root_span.duration());
    println!("子Span持续时间: {:?}", child_span.duration());
    
    println!("\n【Distributed Tracing模式特点】");
    println!("✓ 请求追踪 - 跟踪请求在系统中的完整路径");
    println!("✓ 性能分析 - 分析各个服务的响应时间");
    println!("✓ 依赖分析 - 理解服务间的调用关系");
    println!("✓ 问题诊断 - 快速定位分布式系统中的问题");
} 