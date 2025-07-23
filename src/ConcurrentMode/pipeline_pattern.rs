/*
 * 文件路径: /d%3A/workspace/RustLearn/RustDesignPattern/src/ConcurrentMode/pipeline_pattern.rs
 * 
 * Pipeline模式 (Pipeline Pattern)
 * 
 * 流水线模式将复杂的数据处理任务分解为多个阶段，每个阶段由独立的线程处理。
 * 数据按顺序通过各个阶段，每个阶段专注于特定的处理逻辑，提高系统的吞吐量和可扩展性。
 * 
 * 主要特点：
 * 1. 分阶段处理 - 将复杂任务分解为简单阶段
 * 2. 并行执行 - 多个阶段可以同时处理不同数据
 * 3. 背压控制 - 防止快速阶段压垮慢速阶段
 * 4. 错误隔离 - 错误处理局限在特定阶段
 * 5. 可扩展性 - 可以动态添加或移除阶段
 */

use std::sync::{Arc, Mutex};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};
use std::collections::VecDeque;
use std::fmt;
use std::marker::PhantomData;

// =================
// 流水线阶段特质
// =================

/// 流水线阶段处理器特质
pub trait StageProcessor<Input, Output>: Send + 'static {
    /// 处理输入数据并产生输出
    fn process(&mut self, input: Input) -> Result<Output, ProcessError>;
    
    /// 获取阶段名称
    fn name(&self) -> &str;
    
    /// 阶段初始化（可选）
    fn initialize(&mut self) -> Result<(), ProcessError> {
        Ok(())
    }
    
    /// 阶段清理（可选）
    fn cleanup(&mut self) -> Result<(), ProcessError> {
        Ok(())
    }
}

/// 处理错误类型
#[derive(Debug)]
pub enum ProcessError {
    InvalidInput(String),
    ProcessingFailed(String),
    ResourceUnavailable,
    Timeout,
    Custom(String),
}

impl fmt::Display for ProcessError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessError::InvalidInput(msg) => write!(f, "输入无效: {}", msg),
            ProcessError::ProcessingFailed(msg) => write!(f, "处理失败: {}", msg),
            ProcessError::ResourceUnavailable => write!(f, "资源不可用"),
            ProcessError::Timeout => write!(f, "处理超时"),
            ProcessError::Custom(msg) => write!(f, "自定义错误: {}", msg),
        }
    }
}

// =================
// 数据流和控制信号
// =================

/// 流水线数据项
#[derive(Debug, Clone)]
pub struct DataItem<T> {
    pub data: T,
    pub id: u64,
    pub timestamp: Instant,
    pub metadata: std::collections::HashMap<String, String>,
}

impl<T> DataItem<T> {
    pub fn new(data: T, id: u64) -> Self {
        Self {
            data,
            id,
            timestamp: Instant::now(),
            metadata: std::collections::HashMap::new(),
        }
    }
    
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
    
    pub fn age(&self) -> Duration {
        self.timestamp.elapsed()
    }
}

/// 流水线控制信号
#[derive(Debug)]
pub enum ControlSignal {
    Stop,
    Pause,
    Resume,
    Flush,
}

/// 阶段间消息
#[derive(Debug)]
pub enum StageMessage<T> {
    Data(DataItem<T>),
    Control(ControlSignal),
    Error(ProcessError),
}

// =================
// 流水线阶段实现
// =================

/// 流水线阶段
pub struct PipelineStage<Input, Output> {
    name: String,
    processor: Option<Box<dyn StageProcessor<Input, Output>>>,
    input_receiver: Option<Receiver<StageMessage<Input>>>,
    output_sender: Option<Sender<StageMessage<Output>>>,
    error_sender: Sender<ProcessError>,
    buffer_size: usize,
    processed_count: Arc<Mutex<u64>>,
    error_count: Arc<Mutex<u64>>,
    handle: Option<JoinHandle<()>>,
}

impl<Input: Send + 'static, Output: Send + 'static> PipelineStage<Input, Output> {
    pub fn new(
        name: String,
        processor: Box<dyn StageProcessor<Input, Output>>,
        input_receiver: Receiver<StageMessage<Input>>,
        output_sender: Option<Sender<StageMessage<Output>>>,
        error_sender: Sender<ProcessError>,
        buffer_size: usize,
    ) -> Self {
        Self {
            name,
            processor: Some(processor),
            input_receiver: Some(input_receiver),
            output_sender,
            error_sender,
            buffer_size,
            processed_count: Arc::new(Mutex::new(0)),
            error_count: Arc::new(Mutex::new(0)),
            handle: None,
        }
    }
    
