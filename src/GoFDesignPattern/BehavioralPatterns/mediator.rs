//! 中介者模式 (Mediator Pattern)
//! 
//! 用一个中介对象来封装一系列的对象交互。中介者使各对象不需要显式地相互引用，从而使其耦合松散，而且可以独立地改变它们之间的交互。
//! 文件路径：/d%3A/workspace/RustLearn/RustDesignPattern/src/GoFDesignPattern/BehavioralPatterns/mediator.rs

use std::collections::HashMap;

// 中介者接口
trait Mediator {
    fn notify(&mut self, sender: &str, event: &str, data: Option<String>);
}

// 组件基类
trait Component {
    fn set_mediator(&mut self, mediator: *mut dyn Mediator);
    fn get_name(&self) -> &str;
}

// 具体中介者 - 聊天室
struct ChatRoom {
    users: Vec<String>,
}

impl ChatRoom {
    fn new() -> Self {
        Self {
            users: Vec::new(),
        }
    }

    fn add_user(&mut self, username: String) {
        self.users.push(username.clone());
        println!("聊天室: {} 加入了聊天室", username);
    }

    fn remove_user(&mut self, username: &str) {
        self.users.retain(|user| user != username);
        println!("聊天室: {} 离开了聊天室", username);
    }
}

impl Mediator for ChatRoom {
    fn notify(&mut self, sender: &str, event: &str, data: Option<String>) {
        match event {
            "send_message" => {
                if let Some(message) = data {
                    println!("聊天室广播: {} 说: {}", sender, message);
                    for user in &self.users {
                        if user != sender {
                            println!("  -> {} 收到消息", user);
                        }
                    }
                }
            }
            "private_message" => {
                if let Some(content) = data {
                    let parts: Vec<&str> = content.split('|').collect();
                    if parts.len() == 2 {
                        let target = parts[0];
                        let message = parts[1];
                        if self.users.contains(&target.to_string()) {
                            println!("私聊: {} -> {}: {}", sender, target, message);
                        } else {
                            println!("错误: 用户 {} 不在聊天室", target);
                        }
                    }
                }
            }
            "user_typing" => {
                println!("状态更新: {} 正在输入...", sender);
            }
            _ => {}
        }
    }
}

// 具体组件 - 用户
struct User {
    name: String,
    mediator: Option<*mut dyn Mediator>,
}

impl User {
    fn new(name: String) -> Self {
        Self {
            name,
            mediator: None,
        }
    }

    fn send_message(&mut self, message: String) {
        if let Some(mediator) = self.mediator {
            unsafe {
                (*mediator).notify(&self.name, "send_message", Some(message));
            }
        }
    }

    fn send_private_message(&mut self, target: String, message: String) {
        if let Some(mediator) = self.mediator {
            let data = format!("{}|{}", target, message);
            unsafe {
                (*mediator).notify(&self.name, "private_message", Some(data));
            }
        }
    }

    fn start_typing(&mut self) {
        if let Some(mediator) = self.mediator {
            unsafe {
                (*mediator).notify(&self.name, "user_typing", None);
            }
        }
    }
}

impl Component for User {
    fn set_mediator(&mut self, mediator: *mut dyn Mediator) {
        self.mediator = Some(mediator);
    }

    fn get_name(&self) -> &str {
        &self.name
    }
}

// 另一个例子 - 智能家居系统
struct SmartHomeHub {
    devices: HashMap<String, String>, // 设备名 -> 状态
}

impl SmartHomeHub {
    fn new() -> Self {
        Self {
            devices: HashMap::new(),
        }
    }

    fn register_device(&mut self, device_name: String) {
        self.devices.insert(device_name.clone(), "offline".to_string());
        println!("智能家居中心: 注册设备 {}", device_name);
    }
}

