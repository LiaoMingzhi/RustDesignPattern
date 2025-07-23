//! 状态模式 (State Pattern)
//! 
//! 允许一个对象在其内部状态改变时改变它的行为。对象看起来似乎修改了它的类。
//! 文件路径：/d%3A/workspace/RustLearn/RustDesignPattern/src/GoFDesignPattern/BehavioralPatterns/state.rs

use std::collections::HashMap;

// 状态接口
trait VendingMachineState {
    fn insert_coin(&self, machine: &mut VendingMachine) -> Result<(), String>;
    fn select_product(&self, machine: &mut VendingMachine, product: &str) -> Result<(), String>;
    fn dispense(&self, machine: &mut VendingMachine) -> Result<(), String>;
    fn refund(&self, machine: &mut VendingMachine) -> Result<(), String>;
    fn get_state_name(&self) -> &str;
}

// 状态枚举 - 更简单的状态管理
#[derive(Debug, Clone)]
enum MachineState {
    Idle,
    HasMoney,
    ProductSelected,
}

impl MachineState {
    fn get_name(&self) -> &str {
        match self {
            MachineState::Idle => "空闲状态",
            MachineState::HasMoney => "有钱状态",
            MachineState::ProductSelected => "商品已选择状态",
        }
    }
}

// 上下文 - 自动售货机
struct VendingMachine {
    state: MachineState,
    balance: u32,
    selected_product: Option<String>,
    products: HashMap<String, (u32, u32)>, // 产品名 -> (价格, 库存)
}

impl VendingMachine {
    fn new() -> Self {
        let mut products = HashMap::new();
        products.insert("可乐".to_string(), (300, 5));
        products.insert("水".to_string(), (200, 3));
        products.insert("薯片".to_string(), (250, 2));
        
        Self {
            state: MachineState::Idle,
            balance: 0,
            selected_product: None,
            products,
        }
    }

    fn set_state(&mut self, state: MachineState) {
        println!("状态切换到: {}", state.get_name());
        self.state = state;
    }

    fn add_balance(&mut self, amount: u32) {
        self.balance += amount;
        println!("余额增加 {}，当前余额: {}", amount, self.balance);
    }

    fn deduct_balance(&mut self, amount: u32) -> bool {
        if self.balance >= amount {
            self.balance -= amount;
            println!("扣除 {}，剩余余额: {}", amount, self.balance);
            true
        } else {
            false
        }
    }

    fn get_product_price(&self, product: &str) -> Option<u32> {
        self.products.get(product).map(|(price, _)| *price)
    }

    fn get_product_stock(&self, product: &str) -> Option<u32> {
        self.products.get(product).map(|(_, stock)| *stock)
    }

    fn reduce_stock(&mut self, product: &str) -> bool {
        if let Some((_price, stock)) = self.products.get_mut(product) {
            if *stock > 0 {
                *stock -= 1;
                println!("出货: {}，剩余库存: {}", product, *stock);
                return true;
            }
        }
        false
    }

    fn show_status(&self) {
        println!("售货机状态: {} | 余额: {} | 选择的产品: {:?}", 
                self.state.get_name(), 
                self.balance, 
                self.selected_product);
    }

    // 公共接口 - 根据当前状态执行不同的行为
    fn insert_coin(&mut self) -> Result<(), String> {
        match self.state {
            MachineState::Idle => {
                self.add_balance(100); // 假设投入100的硬币
                self.set_state(MachineState::HasMoney);
                Ok(())
            },
            MachineState::HasMoney | MachineState::ProductSelected => {
                self.add_balance(100);
                Ok(())
            }
        }
    }

    fn select_product(&mut self, product: &str) -> Result<(), String> {
        match self.state {
            MachineState::Idle => {
                Err("请先投币".to_string())
            },
            MachineState::HasMoney | MachineState::ProductSelected => {
                if let Some(price) = self.get_product_price(product) {
                    if let Some(stock) = self.get_product_stock(product) {
                        if stock == 0 {
                            return Err(format!("{} 已售完", product));
                        }

                        if self.balance >= price {
                            self.selected_product = Some(product.to_string());
                            self.set_state(MachineState::ProductSelected);
                            println!("选择商品: {} (价格: {})", product, price);
                            Ok(())
                        } else {
                            Err(format!("余额不足，需要 {}，当前余额 {}", price, self.balance))
                        }
                    } else {
                        Err(format!("商品 {} 不存在", product))
                    }
                } else {
                    Err(format!("商品 {} 不存在", product))
                }
            }
        }
    }

