// 文件路径: /d%3A/workspace/RustLearn/RustDesignPattern/src/EnterpriseAppPattern/ObjectRelationalMetadataMappingPatterns/repository.rs

//! # 仓储模式 (Repository)
//!
//! ## 概述
//! 仓储模式提供了一个更面向对象的视图来访问领域对象。
//! 它封装了访问数据所需的逻辑，并集中化了数据访问逻辑，
//! 使得数据访问架构更容易维护。
//!
//! ## 优点
//! - 集中化数据访问逻辑
//! - 提供了可测试的抽象层
//! - 支持多种数据源
//! - 便于单元测试（可以使用内存实现）
//! - 隔离了领域模型和数据映射层
//!
//! ## 适用场景
//! - 复杂的查询逻辑
//! - 需要支持多种数据源的应用
//! - 领域驱动设计(DDD)项目
//! - 需要良好测试覆盖率的系统

use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, Mutex};

/// 通用错误类型
#[derive(Debug)]
pub enum RepositoryError {
    NotFound(String),
    DatabaseError(String),
    ValidationError(String),
    DuplicateError(String),
}

impl fmt::Display for RepositoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RepositoryError::NotFound(msg) => write!(f, "未找到: {}", msg),
            RepositoryError::DatabaseError(msg) => write!(f, "数据库错误: {}", msg),
            RepositoryError::ValidationError(msg) => write!(f, "验证错误: {}", msg),
            RepositoryError::DuplicateError(msg) => write!(f, "重复错误: {}", msg),
        }
    }
}

impl std::error::Error for RepositoryError {}

/// 通用仓储接口
pub trait Repository<T, ID> {
    /// 根据ID查找实体
    fn find_by_id(&self, id: &ID) -> Result<Option<T>, RepositoryError>;
    
    /// 查找所有实体
    fn find_all(&self) -> Result<Vec<T>, RepositoryError>;
    
    /// 保存实体
    fn save(&self, entity: &T) -> Result<T, RepositoryError>;
    
    /// 删除实体
    fn delete(&self, id: &ID) -> Result<bool, RepositoryError>;
    
    /// 检查实体是否存在
    fn exists(&self, id: &ID) -> Result<bool, RepositoryError>;
    
    /// 获取实体总数
    fn count(&self) -> Result<usize, RepositoryError>;
}

/// 用户实体
#[derive(Debug, Clone, PartialEq)]
pub struct User {
    pub id: Option<u64>,
    pub username: String,
    pub email: String,
    pub full_name: String,
    pub age: u32,
    pub is_active: bool,
}

impl User {
    pub fn new(username: String, email: String, full_name: String, age: u32) -> Self {
        Self {
            id: None,
            username,
            email,
            full_name,
            age,
            is_active: true,
        }
    }

    pub fn with_id(mut self, id: u64) -> Self {
        self.id = Some(id);
        self
    }

    pub fn deactivate(&mut self) {
        self.is_active = false;
    }

    pub fn activate(&mut self) {
        self.is_active = true;
    }
}

impl fmt::Display for User {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "User[{}]: {} ({}) - {}", 
               self.id.unwrap_or(0), 
               self.username, 
               self.email, 
               if self.is_active { "活跃" } else { "非活跃" })
    }
}

/// 用户仓储接口
pub trait UserRepository: Repository<User, u64> {
    /// 根据用户名查找用户
    fn find_by_username(&self, username: &str) -> Result<Option<User>, RepositoryError>;
    
    /// 根据邮箱查找用户
    fn find_by_email(&self, email: &str) -> Result<Option<User>, RepositoryError>;
    
    /// 查找活跃用户
    fn find_active_users(&self) -> Result<Vec<User>, RepositoryError>;
    
    /// 根据年龄范围查找用户
    fn find_by_age_range(&self, min_age: u32, max_age: u32) -> Result<Vec<User>, RepositoryError>;
    
    /// 搜索用户（模糊匹配姓名或用户名）
    fn search_users(&self, keyword: &str) -> Result<Vec<User>, RepositoryError>;
}

/// 内存用户仓储实现
pub struct InMemoryUserRepository {
    users: Arc<Mutex<HashMap<u64, User>>>,
    next_id: Arc<Mutex<u64>>,
}