    /// 启动阶段处理
    pub fn start(&mut self) {
        let name = self.name.clone();
        let mut processor = match self.processor.take() {
            Some(p) => p,
            None => {
                println!("阶段 '{}' 已经启动或处理器不可用", name);
                return;
            }
        };
        
        let input_receiver = match self.input_receiver.take() {
            Some(r) => r,
            None => {
                println!("阶段 '{}' 的输入接收器不可用", name);
                return;
            }
        };
        
        let output_sender = self.output_sender.take();
        let error_sender = self.error_sender.clone();
        let processed_count = Arc::clone(&self.processed_count);
        let error_count = Arc::clone(&self.error_count);
        
        let handle = thread::spawn(move || {
            println!("流水线阶段 '{}' 启动", name);
            
            if let Err(e) = processor.initialize() {
                println!("阶段 '{}' 初始化失败: {}", name, e);
                let _ = error_sender.send(e);
                return;
            }
            
            let mut is_paused = false;
            
            while let Ok(message) = input_receiver.recv() {
                match message {
                    StageMessage::Data(data_item) => {
                        if is_paused {
                            continue;
                        }
                        
                        match processor.process(data_item.data) {
                            Ok(output) => {
                                let output_item = DataItem {
                                    data: output,
                                    id: data_item.id,
                                    timestamp: data_item.timestamp,
                                    metadata: data_item.metadata,
                                };
                                
                                if let Some(ref sender) = output_sender {
                                    if let Err(_) = sender.send(StageMessage::Data(output_item)) {
                                        println!("阶段 '{}' 输出通道已关闭", name);
                                        break;
                                    }
                                }
                                
                                let mut count = processed_count.lock().unwrap();
                                *count += 1;
                            }
                            Err(e) => {
                                println!("阶段 '{}' 处理错误: {}", name, e);
                                let _ = error_sender.send(e);
                                
                                let mut count = error_count.lock().unwrap();
                                *count += 1;
                            }
                        }
                    }
                    StageMessage::Control(signal) => {
                        match signal {
                            ControlSignal::Stop => {
                                println!("阶段 '{}' 收到停止信号", name);
                                break;
                            }
                            ControlSignal::Pause => {
                                println!("阶段 '{}' 暂停", name);
                                is_paused = true;
                            }
                            ControlSignal::Resume => {
                                println!("阶段 '{}' 恢复", name);
                                is_paused = false;
                            }
                            ControlSignal::Flush => {
                                println!("阶段 '{}' 刷新缓冲区", name);
                                // 这里可以实现缓冲区刷新逻辑
                            }
                        }
                        
                        // 传递控制信号给下一阶段
                        if let Some(ref sender) = output_sender {
                            let _ = sender.send(StageMessage::Control(signal));
                        }
                    }
                    StageMessage::Error(e) => {
                        println!("阶段 '{}' 收到错误信号: {}", name, e);
                        let _ = error_sender.send(e);
                    }
                }
            }
            
            if let Err(e) = processor.cleanup() {
                println!("阶段 '{}' 清理失败: {}", name, e);
                let _ = error_sender.send(e);
            }
            
            println!("阶段 '{}' 停止", name);
        });
        
        self.handle = Some(handle);
    }
    
    /// 获取处理统计
    pub fn get_stats(&self) -> (u64, u64) {
        let processed = *self.processed_count.lock().unwrap();
        let errors = *self.error_count.lock().unwrap();
        (processed, errors)
    }
}

/// 虚拟处理器（用于占位）
struct DummyProcessor<Input, Output> {
    _phantom: PhantomData<(Input, Output)>,
}

impl<Input, Output> DummyProcessor<Input, Output> {
    fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<Input: Send + 'static, Output: Send + 'static> StageProcessor<Input, Output> for DummyProcessor<Input, Output> {
    fn process(&mut self, _input: Input) -> Result<Output, ProcessError> {
        panic!("DummyProcessor should not be used")
    }
    
    fn name(&self) -> &str {
        "dummy"
    }
}

// =================
// 流水线构建器
// =================

/// 流水线配置
#[derive(Debug, Clone)]
pub struct PipelineConfig {
    pub buffer_size: usize,
    pub max_processing_time: Option<Duration>,
    pub enable_metrics: bool,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            buffer_size: 100,
            max_processing_time: Some(Duration::from_secs(30)),
            enable_metrics: true,
        }
    }
}

