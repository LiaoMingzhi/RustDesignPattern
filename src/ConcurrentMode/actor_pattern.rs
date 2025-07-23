/*
 * 文件路径: /d%3A/workspace/RustLearn/RustDesignPattern/src/ConcurrentMode/actor_pattern.rs
 * 
 * Actor模式 (Actor Pattern)
 * 
 * Actor模式是一种并发计算的数学模型，其中"actors"是并发计算的通用原语。
 * 每个Actor都有自己的状态和邮箱，通过消息传递进行通信，避免了共享状态的问题。
 * 
 * 主要特点：
 * 1. 封装性 - 每个Actor封装自己的状态
 * 2. 消息传递 - Actor之间只能通过消息通信  
 * 3. 无共享状态 - 避免了数据竞争
 * 4. 监督策略 - 处理Actor故障和重启
 * 5. 位置透明 - Actor可以在不同位置运行
 */

use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};
use std::collections::HashMap;
use std::fmt;

// =================
// 核心Actor特质和消息
// =================

/// Actor消息特质
pub trait Message: Send + 'static {}

/// Actor特质
pub trait Actor: Send + 'static {
    type Message: Message;
    
    /// 处理接收到的消息
    fn receive(&mut self, message: Self::Message, context: &mut ActorContext<Self::Message>);
    
    /// Actor启动时调用
    fn pre_start(&mut self, _context: &mut ActorContext<Self::Message>) {}
    
    /// Actor停止前调用
    fn pre_stop(&mut self, _context: &mut ActorContext<Self::Message>) {}
    
    /// 处理异常
    fn post_restart(&mut self, _context: &mut ActorContext<Self::Message>) {}
}

/// Actor引用，用于向Actor发送消息
#[derive(Clone, Debug)]
pub struct ActorRef<M: Message> {
    sender: Sender<M>,
    name: String,
}

impl<M: Message> ActorRef<M> {
    pub fn new(sender: Sender<M>, name: String) -> Self {
        Self { sender, name }
    }
    
    /// 发送消息给Actor
    pub fn tell(&self, message: M) -> Result<(), ActorError> {
        self.sender.send(message)
            .map_err(|_| ActorError::MailboxClosed(self.name.clone()))
    }
    
    pub fn name(&self) -> &str {
        &self.name
    }
}

/// Actor上下文，提供Actor运行时环境
pub struct ActorContext<M: Message> {
    self_ref: Option<ActorRef<M>>,
    children: HashMap<String, Box<dyn std::any::Any + Send>>,
    should_stop: bool,
}

impl<M: Message> ActorContext<M> {
    pub fn new() -> Self {
        Self {
            self_ref: None,
            children: HashMap::new(),
            should_stop: false,
        }
    }
    
    /// 获取自身引用
    pub fn self_ref(&self) -> Option<&ActorRef<M>> {
        self.self_ref.as_ref()
    }
    
    /// 停止Actor
    pub fn stop(&mut self) {
        self.should_stop = true;
    }
    
    /// 检查是否应该停止
    pub fn should_stop(&self) -> bool {
        self.should_stop
    }
}

// =================
// Actor系统和错误处理
// =================

/// Actor错误类型
#[derive(Debug)]
pub enum ActorError {
    MailboxClosed(String),
    ActorNotFound(String),
    SystemShutdown,
}

impl fmt::Display for ActorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ActorError::MailboxClosed(name) => write!(f, "Actor {} 的邮箱已关闭", name),
            ActorError::ActorNotFound(name) => write!(f, "未找到Actor: {}", name),
            ActorError::SystemShutdown => write!(f, "Actor系统正在关闭"),
        }
    }
}

/// Actor系统，管理所有Actor
pub struct ActorSystem {
    actors: Arc<Mutex<HashMap<String, JoinHandle<()>>>>,
    shutdown: Arc<Mutex<bool>>,
}

