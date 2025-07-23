//! 数据映射器模式 (Data Mapper)
//! 
//! 在内存中的对象和数据库之间移动数据，同时保持彼此独立。
//! 文件路径：/d%3A/workspace/RustLearn/RustDesignPattern/src/EnterpriseAppPattern/DataSourceArchitecturalPatterns/data_mapper.rs

use std::collections::HashMap;
use std::fmt;
use std::sync::{Mutex, OnceLock};

// 数据访问错误
#[derive(Debug)]
pub enum DataMapperError {
    NotFound,
    ValidationError(String),
    DatabaseError(String),
}

impl fmt::Display for DataMapperError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataMapperError::NotFound => write!(f, "记录未找到"),
            DataMapperError::ValidationError(msg) => write!(f, "验证错误: {}", msg),
            DataMapperError::DatabaseError(msg) => write!(f, "数据库错误: {}", msg),
        }
    }
}

// 使用线程安全的全局存储
static USER_DATABASE: OnceLock<Mutex<HashMap<u32, HashMap<String, String>>>> = OnceLock::new();
static NEXT_USER_ID: OnceLock<Mutex<u32>> = OnceLock::new();

fn get_user_database() -> &'static Mutex<HashMap<u32, HashMap<String, String>>> {
    USER_DATABASE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn get_next_user_id() -> u32 {
    let next_id_mutex = NEXT_USER_ID.get_or_init(|| Mutex::new(1));
    let mut next_id = next_id_mutex.lock().unwrap();
    let id = *next_id;
    *next_id += 1;
    id
}

// 领域对象 - 纯粹的业务对象，不包含数据访问逻辑
#[derive(Debug, Clone, PartialEq)]
pub struct User {
    pub id: Option<u32>,
    pub username: String,
    pub email: String,
    pub full_name: String,
    pub age: u32,
    pub balance: f64,
}

impl User {
    pub fn new(username: String, email: String, full_name: String, age: u32) -> Self {
        Self {
            id: None,
            username,
            email,
            full_name,
            age,
            balance: 0.0,
        }
    }

    // 纯业务逻辑，不涉及数据访问
    pub fn can_buy(&self, amount: f64) -> bool {
        self.balance >= amount
    }

    pub fn is_adult(&self) -> bool {
        self.age >= 18
    }

    pub fn deposit(&mut self, amount: f64) -> Result<(), String> {
        if amount <= 0.0 {
            return Err("存款金额必须大于0".to_string());
        }
        self.balance += amount;
        Ok(())
    }

    pub fn withdraw(&mut self, amount: f64) -> Result<(), String> {
        if amount <= 0.0 {
            return Err("取款金额必须大于0".to_string());
        }
        if self.balance < amount {
            return Err("余额不足".to_string());
        }
        self.balance -= amount;
        Ok(())
    }
}

impl fmt::Display for User {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "User[id={:?}, username={}, name={}, age={}, balance={:.2}]", 
               self.id, self.username, self.full_name, self.age, self.balance)
    }
}

// 数据映射器 - 负责对象与数据库之间的映射
pub struct UserMapper;

impl UserMapper {
    pub fn new() -> Self {
        println!("初始化用户数据映射器");
        Self
    }

    // 插入新用户
    pub fn insert(&self, user: &mut User) -> Result<(), DataMapperError> {
        if user.id.is_some() {
            return Err(DataMapperError::ValidationError("不能插入已有ID的用户".to_string()));
        }

        self.validate_user(user)?;

        let db = get_user_database();
        let mut db_guard = db.lock().unwrap();
        
        // 检查用户名是否已存在
        for user_data in db_guard.values() {
            if user_data.get("username") == Some(&user.username) {
                return Err(DataMapperError::ValidationError("用户名已存在".to_string()));
            }
        }

        let new_id = get_next_user_id();
        user.id = Some(new_id);

        let user_data = self.to_database_record(user);
        db_guard.insert(new_id, user_data);

        println!("插入用户到数据库: {}", user);
        Ok(())
    }