/// 流水线构建器
pub struct PipelineBuilder<T> {
    stages: Vec<Box<dyn std::any::Any + Send>>,
    config: PipelineConfig,
    _phantom: PhantomData<T>,
}

impl<T> PipelineBuilder<T> {
    pub fn new() -> Self {
        Self {
            stages: Vec::new(),
            config: PipelineConfig::default(),
            _phantom: PhantomData,
        }
    }
    
    pub fn with_config(mut self, config: PipelineConfig) -> Self {
        self.config = config;
        self
    }
    
    pub fn add_stage<Output>(
        mut self,
        processor: Box<dyn StageProcessor<T, Output>>
    ) -> PipelineBuilder<Output>
    where
        T: Send + 'static,
        Output: Send + 'static,
    {
        self.stages.push(Box::new(processor));
        PipelineBuilder {
            stages: self.stages,
            config: self.config,
            _phantom: PhantomData,
        }
    }
}

// =================
// 示例处理器实现
// =================

/// 字符串处理器：转换为大写
pub struct UppercaseProcessor {
    name: String,
    delay: Duration,
}

impl UppercaseProcessor {
    pub fn new(delay: Duration) -> Self {
        Self {
            name: "大写转换器".to_string(),
            delay,
        }
    }
}

impl StageProcessor<String, String> for UppercaseProcessor {
    fn process(&mut self, input: String) -> Result<String, ProcessError> {
        thread::sleep(self.delay);
        Ok(input.to_uppercase())
    }
    
    fn name(&self) -> &str {
        &self.name
    }
}

/// 数字处理器：平方计算
pub struct SquareProcessor {
    name: String,
    delay: Duration,
}

impl SquareProcessor {
    pub fn new(delay: Duration) -> Self {
        Self {
            name: "平方计算器".to_string(),
            delay,
        }
    }
}

impl StageProcessor<i32, i64> for SquareProcessor {
    fn process(&mut self, input: i32) -> Result<i64, ProcessError> {
        thread::sleep(self.delay);
        Ok((input as i64) * (input as i64))
    }
    
    fn name(&self) -> &str {
        &self.name
    }
}

/// 字符串长度处理器
pub struct LengthProcessor {
    name: String,
    delay: Duration,
}

impl LengthProcessor {
    pub fn new(delay: Duration) -> Self {
        Self {
            name: "长度计算器".to_string(),
            delay,
        }
    }
}

impl StageProcessor<String, usize> for LengthProcessor {
    fn process(&mut self, input: String) -> Result<usize, ProcessError> {
        thread::sleep(self.delay);
        Ok(input.len())
    }
    
    fn name(&self) -> &str {
        &self.name
    }
}

/// 数据验证处理器
pub struct ValidationProcessor {
    name: String,
    min_length: usize,
}

impl ValidationProcessor {
    pub fn new(min_length: usize) -> Self {
        Self {
            name: "数据验证器".to_string(),
            min_length,
        }
    }
}

impl StageProcessor<String, String> for ValidationProcessor {
    fn process(&mut self, input: String) -> Result<String, ProcessError> {
        if input.len() < self.min_length {
            Err(ProcessError::InvalidInput(
                format!("字符串长度 {} 小于最小要求 {}", input.len(), self.min_length)
            ))
        } else {
            Ok(input)
        }
    }
    
    fn name(&self) -> &str {
        &self.name
    }
}

/// 格式化处理器
pub struct FormatProcessor {
    name: String,
    format: String,
}

impl FormatProcessor {
    pub fn new(format: String) -> Self {
        Self {
            name: "格式化器".to_string(),
            format,
        }
    }
}

impl StageProcessor<usize, String> for FormatProcessor {
    fn process(&mut self, input: usize) -> Result<String, ProcessError> {
        Ok(self.format.replace("{}", &input.to_string()))
    }
    
    fn name(&self) -> &str {
        &self.name
    }
}

// =================
// 简单流水线实现
// =================

/// 简单的三阶段流水线示例
pub struct SimpleTextPipeline {
    input_sender: Sender<StageMessage<String>>,
    output_receiver: Receiver<String>,
    error_receiver: Receiver<ProcessError>,
    stages: Vec<Box<dyn std::any::Any + Send>>,
}

