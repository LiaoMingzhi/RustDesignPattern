//! 模板方法模式 (Template Method Pattern)
//! 
//! 定义一个操作中的算法的骨架，而将一些步骤延迟到子类中。
//! 文件路径：/d%3A/workspace/RustLearn/RustDesignPattern/src/GoFDesignPattern/BehavioralPatterns/template_method.rs

// 模板方法trait
trait DataProcessor {
    // 模板方法 - 定义算法骨架
    fn process(&self) {
        println!("=== 开始数据处理 ===");
        
        self.read_data();
        self.validate_data();
        self.transform_data();
        self.save_data();
        
        // 可选的钩子方法
        if self.should_send_notification() {
            self.send_notification();
        }
        
        println!("=== 数据处理完成 ===\n");
    }

    // 抽象方法 - 子类必须实现
    fn read_data(&self);
    fn validate_data(&self);
    fn transform_data(&self);
    fn save_data(&self);

    // 钩子方法 - 子类可以选择重写
    fn should_send_notification(&self) -> bool {
        false
    }

    fn send_notification(&self) {
        println!("发送处理完成通知");
    }
}

// 具体实现 - CSV数据处理器
struct CsvDataProcessor {
    filename: String,
}

impl CsvDataProcessor {
    fn new(filename: String) -> Self {
        Self { filename }
    }
}

impl DataProcessor for CsvDataProcessor {
    fn read_data(&self) {
        println!("从CSV文件读取数据: {}", self.filename);
    }

    fn validate_data(&self) {
        println!("验证CSV数据格式和完整性");
    }

    fn transform_data(&self) {
        println!("转换CSV数据格式");
    }

    fn save_data(&self) {
        println!("保存处理后的CSV数据到数据库");
    }

    fn should_send_notification(&self) -> bool {
        true // CSV处理完成后需要发送通知
    }
}

// 具体实现 - JSON数据处理器
struct JsonDataProcessor {
    api_endpoint: String,
}

impl JsonDataProcessor {
    fn new(api_endpoint: String) -> Self {
        Self { api_endpoint }
    }
}

impl DataProcessor for JsonDataProcessor {
    fn read_data(&self) {
        println!("从API读取JSON数据: {}", self.api_endpoint);
    }

    fn validate_data(&self) {
        println!("验证JSON数据结构");
    }

    fn transform_data(&self) {
        println!("转换JSON数据到标准格式");
    }

    fn save_data(&self) {
        println!("保存JSON数据到缓存");
    }

    fn send_notification(&self) {
        println!("发送JSON数据处理完成的API回调");
    }
}

// 具体实现 - XML数据处理器
struct XmlDataProcessor {
    source: String,
}

impl XmlDataProcessor {
    fn new(source: String) -> Self {
        Self { source }
    }
}

impl DataProcessor for XmlDataProcessor {
    fn read_data(&self) {
        println!("从XML源读取数据: {}", self.source);
    }

    fn validate_data(&self) {
        println!("使用XSD验证XML数据");
    }

    fn transform_data(&self) {
        println!("使用XSLT转换XML数据");
    }

    fn save_data(&self) {
        println!("保存XML数据到文件系统");
    }
}

// 另一个例子 - 饮料制作模板
trait BeverageMaker {
    // 模板方法
    fn prepare_beverage(&self) {
        println!("=== 制作 {} ===", self.get_name());
        
        self.boil_water();
        self.brew();
        self.pour_in_cup();
        
        if self.wants_condiments() {
            self.add_condiments();
        }
        
        println!("=== {} 制作完成 ===\n", self.get_name());
    }

    // 通用步骤
    fn boil_water(&self) {
        println!("烧开水");
    }

    fn pour_in_cup(&self) {
        println!("倒入杯中");
    }

    // 抽象方法
    fn brew(&self);
    fn add_condiments(&self);
    fn get_name(&self) -> &str;

    // 钩子方法
    fn wants_condiments(&self) -> bool {
        true
    }
}

// 咖啡制作器
struct CoffeeMaker;

impl BeverageMaker for CoffeeMaker {
    fn brew(&self) {
        println!("用沸水冲泡咖啡");
    }

    fn add_condiments(&self) {
        println!("加糖和牛奶");
    }

    fn get_name(&self) -> &str {
        "咖啡"
    }
}

// 茶制作器
struct TeaMaker {
    tea_type: String,
}

impl TeaMaker {
    fn new(tea_type: String) -> Self {
        Self { tea_type }
    }
}

impl BeverageMaker for TeaMaker {
    fn brew(&self) {
        println!("浸泡{}茶叶", self.tea_type);
    }

    fn add_condiments(&self) {
        println!("加柠檬");
    }

    fn get_name(&self) -> &str {
        &self.tea_type
    }
}

// 纯茶制作器（不加调料）
struct PlainTeaMaker;

impl BeverageMaker for PlainTeaMaker {
    fn brew(&self) {
        println!("浸泡绿茶茶叶");
    }

    fn add_condiments(&self) {
        println!("不添加任何调料");
    }

    fn get_name(&self) -> &str {
        "纯绿茶"
    }

    fn wants_condiments(&self) -> bool {
        false // 不需要调料
    }
}

pub fn demo() {
    println!("=== 模板方法模式演示 ===");

    // 1. 数据处理器示例
    println!("\n1. 数据处理器模板:");
    
    let csv_processor = CsvDataProcessor::new("data.csv".to_string());
    csv_processor.process();

    let json_processor = JsonDataProcessor::new("https://api.example.com/data".to_string());
    json_processor.process();

    let xml_processor = XmlDataProcessor::new("config.xml".to_string());
    xml_processor.process();

    // 2. 饮料制作示例
    println!("2. 饮料制作模板:");
    
    let coffee_maker = CoffeeMaker;
    coffee_maker.prepare_beverage();

    let tea_maker = TeaMaker::new("红茶".to_string());
    tea_maker.prepare_beverage();

    let plain_tea_maker = PlainTeaMaker;
    plain_tea_maker.prepare_beverage();

    println!("模板方法模式的优点:");
    println!("1. 提高代码复用性，将相同部分的代码放在父类中");
    println!("2. 提高了扩展性，将不同的代码放入不同的子类中");
    println!("3. 符合开闭原则，增加新的实现只需要增加子类");
    println!("4. 符合单一职责原则，每个子类只负责自己的算法实现");
} 