    // 更新用户
    pub fn update(&self, user: &User) -> Result<(), DataMapperError> {
        let id = user.id.ok_or(DataMapperError::ValidationError("更新的用户必须有ID".to_string()))?;
        
        self.validate_user(user)?;

        let db = get_user_database();
        let mut db_guard = db.lock().unwrap();
        
        // 检查用户是否存在
        if !db_guard.contains_key(&id) {
            return Err(DataMapperError::NotFound);
        }

        // 检查用户名是否被其他用户使用
        for (existing_id, user_data) in db_guard.iter() {
            if *existing_id != id && user_data.get("username") == Some(&user.username) {
                return Err(DataMapperError::ValidationError("用户名已被其他用户使用".to_string()));
            }
        }

        let user_data = self.to_database_record(user);
        db_guard.insert(id, user_data);

        println!("更新用户到数据库: {}", user);
        Ok(())
    }

    // 根据ID查找用户
    pub fn find_by_id(&self, id: u32) -> Result<User, DataMapperError> {
        let db = get_user_database();
        let db_guard = db.lock().unwrap();
        
        match db_guard.get(&id) {
            Some(user_data) => {
                let user = self.from_database_record(id, user_data)?;
                println!("从数据库加载用户: {}", user);
                Ok(user)
            },
            None => Err(DataMapperError::NotFound)
        }
    }

    // 根据用户名查找用户
    pub fn find_by_username(&self, username: &str) -> Result<User, DataMapperError> {
        let db = get_user_database();
        
        let db_guard = db.lock().unwrap();
        for (id, user_data) in db_guard.iter() {
            if user_data.get("username") == Some(&username.to_string()) {
                let user = self.from_database_record(*id, user_data)?;
                println!("根据用户名找到用户: {}", user);
                return Ok(user);
            }
        }
        
        Err(DataMapperError::NotFound)
    }

    // 查找所有用户
    pub fn find_all(&self) -> Result<Vec<User>, DataMapperError> {
        let db = get_user_database();
        let db_guard = db.lock().unwrap();
        let mut users = Vec::new();
        
        for (id, user_data) in db_guard.iter() {
            let user = self.from_database_record(*id, user_data)?;
            users.push(user);
        }
        
        println!("从数据库加载所有用户，共 {} 个", users.len());
        Ok(users)
    }

    // 根据年龄范围查找用户
    pub fn find_by_age_range(&self, min_age: u32, max_age: u32) -> Result<Vec<User>, DataMapperError> {
        let db = get_user_database();
        let db_guard = db.lock().unwrap();
        let mut users = Vec::new();
        
        for (id, user_data) in db_guard.iter() {
            let user = self.from_database_record(*id, user_data)?;
            if user.age >= min_age && user.age <= max_age {
                users.push(user);
            }
        }
        
        println!("查找年龄 {}-{} 岁的用户，共 {} 个", min_age, max_age, users.len());
        Ok(users)
    }

    // 根据余额范围查找用户
    pub fn find_by_balance_range(&self, min_balance: f64, max_balance: f64) -> Result<Vec<User>, DataMapperError> {
        let db = get_user_database();
        let db_guard = db.lock().unwrap();
        let mut users = Vec::new();
        
        for (id, user_data) in db_guard.iter() {
            let user = self.from_database_record(*id, user_data)?;
            if user.balance >= min_balance && user.balance <= max_balance {
                users.push(user);
            }
        }
        
        println!("查找余额 {:.2}-{:.2} 的用户，共 {} 个", min_balance, max_balance, users.len());
        Ok(users)
    }

    // 删除用户
    pub fn delete(&self, id: u32) -> Result<User, DataMapperError> {
        let db = get_user_database();
        
        let mut db_guard = db.lock().unwrap();
        match db_guard.remove(&id) {
            Some(user_data) => {
                let user = self.from_database_record(id, &user_data)?;
                println!("从数据库删除用户: {}", user);
                Ok(user)
            },
            None => Err(DataMapperError::NotFound)
        }
    }