impl InMemoryUserRepository {
    pub fn new() -> Self {
        Self {
            users: Arc::new(Mutex::new(HashMap::new())),
            next_id: Arc::new(Mutex::new(1)),
        }
    }

    fn generate_id(&self) -> u64 {
        let mut next_id = self.next_id.lock().unwrap();
        let id = *next_id;
        *next_id += 1;
        id
    }

    fn validate_user(&self, user: &User) -> Result<(), RepositoryError> {
        if user.username.trim().is_empty() {
            return Err(RepositoryError::ValidationError("用户名不能为空".to_string()));
        }
        
        if user.email.trim().is_empty() || !user.email.contains('@') {
            return Err(RepositoryError::ValidationError("邮箱格式无效".to_string()));
        }
        
        if user.full_name.trim().is_empty() {
            return Err(RepositoryError::ValidationError("姓名不能为空".to_string()));
        }
        
        if user.age > 150 {
            return Err(RepositoryError::ValidationError("年龄不能超过150".to_string()));
        }
        
        Ok(())
    }
}

impl Repository<User, u64> for InMemoryUserRepository {
    fn find_by_id(&self, id: &u64) -> Result<Option<User>, RepositoryError> {
        let users = self.users.lock().unwrap();
        Ok(users.get(id).cloned())
    }

    fn find_all(&self) -> Result<Vec<User>, RepositoryError> {
        let users = self.users.lock().unwrap();
        Ok(users.values().cloned().collect())
    }

    fn save(&self, entity: &User) -> Result<User, RepositoryError> {
        self.validate_user(entity)?;
        
        let mut users = self.users.lock().unwrap();
        
        let mut user_to_save = entity.clone();
        
        if let Some(id) = user_to_save.id {
            // 更新现有用户
            if !users.contains_key(&id) {
                return Err(RepositoryError::NotFound(format!("用户ID: {}", id)));
            }
            users.insert(id, user_to_save.clone());
        } else {
            // 创建新用户
            // 检查用户名和邮箱是否已存在
            for existing_user in users.values() {
                if existing_user.username == user_to_save.username {
                    return Err(RepositoryError::DuplicateError(
                        format!("用户名 '{}' 已存在", user_to_save.username)
                    ));
                }
                if existing_user.email == user_to_save.email {
                    return Err(RepositoryError::DuplicateError(
                        format!("邮箱 '{}' 已存在", user_to_save.email)
                    ));
                }
            }
            
            let new_id = self.generate_id();
            user_to_save.id = Some(new_id);
            users.insert(new_id, user_to_save.clone());
        }
        
        Ok(user_to_save)
    }

    fn delete(&self, id: &u64) -> Result<bool, RepositoryError> {
        let mut users = self.users.lock().unwrap();
        Ok(users.remove(id).is_some())
    }

    fn exists(&self, id: &u64) -> Result<bool, RepositoryError> {
        let users = self.users.lock().unwrap();
        Ok(users.contains_key(id))
    }

    fn count(&self) -> Result<usize, RepositoryError> {
        let users = self.users.lock().unwrap();
        Ok(users.len())
    }
}

impl UserRepository for InMemoryUserRepository {
    fn find_by_username(&self, username: &str) -> Result<Option<User>, RepositoryError> {
        let users = self.users.lock().unwrap();
        Ok(users.values()
            .find(|user| user.username == username)
            .cloned())
    }

    fn find_by_email(&self, email: &str) -> Result<Option<User>, RepositoryError> {
        let users = self.users.lock().unwrap();
        Ok(users.values()
            .find(|user| user.email == email)
            .cloned())
    }

    fn find_active_users(&self) -> Result<Vec<User>, RepositoryError> {
        let users = self.users.lock().unwrap();
        Ok(users.values()
            .filter(|user| user.is_active)
            .cloned()
            .collect())
    }

    fn find_by_age_range(&self, min_age: u32, max_age: u32) -> Result<Vec<User>, RepositoryError> {
        let users = self.users.lock().unwrap();
        Ok(users.values()
            .filter(|user| user.age >= min_age && user.age <= max_age)
            .cloned()
            .collect())
    }

    fn search_users(&self, keyword: &str) -> Result<Vec<User>, RepositoryError> {
        let users = self.users.lock().unwrap();
        let keyword_lower = keyword.to_lowercase();
        
        Ok(users.values()
            .filter(|user| {
                user.username.to_lowercase().contains(&keyword_lower) ||
                user.full_name.to_lowercase().contains(&keyword_lower)
            })
            .cloned()
            .collect())
    }
}

