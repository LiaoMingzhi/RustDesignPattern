//! 领域模型模式 (Domain Model)
//! 
//! 构建包含行为和数据的复杂网状结构的对象模型
//! 文件路径：/d%3A/workspace/RustLearn/RustDesignPattern/src/EnterpriseAppPattern/DomainLogicPatterns/domain_model.rs

use std::fmt;

#[derive(Debug)]
pub enum DomainError {
    ValidationError(String),
    InsufficientFunds,
    InvalidState(String),
}

impl fmt::Display for DomainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DomainError::ValidationError(msg) => write!(f, "验证错误: {}", msg),
            DomainError::InsufficientFunds => write!(f, "余额不足"),
            DomainError::InvalidState(msg) => write!(f, "无效状态: {}", msg),
        }
    }
}

// 值对象：金额
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Money {
    amount: f64,
}

impl Money {
    pub fn new(amount: f64) -> Result<Self, DomainError> {
        if amount < 0.0 {
            return Err(DomainError::ValidationError("金额不能为负数".to_string()));
        }
        Ok(Self { amount })
    }

    pub fn zero() -> Self {
        Self { amount: 0.0 }
    }

    pub fn value(&self) -> f64 {
        self.amount
    }

    pub fn add(&self, other: Money) -> Money {
        Money { amount: self.amount + other.amount }
    }

    pub fn subtract(&self, other: Money) -> Result<Money, DomainError> {
        if self.amount < other.amount {
            return Err(DomainError::InsufficientFunds);
        }
        Ok(Money { amount: self.amount - other.amount })
    }

    pub fn is_greater_or_equal(&self, other: Money) -> bool {
        self.amount >= other.amount
    }
}

impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "¥{:.2}", self.amount)
    }
}

// 实体：账户
#[derive(Debug, Clone)]
pub struct Account {
    id: u32,
    owner_name: String,
    balance: Money,
    account_type: AccountType,
}

#[derive(Debug, Clone)]
pub enum AccountType {
    Savings,    // 储蓄账户
    Checking,   // 支票账户
    Credit,     // 信用账户
}

impl AccountType {
    pub fn interest_rate(&self) -> f64 {
        match self {
            AccountType::Savings => 0.02,
            AccountType::Checking => 0.001,
            AccountType::Credit => 0.15,
        }
    }
}

impl fmt::Display for AccountType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let type_name = match self {
            AccountType::Savings => "储蓄账户",
            AccountType::Checking => "支票账户",
            AccountType::Credit => "信用账户",
        };
        write!(f, "{}", type_name)
    }
}

impl Account {
    pub fn new(id: u32, owner_name: String, account_type: AccountType, initial_balance: Money) -> Self {
        println!("创建账户: ID={}, 户主={}, 类型={}, 初始余额={}", 
                id, owner_name, account_type, initial_balance);
        
        Self {
            id,
            owner_name,
            balance: initial_balance,
            account_type,
        }
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn owner_name(&self) -> &str {
        &self.owner_name
    }

    pub fn balance(&self) -> Money {
        self.balance
    }

    pub fn account_type(&self) -> &AccountType {
        &self.account_type
    }

    // 业务方法：存款
    pub fn deposit(&mut self, amount: Money) -> Result<(), DomainError> {
        if amount.value() <= 0.0 {
            return Err(DomainError::ValidationError("存款金额必须大于0".to_string()));
        }

        self.balance = self.balance.add(amount);
        println!("账户 {} 存款 {}，余额: {}", self.owner_name, amount, self.balance);
        Ok(())
    }

    // 业务方法：取款
    pub fn withdraw(&mut self, amount: Money) -> Result<(), DomainError> {
        if amount.value() <= 0.0 {
            return Err(DomainError::ValidationError("取款金额必须大于0".to_string()));
        }

        // 储蓄账户不能透支
        if matches!(self.account_type, AccountType::Savings) && !self.balance.is_greater_or_equal(amount) {
            return Err(DomainError::InsufficientFunds);
        }

        self.balance = self.balance.subtract(amount)?;
        println!("账户 {} 取款 {}，余额: {}", self.owner_name, amount, self.balance);
        Ok(())
    }

    // 业务方法：计算利息
    pub fn calculate_interest(&self) -> Money {
        let interest_rate = self.account_type.interest_rate();
        let interest = Money::new(self.balance.value() * interest_rate).unwrap_or(Money::zero());
        println!("账户 {} 利息计算: 本金={}, 利率={:.2}%, 利息={}", 
                self.owner_name, self.balance, interest_rate * 100.0, interest);
        interest
    }

    // 业务方法：应用利息
    pub fn apply_interest(&mut self) {
        let interest = self.calculate_interest();
        self.balance = self.balance.add(interest);
        println!("账户 {} 利息已计入，新余额: {}", self.owner_name, self.balance);
    }

    // 业务方法：检查是否可以转账
    pub fn can_transfer(&self, amount: Money) -> bool {
        match self.account_type {
            AccountType::Credit => true, // 信用账户可以透支
            _ => self.balance.is_greater_or_equal(amount),
        }
    }
}

impl fmt::Display for Account {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Account[id={}, owner={}, type={}, balance={}]", 
               self.id, self.owner_name, self.account_type, self.balance)
    }
}

// 实体：转账记录
#[derive(Debug, Clone)]
pub struct Transfer {
    id: u32,
    from_account_id: u32,
    to_account_id: u32,
    amount: Money,
    status: TransferStatus,
    timestamp: String,
}

#[derive(Debug, Clone)]
pub enum TransferStatus {
    Pending,
    Completed,
    Failed,
    Cancelled,
}

