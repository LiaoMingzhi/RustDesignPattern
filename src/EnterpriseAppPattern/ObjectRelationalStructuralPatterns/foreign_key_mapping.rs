// 文件路径: /d%3A/workspace/RustLearn/RustDesignPattern/src/EnterpriseAppPattern/ObjectRelationalStructuralPatterns/foreign_key_mapping.rs

//! # 外键映射模式 (Foreign Key Mapping)
//!
//! ## 概述
//! 外键映射模式用于在对象中维护对其他对象的引用，
//! 通过在数据库表中使用外键来表示对象之间的关联关系。
//!
//! ## 优点
//! - 维护数据库的引用完整性
//! - 支持对象间的导航
//! - 提供高效的关联查询
//! - 符合关系数据库设计规范

use std::collections::HashMap;
use std::fmt;

/// 外键引用
#[derive(Debug)]
pub struct ForeignKey<T> {
    pub id: T,
    loaded_object: Option<Box<dyn std::any::Any>>,
}

impl<T: Clone> Clone for ForeignKey<T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            loaded_object: None, // 不克隆loaded_object，因为它是缓存
        }
    }
}

impl<T: PartialEq> PartialEq for ForeignKey<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<T: Clone> ForeignKey<T> {
    pub fn new(id: T) -> Self {
        Self {
            id,
            loaded_object: None,
        }
    }

    pub fn get_id(&self) -> &T {
        &self.id
    }
}

impl<T: fmt::Display> fmt::Display for ForeignKey<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "FK({})", self.id)
    }
}

/// 用户实体
#[derive(Debug, Clone)]
pub struct User {
    pub id: Option<u64>,
    pub username: String,
    pub email: String,
    pub full_name: String,
    pub department_id: Option<ForeignKey<u64>>, // 外键引用部门
}

impl User {
    pub fn new(username: String, email: String, full_name: String) -> Self {
        Self {
            id: None,
            username,
            email,
            full_name,
            department_id: None,
        }
    }

    pub fn set_department(&mut self, department_id: u64) {
        self.department_id = Some(ForeignKey::new(department_id));
    }

    pub fn get_department_id(&self) -> Option<u64> {
        self.department_id.as_ref().map(|fk| fk.id)
    }
}

impl fmt::Display for User {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let id_str = self.id.map_or("NEW".to_string(), |id| id.to_string());
        let dept_str = self.department_id.as_ref()
            .map_or("无".to_string(), |fk| fk.to_string());
        write!(f, "User[{}]: {} - 部门: {}", id_str, self.full_name, dept_str)
    }
}

/// 部门实体
#[derive(Debug, Clone)]
pub struct Department {
    pub id: Option<u64>,
    pub name: String,
    pub description: String,
    pub manager_id: Option<ForeignKey<u64>>, // 外键引用经理
}

impl Department {
    pub fn new(name: String, description: String) -> Self {
        Self {
            id: None,
            name,
            description,
            manager_id: None,
        }
    }

    pub fn set_manager(&mut self, manager_id: u64) {
        self.manager_id = Some(ForeignKey::new(manager_id));
    }

    pub fn get_manager_id(&self) -> Option<u64> {
        self.manager_id.as_ref().map(|fk| fk.id)
    }
}

impl fmt::Display for Department {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let id_str = self.id.map_or("NEW".to_string(), |id| id.to_string());
        let manager_str = self.manager_id.as_ref()
            .map_or("无".to_string(), |fk| fk.to_string());
        write!(f, "Department[{}]: {} - 经理: {}", id_str, self.name, manager_str)
    }
}

/// 订单实体
#[derive(Debug, Clone)]
pub struct Order {
    pub id: Option<u64>,
    pub order_number: String,
    pub user_id: ForeignKey<u64>, // 必需的外键引用用户
    pub total_amount: f64,
    pub status: String,
    pub created_at: u64,
}

impl Order {
    pub fn new(order_number: String, user_id: u64, total_amount: f64) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            id: None,
            order_number,
            user_id: ForeignKey::new(user_id),
            total_amount,
            status: "PENDING".to_string(),
            created_at: now,
        }
    }

    pub fn get_user_id(&self) -> u64 {
        self.user_id.id
    }
}

impl fmt::Display for Order {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let id_str = self.id.map_or("NEW".to_string(), |id| id.to_string());
        write!(f, "Order[{}]: {} - 用户: {} - 金额: {:.2}", 
               id_str, self.order_number, self.user_id, self.total_amount)
    }
}

