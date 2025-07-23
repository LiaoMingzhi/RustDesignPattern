//! 单表继承模式 (Single Table Inheritance)
//! 
//! 将继承层次中的所有类映射到一个数据库表中
//! 文件路径：/d%3A/workspace/RustLearn/RustDesignPattern/src/EnterpriseAppPattern/ObjectRelationalStructuralPatterns/single_table_inheritance.rs

use std::collections::HashMap;
use std::fmt;
use std::error::Error;
use serde::{Serialize, Deserialize};

/// 单表继承错误类型
#[derive(Debug)]
pub enum SingleTableInheritanceError {
    ValidationError(String),
    MappingError(String),
    DatabaseError(String),
    TypeMismatch(String),
    NotFound(String),
}

impl fmt::Display for SingleTableInheritanceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SingleTableInheritanceError::ValidationError(msg) => write!(f, "验证错误: {}", msg),
            SingleTableInheritanceError::MappingError(msg) => write!(f, "映射错误: {}", msg),
            SingleTableInheritanceError::DatabaseError(msg) => write!(f, "数据库错误: {}", msg),
            SingleTableInheritanceError::TypeMismatch(msg) => write!(f, "类型不匹配: {}", msg),
            SingleTableInheritanceError::NotFound(msg) => write!(f, "未找到: {}", msg),
        }
    }
}

impl Error for SingleTableInheritanceError {}

/// 员工基类（父类）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Employee {
    pub id: Option<u32>,
    pub name: String,
    pub email: String,
    pub hire_date: String,
    pub employee_type: String,  // 类型鉴别器字段
    pub base_salary: f64,
    
    // 可选字段，用于不同子类的特有属性
    pub commission_rate: Option<f64>,    // 销售员专用
    pub hourly_rate: Option<f64>,        // 时薪员工专用
    pub overtime_rate: Option<f64>,      // 时薪员工专用
    pub bonus_percentage: Option<f64>,   // 经理专用
    pub department: Option<String>,      // 经理专用
    pub team_size: Option<u32>,          // 经理专用
}

impl Employee {
    pub fn validate(&self) -> Result<(), SingleTableInheritanceError> {
        if self.name.trim().is_empty() {
            return Err(SingleTableInheritanceError::ValidationError("姓名不能为空".to_string()));
        }
        if self.email.trim().is_empty() || !self.email.contains('@') {
            return Err(SingleTableInheritanceError::ValidationError("邮箱格式不正确".to_string()));
        }
        if self.base_salary < 0.0 {
            return Err(SingleTableInheritanceError::ValidationError("基本工资不能为负数".to_string()));
        }
        Ok(())
    }

    pub fn calculate_monthly_salary(&self) -> Result<f64, SingleTableInheritanceError> {
        match self.employee_type.as_str() {
            "SalaryEmployee" => Ok(self.base_salary),
            "HourlyEmployee" => {
                let rate = self.hourly_rate.ok_or_else(|| 
                    SingleTableInheritanceError::MappingError("时薪员工缺少时薪信息".to_string()))?;
                Ok(rate * 160.0) // 假设月工作160小时
            }
            "SalesEmployee" => {
                let commission = self.commission_rate.unwrap_or(0.0);
                Ok(self.base_salary + (self.base_salary * commission))
            }
            "Manager" => {
                let bonus = self.bonus_percentage.unwrap_or(0.0);
                Ok(self.base_salary * (1.0 + bonus))
            }
            _ => Err(SingleTableInheritanceError::TypeMismatch(
                format!("未知的员工类型: {}", self.employee_type)
            ))
        }
    }

    pub fn get_type_display(&self) -> &str {
        match self.employee_type.as_str() {
            "SalaryEmployee" => "固定薪酬员工",
            "HourlyEmployee" => "时薪员工", 
            "SalesEmployee" => "销售员工",
            "Manager" => "管理人员",
            _ => "未知类型"
        }
    }