impl ActorSystem {
    pub fn new() -> Self {
        Self {
            actors: Arc::new(Mutex::new(HashMap::new())),
            shutdown: Arc::new(Mutex::new(false)),
        }
    }
    
    /// 启动一个Actor
    pub fn spawn<A>(&self, mut actor: A, name: String) -> Result<ActorRef<A::Message>, ActorError>
    where
        A: Actor + 'static,
    {
        let (sender, receiver) = mpsc::channel();
        let actor_ref = ActorRef::new(sender.clone(), name.clone());
        let mut context = ActorContext::new();
        context.self_ref = Some(ActorRef::new(sender.clone(), name.clone()));
        
        let handle = thread::spawn(move || {
            actor.pre_start(&mut context);
            
            while let Ok(message) = receiver.recv() {
                actor.receive(message, &mut context);
                
                if context.should_stop() {
                    break;
                }
            }
            
            actor.pre_stop(&mut context);
        });
        
        let mut actors = self.actors.lock().unwrap();
        actors.insert(name, handle);
        
        Ok(actor_ref)
    }
    
    /// 关闭Actor系统
    pub fn shutdown(&self) {
        *self.shutdown.lock().unwrap() = true;
        
        // 等待所有Actor完成
        let mut actors = self.actors.lock().unwrap();
        for (name, handle) in actors.drain() {
            println!("等待Actor {} 停止...", name);
            let _ = handle.join();
        }
    }
}

// =================
// 示例Actor实现
// =================

/// 计数器消息
#[derive(Debug)]
pub enum CounterMessage {
    Increment,
    Decrement,
    GetCount(Sender<i32>),
    Reset,
    Stop,
}

impl Message for CounterMessage {}

/// 计数器Actor
pub struct CounterActor {
    count: i32,
    name: String,
}

impl CounterActor {
    pub fn new(name: String) -> Self {
        Self { count: 0, name }
    }
}

impl Actor for CounterActor {
    type Message = CounterMessage;
    
    fn receive(&mut self, message: Self::Message, context: &mut ActorContext<Self::Message>) {
        match message {
            CounterMessage::Increment => {
                self.count += 1;
                println!("[{}] 计数器增加，当前值: {}", self.name, self.count);
            }
            CounterMessage::Decrement => {
                self.count -= 1;
                println!("[{}] 计数器减少，当前值: {}", self.name, self.count);
            }
            CounterMessage::GetCount(sender) => {
                let _ = sender.send(self.count);
                println!("[{}] 返回当前计数: {}", self.name, self.count);
            }
            CounterMessage::Reset => {
                self.count = 0;
                println!("[{}] 计数器重置", self.name);
            }
            CounterMessage::Stop => {
                println!("[{}] 收到停止消息", self.name);
                context.stop();
            }
        }
    }
    
    fn pre_start(&mut self, _context: &mut ActorContext<Self::Message>) {
        println!("[{}] Counter Actor 启动", self.name);
    }
    
    fn pre_stop(&mut self, _context: &mut ActorContext<Self::Message>) {
        println!("[{}] Counter Actor 停止，最终计数: {}", self.name, self.count);
    }
}

/// 银行账户消息
#[derive(Debug, Clone)]
pub enum BankMessage {
    Deposit(f64),
    Withdraw(f64, Sender<Result<f64, String>>),
    GetBalance(Sender<f64>),
    Transfer(f64, ActorRef<BankMessage>),
    Stop,
}

impl Message for BankMessage {}

/// 银行账户Actor
pub struct BankAccountActor {
    balance: f64,
    account_id: String,
}

impl BankAccountActor {
    pub fn new(account_id: String, initial_balance: f64) -> Self {
        Self {
            balance: initial_balance,
            account_id,
        }
    }
}

impl Actor for BankAccountActor {
    type Message = BankMessage;
    
