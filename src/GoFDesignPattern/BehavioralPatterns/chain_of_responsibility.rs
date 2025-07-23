//! 责任链模式 (Chain of Responsibility Pattern)
//! 
//! 避免请求发送者与接收者耦合在一起，让多个对象都有可能接收请求，将这些对象连接成一条链，并且沿着这条链传递请求，直到有对象处理它为止。
//! 文件路径：/d%3A/workspace/RustLearn/RustDesignPattern/src/GoFDesignPattern/BehavioralPatterns/chain_of_responsibility.rs

// 请求类型
#[derive(Debug, Clone)]
enum RequestType {
    Leave,      // 请假
    Expense,    // 报销
    Purchase,   // 采购
}

// 请求
#[derive(Debug, Clone)]
struct Request {
    request_type: RequestType,
    amount: f64,
    description: String,
}

impl Request {
    fn new(request_type: RequestType, amount: f64, description: String) -> Self {
        Self {
            request_type,
            amount,
            description,
        }
    }
}

// 处理者接口
trait Handler {
    fn set_next(&mut self, handler: Box<dyn Handler>);
    fn handle(&self, request: &Request) -> bool;
}

// 抽象处理者
struct BaseHandler {
    next_handler: Option<Box<dyn Handler>>,
}

impl BaseHandler {
    fn new() -> Self {
        Self { next_handler: None }
    }
    
    fn forward_to_next(&self, request: &Request) -> bool {
        if let Some(ref next) = self.next_handler {
            next.handle(request)
        } else {
            println!("没有处理者能够处理此请求: {:?}", request);
            false
        }
    }
}

// 具体处理者 - 主管
struct Supervisor {
    base: BaseHandler,
}

impl Supervisor {
    fn new() -> Self {
        Self {
            base: BaseHandler::new(),
        }
    }
}

impl Handler for Supervisor {
    fn set_next(&mut self, handler: Box<dyn Handler>) {
        self.base.next_handler = Some(handler);
    }

    fn handle(&self, request: &Request) -> bool {
        match request.request_type {
            RequestType::Leave if request.amount <= 3.0 => {
                println!("主管批准了{}天的请假申请: {}", request.amount, request.description);
                true
            }
            RequestType::Expense if request.amount <= 1000.0 => {
                println!("主管批准了¥{}的报销申请: {}", request.amount, request.description);
                true
            }
            _ => {
                println!("主管无权处理此请求，转交给上级");
                self.base.forward_to_next(request)
            }
        }
    }
}

// 具体处理者 - 经理
struct Manager {
    base: BaseHandler,
}

impl Manager {
    fn new() -> Self {
        Self {
            base: BaseHandler::new(),
        }
    }
}

impl Handler for Manager {
    fn set_next(&mut self, handler: Box<dyn Handler>) {
        self.base.next_handler = Some(handler);
    }

    fn handle(&self, request: &Request) -> bool {
        match request.request_type {
            RequestType::Leave if request.amount <= 7.0 => {
                println!("经理批准了{}天的请假申请: {}", request.amount, request.description);
                true
            }
            RequestType::Expense if request.amount <= 5000.0 => {
                println!("经理批准了¥{}的报销申请: {}", request.amount, request.description);
                true
            }
            RequestType::Purchase if request.amount <= 10000.0 => {
                println!("经理批准了¥{}的采购申请: {}", request.amount, request.description);
                true
            }
            _ => {
                println!("经理无权处理此请求，转交给上级");
                self.base.forward_to_next(request)
            }
        }
    }
}

// 具体处理者 - 总监
struct Director {
    base: BaseHandler,
}

impl Director {
    fn new() -> Self {
        Self {
            base: BaseHandler::new(),
        }
    }
}

impl Handler for Director {
    fn set_next(&mut self, handler: Box<dyn Handler>) {
        self.base.next_handler = Some(handler);
    }

    fn handle(&self, request: &Request) -> bool {
        match request.request_type {
            RequestType::Leave => {
                println!("总监批准了{}天的请假申请: {}", request.amount, request.description);
                true
            }
            RequestType::Expense if request.amount <= 20000.0 => {
                println!("总监批准了¥{}的报销申请: {}", request.amount, request.description);
                true
            }
            RequestType::Purchase if request.amount <= 50000.0 => {
                println!("总监批准了¥{}的采购申请: {}", request.amount, request.description);
                true
            }
            _ => {
                println!("总监无权处理此请求，需要董事会决定");
                self.base.forward_to_next(request)
            }
        }
    }
}

// 责任链构建器
struct ChainBuilder;

impl ChainBuilder {
    fn build_approval_chain() -> Box<dyn Handler> {
        let mut supervisor = Box::new(Supervisor::new());
        let mut manager = Box::new(Manager::new());
        let director = Box::new(Director::new());

        manager.set_next(director);
        supervisor.set_next(manager);
        
        supervisor
    }
}

pub fn demo() {
    println!("=== 责任链模式演示 ===");

    let chain = ChainBuilder::build_approval_chain();

    let requests = vec![
        Request::new(RequestType::Leave, 2.0, "病假".to_string()),
        Request::new(RequestType::Expense, 800.0, "差旅费".to_string()),
        Request::new(RequestType::Leave, 5.0, "年假".to_string()),
        Request::new(RequestType::Expense, 3000.0, "设备采购".to_string()),
        Request::new(RequestType::Purchase, 8000.0, "办公用品".to_string()),
        Request::new(RequestType::Purchase, 60000.0, "服务器采购".to_string()),
    ];

    for (i, request) in requests.iter().enumerate() {
        println!("\n请求 {}: {:?}", i + 1, request);
        let approved = chain.handle(request);
        println!("结果: {}", if approved { "已批准" } else { "被拒绝" });
    }
} 