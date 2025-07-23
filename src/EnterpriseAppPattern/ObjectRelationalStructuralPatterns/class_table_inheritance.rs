//! # 类表继承模式（Class Table Inheritance Pattern）
//!
//! 类表继承模式为继承层次结构中的每个类都创建一个数据库表。
//! 每个子类表只包含该类特有的字段，通过外键关联到父类表。
//! 这种方式保持了数据的规范化，避免了数据冗余。
//!
//! ## 模式特点
//! - **数据规范化**: 每个表只存储相关类的数据
//! - **无冗余**: 避免空值和重复数据
//! - **扩展性好**: 添加新子类时影响最小
//! - **查询复杂**: 需要连接多个表才能获取完整对象
//!
//! ## 使用场景
//! - 继承层次复杂且子类差异较大时
//! - 需要严格数据规范化时
//! - 子类字段较多且独特时
//! - 对存储空间有严格要求时

use std::collections::HashMap;
use std::fmt::{self, Display, Formatter};
use std::error::Error;

/// 类表继承错误类型
#[derive(Debug)]
pub enum ClassTableInheritanceError {
    RecordNotFound(String),
    InvalidType(String),
    DatabaseError(String),
    ValidationError(String),
    IntegrityConstraintViolation(String),
}

impl Display for ClassTableInheritanceError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ClassTableInheritanceError::RecordNotFound(msg) => write!(f, "记录未找到: {}", msg),
            ClassTableInheritanceError::InvalidType(msg) => write!(f, "无效类型: {}", msg),
            ClassTableInheritanceError::DatabaseError(msg) => write!(f, "数据库错误: {}", msg),
            ClassTableInheritanceError::ValidationError(msg) => write!(f, "验证错误: {}", msg),
            ClassTableInheritanceError::IntegrityConstraintViolation(msg) => write!(f, "完整性约束违反: {}", msg),
        }
    }
}

impl Error for ClassTableInheritanceError {}

/// 实体类型枚举
#[derive(Debug, Clone, PartialEq)]
pub enum EntityType {
    Person,
    Employee,
    Customer,
    Supplier,
}

impl Display for EntityType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let type_str = match self {
            EntityType::Person => "Person",
            EntityType::Employee => "Employee", 
            EntityType::Customer => "Customer",
            EntityType::Supplier => "Supplier",
        };
        write!(f, "{}", type_str)
    }
}

/// 基础实体 trait
pub trait Entity {
    fn get_id(&self) -> i64;
    fn get_type(&self) -> EntityType;
    fn validate(&self) -> Result<(), ClassTableInheritanceError>;
}

/// 人员基类（对应 persons 表）
#[derive(Debug, Clone)]
pub struct Person {
    pub id: i64,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub phone: String,
    pub birth_date: String,
    pub created_at: String,
}

impl Person {
    pub fn new(id: i64, first_name: String, last_name: String, email: String) -> Self {
        Self {
            id,
            first_name,
            last_name,
            email,
            phone: String::new(),
            birth_date: String::new(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
        }
    }

    pub fn full_name(&self) -> String {
        format!("{} {}", self.first_name, self.last_name)
    }
}

impl Entity for Person {
    fn get_id(&self) -> i64 {
        self.id
    }

    fn get_type(&self) -> EntityType {
        EntityType::Person
    }

    fn validate(&self) -> Result<(), ClassTableInheritanceError> {
        if self.first_name.trim().is_empty() {
            return Err(ClassTableInheritanceError::ValidationError("名字不能为空".to_string()));
        }
        if self.last_name.trim().is_empty() {
            return Err(ClassTableInheritanceError::ValidationError("姓氏不能为空".to_string()));
        }
        if self.email.trim().is_empty() {
            return Err(ClassTableInheritanceError::ValidationError("邮箱不能为空".to_string()));
        }
        Ok(())
    }
}

impl Display for Person {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Person[{}] - {}, Email: {}, Phone: {}", 
               self.id, self.full_name(), self.email, self.phone)
    }
}

/// 员工类（对应 employees 表）
#[derive(Debug, Clone)]
pub struct Employee {
    pub person: Person,           // 基类数据
    pub employee_id: String,      // 员工编号
    pub department: String,       // 部门
    pub position: String,         // 职位
    pub hire_date: String,        // 入职日期
    pub salary: f64,             // 薪资
    pub manager_id: Option<i64>,  // 上级经理ID
    pub status: EmployeeStatus,   // 员工状态
}

