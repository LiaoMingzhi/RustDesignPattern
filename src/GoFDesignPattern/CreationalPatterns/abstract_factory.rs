//! 抽象工厂模式 (Abstract Factory Pattern)
//! 
//! 提供一个创建一系列相关或相互依赖对象的接口，而无需指定它们具体的类。
//! 文件路径：/d%3A/workspace/RustLearn/RustDesignPattern/src/GoFDesignPattern/CreationalPatterns/abstract_factory.rs

use std::fmt::Debug;

// 抽象产品 - 按钮
trait Button: Debug {
    fn render(&self);
}

// 抽象产品 - 复选框
trait Checkbox: Debug {
    fn render(&self);
}

// 具体产品 - Windows按钮
#[derive(Debug)]
struct WindowsButton;

impl Button for WindowsButton {
    fn render(&self) {
        println!("渲染Windows风格的按钮");
    }
}

// 具体产品 - Windows复选框
#[derive(Debug)]
struct WindowsCheckbox;

impl Checkbox for WindowsCheckbox {
    fn render(&self) {
        println!("渲染Windows风格的复选框");
    }
}

// 具体产品 - MacOS按钮
#[derive(Debug)]
struct MacButton;

impl Button for MacButton {
    fn render(&self) {
        println!("渲染MacOS风格的按钮");
    }
}

// 具体产品 - MacOS复选框
#[derive(Debug)]
struct MacCheckbox;

impl Checkbox for MacCheckbox {
    fn render(&self) {
        println!("渲染MacOS风格的复选框");
    }
}

// 抽象工厂
trait GUIFactory {
    fn create_button(&self) -> Box<dyn Button>;
    fn create_checkbox(&self) -> Box<dyn Checkbox>;
}

// 具体工厂 - Windows工厂
struct WindowsFactory;

impl GUIFactory for WindowsFactory {
    fn create_button(&self) -> Box<dyn Button> {
        Box::new(WindowsButton)
    }

    fn create_checkbox(&self) -> Box<dyn Checkbox> {
        Box::new(WindowsCheckbox)
    }
}

// 具体工厂 - MacOS工厂
struct MacFactory;

impl GUIFactory for MacFactory {
    fn create_button(&self) -> Box<dyn Button> {
        Box::new(MacButton)
    }

    fn create_checkbox(&self) -> Box<dyn Checkbox> {
        Box::new(MacCheckbox)
    }
}

// 客户端代码
struct Application {
    button: Box<dyn Button>,
    checkbox: Box<dyn Checkbox>,
}

impl Application {
    fn new(factory: &dyn GUIFactory) -> Self {
        Self {
            button: factory.create_button(),
            checkbox: factory.create_checkbox(),
        }
    }

    fn render(&self) {
        self.button.render();
        self.checkbox.render();
    }
}

// 工厂选择器
fn get_factory(os_type: &str) -> Box<dyn GUIFactory> {
    match os_type {
        "windows" => Box::new(WindowsFactory),
        "mac" => Box::new(MacFactory),
        _ => panic!("不支持的操作系统类型"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_abstract_factory() {
        // 测试Windows工厂
        let windows_factory = get_factory("windows");
        let windows_app = Application::new(windows_factory.as_ref());
        
        println!("Windows应用程序:");
        windows_app.render();

        // 测试MacOS工厂
        let mac_factory = get_factory("mac");
        let mac_app = Application::new(mac_factory.as_ref());
        
        println!("\nMacOS应用程序:");
        mac_app.render();
    }
}

pub fn demo() {
    println!("=== 抽象工厂模式演示 ===");
    
    // 根据运行环境选择不同的工厂
    let factory = get_factory("windows");
    let app = Application::new(factory.as_ref());
    
    println!("创建的GUI组件:");
    app.render();
} 