/// 用户服务（使用仓储模式）
pub struct UserService {
    repository: Box<dyn UserRepository + Send + Sync>,
}

impl UserService {
    pub fn new(repository: Box<dyn UserRepository + Send + Sync>) -> Self {
        Self { repository }
    }

    /// 创建用户
    pub fn create_user(&self, username: String, email: String, full_name: String, age: u32) -> Result<User, RepositoryError> {
        let user = User::new(username, email, full_name, age);
        self.repository.save(&user)
    }

    /// 获取用户
    pub fn get_user(&self, id: u64) -> Result<User, RepositoryError> {
        self.repository.find_by_id(&id)?
            .ok_or_else(|| RepositoryError::NotFound(format!("用户ID: {}", id)))
    }

    /// 更新用户信息
    pub fn update_user(&self, id: u64, full_name: Option<String>, age: Option<u32>) -> Result<User, RepositoryError> {
        let mut user = self.get_user(id)?;
        
        if let Some(name) = full_name {
            user.full_name = name;
        }
        
        if let Some(new_age) = age {
            user.age = new_age;
        }
        
        self.repository.save(&user)
    }

    /// 停用用户
    pub fn deactivate_user(&self, id: u64) -> Result<User, RepositoryError> {
        let mut user = self.get_user(id)?;
        user.deactivate();
        self.repository.save(&user)
    }

    /// 激活用户
    pub fn activate_user(&self, id: u64) -> Result<User, RepositoryError> {
        let mut user = self.get_user(id)?;
        user.activate();
        self.repository.save(&user)
    }

    /// 删除用户
    pub fn delete_user(&self, id: u64) -> Result<bool, RepositoryError> {
        self.repository.delete(&id)
    }

    /// 获取所有活跃用户
    pub fn get_active_users(&self) -> Result<Vec<User>, RepositoryError> {
        self.repository.find_active_users()
    }

    /// 按年龄组获取用户统计
    pub fn get_age_group_statistics(&self) -> Result<HashMap<String, usize>, RepositoryError> {
        let users = self.repository.find_all()?;
        let mut stats = HashMap::new();
        
        for user in users {
            let age_group = match user.age {
                0..=17 => "未成年",
                18..=30 => "青年",
                31..=50 => "中年",
                51..=70 => "中老年",
                _ => "老年",
            };
            
            *stats.entry(age_group.to_string()).or_insert(0) += 1;
        }
        
        Ok(stats)
    }

    /// 搜索用户
    pub fn search_users(&self, keyword: &str) -> Result<Vec<User>, RepositoryError> {
        self.repository.search_users(keyword)
    }

    /// 获取用户总数
    pub fn get_total_users(&self) -> Result<usize, RepositoryError> {
        self.repository.count()
    }
}