    pub fn format_details(&self) -> String {
        let mut details = format!("姓名: {}, 邮箱: {}, 类型: {}", 
                                 self.name, self.email, self.get_type_display());
        
        match self.employee_type.as_str() {
            "SalaryEmployee" => {
                details.push_str(&format!(", 月薪: ¥{:.2}", self.base_salary));
            }
            "HourlyEmployee" => {
                if let Some(rate) = self.hourly_rate {
                    details.push_str(&format!(", 时薪: ¥{:.2}", rate));
                    if let Some(overtime) = self.overtime_rate {
                        details.push_str(&format!(", 加班时薪: ¥{:.2}", overtime));
                    }
                }
            }
            "SalesEmployee" => {
                details.push_str(&format!(", 基本工资: ¥{:.2}", self.base_salary));
                if let Some(rate) = self.commission_rate {
                    details.push_str(&format!(", 提成比例: {:.1}%", rate * 100.0));
                }
            }
            "Manager" => {
                details.push_str(&format!(", 基本工资: ¥{:.2}", self.base_salary));
                if let Some(bonus) = self.bonus_percentage {
                    details.push_str(&format!(", 奖金比例: {:.1}%", bonus * 100.0));
                }
                if let Some(ref dept) = self.department {
                    details.push_str(&format!(", 部门: {}", dept));
                }
                if let Some(team) = self.team_size {
                    details.push_str(&format!(", 团队规模: {}人", team));
                }
            }
            _ => {}
        }
        
        details
    }
}

/// 固定薪酬员工
pub struct SalaryEmployee;

impl SalaryEmployee {
    pub fn new(name: String, email: String, hire_date: String, salary: f64) -> Employee {
        Employee {
            id: None,
            name,
            email,
            hire_date,
            employee_type: "SalaryEmployee".to_string(),
            base_salary: salary,
            commission_rate: None,
            hourly_rate: None,
            overtime_rate: None,
            bonus_percentage: None,
            department: None,
            team_size: None,
        }
    }
}

/// 时薪员工
pub struct HourlyEmployee;

impl HourlyEmployee {
    pub fn new(name: String, email: String, hire_date: String, 
               hourly_rate: f64, overtime_rate: Option<f64>) -> Employee {
        Employee {
            id: None,
            name,
            email,
            hire_date,
            employee_type: "HourlyEmployee".to_string(),
            base_salary: 0.0,  // 时薪员工没有固定基本工资
            commission_rate: None,
            hourly_rate: Some(hourly_rate),
            overtime_rate,
            bonus_percentage: None,
            department: None,
            team_size: None,
        }
    }

    pub fn calculate_overtime_pay(employee: &Employee, regular_hours: f64, overtime_hours: f64) 
        -> Result<f64, SingleTableInheritanceError> {
        if employee.employee_type != "HourlyEmployee" {
            return Err(SingleTableInheritanceError::TypeMismatch(
                "只有时薪员工才能计算加班费".to_string()
            ));
        }

        let regular_rate = employee.hourly_rate.ok_or_else(|| 
            SingleTableInheritanceError::MappingError("缺少时薪信息".to_string()))?;
        
        let overtime_rate = employee.overtime_rate.unwrap_or(regular_rate * 1.5);
        
        Ok(regular_hours * regular_rate + overtime_hours * overtime_rate)
    }
}

/// 销售员工
pub struct SalesEmployee;

impl SalesEmployee {
    pub fn new(name: String, email: String, hire_date: String, 
               base_salary: f64, commission_rate: f64) -> Employee {
        Employee {
            id: None,
            name,
            email,
            hire_date,
            employee_type: "SalesEmployee".to_string(),
            base_salary,
            commission_rate: Some(commission_rate),
            hourly_rate: None,
            overtime_rate: None,
            bonus_percentage: None,
            department: None,
            team_size: None,
        }
    }

    pub fn calculate_commission(employee: &Employee, sales_amount: f64) 
        -> Result<f64, SingleTableInheritanceError> {
        if employee.employee_type != "SalesEmployee" {
            return Err(SingleTableInheritanceError::TypeMismatch(
                "只有销售员工才能计算提成".to_string()
            ));
        }

        let rate = employee.commission_rate.ok_or_else(|| 
            SingleTableInheritanceError::MappingError("缺少提成比例信息".to_string()))?;
        
        Ok(sales_amount * rate)
    }
}

/// 管理人员
pub struct Manager;

impl Manager {
    pub fn new(name: String, email: String, hire_date: String, 
               base_salary: f64, bonus_percentage: f64, 
               department: String, team_size: u32) -> Employee {
        Employee {
            id: None,
            name,
            email,
            hire_date,
            employee_type: "Manager".to_string(),
            base_salary,
            commission_rate: None,
            hourly_rate: None,
            overtime_rate: None,
            bonus_percentage: Some(bonus_percentage),
            department: Some(department),
            team_size: Some(team_size),
        }
    }