impl SimpleTextPipeline {
    pub fn new() -> Self {
        let (input_sender, stage1_receiver) = mpsc::channel();
        let (stage1_sender, stage2_receiver) = mpsc::channel();
        let (stage2_sender, stage3_receiver) = mpsc::channel();
        let (output_sender, output_receiver) = mpsc::channel();
        let (error_sender, error_receiver) = mpsc::channel();
        
        // 阶段1: 验证
        let validation_processor = Box::new(ValidationProcessor::new(3));
        let mut stage1 = PipelineStage::new(
            "验证阶段".to_string(),
            validation_processor,
            stage1_receiver,
            Some(stage1_sender),
            error_sender.clone(),
            100,
        );
        
        // 阶段2: 转大写
        let uppercase_processor = Box::new(UppercaseProcessor::new(Duration::from_millis(10)));
        let mut stage2 = PipelineStage::new(
            "大写转换阶段".to_string(),
            uppercase_processor,
            stage2_receiver,
            Some(stage2_sender),
            error_sender.clone(),
            100,
        );
        
        // 阶段3: 计算长度并格式化
        let length_processor = Box::new(LengthProcessor::new(Duration::from_millis(5)));
        
        // 启动阶段
        stage1.start();
        stage2.start();
        
        // 启动最后阶段（单独处理）
        let stage3_error_sender = error_sender.clone();
        let stage3_handle = thread::spawn(move || {
            let mut processor = length_processor;
            if let Err(e) = processor.initialize() {
                let _ = stage3_error_sender.send(e);
                return;
            }
            
            while let Ok(message) = stage3_receiver.recv() {
                match message {
                    StageMessage::Data(data_item) => {
                        match processor.process(data_item.data) {
                            Ok(length) => {
                                let result = format!("长度: {}", length);
                                if output_sender.send(result).is_err() {
                                    break;
                                }
                            }
                            Err(e) => {
                                let _ = stage3_error_sender.send(e);
                            }
                        }
                    }
                    StageMessage::Control(ControlSignal::Stop) => break,
                    _ => {}
                }
            }
            
            let _ = processor.cleanup();
        });
        
        let mut stages = Vec::new();
        stages.push(Box::new(stage1) as Box<dyn std::any::Any + Send>);
        stages.push(Box::new(stage2) as Box<dyn std::any::Any + Send>);
        stages.push(Box::new(stage3_handle) as Box<dyn std::any::Any + Send>);
        
        Self {
            input_sender,
            output_receiver,
            error_receiver,
            stages,
        }
    }
    
    /// 处理输入数据
    pub fn process(&self, data: String) -> Result<(), ProcessError> {
        let data_item = DataItem::new(data, 1);
        self.input_sender.send(StageMessage::Data(data_item))
            .map_err(|_| ProcessError::Custom("发送失败".to_string()))
    }
    
    /// 获取处理结果
    pub fn get_result(&self, timeout: Duration) -> Result<String, ProcessError> {
        self.output_receiver.recv_timeout(timeout)
            .map_err(|_| ProcessError::Timeout)
    }
    
    /// 检查错误
    pub fn check_errors(&self) -> Vec<ProcessError> {
        let mut errors = Vec::new();
        while let Ok(error) = self.error_receiver.try_recv() {
            errors.push(error);
        }
        errors
    }
    
    /// 停止流水线
    pub fn stop(&self) {
        let _ = self.input_sender.send(StageMessage::Control(ControlSignal::Stop));
    }
}

// =================
// 演示函数
// =================