/// 演示仓储模式
pub fn demo() {
    println!("=== 仓储模式演示 ===\n");
    
    // 创建仓储和服务
    let repository = Box::new(InMemoryUserRepository::new());
    let user_service = UserService::new(repository);
    
    println!("1. 创建用户");
    let users_data = vec![
        ("alice", "alice@example.com", "Alice Johnson", 25),
        ("bob", "bob@example.com", "Bob Smith", 32),
        ("charlie", "charlie@example.com", "Charlie Brown", 28),
        ("diana", "diana@example.com", "Diana Prince", 35),
        ("eve", "eve@example.com", "Eve Davis", 22),
    ];
    
    let mut created_users = Vec::new();
    for (username, email, full_name, age) in users_data {
        match user_service.create_user(username.to_string(), email.to_string(), full_name.to_string(), age) {
            Ok(user) => {
                println!("   ✅ 创建用户: {}", user);
                created_users.push(user);
            }
            Err(e) => println!("   ❌ 创建用户失败: {}", e),
        }
    }
    
    println!("\n2. 查询操作");
    
    // 根据ID查找
    if let Some(first_user) = created_users.first() {
        if let Some(user_id) = first_user.id {
            match user_service.get_user(user_id) {
                Ok(user) => println!("   根据ID查找: {}", user),
                Err(e) => println!("   查找失败: {}", e),
            }
        }
    }
    
    // 根据用户名查找
    match user_service.repository.find_by_username("alice") {
        Ok(Some(user)) => println!("   根据用户名查找: {}", user),
        Ok(None) => println!("   用户名未找到"),
        Err(e) => println!("   查找失败: {}", e),
    }
    
    // 根据邮箱查找
    match user_service.repository.find_by_email("bob@example.com") {
        Ok(Some(user)) => println!("   根据邮箱查找: {}", user),
        Ok(None) => println!("   邮箱未找到"),
        Err(e) => println!("   查找失败: {}", e),
    }
    
    println!("\n3. 高级查询");
    
    // 按年龄范围查找
    match user_service.repository.find_by_age_range(25, 35) {
        Ok(users) => {
            println!("   年龄25-35岁的用户:");
            for user in users {
                println!("     - {}", user);
            }
        }
        Err(e) => println!("   查询失败: {}", e),
    }
    
    // 搜索用户
    match user_service.search_users("Alice") {
        Ok(users) => {
            println!("   搜索'Alice'的结果:");
            for user in users {
                println!("     - {}", user);
            }
        }
        Err(e) => println!("   搜索失败: {}", e),
    }
    
    println!("\n4. 更新操作");
    if let Some(first_user) = created_users.first() {
        if let Some(user_id) = first_user.id {
            match user_service.update_user(user_id, Some("Alice Updated".to_string()), Some(26)) {
                Ok(user) => println!("   ✅ 更新用户: {}", user),
                Err(e) => println!("   ❌ 更新失败: {}", e),
            }
        }
    }
    
    println!("\n5. 停用和激活用户");
    if let Some(second_user) = created_users.get(1) {
        if let Some(user_id) = second_user.id {
            match user_service.deactivate_user(user_id) {
                Ok(user) => println!("   ✅ 停用用户: {}", user),
                Err(e) => println!("   ❌ 停用失败: {}", e),
            }
            
            match user_service.activate_user(user_id) {
                Ok(user) => println!("   ✅ 激活用户: {}", user),
                Err(e) => println!("   ❌ 激活失败: {}", e),
            }
        }
    }
    
    println!("\n6. 统计信息");
    match user_service.get_total_users() {
        Ok(count) => println!("   总用户数: {}", count),
        Err(e) => println!("   获取用户数失败: {}", e),
    }
    
    match user_service.get_age_group_statistics() {
        Ok(stats) => {
            println!("   年龄组统计:");
            for (group, count) in stats {
                println!("     - {}: {} 人", group, count);
            }
        }
        Err(e) => println!("   获取统计失败: {}", e),
    }
    
    println!("\n7. 删除操作");
    if let Some(last_user) = created_users.last() {
        if let Some(user_id) = last_user.id {
            match user_service.delete_user(user_id) {
                Ok(true) => println!("   ✅ 删除用户成功"),
                Ok(false) => println!("   ❌ 用户不存在"),
                Err(e) => println!("   ❌ 删除失败: {}", e),
            }
        }
    }
    
    // 最终用户数
    match user_service.get_total_users() {
        Ok(count) => println!("   删除后总用户数: {}", count),
        Err(e) => println!("   获取用户数失败: {}", e),
    }
    
    println!("\n=== 仓储模式演示完成 ===");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_creation_and_retrieval() {
        let repo = InMemoryUserRepository::new();
        let user = User::new("testuser".to_string(), "test@example.com".to_string(), "Test User".to_string(), 25);
        
        let saved_user = repo.save(&user).unwrap();
        assert!(saved_user.id.is_some());
        
        let retrieved = repo.find_by_id(&saved_user.id.unwrap()).unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().username, "testuser");
    }

    #[test]
    fn test_user_validation() {
        let repo = InMemoryUserRepository::new();
        let invalid_user = User::new("".to_string(), "invalid".to_string(), "".to_string(), 200);
        
        assert!(repo.save(&invalid_user).is_err());
    }

    #[test]
    fn test_duplicate_prevention() {
        let repo = InMemoryUserRepository::new();
        let user1 = User::new("duplicate".to_string(), "dup@example.com".to_string(), "User 1".to_string(), 25);
        let user2 = User::new("duplicate".to_string(), "other@example.com".to_string(), "User 2".to_string(), 30);
        
        assert!(repo.save(&user1).is_ok());
        assert!(repo.save(&user2).is_err()); // 应该失败，因为用户名重复
    }
} 