    pub fn calculate_team_bonus(employee: &Employee, team_performance: f64) 
        -> Result<f64, SingleTableInheritanceError> {
        if employee.employee_type != "Manager" {
            return Err(SingleTableInheritanceError::TypeMismatch(
                "只有管理人员才能计算团队奖金".to_string()
            ));
        }

        let bonus_rate = employee.bonus_percentage.ok_or_else(|| 
            SingleTableInheritanceError::MappingError("缺少奖金比例信息".to_string()))?;
        
        let team_size = employee.team_size.unwrap_or(1) as f64;
        
        Ok(employee.base_salary * bonus_rate * team_performance * team_size / 10.0)
    }
}

/// 单表继承映射器
pub struct EmployeeSingleTableMapper {
    employees: HashMap<u32, Employee>,
    next_id: u32,
}

impl EmployeeSingleTableMapper {
    pub fn new() -> Self {
        Self {
            employees: HashMap::new(),
            next_id: 1,
        }
    }

    /// 保存员工（单表继承映射）
    pub fn save(&mut self, mut employee: Employee) -> Result<Employee, SingleTableInheritanceError> {
        employee.validate()?;

        let id = match employee.id {
            Some(existing_id) => {
                println!("更新员工: {}", employee.name);
                existing_id
            }
            None => {
                let new_id = self.next_id;
                self.next_id += 1;
                employee.id = Some(new_id);
                println!("创建员工: {}", employee.name);
                new_id
            }
        };

        // 生成单表继承的SQL语句
        self.generate_single_table_sql(&employee)?;
        
        self.employees.insert(id, employee.clone());
        Ok(employee)
    }

    /// 查找员工
    pub fn find(&self, id: u32) -> Result<Employee, SingleTableInheritanceError> {
        match self.employees.get(&id) {
            Some(employee) => {
                println!("从数据库加载员工: {} ({})", employee.name, employee.get_type_display());
                Ok(employee.clone())
            }
            None => Err(SingleTableInheritanceError::NotFound(format!("员工ID {} 不存在", id)))
        }
    }

    /// 按类型查找员工
    pub fn find_by_type(&self, employee_type: &str) -> Vec<Employee> {
        self.employees.values()
            .filter(|emp| emp.employee_type == employee_type)
            .cloned()
            .collect()
    }

    /// 查找所有经理
    pub fn find_managers(&self) -> Vec<Employee> {
        self.find_by_type("Manager")
    }

    /// 查找所有销售员工
    pub fn find_sales_employees(&self) -> Vec<Employee> {
        self.find_by_type("SalesEmployee")
    }

    /// 查找所有时薪员工
    pub fn find_hourly_employees(&self) -> Vec<Employee> {
        self.find_by_type("HourlyEmployee")
    }

    /// 查找所有固定薪酬员工
    pub fn find_salary_employees(&self) -> Vec<Employee> {
        self.find_by_type("SalaryEmployee")
    }

    /// 按部门查找员工（只适用于经理）
    pub fn find_by_department(&self, department: &str) -> Vec<Employee> {
        self.employees.values()
            .filter(|emp| {
                emp.employee_type == "Manager" && 
                emp.department.as_ref().map_or(false, |d| d.contains(department))
            })
            .cloned()
            .collect()
    }

    /// 按薪资范围查找员工
    pub fn find_by_salary_range(&self, min_salary: f64, max_salary: f64) -> Vec<Employee> {
        self.employees.values()
            .filter(|emp| {
                if let Ok(monthly_salary) = emp.calculate_monthly_salary() {
                    monthly_salary >= min_salary && monthly_salary <= max_salary
                } else {
                    false
                }
            })
            .cloned()
            .collect()
    }

    /// 删除员工
    pub fn delete(&mut self, id: u32) -> Result<(), SingleTableInheritanceError> {
        match self.employees.remove(&id) {
            Some(employee) => {
                println!("删除员工: {}", employee.name);
                println!("生成SQL: DELETE FROM employees WHERE id = {}", id);
                Ok(())
            }
            None => Err(SingleTableInheritanceError::NotFound(format!("员工ID {} 不存在", id)))
        }
    }

