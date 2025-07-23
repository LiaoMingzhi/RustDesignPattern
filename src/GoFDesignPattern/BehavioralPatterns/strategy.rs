//! 策略模式 (Strategy Pattern)
//! 
//! 定义一系列的算法，把它们一个个封装起来，并且使它们可相互替换。
//! 文件路径：/d%3A/workspace/RustLearn/RustDesignPattern/src/GoFDesignPattern/BehavioralPatterns/strategy.rs

// 策略接口
trait PaymentStrategy {
    fn pay(&self, amount: f64) -> Result<String, String>;
    fn get_name(&self) -> &str;
}

// 具体策略 - 信用卡支付
struct CreditCardPayment {
    card_number: String,
    cvv: String,
}

impl CreditCardPayment {
    fn new(card_number: String, cvv: String) -> Self {
        Self { card_number, cvv }
    }
}

impl PaymentStrategy for CreditCardPayment {
    fn pay(&self, amount: f64) -> Result<String, String> {
        if self.card_number.len() != 16 {
            return Err("无效的信用卡号".to_string());
        }
        println!("使用信用卡支付 ¥{:.2}", amount);
        println!("信用卡号: ****-****-****-{}", &self.card_number[12..]);
        Ok(format!("信用卡支付 ¥{:.2} 成功", amount))
    }

    fn get_name(&self) -> &str {
        "信用卡支付"
    }
}

// 具体策略 - 支付宝支付
struct AlipayPayment {
    account: String,
}

impl AlipayPayment {
    fn new(account: String) -> Self {
        Self { account }
    }
}

impl PaymentStrategy for AlipayPayment {
    fn pay(&self, amount: f64) -> Result<String, String> {
        println!("使用支付宝支付 ¥{:.2}", amount);
        println!("支付宝账号: {}", self.account);
        Ok(format!("支付宝支付 ¥{:.2} 成功", amount))
    }

    fn get_name(&self) -> &str {
        "支付宝支付"
    }
}

// 具体策略 - 微信支付
struct WechatPayment {
    phone_number: String,
}

impl WechatPayment {
    fn new(phone_number: String) -> Self {
        Self { phone_number }
    }
}

impl PaymentStrategy for WechatPayment {
    fn pay(&self, amount: f64) -> Result<String, String> {
        println!("使用微信支付 ¥{:.2}", amount);
        println!("微信绑定手机: {}", self.phone_number);
        Ok(format!("微信支付 ¥{:.2} 成功", amount))
    }

    fn get_name(&self) -> &str {
        "微信支付"
    }
}

// 上下文 - 购物车
struct ShoppingCart {
    items: Vec<(String, f64)>,
    payment_strategy: Option<Box<dyn PaymentStrategy>>,
}

impl ShoppingCart {
    fn new() -> Self {
        Self {
            items: Vec::new(),
            payment_strategy: None,
        }
    }

    fn add_item(&mut self, name: String, price: f64) {
        self.items.push((name, price));
        println!("添加商品: {} - ¥{:.2}", self.items.last().unwrap().0, price);
    }

    fn set_payment_strategy(&mut self, strategy: Box<dyn PaymentStrategy>) {
        println!("设置支付方式: {}", strategy.get_name());
        self.payment_strategy = Some(strategy);
    }

    fn calculate_total(&self) -> f64 {
        self.items.iter().map(|(_, price)| price).sum()
    }

    fn checkout(&self) -> Result<String, String> {
        if self.items.is_empty() {
            return Err("购物车为空".to_string());
        }

        let total = self.calculate_total();
        println!("\n=== 结账 ===");
        for (name, price) in &self.items {
            println!("{}: ¥{:.2}", name, price);
        }
        println!("总计: ¥{:.2}", total);

        if let Some(ref strategy) = self.payment_strategy {
            strategy.pay(total)
        } else {
            Err("请选择支付方式".to_string())
        }
    }

    fn show_cart(&self) {
        println!("\n购物车内容:");
        if self.items.is_empty() {
            println!("  (空)");
        } else {
            for (name, price) in &self.items {
                println!("  {} - ¥{:.2}", name, price);
            }
            println!("  总计: ¥{:.2}", self.calculate_total());
        }
    }
}