    fn dispense(&mut self) -> Result<(), String> {
        match self.state {
            MachineState::Idle => {
                Err("请先投币并选择商品".to_string())
            },
            MachineState::HasMoney => {
                Err("请先选择商品".to_string())
            },
            MachineState::ProductSelected => {
                if let Some(ref product) = self.selected_product.clone() {
                    if let Some(price) = self.get_product_price(product) {
                        if self.deduct_balance(price) && self.reduce_stock(product) {
                            self.selected_product = None;
                            
                            if self.balance > 0 {
                                self.set_state(MachineState::HasMoney);
                            } else {
                                self.set_state(MachineState::Idle);
                            }
                            
                            println!("成功出货: {}", product);
                            Ok(())
                        } else {
                            Err("出货失败".to_string())
                        }
                    } else {
                        Err("商品价格错误".to_string())
                    }
                } else {
                    Err("没有选择商品".to_string())
                }
            }
        }
    }

    fn refund(&mut self) -> Result<(), String> {
        match self.state {
            MachineState::Idle => {
                Err("没有余额可退".to_string())
            },
            MachineState::HasMoney => {
                let refund_amount = self.balance;
                self.balance = 0;
                self.set_state(MachineState::Idle);
                println!("退币: {}", refund_amount);
                Ok(())
            },
            MachineState::ProductSelected => {
                let refund_amount = self.balance;
                self.balance = 0;
                self.selected_product = None;
                self.set_state(MachineState::Idle);
                println!("退币: {}，取消商品选择", refund_amount);
                Ok(())
            }
        }
    }
}

pub fn demo() {
    println!("=== 状态模式演示 ===");

    let mut machine = VendingMachine::new();
    machine.show_status();

    // 测试各种操作
    println!("\n1. 尝试在空闲状态选择商品:");
    if let Err(e) = machine.select_product("可乐") {
        println!("错误: {}", e);
    }

    println!("\n2. 投币:");
    if let Err(e) = machine.insert_coin() {
        println!("投币失败: {}", e);
        return;
    }
    machine.show_status();

    println!("\n3. 选择商品:");
    if let Err(e) = machine.select_product("可乐") {
        println!("选择商品失败: {}", e);
        return;
    }
    machine.show_status();

    println!("\n4. 出货:");
    if let Err(e) = machine.dispense() {
        println!("出货失败: {}", e);
        return;
    }
    machine.show_status();

    println!("\n5. 继续投币和购买:");
    if let Err(e) = machine.insert_coin() {
        println!("投币失败: {}", e);
        return;
    }
    if let Err(e) = machine.insert_coin() {
        println!("投币失败: {}", e);
        return;
    }
    if let Err(e) = machine.select_product("薯片") {
        println!("选择商品失败: {}", e);
        return;
    }
    if let Err(e) = machine.dispense() {
        println!("出货失败: {}", e);
        return;
    }
    machine.show_status();

    println!("\n6. 投币后退币:");
    if let Err(e) = machine.insert_coin() {
        println!("投币失败: {}", e);
        return;
    }
    if let Err(e) = machine.refund() {
        println!("退币失败: {}", e);
        return;
    }
    machine.show_status();

    println!("\n7. 测试余额不足的情况:");
    if let Err(e) = machine.insert_coin() {
        println!("投币失败: {}", e);
        return;
    }
    // 只投一次币（100），尝试买可乐（300）
    if let Err(e) = machine.select_product("可乐") {
        println!("选择商品失败: {}", e);
    }
    // 投足够的币
    if let Err(e) = machine.insert_coin() {
        println!("投币失败: {}", e);
        return;
    }
    if let Err(e) = machine.insert_coin() {
        println!("投币失败: {}", e);
        return;
    }
    if let Err(e) = machine.select_product("可乐") {
        println!("选择商品失败: {}", e);
        return;
    }
    if let Err(e) = machine.dispense() {
        println!("出货失败: {}", e);
        return;
    }
    machine.show_status();

    println!("\n状态模式的优点:");
    println!("1. 将状态相关的行为局部化，并且将不同状态的行为分割开来");
    println!("2. 使状态转换显式化");
    println!("3. 可以让多个环境对象共享一个状态对象");
    println!("4. 状态类职责明确，有利于程序的扩展");
    
    println!("\n状态模式的应用场景:");
    println!("1. 当一个对象的行为取决于它的状态时");
    println!("2. 当一个操作中含有庞大的多分支的条件语句时");
    println!("3. 状态转换逻辑复杂时");
} 