    /// 获取所有员工
    pub fn find_all(&self) -> Vec<Employee> {
        self.employees.values().cloned().collect()
    }

    /// 生成单表继承的SQL语句
    fn generate_single_table_sql(&self, employee: &Employee) -> Result<(), SingleTableInheritanceError> {
        match employee.id {
            Some(id) => {
                println!("生成UPDATE SQL (单表继承):");
                println!("UPDATE employees SET");
                println!("  name = '{}',", employee.name);
                println!("  email = '{}',", employee.email);
                println!("  hire_date = '{}',", employee.hire_date);
                println!("  employee_type = '{}',", employee.employee_type);
                println!("  base_salary = {},", employee.base_salary);
                
                // 可选字段处理
                match employee.commission_rate {
                    Some(rate) => println!("  commission_rate = {},", rate),
                    None => println!("  commission_rate = NULL,"),
                }
                
                match employee.hourly_rate {
                    Some(rate) => println!("  hourly_rate = {},", rate),
                    None => println!("  hourly_rate = NULL,"),
                }
                
                match employee.overtime_rate {
                    Some(rate) => println!("  overtime_rate = {},", rate),
                    None => println!("  overtime_rate = NULL,"),
                }
                
                match employee.bonus_percentage {
                    Some(bonus) => println!("  bonus_percentage = {},", bonus),
                    None => println!("  bonus_percentage = NULL,"),
                }
                
                match &employee.department {
                    Some(dept) => println!("  department = '{}',", dept),
                    None => println!("  department = NULL,"),
                }
                
                match employee.team_size {
                    Some(size) => println!("  team_size = {}", size),
                    None => println!("  team_size = NULL"),
                }
                
                println!("WHERE id = {};", id);
            }
            None => {
                println!("生成INSERT SQL (单表继承):");
                println!("INSERT INTO employees (");
                println!("  name, email, hire_date, employee_type, base_salary,");
                println!("  commission_rate, hourly_rate, overtime_rate,");
                println!("  bonus_percentage, department, team_size");
                println!(") VALUES (");
                println!("  '{}', '{}', '{}', '{}', {},",
                        employee.name, employee.email, employee.hire_date, 
                        employee.employee_type, employee.base_salary);
                        
                print!("  ");
                match employee.commission_rate {
                    Some(rate) => print!("{}, ", rate),
                    None => print!("NULL, "),
                }
                
                match employee.hourly_rate {
                    Some(rate) => print!("{}, ", rate),
                    None => print!("NULL, "),
                }
                
                match employee.overtime_rate {
                    Some(rate) => print!("{}, ", rate),
                    None => print!("NULL, "),
                }
                
                match employee.bonus_percentage {
                    Some(bonus) => print!("{}, ", bonus),
                    None => print!("NULL, "),
                }
                
                match &employee.department {
                    Some(dept) => print!("'{}', ", dept),
                    None => print!("NULL, "),
                }
                
                match employee.team_size {
                    Some(size) => println!("{}", size),
                    None => println!("NULL"),
                }
                
                println!(");");
            }
        }
        Ok(())
    }

    /// 获取统计信息
    pub fn get_statistics(&self) -> EmployeeStatistics {
        if self.employees.is_empty() {
            return EmployeeStatistics::default();
        }

        let employees: Vec<&Employee> = self.employees.values().collect();
        let total_employees = employees.len();
        
        // 按类型统计
        let mut type_counts = HashMap::new();
        let mut total_salary_by_type = HashMap::new();
        
        for employee in &employees {
            let type_display = employee.get_type_display();
            *type_counts.entry(type_display.to_string()).or_insert(0) += 1;
            
            if let Ok(monthly_salary) = employee.calculate_monthly_salary() {
                let entry = total_salary_by_type.entry(type_display.to_string()).or_insert(0.0);
                *entry += monthly_salary;
            }
        }

        // 计算平均薪资
        let total_monthly_salary: f64 = employees.iter()
            .filter_map(|emp| emp.calculate_monthly_salary().ok())
            .sum();
        let average_salary = total_monthly_salary / total_employees as f64;

        // 部门统计（仅经理）
        let mut department_counts = HashMap::new();
        for employee in &employees {
            if employee.employee_type == "Manager" {
                if let Some(ref dept) = employee.department {
                    *department_counts.entry(dept.clone()).or_insert(0) += 1;
                }
            }
        }

        EmployeeStatistics {
            total_employees,
            type_distribution: type_counts,
            total_salary_by_type,
            average_salary,
            total_monthly_salary,
            department_distribution: department_counts,
        }
    }
}