    fn receive(&mut self, message: Self::Message, context: &mut ActorContext<Self::Message>) {
        match message {
            BankMessage::Deposit(amount) => {
                if amount > 0.0 {
                    self.balance += amount;
                    println!("[{}] 存款 {:.2}，余额: {:.2}", self.account_id, amount, self.balance);
                }
            }
            BankMessage::Withdraw(amount, sender) => {
                if amount > 0.0 && amount <= self.balance {
                    self.balance -= amount;
                    let _ = sender.send(Ok(self.balance));
                    println!("[{}] 取款 {:.2}，余额: {:.2}", self.account_id, amount, self.balance);
                } else {
                    let _ = sender.send(Err("余额不足或金额无效".to_string()));
                    println!("[{}] 取款失败，余额: {:.2}", self.account_id, self.balance);
                }
            }
            BankMessage::GetBalance(sender) => {
                let _ = sender.send(self.balance);
            }
            BankMessage::Transfer(amount, target_account) => {
                if amount > 0.0 && amount <= self.balance {
                    self.balance -= amount;
                    let _ = target_account.tell(BankMessage::Deposit(amount));
                    println!("[{}] 转账 {:.2} 到 {}，余额: {:.2}", 
                             self.account_id, amount, target_account.name(), self.balance);
                } else {
                    println!("[{}] 转账失败，余额不足", self.account_id);
                }
            }
            BankMessage::Stop => {
                context.stop();
            }
        }
    }
    
    fn pre_start(&mut self, _context: &mut ActorContext<Self::Message>) {
        println!("[{}] 银行账户Actor启动，初始余额: {:.2}", self.account_id, self.balance);
    }
    
    fn pre_stop(&mut self, _context: &mut ActorContext<Self::Message>) {
        println!("[{}] 银行账户Actor停止，最终余额: {:.2}", self.account_id, self.balance);
    }
}

/// 消息路由器Actor
pub struct RouterActor {
    workers: Vec<ActorRef<CounterMessage>>,
    current_index: usize,
}

impl RouterActor {
    pub fn new(workers: Vec<ActorRef<CounterMessage>>) -> Self {
        Self {
            workers,
            current_index: 0,
        }
    }
}

/// 路由器消息
#[derive(Debug)]
pub enum RouterMessage {
    Route(CounterMessage),
    Stop,
}

impl Message for RouterMessage {}

impl Actor for RouterActor {
    type Message = RouterMessage;
    
    fn receive(&mut self, message: Self::Message, context: &mut ActorContext<Self::Message>) {
        match message {
            RouterMessage::Route(counter_msg) => {
                if !self.workers.is_empty() {
                    let worker = &self.workers[self.current_index];
                    let _ = worker.tell(counter_msg);
                    self.current_index = (self.current_index + 1) % self.workers.len();
                }
            }
            RouterMessage::Stop => {
                // 停止所有工作者
                for worker in &self.workers {
                    let _ = worker.tell(CounterMessage::Stop);
                }
                context.stop();
            }
        }
    }
    
    fn pre_start(&mut self, _context: &mut ActorContext<Self::Message>) {
        println!("路由器Actor启动，管理 {} 个工作者", self.workers.len());
    }
}

// =================
// 演示函数
// =================

