//! 行数据入口模式 (Row Data Gateway)
//! 
//! 一个对象作为单条记录的入口，一个实例对应一行数据。
//! 文件路径：/d%3A/workspace/RustLearn/RustDesignPattern/src/EnterpriseAppPattern/DataSourceArchitecturalPatterns/row_data_gateway.rs

use std::collections::HashMap;
use std::fmt;
use std::sync::{Mutex, OnceLock};

// 简化的数据库错误类型
#[derive(Debug)]
pub enum DatabaseError {
    NotFound,
    ValidationError(String),
}

impl fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DatabaseError::NotFound => write!(f, "记录未找到"),
            DatabaseError::ValidationError(msg) => write!(f, "验证错误: {}", msg),
        }
    }
}

// 使用线程安全的全局存储
static USER_STORAGE: OnceLock<Mutex<HashMap<u32, HashMap<String, String>>>> = OnceLock::new();
static NEXT_USER_ID: OnceLock<Mutex<u32>> = OnceLock::new();

fn get_user_storage() -> &'static Mutex<HashMap<u32, HashMap<String, String>>> {
    USER_STORAGE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn get_next_user_id() -> u32 {
    let next_id_mutex = NEXT_USER_ID.get_or_init(|| Mutex::new(1));
    let mut next_id = next_id_mutex.lock().unwrap();
    let id = *next_id;
    *next_id += 1;
    id
}

// 用户行数据入口
#[derive(Debug, Clone)]
pub struct UserRowDataGateway {
    id: Option<u32>,
    username: String,
    email: String,
    created_at: String,
    is_dirty: bool,
}

impl UserRowDataGateway {
    // 创建新用户
    pub fn new(username: String, email: String) -> Self {
        println!("创建新的用户行数据入口: username={}, email={}", username, email);
        Self {
            id: None,
            username,
            email,
            created_at: "2024-01-01 00:00:00".to_string(),
            is_dirty: true,
        }
    }

    // 从数据库加载用户
    pub fn load(id: u32) -> Result<Self, DatabaseError> {
        let storage = get_user_storage();
        let storage_guard = storage.lock().unwrap();
        
        if let Some(user_data) = storage_guard.get(&id) {
            let user = Self {
                id: Some(id),
                username: user_data.get("username").unwrap_or(&String::new()).clone(),
                email: user_data.get("email").unwrap_or(&String::new()).clone(),
                created_at: user_data.get("created_at").unwrap_or(&String::new()).clone(),
                is_dirty: false,
            };
            println!("从数据库加载用户: {}", user);
            Ok(user)
        } else {
            Err(DatabaseError::NotFound)
        }
    }

    // 保存到数据库
    pub fn save(&mut self) -> Result<(), DatabaseError> {
        // 验证数据
        if self.username.is_empty() {
            return Err(DatabaseError::ValidationError("用户名不能为空".to_string()));
        }
        if self.email.is_empty() {
            return Err(DatabaseError::ValidationError("邮箱不能为空".to_string()));
        }

        let storage = get_user_storage();
        let mut storage_guard = storage.lock().unwrap();

        match self.id {
            Some(id) => {
                // 更新现有用户
                let mut user_data = HashMap::new();
                user_data.insert("username".to_string(), self.username.clone());
                user_data.insert("email".to_string(), self.email.clone());
                user_data.insert("created_at".to_string(), self.created_at.clone());
                
                storage_guard.insert(id, user_data);
                println!("更新用户到数据库: {}", self);
            },
            None => {
                // 插入新用户
                let new_id = get_next_user_id();
                self.id = Some(new_id);
                
                let mut user_data = HashMap::new();
                user_data.insert("username".to_string(), self.username.clone());
                user_data.insert("email".to_string(), self.email.clone());
                user_data.insert("created_at".to_string(), self.created_at.clone());
                
                storage_guard.insert(new_id, user_data);
                println!("插入新用户到数据库: {}", self);
            }
        }

        self.is_dirty = false;
        Ok(())
    }