/// 订单项实体
#[derive(Debug, Clone)]
pub struct OrderItem {
    pub id: Option<u64>,
    pub order_id: ForeignKey<u64>, // 外键引用订单
    pub product_name: String,
    pub quantity: u32,
    pub unit_price: f64,
    pub total_price: f64,
}

impl OrderItem {
    pub fn new(order_id: u64, product_name: String, quantity: u32, unit_price: f64) -> Self {
        let total_price = quantity as f64 * unit_price;
        
        Self {
            id: None,
            order_id: ForeignKey::new(order_id),
            product_name,
            quantity,
            unit_price,
            total_price,
        }
    }

    pub fn get_order_id(&self) -> u64 {
        self.order_id.id
    }
}

impl fmt::Display for OrderItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let id_str = self.id.map_or("NEW".to_string(), |id| id.to_string());
        write!(f, "OrderItem[{}]: {} x{} - 订单: {} - 总价: {:.2}", 
               id_str, self.product_name, self.quantity, self.order_id, self.total_price)
    }
}

/// 外键映射管理器
pub struct ForeignKeyMappingManager {
    users: HashMap<u64, User>,
    departments: HashMap<u64, Department>,
    orders: HashMap<u64, Order>,
    order_items: HashMap<u64, OrderItem>,
    next_user_id: u64,
    next_dept_id: u64,
    next_order_id: u64,
    next_item_id: u64,
}

impl ForeignKeyMappingManager {
    pub fn new() -> Self {
        Self {
            users: HashMap::new(),
            departments: HashMap::new(),
            orders: HashMap::new(),
            order_items: HashMap::new(),
            next_user_id: 1,
            next_dept_id: 1,
            next_order_id: 1,
            next_item_id: 1,
        }
    }

    /// 保存部门
    pub fn save_department(&mut self, mut department: Department) -> Result<Department, String> {
        // 验证经理是否存在
        if let Some(manager_id) = department.get_manager_id() {
            if !self.users.contains_key(&manager_id) {
                return Err(format!("经理ID {} 不存在", manager_id));
            }
        }

        if department.id.is_none() {
            department.id = Some(self.next_dept_id);
            self.next_dept_id += 1;
        }

        let id = department.id.unwrap();
        self.departments.insert(id, department.clone());
        Ok(department)
    }

    /// 保存用户
    pub fn save_user(&mut self, mut user: User) -> Result<User, String> {
        // 验证部门是否存在
        if let Some(dept_id) = user.get_department_id() {
            if !self.departments.contains_key(&dept_id) {
                return Err(format!("部门ID {} 不存在", dept_id));
            }
        }

        if user.id.is_none() {
            user.id = Some(self.next_user_id);
            self.next_user_id += 1;
        }

        let id = user.id.unwrap();
        self.users.insert(id, user.clone());
        Ok(user)
    }

    /// 保存订单
    pub fn save_order(&mut self, mut order: Order) -> Result<Order, String> {
        // 验证用户是否存在
        let user_id = order.get_user_id();
        if !self.users.contains_key(&user_id) {
            return Err(format!("用户ID {} 不存在", user_id));
        }

        if order.id.is_none() {
            order.id = Some(self.next_order_id);
            self.next_order_id += 1;
        }

        let id = order.id.unwrap();
        self.orders.insert(id, order.clone());
        Ok(order)
    }

    /// 保存订单项
    pub fn save_order_item(&mut self, mut item: OrderItem) -> Result<OrderItem, String> {
        // 验证订单是否存在
        let order_id = item.get_order_id();
        if !self.orders.contains_key(&order_id) {
            return Err(format!("订单ID {} 不存在", order_id));
        }

        if item.id.is_none() {
            item.id = Some(self.next_item_id);
            self.next_item_id += 1;
        }

        let id = item.id.unwrap();
        self.order_items.insert(id, item.clone());
        Ok(item)
    }

    /// 获取用户及其部门信息
    pub fn get_user_with_department(&self, user_id: u64) -> Option<(User, Option<Department>)> {
        if let Some(user) = self.users.get(&user_id) {
            let department = user.get_department_id()
                .and_then(|dept_id| self.departments.get(&dept_id))
                .cloned();
            Some((user.clone(), department))
        } else {
            None
        }
    }

    /// 获取部门及其经理信息
    pub fn get_department_with_manager(&self, dept_id: u64) -> Option<(Department, Option<User>)> {
        if let Some(department) = self.departments.get(&dept_id) {
            let manager = department.get_manager_id()
                .and_then(|manager_id| self.users.get(&manager_id))
                .cloned();
            Some((department.clone(), manager))
        } else {
            None
        }
    }