/// Pipeline模式演示
pub fn demo_pipeline() {
    println!("=== Pipeline模式演示 ===\n");
    
    // 1. 简单文本处理流水线
    println!("1. 简单文本处理流水线:");
    {
        let pipeline = SimpleTextPipeline::new();
        
        let test_inputs = vec![
            "hello world".to_string(),
            "rust".to_string(),
            "pipeline pattern".to_string(),
            "ok".to_string(), // 这个会因为长度不足而失败
            "concurrent programming".to_string(),
        ];
        
        for (i, input) in test_inputs.iter().enumerate() {
            println!("处理输入 {}: '{}'", i + 1, input);
            
            match pipeline.process(input.clone()) {
                Ok(_) => {
                    match pipeline.get_result(Duration::from_millis(500)) {
                        Ok(result) => println!("  结果: {}", result),
                        Err(e) => println!("  获取结果失败: {}", e),
                    }
                }
                Err(e) => println!("  处理失败: {}", e),
            }
            
            // 检查错误
            let errors = pipeline.check_errors();
            for error in errors {
                println!("  错误: {}", error);
            }
            
            thread::sleep(Duration::from_millis(50));
        }
        
        pipeline.stop();
        thread::sleep(Duration::from_millis(100));
    }
    
    println!("\n{}\n", "=".repeat(50));
    
    // 2. 数字处理流水线演示
    println!("2. 数字处理流水线演示:");
    {
        // 创建数字处理通道
        let (input_sender, stage1_receiver) = mpsc::channel();
        let (stage1_sender, output_receiver) = mpsc::channel();
        let (error_sender, error_receiver) = mpsc::channel();
        
        // 启动平方计算阶段
        let square_processor = Box::new(SquareProcessor::new(Duration::from_millis(20)));
        let mut square_stage = PipelineStage::new(
            "平方计算".to_string(),
            square_processor,
            stage1_receiver,
            Some(stage1_sender),
            error_sender.clone(),
            50,
        );
        
        square_stage.start();
        
        // 启动输出处理线程
        let output_handle = thread::spawn(move || {
            let mut results = Vec::new();
            while let Ok(message) = output_receiver.recv() {
                match message {
                    StageMessage::Data(data_item) => {
                        results.push((data_item.id, data_item.data));
                        println!("  数字 {} 的平方: {}", data_item.id, data_item.data);
                    }
                    StageMessage::Control(ControlSignal::Stop) => break,
                    _ => {}
                }
            }
            results
        });
        
        // 发送测试数据
        for i in 1..=10 {
            let data_item = DataItem::new(i, i as u64);
            let _ = input_sender.send(StageMessage::Data(data_item));
        }
        
        // 发送停止信号
        let _ = input_sender.send(StageMessage::Control(ControlSignal::Stop));
        
        // 等待处理完成
        if let Ok(results) = output_handle.join() {
            println!("处理完成，共处理 {} 个数字", results.len());
        }
        
        // 检查阶段统计
        let (processed, errors) = square_stage.get_stats();
        println!("阶段统计: 处理 {} 项，错误 {} 项", processed, errors);
        
        // 检查错误
        while let Ok(error) = error_receiver.try_recv() {
            println!("错误: {}", error);
        }
    }
    
    println!("\n{}\n", "=".repeat(50));
    
    // 3. 性能测试
    println!("3. 流水线性能测试:");
    {
        let (input_sender, receiver) = mpsc::channel();
        let (output_sender, output_receiver) = mpsc::channel();
        let (error_sender, _error_receiver) = mpsc::channel();
        
        // 快速处理器
        let fast_processor = Box::new(SquareProcessor::new(Duration::from_millis(1)));
        let mut fast_stage = PipelineStage::new(
            "快速处理".to_string(),
            fast_processor,
            receiver,
            Some(output_sender),
            error_sender,
            1000,
        );
        
        fast_stage.start();
        
        let start_time = Instant::now();
        const TEST_COUNT: u64 = 1000;
        
        // 发送大量数据
        for i in 1..=TEST_COUNT {
            let data_item = DataItem::new(i as i32, i);
            let _ = input_sender.send(StageMessage::Data(data_item));
        }
        
        // 等待处理完成
        let mut completed = 0;
        while let Ok(message) = output_receiver.recv() {
            if let StageMessage::Data(_) = message {
                completed += 1;
                if completed >= TEST_COUNT {
                    break;
                }
            }
        }
        
        let elapsed = start_time.elapsed();
        let (processed, errors) = fast_stage.get_stats();
        
        println!("处理 {} 项数据耗时: {:?}", TEST_COUNT, elapsed);
        println!("平均每项: {:?}", elapsed / TEST_COUNT as u32);
        println!("吞吐量: {:.2} 项/秒", TEST_COUNT as f64 / elapsed.as_secs_f64());
        println!("阶段统计: 处理 {} 项，错误 {} 项", processed, errors);
        
        // 停止阶段
        let _ = input_sender.send(StageMessage::Control(ControlSignal::Stop));
    }
    
    println!("\n【Pipeline模式特点】");
    println!("✓ 分阶段处理 - 将复杂任务分解为简单阶段");
    println!("✓ 并行执行 - 多个阶段同时处理不同数据");
    println!("✓ 背压控制 - 防止快速阶段压垮慢速阶段");
    println!("✓ 错误隔离 - 错误处理局限在特定阶段");
    println!("✓ 可扩展性 - 可以动态添加或移除阶段");
    println!("✓ 高吞吐量 - 提高系统整体处理能力");
} 