#[derive(Debug, Clone, PartialEq)]
pub enum EmployeeStatus {
    Active,
    Inactive,
    Terminated,
    OnLeave,
}

impl Employee {
    pub fn new(person: Person, employee_id: String, department: String, position: String) -> Self {
        Self {
            person,
            employee_id,
            department,
            position,
            hire_date: "2024-01-01".to_string(),
            salary: 0.0,
            manager_id: None,
            status: EmployeeStatus::Active,
        }
    }

    pub fn get_annual_salary(&self) -> f64 {
        self.salary * 12.0
    }

    pub fn update_salary(&mut self, new_salary: f64) -> Result<(), ClassTableInheritanceError> {
        if new_salary < 0.0 {
            return Err(ClassTableInheritanceError::ValidationError("薪资不能为负数".to_string()));
        }
        self.salary = new_salary;
        Ok(())
    }

    pub fn assign_manager(&mut self, manager_id: i64) {
        self.manager_id = Some(manager_id);
    }

    pub fn promote(&mut self, new_position: String, new_salary: f64) -> Result<(), ClassTableInheritanceError> {
        self.position = new_position;
        self.update_salary(new_salary)?;
        Ok(())
    }
}

impl Entity for Employee {
    fn get_id(&self) -> i64 {
        self.person.id
    }

    fn get_type(&self) -> EntityType {
        EntityType::Employee
    }

    fn validate(&self) -> Result<(), ClassTableInheritanceError> {
        self.person.validate()?;
        if self.employee_id.trim().is_empty() {
            return Err(ClassTableInheritanceError::ValidationError("员工编号不能为空".to_string()));
        }
        if self.department.trim().is_empty() {
            return Err(ClassTableInheritanceError::ValidationError("部门不能为空".to_string()));
        }
        if self.position.trim().is_empty() {
            return Err(ClassTableInheritanceError::ValidationError("职位不能为空".to_string()));
        }
        if self.salary < 0.0 {
            return Err(ClassTableInheritanceError::ValidationError("薪资不能为负数".to_string()));
        }
        Ok(())
    }
}

impl Display for Employee {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Employee[{}] - {}, ID: {}, Dept: {}, Position: {}, Salary: {:.2}", 
               self.person.id, self.person.full_name(), self.employee_id, 
               self.department, self.position, self.salary)
    }
}

/// 客户类（对应 customers 表）
#[derive(Debug, Clone)]
pub struct Customer {
    pub person: Person,              // 基类数据
    pub customer_number: String,     // 客户编号
    pub registration_date: String,   // 注册日期
    pub credit_limit: f64,           // 信用额度
    pub credit_rating: CreditRating, // 信用评级
    pub preferred_contact: String,   // 首选联系方式
    pub loyalty_points: i32,         // 忠诚度积分
    pub status: CustomerStatus,      // 客户状态
}

#[derive(Debug, Clone, PartialEq)]
pub enum CreditRating {
    Excellent,
    Good,
    Fair,
    Poor,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CustomerStatus {
    Active,
    Inactive,
    Suspended,
    Blacklisted,
}

impl Customer {
    pub fn new(person: Person, customer_number: String) -> Self {
        Self {
            person,
            customer_number,
            registration_date: "2024-01-01".to_string(),
            credit_limit: 5000.0,
            credit_rating: CreditRating::Good,
            preferred_contact: "email".to_string(),
            loyalty_points: 0,
            status: CustomerStatus::Active,
        }
    }

    pub fn update_credit_limit(&mut self, new_limit: f64) -> Result<(), ClassTableInheritanceError> {
        if new_limit < 0.0 {
            return Err(ClassTableInheritanceError::ValidationError("信用额度不能为负数".to_string()));
        }
        self.credit_limit = new_limit;
        Ok(())
    }

    pub fn add_loyalty_points(&mut self, points: i32) {
        self.loyalty_points += points;
    }

    pub fn redeem_points(&mut self, points: i32) -> Result<(), ClassTableInheritanceError> {
        if points > self.loyalty_points {
            return Err(ClassTableInheritanceError::ValidationError("积分不足".to_string()));
        }
        self.loyalty_points -= points;
        Ok(())
    }