    /// 获取用户的所有订单
    pub fn get_orders_by_user(&self, user_id: u64) -> Vec<Order> {
        self.orders.values()
            .filter(|order| order.get_user_id() == user_id)
            .cloned()
            .collect()
    }

    /// 获取订单及其所有订单项
    pub fn get_order_with_items(&self, order_id: u64) -> Option<(Order, Vec<OrderItem>)> {
        if let Some(order) = self.orders.get(&order_id) {
            let items: Vec<OrderItem> = self.order_items.values()
                .filter(|item| item.get_order_id() == order_id)
                .cloned()
                .collect();
            Some((order.clone(), items))
        } else {
            None
        }
    }

    /// 获取部门的所有用户
    pub fn get_users_by_department(&self, dept_id: u64) -> Vec<User> {
        self.users.values()
            .filter(|user| user.get_department_id() == Some(dept_id))
            .cloned()
            .collect()
    }

    /// 删除实体（带级联检查）
    pub fn delete_user(&mut self, user_id: u64) -> Result<bool, String> {
        // 检查是否有订单引用该用户
        let has_orders = self.orders.values()
            .any(|order| order.get_user_id() == user_id);
        
        if has_orders {
            return Err("无法删除用户，存在关联的订单".to_string());
        }

        // 检查是否是某个部门的经理
        let is_manager = self.departments.values()
            .any(|dept| dept.get_manager_id() == Some(user_id));
        
        if is_manager {
            return Err("无法删除用户，该用户是某个部门的经理".to_string());
        }

        Ok(self.users.remove(&user_id).is_some())
    }

    /// 删除部门（带级联检查）
    pub fn delete_department(&mut self, dept_id: u64) -> Result<bool, String> {
        // 检查是否有用户属于该部门
        let has_users = self.users.values()
            .any(|user| user.get_department_id() == Some(dept_id));
        
        if has_users {
            return Err("无法删除部门，存在属于该部门的用户".to_string());
        }

        Ok(self.departments.remove(&dept_id).is_some())
    }

    /// 获取统计信息
    pub fn get_statistics(&self) -> HashMap<String, usize> {
        let mut stats = HashMap::new();
        stats.insert("用户数量".to_string(), self.users.len());
        stats.insert("部门数量".to_string(), self.departments.len());
        stats.insert("订单数量".to_string(), self.orders.len());
        stats.insert("订单项数量".to_string(), self.order_items.len());
        stats
    }
}