// 另一个例子 - 排序策略
trait SortStrategy {
    fn sort(&self, data: &mut Vec<i32>);
    fn get_name(&self) -> &str;
}

struct BubbleSort;

impl SortStrategy for BubbleSort {
    fn sort(&self, data: &mut Vec<i32>) {
        let n = data.len();
        for i in 0..n {
            for j in 0..n - 1 - i {
                if data[j] > data[j + 1] {
                    data.swap(j, j + 1);
                }
            }
        }
        println!("使用冒泡排序完成");
    }

    fn get_name(&self) -> &str {
        "冒泡排序"
    }
}

struct QuickSort;

impl SortStrategy for QuickSort {
    fn sort(&self, data: &mut Vec<i32>) {
        if data.len() <= 1 {
            return;
        }
        self.quick_sort_recursive(data, 0, data.len() - 1);
        println!("使用快速排序完成");
    }

    fn get_name(&self) -> &str {
        "快速排序"
    }
}

impl QuickSort {
    fn quick_sort_recursive(&self, data: &mut Vec<i32>, low: usize, high: usize) {
        if low < high {
            let pi = self.partition(data, low, high);
            if pi > 0 {
                self.quick_sort_recursive(data, low, pi - 1);
            }
            self.quick_sort_recursive(data, pi + 1, high);
        }
    }

    fn partition(&self, data: &mut Vec<i32>, low: usize, high: usize) -> usize {
        let pivot = data[high];
        let mut i = low;

        for j in low..high {
            if data[j] <= pivot {
                data.swap(i, j);
                i += 1;
            }
        }
        data.swap(i, high);
        i
    }
}

struct SortContext {
    strategy: Option<Box<dyn SortStrategy>>,
}

impl SortContext {
    fn new() -> Self {
        Self { strategy: None }
    }

    fn set_strategy(&mut self, strategy: Box<dyn SortStrategy>) {
        println!("设置排序策略: {}", strategy.get_name());
        self.strategy = Some(strategy);
    }

    fn sort(&self, data: &mut Vec<i32>) {
        if let Some(ref strategy) = self.strategy {
            println!("排序前: {:?}", data);
            strategy.sort(data);
            println!("排序后: {:?}", data);
        } else {
            println!("请先设置排序策略");
        }
    }
}

pub fn demo() {
    println!("=== 策略模式演示 ===");

    // 1. 支付策略示例
    println!("\n1. 支付策略:");
    let mut cart = ShoppingCart::new();
    
    cart.add_item("笔记本电脑".to_string(), 5999.0);
    cart.add_item("鼠标".to_string(), 199.0);
    cart.add_item("键盘".to_string(), 299.0);
    
    cart.show_cart();

    // 使用不同的支付策略
    println!("\n使用信用卡支付:");
    cart.set_payment_strategy(Box::new(CreditCardPayment::new(
        "1234567890123456".to_string(),
        "123".to_string(),
    )));
    match cart.checkout() {
        Ok(result) => println!("{}", result),
        Err(e) => println!("支付失败: {}", e),
    }

    println!("\n切换到支付宝支付:");
    cart.set_payment_strategy(Box::new(AlipayPayment::new(
        "user@example.com".to_string(),
    )));
    match cart.checkout() {
        Ok(result) => println!("{}", result),
        Err(e) => println!("支付失败: {}", e),
    }

    // 2. 排序策略示例
    println!("\n\n2. 排序策略:");
    let mut sort_context = SortContext::new();
    
    let mut data1 = vec![64, 34, 25, 12, 22, 11, 90];
    sort_context.set_strategy(Box::new(BubbleSort));
    sort_context.sort(&mut data1);

    let mut data2 = vec![64, 34, 25, 12, 22, 11, 90];
    sort_context.set_strategy(Box::new(QuickSort));
    sort_context.sort(&mut data2);

    println!("\n策略模式的优点:");
    println!("1. 算法可以自由切换");
    println!("2. 避免使用多重条件判断");
    println!("3. 扩展性良好，易于增加新的策略");
    println!("4. 符合开闭原则");
} 