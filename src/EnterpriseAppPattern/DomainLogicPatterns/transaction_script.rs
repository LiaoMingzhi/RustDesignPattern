//! 事务脚本模式 (Transaction Script)
//! 
//! 将业务逻辑组织成一个个处理单个请求的脚本
//! 文件路径：/d%3A/workspace/RustLearn/RustDesignPattern/src/EnterpriseAppPattern/DomainLogicPatterns/transaction_script.rs

use std::collections::HashMap;
use std::fmt;
use std::sync::{Mutex, OnceLock};

#[derive(Debug)]
pub enum BusinessError {
    ValidationError(String),
    InsufficientFunds,
    NotFound,
}

impl fmt::Display for BusinessError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BusinessError::ValidationError(msg) => write!(f, "验证错误: {}", msg),
            BusinessError::InsufficientFunds => write!(f, "余额不足"),
            BusinessError::NotFound => write!(f, "记录不存在"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Account {
    pub id: u32,
    pub name: String,
    pub balance: f64,
}

#[derive(Debug, Clone)]
pub struct TransferRecord {
    pub from_account: u32,
    pub to_account: u32,
    pub amount: f64,
    pub timestamp: String,
}

// 使用线程安全的全局存储
static ACCOUNTS: OnceLock<Mutex<HashMap<u32, Account>>> = OnceLock::new();
static TRANSFERS: OnceLock<Mutex<Vec<TransferRecord>>> = OnceLock::new();

fn get_accounts() -> &'static Mutex<HashMap<u32, Account>> {
    ACCOUNTS.get_or_init(|| {
        let mut accounts = HashMap::new();
        accounts.insert(1, Account { id: 1, name: "张三".to_string(), balance: 1000.0 });
        accounts.insert(2, Account { id: 2, name: "李四".to_string(), balance: 500.0 });
        Mutex::new(accounts)
    })
}

fn get_transfers() -> &'static Mutex<Vec<TransferRecord>> {
    TRANSFERS.get_or_init(|| Mutex::new(Vec::new()))
}

// 事务脚本集合
pub struct BankingService;

impl BankingService {
    pub fn new() -> Self {
        println!("初始化银行业务服务");
        Self
    }

    // 事务脚本：转账
    pub fn transfer_money(&self, from_id: u32, to_id: u32, amount: f64) -> Result<(), BusinessError> {
        println!("执行转账事务: {} -> {}, 金额: {:.2}", from_id, to_id, amount);
        
        // 验证输入
        if amount <= 0.0 {
            return Err(BusinessError::ValidationError("转账金额必须大于0".to_string()));
        }
        if from_id == to_id {
            return Err(BusinessError::ValidationError("不能向自己转账".to_string()));
        }

        let accounts = get_accounts();
        let transfers = get_transfers();
        
        let mut accounts_guard = accounts.lock().unwrap();
        let mut transfers_guard = transfers.lock().unwrap();

        // 检查账户是否存在
        if !accounts_guard.contains_key(&from_id) || !accounts_guard.contains_key(&to_id) {
            return Err(BusinessError::NotFound);
        }

        // 检查余额
        let from_account = accounts_guard.get(&from_id).unwrap();
        if from_account.balance < amount {
            return Err(BusinessError::InsufficientFunds);
        }

        // 执行转账
        accounts_guard.get_mut(&from_id).unwrap().balance -= amount;
        accounts_guard.get_mut(&to_id).unwrap().balance += amount;

        // 记录转账
        let record = TransferRecord {
            from_account: from_id,
            to_account: to_id,
            amount,
            timestamp: "2024-01-01 00:00:00".to_string(),
        };
        transfers_guard.push(record);

        println!("转账成功");
        Ok(())
    }