/// Actor模式演示
pub fn demo_actor_pattern() {
    println!("=== Actor模式演示 ===\n");
    
    let system = ActorSystem::new();
    
    // 1. 基本Actor演示
    println!("1. 基本Counter Actor演示:");
    let counter = CounterActor::new("主计数器".to_string());
    let counter_ref = system.spawn(counter, "counter1".to_string()).unwrap();
    
    // 发送消息
    counter_ref.tell(CounterMessage::Increment).unwrap();
    counter_ref.tell(CounterMessage::Increment).unwrap();
    counter_ref.tell(CounterMessage::Decrement).unwrap();
    
    // 查询计数
    let (sender, receiver) = mpsc::channel();
    counter_ref.tell(CounterMessage::GetCount(sender)).unwrap();
    if let Ok(count) = receiver.recv_timeout(Duration::from_millis(100)) {
        println!("最终计数: {}", count);
    }
    
    thread::sleep(Duration::from_millis(100));
    println!();
    
    // 2. 银行账户Actor演示
    println!("2. 银行账户Actor演示:");
    let account1 = BankAccountActor::new("账户001".to_string(), 1000.0);
    let account2 = BankAccountActor::new("账户002".to_string(), 500.0);
    
    let account1_ref = system.spawn(account1, "account1".to_string()).unwrap();
    let account2_ref = system.spawn(account2, "account2".to_string()).unwrap();
    
    // 存款和取款操作
    account1_ref.tell(BankMessage::Deposit(200.0)).unwrap();
    
    let (sender, receiver) = mpsc::channel();
    account1_ref.tell(BankMessage::Withdraw(150.0, sender)).unwrap();
    if let Ok(result) = receiver.recv_timeout(Duration::from_millis(100)) {
        match result {
            Ok(balance) => println!("取款成功，剩余余额: {:.2}", balance),
            Err(err) => println!("取款失败: {}", err),
        }
    }
    
    // 转账操作
    account1_ref.tell(BankMessage::Transfer(300.0, account2_ref.clone())).unwrap();
    
    thread::sleep(Duration::from_millis(200));
    println!();
    
    // 3. 路由器Actor演示
    println!("3. 路由器Actor演示:");
    let mut workers = Vec::new();
    for i in 1..=3 {
        let worker = CounterActor::new(format!("工作者{}", i));
        let worker_ref = system.spawn(worker, format!("worker{}", i)).unwrap();
        workers.push(worker_ref);
    }
    
    let router = RouterActor::new(workers);
    let router_ref = system.spawn(router, "router".to_string()).unwrap();
    
    // 通过路由器发送消息
    for i in 0..6 {
        router_ref.tell(RouterMessage::Route(CounterMessage::Increment)).unwrap();
        thread::sleep(Duration::from_millis(50));
    }
    
    thread::sleep(Duration::from_millis(200));
    
    // 4. 性能测试
    println!("\n4. Actor性能测试:");
    let start_time = Instant::now();
    let test_counter = CounterActor::new("性能测试".to_string());
    let test_ref = system.spawn(test_counter, "perf_test".to_string()).unwrap();
    
    const MESSAGE_COUNT: i32 = 10000;
    for _ in 0..MESSAGE_COUNT {
        test_ref.tell(CounterMessage::Increment).unwrap();
    }
    
    // 等待处理完成
    let (sender, receiver) = mpsc::channel();
    test_ref.tell(CounterMessage::GetCount(sender)).unwrap();
    if let Ok(final_count) = receiver.recv_timeout(Duration::from_secs(2)) {
        let elapsed = start_time.elapsed();
        println!("处理 {} 条消息耗时: {:?}", final_count, elapsed);
        println!("平均每秒处理: {:.0} 条消息", final_count as f64 / elapsed.as_secs_f64());
    }
    
    // 停止所有Actor
    println!("\n5. 停止所有Actor:");
    counter_ref.tell(CounterMessage::Stop).unwrap();
    account1_ref.tell(BankMessage::Stop).unwrap();
    account2_ref.tell(BankMessage::Stop).unwrap();
    router_ref.tell(RouterMessage::Stop).unwrap();
    test_ref.tell(CounterMessage::Stop).unwrap();
    
    thread::sleep(Duration::from_millis(200));
    
    // 关闭系统
    system.shutdown();
    
    println!("\n【Actor模式特点】");
    println!("✓ 消息传递 - Actor之间通过异步消息通信");
    println!("✓ 状态封装 - 每个Actor封装自己的状态，避免共享");
    println!("✓ 并发安全 - 无锁并发，避免数据竞争");
    println!("✓ 容错处理 - 支持Actor监督和重启策略");
    println!("✓ 位置透明 - Actor可以在不同位置运行");
    println!("✓ 背压处理 - 通过邮箱大小控制消息流量");
} 