impl Mediator for SmartHomeHub {
    fn notify(&mut self, sender: &str, event: &str, data: Option<String>) {
        match event {
            "motion_detected" => {
                println!("智能家居中心: {} 检测到运动", sender);
                // 自动开灯
                println!("  -> 自动开启客厅灯光");
                println!("  -> 启动安全摄像头录制");
            }
            "door_opened" => {
                println!("智能家居中心: {} 门被打开", sender);
                // 关闭安全系统
                println!("  -> 关闭安全系统");
                println!("  -> 开启欢迎灯光");
            }
            "temperature_change" => {
                if let Some(temp) = data {
                    println!("智能家居中心: {} 温度变化为 {}°C", sender, temp);
                    if let Ok(temperature) = temp.parse::<i32>() {
                        if temperature > 25 {
                            println!("  -> 启动空调制冷");
                        } else if temperature < 18 {
                            println!("  -> 启动暖气");
                        }
                    }
                }
            }
            "device_status" => {
                if let Some(status) = data {
                    self.devices.insert(sender.to_string(), status.clone());
                    println!("智能家居中心: {} 状态更新为 {}", sender, status);
                }
            }
            _ => {}
        }
    }
}

// 智能设备基类
struct SmartDevice {
    name: String,
    mediator: Option<*mut dyn Mediator>,
}

impl SmartDevice {
    fn new(name: String) -> Self {
        Self {
            name,
            mediator: None,
        }
    }

    fn send_event(&mut self, event: &str, data: Option<String>) {
        if let Some(mediator) = self.mediator {
            unsafe {
                (*mediator).notify(&self.name, event, data);
            }
        }
    }
}

impl Component for SmartDevice {
    fn set_mediator(&mut self, mediator: *mut dyn Mediator) {
        self.mediator = Some(mediator);
    }

    fn get_name(&self) -> &str {
        &self.name
    }
}

pub fn demo() {
    println!("=== 中介者模式演示 ===");

    // 1. 聊天室示例
    println!("\n1. 聊天室中介者:");
    let mut chat_room = ChatRoom::new();
    
    let mut alice = User::new("Alice".to_string());
    let mut bob = User::new("Bob".to_string());
    let mut charlie = User::new("Charlie".to_string());
    
    // 设置中介者
    alice.set_mediator(&mut chat_room as *mut dyn Mediator);
    bob.set_mediator(&mut chat_room as *mut dyn Mediator);
    charlie.set_mediator(&mut chat_room as *mut dyn Mediator);
    
    // 添加用户到聊天室
    chat_room.add_user("Alice".to_string());
    chat_room.add_user("Bob".to_string());
    chat_room.add_user("Charlie".to_string());
    
    // 用户交互
    println!();
    alice.send_message("大家好！".to_string());
    bob.start_typing();
    bob.send_message("你好Alice！".to_string());
    charlie.send_private_message("Alice".to_string(), "私聊消息".to_string());

    // 2. 智能家居示例
    println!("\n\n2. 智能家居中介者:");
    let mut smart_home = SmartHomeHub::new();
    
    let mut motion_sensor = SmartDevice::new("客厅运动传感器".to_string());
    let mut door_sensor = SmartDevice::new("前门传感器".to_string());
    let mut temperature_sensor = SmartDevice::new("温度传感器".to_string());
    
    // 设置中介者
    motion_sensor.set_mediator(&mut smart_home as *mut dyn Mediator);
    door_sensor.set_mediator(&mut smart_home as *mut dyn Mediator);
    temperature_sensor.set_mediator(&mut smart_home as *mut dyn Mediator);
    
    // 注册设备
    smart_home.register_device("客厅运动传感器".to_string());
    smart_home.register_device("前门传感器".to_string());
    smart_home.register_device("温度传感器".to_string());
    
    // 模拟设备事件
    println!();
    motion_sensor.send_event("motion_detected", None);
    door_sensor.send_event("door_opened", None);
    temperature_sensor.send_event("temperature_change", Some("28".to_string()));
    
    motion_sensor.send_event("device_status", Some("online".to_string()));

    println!("\n中介者模式的优点:");
    println!("1. 减少了类间的依赖，将多对多的依赖转化为一对多");
    println!("2. 提高了系统的灵活性，使得系统易于维护和扩展");
    println!("3. 简化了对象之间的交互");
    println!("4. 将控制逻辑集中化");
} 