    // 获取用户总数
    pub fn count(&self) -> usize {
        let db = get_user_database();
        let db_guard = db.lock().unwrap();
        let count = db_guard.len();
        println!("数据库中用户总数: {}", count);
        count
    }

    // 私有辅助方法：验证用户数据
    fn validate_user(&self, user: &User) -> Result<(), DataMapperError> {
        if user.username.is_empty() {
            return Err(DataMapperError::ValidationError("用户名不能为空".to_string()));
        }
        if user.email.is_empty() {
            return Err(DataMapperError::ValidationError("邮箱不能为空".to_string()));
        }
        if !user.email.contains('@') {
            return Err(DataMapperError::ValidationError("邮箱格式不正确".to_string()));
        }
        if user.full_name.is_empty() {
            return Err(DataMapperError::ValidationError("姓名不能为空".to_string()));
        }
        if user.age > 150 {
            return Err(DataMapperError::ValidationError("年龄不能超过150岁".to_string()));
        }
        Ok(())
    }

    // 私有辅助方法：将用户对象转换为数据库记录
    fn to_database_record(&self, user: &User) -> HashMap<String, String> {
        let mut record = HashMap::new();
        record.insert("username".to_string(), user.username.clone());
        record.insert("email".to_string(), user.email.clone());
        record.insert("full_name".to_string(), user.full_name.clone());
        record.insert("age".to_string(), user.age.to_string());
        record.insert("balance".to_string(), user.balance.to_string());
        record
    }

    // 私有辅助方法：将数据库记录转换为用户对象
    fn from_database_record(&self, id: u32, record: &HashMap<String, String>) -> Result<User, DataMapperError> {
        let username = record.get("username")
            .ok_or(DataMapperError::DatabaseError("缺少用户名字段".to_string()))?
            .clone();
        
        let email = record.get("email")
            .ok_or(DataMapperError::DatabaseError("缺少邮箱字段".to_string()))?
            .clone();
        
        let full_name = record.get("full_name")
            .ok_or(DataMapperError::DatabaseError("缺少姓名字段".to_string()))?
            .clone();
        
        let age: u32 = record.get("age")
            .ok_or(DataMapperError::DatabaseError("缺少年龄字段".to_string()))?
            .parse()
            .map_err(|_| DataMapperError::DatabaseError("年龄字段格式错误".to_string()))?;
        
        let balance: f64 = record.get("balance")
            .ok_or(DataMapperError::DatabaseError("缺少余额字段".to_string()))?
            .parse()
            .map_err(|_| DataMapperError::DatabaseError("余额字段格式错误".to_string()))?;

        let mut user = User::new(username, email, full_name, age);
        user.id = Some(id);
        user.balance = balance;
        Ok(user)
    }
}

// 用户服务 - 使用数据映射器进行数据访问
pub struct UserService {
    mapper: UserMapper,
}

impl UserService {
    pub fn new() -> Self {
        Self {
            mapper: UserMapper::new(),
        }
    }

    // 创建新用户
    pub fn create_user(&self, username: String, email: String, full_name: String, age: u32) -> Result<User, DataMapperError> {
        let mut user = User::new(username, email, full_name, age);
        self.mapper.insert(&mut user)?;
        Ok(user)
    }

    // 用户存款
    pub fn deposit(&self, user_id: u32, amount: f64) -> Result<User, DataMapperError> {
        let mut user = self.mapper.find_by_id(user_id)?;
        
        user.deposit(amount)
            .map_err(|e| DataMapperError::ValidationError(e))?;
        
        self.mapper.update(&user)?;
        println!("用户 {} 存款 {:.2}，余额: {:.2}", user.username, amount, user.balance);
        Ok(user)
    }