/// 演示外键映射模式
pub fn demo() {
    println!("=== 外键映射模式演示 ===\n");

    let mut manager = ForeignKeyMappingManager::new();

    println!("1. 创建部门");
    let it_dept = Department::new("IT部".to_string(), "信息技术部门".to_string());
    let hr_dept = Department::new("HR部".to_string(), "人力资源部门".to_string());

    let it_dept = manager.save_department(it_dept).unwrap();
    let hr_dept = manager.save_department(hr_dept).unwrap();
    
    println!("   ✅ 创建部门: {}", it_dept);
    println!("   ✅ 创建部门: {}", hr_dept);

    println!("\n2. 创建用户");
    let mut alice = User::new("alice".to_string(), "alice@company.com".to_string(), "Alice Johnson".to_string());
    alice.set_department(it_dept.id.unwrap());
    
    let mut bob = User::new("bob".to_string(), "bob@company.com".to_string(), "Bob Smith".to_string());
    bob.set_department(it_dept.id.unwrap());
    
    let mut carol = User::new("carol".to_string(), "carol@company.com".to_string(), "Carol Brown".to_string());
    carol.set_department(hr_dept.id.unwrap());

    let alice = manager.save_user(alice).unwrap();
    let bob = manager.save_user(bob).unwrap();
    let carol = manager.save_user(carol).unwrap();

    println!("   ✅ 创建用户: {}", alice);
    println!("   ✅ 创建用户: {}", bob);
    println!("   ✅ 创建用户: {}", carol);

    println!("\n3. 设置部门经理");
    let mut updated_it_dept = it_dept.clone();
    updated_it_dept.set_manager(alice.id.unwrap());
    let updated_it_dept = manager.save_department(updated_it_dept).unwrap();
    println!("   ✅ 设置IT部经理: {}", updated_it_dept);

    println!("\n4. 创建订单");
    let order1 = Order::new("ORD-001".to_string(), alice.id.unwrap(), 1299.99);
    let order2 = Order::new("ORD-002".to_string(), bob.id.unwrap(), 899.50);

    let order1 = manager.save_order(order1).unwrap();
    let order2 = manager.save_order(order2).unwrap();

    println!("   ✅ 创建订单: {}", order1);
    println!("   ✅ 创建订单: {}", order2);

    println!("\n5. 创建订单项");
    let item1 = OrderItem::new(order1.id.unwrap(), "笔记本电脑".to_string(), 1, 1299.99);
    let item2 = OrderItem::new(order2.id.unwrap(), "显示器".to_string(), 1, 599.50);
    let item3 = OrderItem::new(order2.id.unwrap(), "键盘".to_string(), 1, 300.00);

    let item1 = manager.save_order_item(item1).unwrap();
    let item2 = manager.save_order_item(item2).unwrap();
    let item3 = manager.save_order_item(item3).unwrap();

    println!("   ✅ 创建订单项: {}", item1);
    println!("   ✅ 创建订单项: {}", item2);
    println!("   ✅ 创建订单项: {}", item3);

    println!("\n6. 关联查询演示");
    
    // 用户及其部门
    if let Some((user, department)) = manager.get_user_with_department(alice.id.unwrap()) {
        println!("   用户及部门: {} 属于 {}", user.full_name, 
                 department.map_or("无部门".to_string(), |d| d.name));
    }

    // 部门及其经理
    if let Some((department, manager_user)) = manager.get_department_with_manager(updated_it_dept.id.unwrap()) {
        println!("   部门及经理: {} 的经理是 {}", department.name,
                 manager_user.map_or("无经理".to_string(), |u| u.full_name));
    }

    // 用户的订单
    let alice_orders = manager.get_orders_by_user(alice.id.unwrap());
    println!("   Alice的订单数: {}", alice_orders.len());
    for order in alice_orders {
        println!("     - {}", order);
    }

    // 订单及其订单项
    if let Some((order, items)) = manager.get_order_with_items(order2.id.unwrap()) {
        println!("   订单及订单项: {} 包含 {} 个项目", order.order_number, items.len());
        for item in items {
            println!("     - {}", item);
        }
    }

    // 部门的用户
    let it_users = manager.get_users_by_department(updated_it_dept.id.unwrap());
    println!("   IT部用户数: {}", it_users.len());
    for user in it_users {
        println!("     - {}", user.full_name);
    }

    println!("\n7. 引用完整性验证");
    
    // 尝试创建引用不存在用户的订单
    let invalid_order = Order::new("ORD-999".to_string(), 999, 100.0);
    match manager.save_order(invalid_order) {
        Ok(_) => println!("   ❌ 意外成功"),
        Err(e) => println!("   ✅ 正确阻止: {}", e),
    }

    // 尝试删除有订单的用户
    match manager.delete_user(alice.id.unwrap()) {
        Ok(_) => println!("   ❌ 意外成功"),
        Err(e) => println!("   ✅ 正确阻止: {}", e),
    }

    // 尝试删除有用户的部门
    match manager.delete_department(updated_it_dept.id.unwrap()) {
        Ok(_) => println!("   ❌ 意外成功"),
        Err(e) => println!("   ✅ 正确阻止: {}", e),
    }

    println!("\n8. 统计信息");
    let stats = manager.get_statistics();
    for (key, value) in stats {
        println!("   {}: {}", key, value);
    }

    println!("\n=== 外键映射模式演示完成 ===");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_foreign_key_creation() {
        let fk = ForeignKey::new(123u64);
        assert_eq!(fk.get_id(), &123u64);
    }

    #[test]
    fn test_user_department_relationship() {
        let mut user = User::new("test".to_string(), "test@example.com".to_string(), "Test User".to_string());
        assert_eq!(user.get_department_id(), None);
        
        user.set_department(1);
        assert_eq!(user.get_department_id(), Some(1));
    }

    #[test]
    fn test_foreign_key_validation() {
        let mut manager = ForeignKeyMappingManager::new();
        
        // 创建用户引用不存在的部门应该失败
        let mut user = User::new("test".to_string(), "test@example.com".to_string(), "Test User".to_string());
        user.set_department(999);
        
        assert!(manager.save_user(user).is_err());
    }

    #[test]
    fn test_cascade_delete_prevention() {
        let mut manager = ForeignKeyMappingManager::new();
        
        // 创建部门和用户
        let dept = Department::new("Test Dept".to_string(), "Test".to_string());
        let dept = manager.save_department(dept).unwrap();
        
        let mut user = User::new("test".to_string(), "test@example.com".to_string(), "Test User".to_string());
        user.set_department(dept.id.unwrap());
        let user = manager.save_user(user).unwrap();
        
        // 尝试删除有用户的部门应该失败
        assert!(manager.delete_department(dept.id.unwrap()).is_err());
    }
} 