    pub fn upgrade_credit_rating(&mut self) {
        self.credit_rating = match self.credit_rating {
            CreditRating::Poor => CreditRating::Fair,
            CreditRating::Fair => CreditRating::Good,
            CreditRating::Good => CreditRating::Excellent,
            CreditRating::Excellent => CreditRating::Excellent,
        };
    }
}

impl Entity for Customer {
    fn get_id(&self) -> i64 {
        self.person.id
    }

    fn get_type(&self) -> EntityType {
        EntityType::Customer
    }

    fn validate(&self) -> Result<(), ClassTableInheritanceError> {
        self.person.validate()?;
        if self.customer_number.trim().is_empty() {
            return Err(ClassTableInheritanceError::ValidationError("客户编号不能为空".to_string()));
        }
        if self.credit_limit < 0.0 {
            return Err(ClassTableInheritanceError::ValidationError("信用额度不能为负数".to_string()));
        }
        if self.loyalty_points < 0 {
            return Err(ClassTableInheritanceError::ValidationError("忠诚度积分不能为负数".to_string()));
        }
        Ok(())
    }
}

impl Display for Customer {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Customer[{}] - {}, Number: {}, Credit: {:.2}, Rating: {:?}, Points: {}", 
               self.person.id, self.person.full_name(), self.customer_number, 
               self.credit_limit, self.credit_rating, self.loyalty_points)
    }
}

/// 供应商类（对应 suppliers 表）
#[derive(Debug, Clone)]
pub struct Supplier {
    pub person: Person,           // 基类数据
    pub supplier_code: String,    // 供应商代码
    pub company_name: String,     // 公司名称
    pub business_type: String,    // 业务类型
    pub tax_id: String,           // 税务ID
    pub credit_terms: i32,        // 信用期限（天）
    pub payment_terms: String,    // 付款条件
    pub quality_rating: f64,      // 质量评级 (0.0-5.0)
    pub status: SupplierStatus,   // 供应商状态
}

#[derive(Debug, Clone, PartialEq)]
pub enum SupplierStatus {
    Active,
    Inactive,
    Suspended,
    Blacklisted,
}

impl Supplier {
    pub fn new(person: Person, supplier_code: String, company_name: String) -> Self {
        Self {
            person,
            supplier_code,
            company_name,
            business_type: String::new(),
            tax_id: String::new(),
            credit_terms: 30,
            payment_terms: "Net 30".to_string(),
            quality_rating: 3.0,
            status: SupplierStatus::Active,
        }
    }

    pub fn update_quality_rating(&mut self, rating: f64) -> Result<(), ClassTableInheritanceError> {
        if rating < 0.0 || rating > 5.0 {
            return Err(ClassTableInheritanceError::ValidationError("质量评级必须在0.0-5.0之间".to_string()));
        }
        self.quality_rating = rating;
        Ok(())
    }

    pub fn update_credit_terms(&mut self, days: i32) -> Result<(), ClassTableInheritanceError> {
        if days < 0 {
            return Err(ClassTableInheritanceError::ValidationError("信用期限不能为负数".to_string()));
        }
        self.credit_terms = days;
        Ok(())
    }

    pub fn is_preferred_supplier(&self) -> bool {
        self.quality_rating >= 4.0 && self.status == SupplierStatus::Active
    }
}

impl Entity for Supplier {
    fn get_id(&self) -> i64 {
        self.person.id
    }

    fn get_type(&self) -> EntityType {
        EntityType::Supplier
    }

    fn validate(&self) -> Result<(), ClassTableInheritanceError> {
        self.person.validate()?;
        if self.supplier_code.trim().is_empty() {
            return Err(ClassTableInheritanceError::ValidationError("供应商代码不能为空".to_string()));
        }
        if self.company_name.trim().is_empty() {
            return Err(ClassTableInheritanceError::ValidationError("公司名称不能为空".to_string()));
        }
        if self.quality_rating < 0.0 || self.quality_rating > 5.0 {
            return Err(ClassTableInheritanceError::ValidationError("质量评级必须在0.0-5.0之间".to_string()));
        }
        if self.credit_terms < 0 {
            return Err(ClassTableInheritanceError::ValidationError("信用期限不能为负数".to_string()));
        }
        Ok(())
    }
}