    // 用户取款
    pub fn withdraw(&self, user_id: u32, amount: f64) -> Result<User, DataMapperError> {
        let mut user = self.mapper.find_by_id(user_id)?;
        
        user.withdraw(amount)
            .map_err(|e| DataMapperError::ValidationError(e))?;
        
        self.mapper.update(&user)?;
        println!("用户 {} 取款 {:.2}，余额: {:.2}", user.username, amount, user.balance);
        Ok(user)
    }

    // 转账
    pub fn transfer(&self, from_user_id: u32, to_user_id: u32, amount: f64) -> Result<(User, User), DataMapperError> {
        let mut from_user = self.mapper.find_by_id(from_user_id)?;
        let mut to_user = self.mapper.find_by_id(to_user_id)?;

        // 检查转账条件
        if !from_user.can_buy(amount) {
            return Err(DataMapperError::ValidationError("转出用户余额不足".to_string()));
        }

        // 执行转账
        from_user.withdraw(amount)
            .map_err(|e| DataMapperError::ValidationError(e))?;
        to_user.deposit(amount)
            .map_err(|e| DataMapperError::ValidationError(e))?;

        // 保存更改
        self.mapper.update(&from_user)?;
        self.mapper.update(&to_user)?;

        println!("转账成功: {} -> {}, 金额: {:.2}", from_user.username, to_user.username, amount);
        Ok((from_user, to_user))
    }

    // 查找成年用户
    pub fn find_adult_users(&self) -> Result<Vec<User>, DataMapperError> {
        let all_users = self.mapper.find_all()?;
        let adult_users: Vec<User> = all_users.into_iter()
            .filter(|user| user.is_adult())
            .collect();
        
        println!("找到 {} 个成年用户", adult_users.len());
        Ok(adult_users)
    }

    // 查找富有用户
    pub fn find_wealthy_users(&self, min_balance: f64) -> Result<Vec<User>, DataMapperError> {
        let wealthy_users = self.mapper.find_by_balance_range(min_balance, f64::MAX)?;
        println!("找到 {} 个余额超过 {:.2} 的用户", wealthy_users.len(), min_balance);
        Ok(wealthy_users)
    }

    // 获取用户统计信息
    pub fn get_user_statistics(&self) -> Result<UserStatistics, DataMapperError> {
        let all_users = self.mapper.find_all()?;
        
        if all_users.is_empty() {
            return Ok(UserStatistics::default());
        }

        let total_count = all_users.len();
        let adult_count = all_users.iter().filter(|u| u.is_adult()).count();
        let total_balance: f64 = all_users.iter().map(|u| u.balance).sum();
        let avg_balance = total_balance / total_count as f64;
        let avg_age: f64 = all_users.iter().map(|u| u.age as f64).sum::<f64>() / total_count as f64;
        
        let stats = UserStatistics {
            total_users: total_count,
            adult_users: adult_count,
            total_balance,
            average_balance: avg_balance,
            average_age: avg_age as u32,
        };

        println!("用户统计信息: {}", stats);
        Ok(stats)
    }
}

// 用户统计信息
#[derive(Debug)]
pub struct UserStatistics {
    pub total_users: usize,
    pub adult_users: usize,
    pub total_balance: f64,
    pub average_balance: f64,
    pub average_age: u32,
}

impl Default for UserStatistics {
    fn default() -> Self {
        Self {
            total_users: 0,
            adult_users: 0,
            total_balance: 0.0,
            average_balance: 0.0,
            average_age: 0,
        }
    }
}

impl fmt::Display for UserStatistics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "用户统计[总数: {}, 成年人: {}, 总余额: {:.2}, 平均余额: {:.2}, 平均年龄: {}]",
               self.total_users, self.adult_users, self.total_balance, self.average_balance, self.average_age)
    }
}

