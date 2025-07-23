//! 装饰器模式 (Decorator Pattern)
//! 
//! 动态地给一个对象添加一些额外的职责。就增加功能来说，装饰器模式相比生成子类更为灵活。
//! 文件路径：/d%3A/workspace/RustLearn/RustDesignPattern/src/GoFDesignPattern/StructuralPatterns/decorator.rs

// 组件接口
trait Coffee {
    fn cost(&self) -> f64;
    fn description(&self) -> String;
}

// 具体组件 - 基础咖啡
struct SimpleCoffee;

impl Coffee for SimpleCoffee {
    fn cost(&self) -> f64 {
        10.0
    }

    fn description(&self) -> String {
        "简单咖啡".to_string()
    }
}

// 装饰器基类
trait CoffeeDecorator: Coffee {
    fn get_coffee(&self) -> &dyn Coffee;
}

// 具体装饰器 - 牛奶
struct MilkDecorator {
    coffee: Box<dyn Coffee>,
}

impl MilkDecorator {
    fn new(coffee: Box<dyn Coffee>) -> Self {
        Self { coffee }
    }
}

impl Coffee for MilkDecorator {
    fn cost(&self) -> f64 {
        self.coffee.cost() + 2.0
    }

    fn description(&self) -> String {
        format!("{} + 牛奶", self.coffee.description())
    }
}

impl CoffeeDecorator for MilkDecorator {
    fn get_coffee(&self) -> &dyn Coffee {
        self.coffee.as_ref()
    }
}

// 具体装饰器 - 糖
struct SugarDecorator {
    coffee: Box<dyn Coffee>,
}

impl SugarDecorator {
    fn new(coffee: Box<dyn Coffee>) -> Self {
        Self { coffee }
    }
}

impl Coffee for SugarDecorator {
    fn cost(&self) -> f64 {
        self.coffee.cost() + 1.0
    }

    fn description(&self) -> String {
        format!("{} + 糖", self.coffee.description())
    }
}

impl CoffeeDecorator for SugarDecorator {
    fn get_coffee(&self) -> &dyn Coffee {
        self.coffee.as_ref()
    }
}

// 具体装饰器 - 巧克力
struct ChocolateDecorator {
    coffee: Box<dyn Coffee>,
}

impl ChocolateDecorator {
    fn new(coffee: Box<dyn Coffee>) -> Self {
        Self { coffee }
    }
}

impl Coffee for ChocolateDecorator {
    fn cost(&self) -> f64 {
        self.coffee.cost() + 3.0
    }

    fn description(&self) -> String {
        format!("{} + 巧克力", self.coffee.description())
    }
}

impl CoffeeDecorator for ChocolateDecorator {
    fn get_coffee(&self) -> &dyn Coffee {
        self.coffee.as_ref()
    }
}

pub fn demo() {
    println!("=== 装饰器模式演示 ===");

    // 基础咖啡
    let coffee = Box::new(SimpleCoffee);
    println!("{}: ¥{:.2}", coffee.description(), coffee.cost());

    // 添加牛奶
    let milk_coffee = Box::new(MilkDecorator::new(coffee));
    println!("{}: ¥{:.2}", milk_coffee.description(), milk_coffee.cost());

    // 继续添加糖
    let sweet_coffee = Box::new(SugarDecorator::new(milk_coffee));
    println!("{}: ¥{:.2}", sweet_coffee.description(), sweet_coffee.cost());

    // 最后添加巧克力
    let luxury_coffee = Box::new(ChocolateDecorator::new(sweet_coffee));
    println!("{}: ¥{:.2}", luxury_coffee.description(), luxury_coffee.cost());
} 