impl Display for Supplier {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Supplier[{}] - {}, Code: {}, Company: {}, Quality: {:.1}, Terms: {} days", 
               self.person.id, self.person.full_name(), self.supplier_code, 
               self.company_name, self.quality_rating, self.credit_terms)
    }
}

/// 类表继承数据访问对象
pub struct ClassTableInheritanceDAO {
    // 模拟数据库表
    persons: HashMap<i64, Person>,
    employees: HashMap<i64, Employee>,
    customers: HashMap<i64, Customer>,
    suppliers: HashMap<i64, Supplier>,
    next_id: i64,
}

impl ClassTableInheritanceDAO {
    pub fn new() -> Self {
        Self {
            persons: HashMap::new(),
            employees: HashMap::new(),
            customers: HashMap::new(),
            suppliers: HashMap::new(),
            next_id: 1,
        }
    }

    /// 生成新的ID
    fn generate_id(&mut self) -> i64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    /// 保存人员（基类）
    pub fn save_person(&mut self, mut person: Person) -> Result<i64, ClassTableInheritanceError> {
        if person.id == 0 {
            person.id = self.generate_id();
        }
        person.validate()?;
        
        let id = person.id;
        self.persons.insert(id, person);
        println!("💾 保存人员记录: ID {}", id);
        Ok(id)
    }

    /// 保存员工
    pub fn save_employee(&mut self, mut employee: Employee) -> Result<i64, ClassTableInheritanceError> {
        if employee.person.id == 0 {
            employee.person.id = self.generate_id();
        }
        employee.validate()?;

        let id = employee.person.id;
        
        // 保存到persons表
        self.persons.insert(id, employee.person.clone());
        
        // 保存到employees表  
        self.employees.insert(id, employee);
        
        println!("💾 保存员工记录: ID {}", id);
        Ok(id)
    }

    /// 保存客户
    pub fn save_customer(&mut self, mut customer: Customer) -> Result<i64, ClassTableInheritanceError> {
        if customer.person.id == 0 {
            customer.person.id = self.generate_id();
        }
        customer.validate()?;

        let id = customer.person.id;
        
        // 保存到persons表
        self.persons.insert(id, customer.person.clone());
        
        // 保存到customers表
        self.customers.insert(id, customer);
        
        println!("💾 保存客户记录: ID {}", id);
        Ok(id)
    }

    /// 保存供应商
    pub fn save_supplier(&mut self, mut supplier: Supplier) -> Result<i64, ClassTableInheritanceError> {
        if supplier.person.id == 0 {
            supplier.person.id = self.generate_id();
        }
        supplier.validate()?;

        let id = supplier.person.id;
        
        // 保存到persons表
        self.persons.insert(id, supplier.person.clone());
        
        // 保存到suppliers表
        self.suppliers.insert(id, supplier);
        
        println!("💾 保存供应商记录: ID {}", id);
        Ok(id)
    }

    /// 查找人员
    pub fn find_person(&self, id: i64) -> Result<Person, ClassTableInheritanceError> {
        self.persons.get(&id)
            .cloned()
            .ok_or_else(|| ClassTableInheritanceError::RecordNotFound(format!("人员 {} 不存在", id)))
    }

    /// 查找员工
    pub fn find_employee(&self, id: i64) -> Result<Employee, ClassTableInheritanceError> {
        self.employees.get(&id)
            .cloned()
            .ok_or_else(|| ClassTableInheritanceError::RecordNotFound(format!("员工 {} 不存在", id)))
    }

    /// 查找客户
    pub fn find_customer(&self, id: i64) -> Result<Customer, ClassTableInheritanceError> {
        self.customers.get(&id)
            .cloned()
            .ok_or_else(|| ClassTableInheritanceError::RecordNotFound(format!("客户 {} 不存在", id)))
    }

    /// 查找供应商
    pub fn find_supplier(&self, id: i64) -> Result<Supplier, ClassTableInheritanceError> {
        self.suppliers.get(&id)
            .cloned()
            .ok_or_else(|| ClassTableInheritanceError::RecordNotFound(format!("供应商 {} 不存在", id)))
    }