pub fn demo() {
    println!("=== 数据映射器模式演示 ===");

    // 1. 创建用户服务
    println!("\n1. 初始化用户服务:");
    let user_service = UserService::new();

    // 2. 创建用户
    println!("\n2. 创建用户:");
    let users_data = vec![
        ("张三", "zhangsan@example.com", "张三丰", 25),
        ("李四", "lisi@company.com", "李小四", 30),
        ("王五", "wangwu@test.com", "王老五", 17),
        ("赵六", "zhaoliu@demo.com", "赵小六", 45),
    ];

    let mut created_users = Vec::new();
    for (username, email, full_name, age) in users_data {
        match user_service.create_user(username.to_string(), email.to_string(), 
                                      full_name.to_string(), age) {
            Ok(user) => {
                println!("✓ 创建用户成功: {}", user);
                created_users.push(user);
            },
            Err(e) => println!("✗ 创建用户失败: {}", e),
        }
    }

    // 3. 业务操作演示
    println!("\n3. 业务操作演示:");
    
    // 存款操作
    if let Ok(user) = user_service.deposit(1, 1000.0) {
        println!("✓ 存款操作成功");
    }
    
    if let Ok(user) = user_service.deposit(2, 1500.0) {
        println!("✓ 存款操作成功");
    }

    // 取款操作
    if let Ok(user) = user_service.withdraw(1, 200.0) {
        println!("✓ 取款操作成功");
    }

    // 转账操作
    match user_service.transfer(1, 2, 300.0) {
        Ok((from_user, to_user)) => {
            println!("✓ 转账成功: {} -> {}", from_user.username, to_user.username);
        },
        Err(e) => println!("✗ 转账失败: {}", e),
    }

    // 4. 查询操作演示
    println!("\n4. 查询操作演示:");

    // 直接使用映射器查询
    let mapper = UserMapper::new();
    
    // 根据ID查找
    if let Ok(user) = mapper.find_by_id(1) {
        println!("根据ID找到用户: {}", user);
    }

    // 根据用户名查找
    if let Ok(user) = mapper.find_by_username("李四") {
        println!("根据用户名找到用户: {}", user);
    }

    // 根据年龄范围查找
    if let Ok(users) = mapper.find_by_age_range(20, 35) {
        println!("20-35岁用户:");
        for user in &users {
            println!("  - {}", user);
        }
    }

    // 根据余额范围查找
    if let Ok(users) = mapper.find_by_balance_range(500.0, 2000.0) {
        println!("余额 500-2000 的用户:");
        for user in &users {
            println!("  - {}", user);
        }
    }

    // 5. 业务服务查询演示
    println!("\n5. 业务服务查询演示:");

    // 查找成年用户
    if let Ok(adult_users) = user_service.find_adult_users() {
        println!("成年用户:");
        for user in &adult_users {
            println!("  - {}", user);
        }
    }

    // 查找富有用户
    if let Ok(wealthy_users) = user_service.find_wealthy_users(1000.0) {
        println!("富有用户 (余额 > 1000):");
        for user in &wealthy_users {
            println!("  - {}", user);
        }
    }

    // 获取统计信息
    if let Ok(stats) = user_service.get_user_statistics() {
        println!("用户统计信息: {}", stats);
    }

    // 6. 查找所有用户
    println!("\n6. 所有用户列表:");
    if let Ok(all_users) = mapper.find_all() {
        for user in &all_users {
            println!("  - {}", user);
        }
    }

    println!("\n数据映射器模式的优点:");
    println!("1. 将领域对象与数据库完全分离");
    println!("2. 领域对象专注于业务逻辑");
    println!("3. 数据映射器负责对象-关系映射");
    println!("4. 支持复杂的查询和映射逻辑");
    println!("5. 易于测试和维护");

    println!("\n适用场景:");
    println!("1. 复杂的领域模型");
    println!("2. 对象结构与数据库结构差异较大");
    println!("3. 需要复杂的查询逻辑");
    println!("4. 要求高度的关注点分离");
} 