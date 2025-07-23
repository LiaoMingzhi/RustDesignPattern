//! 观察者模式 (Observer Pattern)
//! 
//! 定义对象间的一种一对多的依赖关系，当一个对象的状态发生改变时，所有依赖于它的对象都得到通知并被自动更新。
//! 文件路径：/d%3A/workspace/RustLearn/RustDesignPattern/src/GoFDesignPattern/BehavioralPatterns/observer.rs

use std::collections::HashMap;

// 观察者trait
trait Observer {
    fn update(&mut self, subject_name: &str, data: &ObserverData);
    fn get_name(&self) -> &str;
}

// 通知数据
#[derive(Debug, Clone)]
enum ObserverData {
    WeatherUpdate { temperature: f32, humidity: f32, pressure: f32 },
    StockUpdate { symbol: String, price: f32, change: f32 },
    NewsUpdate { headline: String, content: String },
}

// 主题trait
trait Subject {
    fn attach(&mut self, observer: Box<dyn Observer>);
    fn detach(&mut self, observer_name: &str);
    fn notify(&mut self, data: &ObserverData);
}

// 具体主题 - 天气站
struct WeatherStation {
    observers: Vec<Box<dyn Observer>>,
    temperature: f32,
    humidity: f32,
    pressure: f32,
}

impl WeatherStation {
    fn new() -> Self {
        Self {
            observers: Vec::new(),
            temperature: 0.0,
            humidity: 0.0,
            pressure: 0.0,
        }
    }

    fn set_measurements(&mut self, temperature: f32, humidity: f32, pressure: f32) {
        self.temperature = temperature;
        self.humidity = humidity;
        self.pressure = pressure;
        
        let data = ObserverData::WeatherUpdate {
            temperature,
            humidity,
            pressure,
        };
        
        self.notify(&data);
    }
}

impl Subject for WeatherStation {
    fn attach(&mut self, observer: Box<dyn Observer>) {
        println!("天气站: 添加观察者 {}", observer.get_name());
        self.observers.push(observer);
    }

    fn detach(&mut self, observer_name: &str) {
        self.observers.retain(|observer| {
            let should_retain = observer.get_name() != observer_name;
            if !should_retain {
                println!("天气站: 移除观察者 {}", observer_name);
            }
            should_retain
        });
    }

    fn notify(&mut self, data: &ObserverData) {
        println!("天气站: 通知所有观察者");
        for observer in &mut self.observers {
            observer.update("天气站", data);
        }
    }
}

// 具体观察者 - 手机应用
struct MobileApp {
    name: String,
}

impl MobileApp {
    fn new(name: String) -> Self {
        Self { name }
    }
}

impl Observer for MobileApp {
    fn update(&mut self, subject_name: &str, data: &ObserverData) {
        match data {
            ObserverData::WeatherUpdate { temperature, humidity, pressure } => {
                println!(
                    "手机应用 {}: 收到{}更新 - 温度: {:.1}°C, 湿度: {:.1}%, 气压: {:.1}hPa",
                    self.name, subject_name, temperature, humidity, pressure
                );
                
                // 手机应用可能会发送推送通知
                if *temperature > 35.0 {
                    println!("  📱 推送通知: 高温预警！");
                }
            }
            _ => {}
        }
    }

    fn get_name(&self) -> &str {
        &self.name
    }
}

// 具体观察者 - 网站显示
struct WebDisplay {
    name: String,
}

impl WebDisplay {
    fn new(name: String) -> Self {
        Self { name }
    }
}

impl Observer for WebDisplay {
    fn update(&mut self, subject_name: &str, data: &ObserverData) {
        match data {
            ObserverData::WeatherUpdate { temperature, humidity, pressure } => {
                println!(
                    "网站显示 {}: 更新{}页面",
                    self.name, subject_name
                );
                println!(
                    "  🌡️  温度: {:.1}°C | 💧 湿度: {:.1}% | 🌪️  气压: {:.1}hPa",
                    temperature, humidity, pressure
                );
            }
            _ => {}
        }
    }

    fn get_name(&self) -> &str {
        &self.name
    }
}

// 具体观察者 - 数据记录器
struct DataLogger {
    name: String,
    records: Vec<String>,
}