    /// 根据ID和类型查找实体
    pub fn find_entity(&self, id: i64, entity_type: EntityType) -> Result<Box<dyn Entity>, ClassTableInheritanceError> {
        match entity_type {
            EntityType::Person => {
                let person = self.find_person(id)?;
                Ok(Box::new(person))
            }
            EntityType::Employee => {
                let employee = self.find_employee(id)?;
                Ok(Box::new(employee))
            }
            EntityType::Customer => {
                let customer = self.find_customer(id)?;
                Ok(Box::new(customer))
            }
            EntityType::Supplier => {
                let supplier = self.find_supplier(id)?;
                Ok(Box::new(supplier))
            }
        }
    }

    /// 删除实体（级联删除）
    pub fn delete_entity(&mut self, id: i64) -> Result<(), ClassTableInheritanceError> {
        // 检查实体是否存在
        if !self.persons.contains_key(&id) {
            return Err(ClassTableInheritanceError::RecordNotFound(format!("实体 {} 不存在", id)));
        }

        // 级联删除：先删除子表记录，再删除父表记录
        self.employees.remove(&id);
        self.customers.remove(&id);
        self.suppliers.remove(&id);
        self.persons.remove(&id);

        println!("🗑️ 删除实体记录: ID {}", id);
        Ok(())
    }

    /// 获取所有员工
    pub fn get_all_employees(&self) -> Vec<Employee> {
        self.employees.values().cloned().collect()
    }

    /// 获取所有客户
    pub fn get_all_customers(&self) -> Vec<Customer> {
        self.customers.values().cloned().collect()
    }

    /// 获取所有供应商
    pub fn get_all_suppliers(&self) -> Vec<Supplier> {
        self.suppliers.values().cloned().collect()
    }

    /// 按部门查找员工
    pub fn find_employees_by_department(&self, department: &str) -> Vec<Employee> {
        self.employees.values()
            .filter(|emp| emp.department == department)
            .cloned()
            .collect()
    }

    /// 按信用评级查找客户
    pub fn find_customers_by_credit_rating(&self, rating: CreditRating) -> Vec<Customer> {
        self.customers.values()
            .filter(|cust| cust.credit_rating == rating)
            .cloned()
            .collect()
    }

    /// 查找优质供应商
    pub fn find_preferred_suppliers(&self) -> Vec<Supplier> {
        self.suppliers.values()
            .filter(|supplier| supplier.is_preferred_supplier())
            .cloned()
            .collect()
    }

    /// 获取统计信息
    pub fn get_statistics(&self) -> ClassTableStatistics {
        ClassTableStatistics {
            total_persons: self.persons.len(),
            total_employees: self.employees.len(),
            total_customers: self.customers.len(),
            total_suppliers: self.suppliers.len(),
        }
    }
}

/// 统计信息
#[derive(Debug)]
pub struct ClassTableStatistics {
    pub total_persons: usize,
    pub total_employees: usize,
    pub total_customers: usize,
    pub total_suppliers: usize,
}

impl Display for ClassTableStatistics {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "统计信息 - 人员: {}, 员工: {}, 客户: {}, 供应商: {}", 
               self.total_persons, self.total_employees, self.total_customers, self.total_suppliers)
    }
}