impl fmt::Display for TransferStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let status = match self {
            TransferStatus::Pending => "待处理",
            TransferStatus::Completed => "已完成",
            TransferStatus::Failed => "失败",
            TransferStatus::Cancelled => "已取消",
        };
        write!(f, "{}", status)
    }
}

impl Transfer {
    pub fn new(id: u32, from_account_id: u32, to_account_id: u32, amount: Money) -> Self {
        Self {
            id,
            from_account_id,
            to_account_id,
            amount,
            status: TransferStatus::Pending,
            timestamp: "2024-01-01 00:00:00".to_string(),
        }
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn amount(&self) -> Money {
        self.amount
    }

    pub fn status(&self) -> &TransferStatus {
        &self.status
    }

    // 业务方法：标记为完成
    pub fn mark_completed(&mut self) {
        self.status = TransferStatus::Completed;
        println!("转账 {} 标记为完成", self.id);
    }

    // 业务方法：标记为失败
    pub fn mark_failed(&mut self) {
        self.status = TransferStatus::Failed;
        println!("转账 {} 标记为失败", self.id);
    }
}

impl fmt::Display for Transfer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Transfer[id={}, from={}, to={}, amount={}, status={}]", 
               self.id, self.from_account_id, self.to_account_id, self.amount, self.status)
    }
}

// 领域服务：银行服务
pub struct BankService;

impl BankService {
    // 领域服务：执行转账
    pub fn execute_transfer(
        transfer: &mut Transfer, 
        from_account: &mut Account, 
        to_account: &mut Account
    ) -> Result<(), DomainError> {
        println!("开始执行转账: {}", transfer);
        
        // 验证转账
        if !matches!(transfer.status, TransferStatus::Pending) {
            return Err(DomainError::InvalidState("转账状态不正确".to_string()));
        }

        if !from_account.can_transfer(transfer.amount) {
            transfer.mark_failed();
            return Err(DomainError::InsufficientFunds);
        }

        // 执行转账
        from_account.withdraw(transfer.amount)?;
        to_account.deposit(transfer.amount)?;
        
        // 标记转账完成
        transfer.mark_completed();
        
        println!("转账执行完成: {}", transfer);
        Ok(())
    }

    // 领域服务：批量计算利息
    pub fn calculate_monthly_interest(accounts: &mut [Account]) {
        println!("开始批量计算月利息");
        
        for account in accounts.iter_mut() {
            account.apply_interest();
        }
        
        println!("月利息计算完成");
    }

    // 领域服务：账户升级
    pub fn upgrade_account_if_eligible(account: &mut Account) -> bool {
        let balance = account.balance().value();
        
        match account.account_type {
            AccountType::Checking if balance >= 10000.0 => {
                account.account_type = AccountType::Savings;
                println!("账户 {} 升级为储蓄账户", account.owner_name);
                true
            },
            AccountType::Savings if balance >= 50000.0 => {
                account.account_type = AccountType::Credit;
                println!("账户 {} 升级为信用账户", account.owner_name);
                true
            },
            _ => false,
        }
    }
}

pub fn demo() {
    println!("=== 领域模型模式演示 ===");

    // 1. 创建账户
    println!("\n1. 创建账户:");
    let initial_balance1 = Money::new(1000.0).unwrap();
    let mut account1 = Account::new(1, "张三".to_string(), AccountType::Checking, initial_balance1);
    
    let initial_balance2 = Money::new(500.0).unwrap();
    let mut account2 = Account::new(2, "李四".to_string(), AccountType::Savings, initial_balance2);

    // 2. 账户操作
    println!("\n2. 账户操作:");
    let deposit_amount = Money::new(500.0).unwrap();
    account1.deposit(deposit_amount).ok();
    
    let withdraw_amount = Money::new(200.0).unwrap();
    account2.withdraw(withdraw_amount).ok();

    // 3. 转账操作
    println!("\n3. 转账操作:");
    let transfer_amount = Money::new(300.0).unwrap();
    let mut transfer = Transfer::new(1001, account1.id(), account2.id(), transfer_amount);
    
    match BankService::execute_transfer(&mut transfer, &mut account1, &mut account2) {
        Ok(_) => println!("✓ 转账成功"),
        Err(e) => println!("✗ 转账失败: {}", e),
    }

    // 4. 利息计算
    println!("\n4. 利息计算:");
    account1.calculate_interest();
    account2.calculate_interest();

    // 5. 批量利息计算
    println!("\n5. 批量利息计算:");
    let mut accounts = vec![account1.clone(), account2.clone()];
    BankService::calculate_monthly_interest(&mut accounts);

    // 6. 账户升级
    println!("\n6. 账户升级检查:");
    // 给账户1增加余额以满足升级条件
    let large_deposit = Money::new(9000.0).unwrap();
    account1.deposit(large_deposit).ok();
    
    if BankService::upgrade_account_if_eligible(&mut account1) {
        println!("✓ 账户升级成功: {}", account1);
    } else {
        println!("账户暂不满足升级条件");
    }

    // 7. 显示最终状态
    println!("\n7. 最终账户状态:");
    println!("账户1: {}", account1);
    println!("账户2: {}", account2);
    println!("转账记录: {}", transfer);

    println!("\n领域模型模式的优点:");
    println!("1. 将业务逻辑封装在领域对象中");
    println!("2. 对象之间形成丰富的交互");
    println!("3. 高度表达业务概念和规则");
    println!("4. 便于应对复杂的业务逻辑");

    println!("\n适用场景:");
    println!("1. 复杂的业务逻辑");
    println!("2. 丰富的对象交互");
    println!("3. 长期维护的系统");
} 