/// 员工统计信息
#[derive(Debug, Default)]
pub struct EmployeeStatistics {
    pub total_employees: usize,
    pub type_distribution: HashMap<String, u32>,
    pub total_salary_by_type: HashMap<String, f64>,
    pub average_salary: f64,
    pub total_monthly_salary: f64,
    pub department_distribution: HashMap<String, u32>,
}

impl fmt::Display for EmployeeStatistics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "=== 员工统计信息 ===")?;
        writeln!(f, "总员工数: {}", self.total_employees)?;
        writeln!(f, "平均月薪: ¥{:.2}", self.average_salary)?;
        writeln!(f, "总月薪支出: ¥{:.2}", self.total_monthly_salary)?;
        
        writeln!(f, "\n员工类型分布:")?;
        for (emp_type, count) in &self.type_distribution {
            let total_salary = self.total_salary_by_type.get(emp_type).unwrap_or(&0.0);
            let avg_salary = if *count > 0 { total_salary / *count as f64 } else { 0.0 };
            writeln!(f, "  {}: {} 人 (平均月薪: ¥{:.2})", emp_type, count, avg_salary)?;
        }
        
        if !self.department_distribution.is_empty() {
            writeln!(f, "\n部门分布 (仅管理人员):")?;
            for (dept, count) in &self.department_distribution {
                writeln!(f, "  {}: {} 人", dept, count)?;
            }
        }
        
        Ok(())
    }
}

