//! 活动记录模式 (Active Record)
//! 
//! 将数据访问逻辑嵌入到领域对象中
//! 文件路径：/d%3A/workspace/RustLearn/RustDesignPattern/src/EnterpriseAppPattern/DataSourceArchitecturalPatterns/active_record.rs

use std::collections::HashMap;
use std::fmt;
use std::sync::{Mutex, OnceLock};

#[derive(Debug)]
pub enum ActiveRecordError {
    NotFound,
    ValidationError(String),
}

impl fmt::Display for ActiveRecordError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ActiveRecordError::NotFound => write!(f, "记录未找到"),
            ActiveRecordError::ValidationError(msg) => write!(f, "验证错误: {}", msg),
        }
    }
}

// 使用线程安全的全局存储
static USER_STORAGE: OnceLock<Mutex<HashMap<u32, User>>> = OnceLock::new();
static NEXT_ID: OnceLock<Mutex<u32>> = OnceLock::new();

fn get_storage() -> &'static Mutex<HashMap<u32, User>> {
    USER_STORAGE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn get_next_id() -> u32 {
    let next_id_mutex = NEXT_ID.get_or_init(|| Mutex::new(1));
    let mut next_id = next_id_mutex.lock().unwrap();
    let id = *next_id;
    *next_id += 1;
    id
}

// 用户活动记录
#[derive(Debug, Clone)]
pub struct User {
    pub id: Option<u32>,
    pub username: String,
    pub email: String,
    pub balance: f64,
}

impl User {
    pub fn new(username: String, email: String) -> Self {
        Self {
            id: None,
            username,
            email,
            balance: 0.0,
        }
    }

    // 保存到数据库
    pub fn save(&mut self) -> Result<(), ActiveRecordError> {
        if self.username.is_empty() {
            return Err(ActiveRecordError::ValidationError("用户名不能为空".to_string()));
        }

        let storage = get_storage();
        let mut storage_guard = storage.lock().unwrap();

        match self.id {
            Some(id) => {
                storage_guard.insert(id, self.clone());
                println!("更新用户: {}", self);
            },
            None => {
                let new_id = get_next_id();
                self.id = Some(new_id);
                storage_guard.insert(new_id, self.clone());
                println!("创建用户: {}", self);
            }
        }
        Ok(())
    }

    // 静态查找方法
    pub fn find(id: u32) -> Result<User, ActiveRecordError> {
        let storage = get_storage();
        let storage_guard = storage.lock().unwrap();
        match storage_guard.get(&id) {
            Some(user) => Ok(user.clone()),
            None => Err(ActiveRecordError::NotFound)
        }
    }

    pub fn find_all() -> Vec<User> {
        let storage = get_storage();
        let storage_guard = storage.lock().unwrap();
        storage_guard.values().cloned().collect()
    }

    // 业务方法
    pub fn deposit(&mut self, amount: f64) -> Result<(), ActiveRecordError> {
        if amount <= 0.0 {
            return Err(ActiveRecordError::ValidationError("金额必须大于0".to_string()));
        }
        self.balance += amount;
        self.save()?;
        println!("用户 {} 存款 {:.2}，余额: {:.2}", self.username, amount, self.balance);
        Ok(())
    }

    pub fn withdraw(&mut self, amount: f64) -> Result<(), ActiveRecordError> {
        if amount <= 0.0 {
            return Err(ActiveRecordError::ValidationError("金额必须大于0".to_string()));
        }
        if self.balance < amount {
            return Err(ActiveRecordError::ValidationError("余额不足".to_string()));
        }
        self.balance -= amount;
        self.save()?;
        println!("用户 {} 取款 {:.2}，余额: {:.2}", self.username, amount, self.balance);
        Ok(())
    }
}

impl fmt::Display for User {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "User[id={:?}, username={}, email={}, balance={:.2}]", 
               self.id, self.username, self.email, self.balance)
    }
}

pub fn demo() {
    println!("=== 活动记录模式演示 ===");

    // 创建用户
    println!("\n1. 创建用户:");
    let mut user1 = User::new("张三".to_string(), "zhangsan@example.com".to_string());
    let mut user2 = User::new("李四".to_string(), "lisi@company.com".to_string());

    user1.save().ok();
    user2.save().ok();

    // 业务操作
    println!("\n2. 业务操作:");
    user1.deposit(1000.0).ok();
    user1.withdraw(200.0).ok();

    // 查找
    println!("\n3. 查找操作:");
    if let Ok(found_user) = User::find(1) {
        println!("找到用户: {}", found_user);
    }

    let all_users = User::find_all();
    println!("所有用户:");
    for user in &all_users {
        println!("  - {}", user);
    }

    println!("\n活动记录模式的优点:");
    println!("1. 将数据和行为封装在一起");
    println!("2. 简单直观，易于理解");
    println!("3. 包含业务逻辑和验证");
    println!("4. 需要快速开发原型");
}