    // 删除用户
    pub fn delete(&mut self) -> Result<(), DatabaseError> {
        if let Some(id) = self.id {
            let storage = get_user_storage();
            let mut storage_guard = storage.lock().unwrap();
            
            if storage_guard.remove(&id).is_some() {
                println!("从数据库删除用户: {}", self);
                self.id = None;
                Ok(())
            } else {
                Err(DatabaseError::NotFound)
            }
        } else {
            Err(DatabaseError::ValidationError("不能删除未保存的用户".to_string()))
        }
    }

    // Getters
    pub fn id(&self) -> Option<u32> {
        self.id
    }

    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn email(&self) -> &str {
        &self.email
    }

    // Setters
    pub fn set_username(&mut self, username: String) {
        if self.username != username {
            self.username = username;
            self.is_dirty = true;
            println!("用户名已修改，标记为dirty");
        }
    }

    pub fn set_email(&mut self, email: String) {
        if self.email != email {
            self.email = email;
            self.is_dirty = true;
            println!("邮箱已修改，标记为dirty");
        }
    }

    // 状态检查
    pub fn is_dirty(&self) -> bool {
        self.is_dirty
    }

    pub fn is_new(&self) -> bool {
        self.id.is_none()
    }
}

impl fmt::Display for UserRowDataGateway {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "UserRow[id={:?}, username={}, email={}, dirty={}]", 
               self.id, self.username, self.email, self.is_dirty)
    }
}

pub fn demo() {
    println!("=== 行数据入口模式演示 ===");

    // 1. 创建新用户
    println!("\n1. 创建新用户:");
    let mut user1 = UserRowDataGateway::new("张三".to_string(), "zhangsan@example.com".to_string());
    let mut user2 = UserRowDataGateway::new("李四".to_string(), "lisi@company.com".to_string());

    println!("用户1状态: 新建={}, 脏数据={}", user1.is_new(), user1.is_dirty());

    // 2. 保存用户
    println!("\n2. 保存用户:");
    match user1.save() {
        Ok(_) => println!("✓ 用户1保存成功"),
        Err(e) => println!("✗ 用户1保存失败: {}", e),
    }

    match user2.save() {
        Ok(_) => println!("✓ 用户2保存成功"),
        Err(e) => println!("✗ 用户2保存失败: {}", e),
    }

    println!("用户1状态: 新建={}, 脏数据={}", user1.is_new(), user1.is_dirty());

    // 3. 修改用户信息
    println!("\n3. 修改用户信息:");
    user1.set_email("zhangsan_new@example.com".to_string());
    println!("用户1状态: 脏数据={}", user1.is_dirty());

    // 4. 保存修改
    match user1.save() {
        Ok(_) => println!("✓ 用户1更新成功"),
        Err(e) => println!("✗ 用户1更新失败: {}", e),
    }

    // 5. 从数据库加载用户
    println!("\n5. 从数据库加载用户:");
    match UserRowDataGateway::load(1) {
        Ok(loaded_user) => {
            println!("✓ 加载成功: {}", loaded_user);
            println!("用户邮箱: {}", loaded_user.email());
        },
        Err(e) => println!("✗ 加载失败: {}", e),
    }

    // 6. 删除操作
    println!("\n6. 删除操作演示:");
    let mut temp_user = UserRowDataGateway::new("临时用户".to_string(), "temp@example.com".to_string());
    temp_user.save().ok();
    
    match temp_user.delete() {
        Ok(_) => println!("✓ 用户删除成功"),
        Err(e) => println!("✗ 用户删除失败: {}", e),
    }

    println!("\n行数据入口模式的优点:");
    println!("1. 每个对象代表一行数据，概念简单清晰");
    println!("2. 包含行级的业务逻辑和验证");
    println!("3. 支持脏数据跟踪");
    println!("4. 便于实现对象状态管理");
    println!("5. 对象状态与数据库状态直接对应");

    println!("\n适用场景:");
    println!("1. 行级业务逻辑较多的情况");
    println!("2. 需要跟踪对象状态变化");
    println!("3. 复杂的对象生命周期管理");
} 