/// 演示单表继承模式
pub fn demo() {
    println!("=== 单表继承模式演示 ===\n");
    
    let mut mapper = EmployeeSingleTableMapper::new();
    
    println!("1. 创建不同类型的员工:");
    
    // 创建固定薪酬员工
    let salary_emp = SalaryEmployee::new(
        "张三".to_string(),
        "zhangsan@company.com".to_string(),
        "2023-01-15".to_string(),
        12000.0
    );
    
    // 创建时薪员工
    let hourly_emp = HourlyEmployee::new(
        "李四".to_string(),
        "lisi@company.com".to_string(),
        "2023-03-01".to_string(),
        50.0,
        Some(75.0)  // 加班时薪
    );
    
    // 创建销售员工
    let sales_emp = SalesEmployee::new(
        "王五".to_string(),
        "wangwu@company.com".to_string(),
        "2023-02-10".to_string(),
        8000.0,
        0.05  // 5%提成
    );
    
    // 创建管理人员
    let manager = Manager::new(
        "赵六".to_string(),
        "zhaoliu@company.com".to_string(),
        "2022-06-01".to_string(),
        20000.0,
        0.20,  // 20%奖金
        "技术部".to_string(),
        15
    );
    
    // 保存所有员工
    let employees = vec![salary_emp, hourly_emp, sales_emp, manager];
    let mut saved_employees = Vec::new();
    
    for employee in employees {
        match mapper.save(employee) {
            Ok(saved) => {
                println!("✅ 员工保存成功: {}", saved.format_details());
                if let Ok(monthly_salary) = saved.calculate_monthly_salary() {
                    println!("   预计月薪: ¥{:.2}", monthly_salary);
                }
                saved_employees.push(saved);
            }
            Err(e) => println!("❌ 保存失败: {}", e),
        }
        println!();
    }
    
    println!("\n{}", "=".repeat(50));
    
    // 按类型查询
    println!("2. 按类型查询员工:");
    
    println!("\n固定薪酬员工:");
    let salary_employees = mapper.find_salary_employees();
    for emp in &salary_employees {
        println!("  - {}", emp.format_details());
    }
    
    println!("\n时薪员工:");
    let hourly_employees = mapper.find_hourly_employees();
    for emp in &hourly_employees {
        println!("  - {}", emp.format_details());
    }
    
    println!("\n销售员工:");
    let sales_employees = mapper.find_sales_employees();
    for emp in &sales_employees {
        println!("  - {}", emp.format_details());
    }
    
    println!("\n管理人员:");
    let managers = mapper.find_managers();
    for emp in &managers {
        println!("  - {}", emp.format_details());
    }
    
    println!("\n{}", "=".repeat(50));
    
    // 特定类型的业务操作
    println!("3. 特定类型的业务操作:");
    
    // 时薪员工加班费计算
    println!("\n时薪员工加班费计算:");
    if let Some(hourly_emp) = hourly_employees.first() {
        match HourlyEmployee::calculate_overtime_pay(hourly_emp, 160.0, 20.0) {
            Ok(total_pay) => {
                println!("  员工: {}", hourly_emp.name);
                println!("  正常工时: 160小时, 加班工时: 20小时");
                println!("  总薪酬: ¥{:.2}", total_pay);
            }
            Err(e) => println!("  ❌ 计算失败: {}", e),
        }
    }
    
    // 销售员工提成计算
    println!("\n销售员工提成计算:");
    if let Some(sales_emp) = sales_employees.first() {
        let sales_amount = 100000.0;
        match SalesEmployee::calculate_commission(sales_emp, sales_amount) {
            Ok(commission) => {
                println!("  员工: {}", sales_emp.name);
                println!("  销售额: ¥{:.2}", sales_amount);
                println!("  提成: ¥{:.2}", commission);
                println!("  总收入: ¥{:.2}", sales_emp.base_salary + commission);
            }
            Err(e) => println!("  ❌ 计算失败: {}", e),
        }
    }
    
    // 管理人员团队奖金计算
    println!("\n管理人员团队奖金计算:");
    if let Some(manager) = managers.first() {
        let team_performance = 1.2; // 120%完成率
        match Manager::calculate_team_bonus(manager, team_performance) {
            Ok(team_bonus) => {
                println!("  经理: {}", manager.name);
                println!("  团队完成率: {:.0}%", team_performance * 100.0);
                println!("  团队奖金: ¥{:.2}", team_bonus);
                if let Ok(monthly_salary) = manager.calculate_monthly_salary() {
                    println!("  总收入: ¥{:.2}", monthly_salary + team_bonus);
                }
            }
            Err(e) => println!("  ❌ 计算失败: {}", e),
        }
    }
    
    println!("\n{}", "=".repeat(50));
    
    // 复杂查询
    println!("4. 复杂查询操作:");
    
    // 按薪资范围查询
    println!("\n月薪在10000-15000之间的员工:");
    let mid_range_employees = mapper.find_by_salary_range(10000.0, 15000.0);
    for emp in &mid_range_employees {
        if let Ok(salary) = emp.calculate_monthly_salary() {
            println!("  - {} ({}): ¥{:.2}", emp.name, emp.get_type_display(), salary);
        }
    }
    
    // 按部门查询
    println!("\n技术部门员工:");
    let tech_employees = mapper.find_by_department("技术");
    for emp in &tech_employees {
        println!("  - {}", emp.format_details());
    }
    
    // 查找特定员工
    println!("\n查找特定员工 (ID: 1):");
    match mapper.find(1) {
        Ok(employee) => {
            println!("  {}", employee.format_details());
            if let Ok(salary) = employee.calculate_monthly_salary() {
                println!("  月薪: ¥{:.2}", salary);
            }
        }
        Err(e) => println!("  ❌ {}", e),
    }
    
    println!("\n{}", "=".repeat(50));
    
    // 统计信息
    println!("5. 统计信息:");
    let stats = mapper.get_statistics();
    println!("{}", stats);
    
    println!("\n{}", "=".repeat(50));
    
    println!("单表继承模式的特点:");
    println!("✅ 所有子类映射到同一个表中");
    println!("✅ 使用类型鉴别器字段区分不同类型");
    println!("✅ 查询简单，无需JOIN操作");
    println!("✅ 添加新子类只需修改应用代码");
    println!("✅ 适合继承层次较浅的场景");
    
    println!("\n适用场景:");
    println!("• 继承层次较浅且稳定");
    println!("• 子类间差异不大");
    println!("• 需要频繁的多态查询");
    println!("• 性能要求较高的场景");
    
    println!("\n注意事项:");
    println!("• 表结构可能变得复杂（很多可选字段）");
    println!("• 存在数据浪费（NULL值较多）");
    println!("• 缺乏数据库层面的约束");
    println!("• 表大小可能影响查询性能");
    println!("• 需要在应用层维护类型一致性");
} 