impl DataLogger {
    fn new(name: String) -> Self {
        Self {
            name,
            records: Vec::new(),
        }
    }

    fn show_records(&self) {
        println!("数据记录器 {} 的历史记录:", self.name);
        for (i, record) in self.records.iter().enumerate() {
            println!("  {}. {}", i + 1, record);
        }
    }
}

impl Observer for DataLogger {
    fn update(&mut self, subject_name: &str, data: &ObserverData) {
        match data {
            ObserverData::WeatherUpdate { temperature, humidity, pressure } => {
                let record = format!(
                    "来自{}: T={:.1}°C, H={:.1}%, P={:.1}hPa",
                    subject_name, temperature, humidity, pressure
                );
                self.records.push(record.clone());
                println!("数据记录器 {}: 记录数据 - {}", self.name, record);
            }
            _ => {}
        }
    }

    fn get_name(&self) -> &str {
        &self.name
    }
}

// 事件管理器 - 更高级的观察者模式实现
struct EventManager {
    listeners: HashMap<String, Vec<Box<dyn Observer>>>,
}

impl EventManager {
    fn new() -> Self {
        Self {
            listeners: HashMap::new(),
        }
    }

    fn subscribe(&mut self, event_type: &str, observer: Box<dyn Observer>) {
        println!("事件管理器: {} 订阅了事件 '{}'", observer.get_name(), event_type);
        self.listeners
            .entry(event_type.to_string())
            .or_insert_with(Vec::new)
            .push(observer);
    }

    fn unsubscribe(&mut self, event_type: &str, observer_name: &str) {
        if let Some(observers) = self.listeners.get_mut(event_type) {
            observers.retain(|observer| {
                let should_retain = observer.get_name() != observer_name;
                if !should_retain {
                    println!("事件管理器: {} 取消订阅事件 '{}'", observer_name, event_type);
                }
                should_retain
            });
        }
    }

    fn notify(&mut self, event_type: &str, data: &ObserverData) {
        if let Some(observers) = self.listeners.get_mut(event_type) {
            println!("事件管理器: 触发事件 '{}'，通知 {} 个观察者", event_type, observers.len());
            for observer in observers {
                observer.update(event_type, data);
            }
        }
    }
}

pub fn demo() {
    println!("=== 观察者模式演示 ===");

    // 1. 基本观察者模式
    println!("\n1. 基本观察者模式 - 天气站:");
    let mut weather_station = WeatherStation::new();

    // 添加观察者
    weather_station.attach(Box::new(MobileApp::new("天气助手".to_string())));
    weather_station.attach(Box::new(WebDisplay::new("天气网站".to_string())));
    weather_station.attach(Box::new(DataLogger::new("气象数据库".to_string())));

    // 更新天气数据
    weather_station.set_measurements(25.5, 65.0, 1013.25);
    
    println!();
    weather_station.set_measurements(38.2, 45.0, 1008.5);

    // 移除观察者
    println!();
    weather_station.detach("天气助手");
    weather_station.set_measurements(20.0, 70.0, 1015.0);

    // 2. 事件管理器模式
    println!("\n\n2. 事件管理器模式:");
    let mut event_manager = EventManager::new();

    // 订阅不同事件
    event_manager.subscribe("weather", Box::new(MobileApp::new("天气APP".to_string())));
    event_manager.subscribe("weather", Box::new(DataLogger::new("天气记录器".to_string())));
    event_manager.subscribe("stock", Box::new(MobileApp::new("股票APP".to_string())));

    // 触发天气事件
    let weather_data = ObserverData::WeatherUpdate {
        temperature: 28.0,
        humidity: 60.0,
        pressure: 1012.0,
    };
    event_manager.notify("weather", &weather_data);

    // 触发股票事件
    println!();
    let stock_data = ObserverData::StockUpdate {
        symbol: "AAPL".to_string(),
        price: 150.25,
        change: 2.5,
    };
    event_manager.notify("stock", &stock_data);

    println!("\n观察者模式的优点:");
    println!("1. 建立了抽象的耦合，主题只知道观察者的抽象接口");
    println!("2. 支持广播通信，可以同时通知多个观察者");
    println!("3. 支持动态增加和删除观察者");
    println!("4. 符合开闭原则，可以独立扩展主题和观察者");
} 