/// 演示类表继承模式
pub fn demo() {
    println!("=== 类表继承模式演示 ===\n");

    let mut dao = ClassTableInheritanceDAO::new();

    // 创建并保存员工
    println!("1. 创建并保存员工");
    let person1 = Person::new(0, "张".to_string(), "三".to_string(), "zhang.san@company.com".to_string());
    let mut employee1 = Employee::new(person1, "EMP001".to_string(), "技术部".to_string(), "软件工程师".to_string());
    employee1.salary = 15000.0;
    employee1.hire_date = "2023-01-15".to_string();

    match dao.save_employee(employee1) {
        Ok(id) => println!("   员工保存成功，ID: {}", id),
        Err(e) => println!("   员工保存失败: {}", e),
    }

    let person2 = Person::new(0, "李".to_string(), "四".to_string(), "li.si@company.com".to_string());
    let mut employee2 = Employee::new(person2, "EMP002".to_string(), "销售部".to_string(), "销售经理".to_string());
    employee2.salary = 18000.0;
    employee2.manager_id = Some(1);

    match dao.save_employee(employee2) {
        Ok(id) => println!("   员工保存成功，ID: {}", id),
        Err(e) => println!("   员工保存失败: {}", e),
    }

    // 创建并保存客户
    println!("\n2. 创建并保存客户");
    let person3 = Person::new(0, "王".to_string(), "五".to_string(), "wang.wu@example.com".to_string());
    let mut customer1 = Customer::new(person3, "CUST001".to_string());
    customer1.credit_limit = 20000.0;
    customer1.credit_rating = CreditRating::Excellent;
    customer1.loyalty_points = 1500;

    match dao.save_customer(customer1) {
        Ok(id) => println!("   客户保存成功，ID: {}", id),
        Err(e) => println!("   客户保存失败: {}", e),
    }

    let person4 = Person::new(0, "赵".to_string(), "六".to_string(), "zhao.liu@example.com".to_string());
    let mut customer2 = Customer::new(person4, "CUST002".to_string());
    customer2.credit_limit = 10000.0;
    customer2.credit_rating = CreditRating::Good;

    match dao.save_customer(customer2) {
        Ok(id) => println!("   客户保存成功，ID: {}", id),
        Err(e) => println!("   客户保存失败: {}", e),
    }

    // 创建并保存供应商
    println!("\n3. 创建并保存供应商");
    let person5 = Person::new(0, "孙".to_string(), "七".to_string(), "sun.qi@supplier.com".to_string());
    let mut supplier1 = Supplier::new(person5, "SUP001".to_string(), "科技有限公司".to_string());
    supplier1.quality_rating = 4.5;
    supplier1.credit_terms = 45;
    supplier1.business_type = "软件开发".to_string();

    match dao.save_supplier(supplier1) {
        Ok(id) => println!("   供应商保存成功，ID: {}", id),
        Err(e) => println!("   供应商保存失败: {}", e),
    }

    // 查询和显示数据
    println!("\n4. 查询数据");
    
    // 查询员工
    match dao.find_employee(1) {
        Ok(employee) => println!("   查询员工: {}", employee),
        Err(e) => println!("   查询员工失败: {}", e),
    }

    // 查询客户
    match dao.find_customer(3) {
        Ok(customer) => println!("   查询客户: {}", customer),
        Err(e) => println!("   查询客户失败: {}", e),
    }

    // 查询供应商
    match dao.find_supplier(5) {
        Ok(supplier) => println!("   查询供应商: {}", supplier),
        Err(e) => println!("   查询供应商失败: {}", e),
    }

    // 演示业务操作
    println!("\n5. 业务操作演示");
    
    // 员工升职
    if let Ok(mut employee) = dao.find_employee(1) {
        match employee.promote("高级软件工程师".to_string(), 18000.0) {
            Ok(_) => {
                println!("   员工升职成功");
                let _ = dao.save_employee(employee);
            }
            Err(e) => println!("   员工升职失败: {}", e),
        }
    }

    // 客户积分操作
    if let Ok(mut customer) = dao.find_customer(3) {
        customer.add_loyalty_points(500);
        match customer.redeem_points(200) {
            Ok(_) => {
                println!("   客户积分操作成功，当前积分: {}", customer.loyalty_points);
                let _ = dao.save_customer(customer);
            }
            Err(e) => println!("   客户积分操作失败: {}", e),
        }
    }

    // 供应商评级更新
    if let Ok(mut supplier) = dao.find_supplier(5) {
        match supplier.update_quality_rating(4.8) {
            Ok(_) => {
                println!("   供应商评级更新成功，新评级: {:.1}", supplier.quality_rating);
                let _ = dao.save_supplier(supplier);
            }
            Err(e) => println!("   供应商评级更新失败: {}", e),
        }
    }

    // 复杂查询演示
    println!("\n6. 复杂查询演示");
    
    // 按部门查询员工
    let tech_employees = dao.find_employees_by_department("技术部");
    println!("   技术部员工数量: {}", tech_employees.len());
    for emp in tech_employees {
        println!("     {}", emp);
    }

    // 按信用评级查询客户
    let excellent_customers = dao.find_customers_by_credit_rating(CreditRating::Excellent);
    println!("   优秀信用客户数量: {}", excellent_customers.len());
    for cust in excellent_customers {
        println!("     {}", cust);
    }

    // 查询优质供应商
    let preferred_suppliers = dao.find_preferred_suppliers();
    println!("   优质供应商数量: {}", preferred_suppliers.len());
    for supplier in preferred_suppliers {
        println!("     {}", supplier);
    }

    // 显示统计信息
    println!("\n7. 统计信息");
    let stats = dao.get_statistics();
    println!("   {}", stats);

    // 演示多态查询
    println!("\n8. 多态查询演示");
    for id in 1..=5 {
        // 首先确定实体类型
        if dao.employees.contains_key(&id) {
            if let Ok(entity) = dao.find_entity(id, EntityType::Employee) {
                println!("   实体[{}]: 类型 {}, ID: {}", id, entity.get_type(), entity.get_id());
            }
        } else if dao.customers.contains_key(&id) {
            if let Ok(entity) = dao.find_entity(id, EntityType::Customer) {
                println!("   实体[{}]: 类型 {}, ID: {}", id, entity.get_type(), entity.get_id());
            }
        } else if dao.suppliers.contains_key(&id) {
            if let Ok(entity) = dao.find_entity(id, EntityType::Supplier) {
                println!("   实体[{}]: 类型 {}, ID: {}", id, entity.get_type(), entity.get_id());
            }
        }
    }

    println!("\n=== 类表继承模式演示完成 ===");

    println!("\n💡 类表继承模式的优势:");
    println!("1. 数据规范化 - 每个表只存储相关类的数据，避免冗余");
    println!("2. 扩展性好 - 添加新子类时对现有结构影响最小");
    println!("3. 存储效率 - 没有空值浪费，存储空间利用率高");
    println!("4. 数据完整性 - 通过外键约束保证数据完整性");

    println!("\n⚠️ 设计考虑:");
    println!("1. 查询复杂 - 需要连接多个表才能获取完整对象");
    println!("2. 性能影响 - 多表连接可能影响查询性能");
    println!("3. 重构成本 - 修改基类结构影响所有子类");
    println!("4. 映射复杂 - ORM映射相对复杂");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_person_creation_and_validation() {
        let person = Person::new(1, "John".to_string(), "Doe".to_string(), "john@example.com".to_string());
        assert_eq!(person.get_id(), 1);
        assert_eq!(person.get_type(), EntityType::Person);
        assert!(person.validate().is_ok());
        assert_eq!(person.full_name(), "John Doe");
    }

    #[test]
    fn test_employee_operations() {
        let person = Person::new(1, "Jane".to_string(), "Smith".to_string(), "jane@company.com".to_string());
        let mut employee = Employee::new(person, "EMP001".to_string(), "IT".to_string(), "Developer".to_string());
        
        assert_eq!(employee.get_type(), EntityType::Employee);
        assert!(employee.validate().is_ok());
        
        // 测试升职
        assert!(employee.promote("Senior Developer".to_string(), 20000.0).is_ok());
        assert_eq!(employee.position, "Senior Developer");
        assert_eq!(employee.salary, 20000.0);
        
        // 测试薪资计算
        assert_eq!(employee.get_annual_salary(), 240000.0);
    }

    #[test]
    fn test_customer_operations() {
        let person = Person::new(1, "Bob".to_string(), "Johnson".to_string(), "bob@example.com".to_string());
        let mut customer = Customer::new(person, "CUST001".to_string());
        
        assert_eq!(customer.get_type(), EntityType::Customer);
        assert!(customer.validate().is_ok());
        
        // 测试积分操作
        customer.add_loyalty_points(1000);
        assert_eq!(customer.loyalty_points, 1000);
        
        assert!(customer.redeem_points(300).is_ok());
        assert_eq!(customer.loyalty_points, 700);
        
        // 测试积分不足的情况
        assert!(customer.redeem_points(800).is_err());
    }

    #[test]
    fn test_dao_operations() {
        let mut dao = ClassTableInheritanceDAO::new();
        
        // 测试保存员工
        let person = Person::new(0, "Test".to_string(), "User".to_string(), "test@example.com".to_string());
        let employee = Employee::new(person, "TEST001".to_string(), "Test Dept".to_string(), "Tester".to_string());
        
        let id = dao.save_employee(employee).unwrap();
        assert!(id > 0);
        
        // 测试查询员工
        let found_employee = dao.find_employee(id).unwrap();
        assert_eq!(found_employee.employee_id, "TEST001");
        
        // 测试删除
        assert!(dao.delete_entity(id).is_ok());
        assert!(dao.find_employee(id).is_err());
    }
} 