    // 事务脚本：查询余额
    pub fn get_balance(&self, account_id: u32) -> Result<f64, BusinessError> {
        let accounts = get_accounts();
        let accounts_guard = accounts.lock().unwrap();
        
        match accounts_guard.get(&account_id) {
            Some(account) => {
                println!("账户 {} 余额: {:.2}", account.name, account.balance);
                Ok(account.balance)
            },
            None => Err(BusinessError::NotFound)
        }
    }

    // 事务脚本：存款
    pub fn deposit(&self, account_id: u32, amount: f64) -> Result<(), BusinessError> {
        println!("执行存款事务: 账户 {}, 金额: {:.2}", account_id, amount);
        
        if amount <= 0.0 {
            return Err(BusinessError::ValidationError("存款金额必须大于0".to_string()));
        }

        let accounts = get_accounts();
        let mut accounts_guard = accounts.lock().unwrap();
        
        match accounts_guard.get_mut(&account_id) {
            Some(account) => {
                account.balance += amount;
                println!("存款成功，账户 {} 余额: {:.2}", account.name, account.balance);
                Ok(())
            },
            None => Err(BusinessError::NotFound)
        }
    }

    // 事务脚本：取款
    pub fn withdraw(&self, account_id: u32, amount: f64) -> Result<(), BusinessError> {
        println!("执行取款事务: 账户 {}, 金额: {:.2}", account_id, amount);
        
        if amount <= 0.0 {
            return Err(BusinessError::ValidationError("取款金额必须大于0".to_string()));
        }

        let accounts = get_accounts();
        
        let mut accounts_guard = accounts.lock().unwrap();
        let account = accounts_guard.get_mut(&account_id).unwrap();
        if account.balance < amount {
            return Err(BusinessError::InsufficientFunds);
        }
        account.balance -= amount;
        println!("取款成功，账户 {} 余额: {:.2}", account.name, account.balance);
        Ok(())
    }

    // 事务脚本：获取转账记录
    pub fn get_transfer_history(&self, account_id: u32) -> Vec<TransferRecord> {
        let accounts = get_accounts();
        let accounts_guard = accounts.lock().unwrap();
        let transfers = get_transfers();
        let transfers_guard = transfers.lock().unwrap();
        
        transfers_guard.iter()
            .filter(|record| record.from_account == account_id || record.to_account == account_id)
            .cloned()
            .collect()
    }
}

pub fn demo() {
    println!("=== 事务脚本模式演示 ===");

    let service = BankingService::new();

    // 1. 查询初始余额
    println!("\n1. 查询初始余额:");
    service.get_balance(1).ok();
    service.get_balance(2).ok();

    // 2. 存款
    println!("\n2. 存款操作:");
    match service.deposit(1, 200.0) {
        Ok(_) => println!("✓ 存款成功"),
        Err(e) => println!("✗ 存款失败: {}", e),
    }

    // 3. 取款
    println!("\n3. 取款操作:");
    match service.withdraw(2, 100.0) {
        Ok(_) => println!("✓ 取款成功"),
        Err(e) => println!("✗ 取款失败: {}", e),
    }

    // 4. 转账
    println!("\n4. 转账操作:");
    match service.transfer_money(1, 2, 300.0) {
        Ok(_) => println!("✓ 转账成功"),
        Err(e) => println!("✗ 转账失败: {}", e),
    }

    // 5. 查询最终余额
    println!("\n5. 查询最终余额:");
    service.get_balance(1).ok();
    service.get_balance(2).ok();

    // 6. 查询转账记录
    println!("\n6. 转账记录:");
    let history = service.get_transfer_history(1);
    for record in &history {
        println!("  转账: {} -> {}, 金额: {:.2}", record.from_account, record.to_account, record.amount);
    }

    println!("\n事务脚本模式的优点:");
    println!("1. 简单直观，容易理解");
    println!("2. 每个脚本处理一个完整事务");
    println!("3. 便于测试和调试");
    println!("4. 适合简单业务逻辑");

    println!("\n适用场景:");
    println!("1. 业务逻辑相对简单");
    println!("2. 事务处理较为独立");
    println!("3. 快速原型开发");
} 