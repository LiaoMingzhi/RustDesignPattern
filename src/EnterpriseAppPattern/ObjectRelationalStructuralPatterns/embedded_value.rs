//! 嵌入值模式 (Embedded Value)
//! 
//! 将值对象的属性直接嵌入到拥有它的对象的表中
//! 文件路径：/d%3A/workspace/RustLearn/RustDesignPattern/src/EnterpriseAppPattern/ObjectRelationalStructuralPatterns/embedded_value.rs

use std::collections::HashMap;
use std::fmt;
use std::error::Error;

/// 嵌入值错误类型
#[derive(Debug)]
pub enum EmbeddedValueError {
    ValidationError(String),
    MappingError(String),
    DatabaseError(String),
    NotFound(String),
}

impl fmt::Display for EmbeddedValueError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EmbeddedValueError::ValidationError(msg) => write!(f, "验证错误: {}", msg),
            EmbeddedValueError::MappingError(msg) => write!(f, "映射错误: {}", msg),
            EmbeddedValueError::DatabaseError(msg) => write!(f, "数据库错误: {}", msg),
            EmbeddedValueError::NotFound(msg) => write!(f, "未找到: {}", msg),
        }
    }
}

impl Error for EmbeddedValueError {}

/// 金额值对象（将被嵌入）
#[derive(Debug, Clone, PartialEq)]
pub struct Money {
    pub amount: f64,
    pub currency: String,
}

impl Money {
    pub fn new(amount: f64, currency: &str) -> Result<Self, EmbeddedValueError> {
        if amount < 0.0 {
            return Err(EmbeddedValueError::ValidationError("金额不能为负数".to_string()));
        }
        if currency.trim().is_empty() {
            return Err(EmbeddedValueError::ValidationError("货币类型不能为空".to_string()));
        }
        
        Ok(Self {
            amount,
            currency: currency.to_uppercase(),
        })
    }

    pub fn zero(currency: &str) -> Self {
        Self {
            amount: 0.0,
            currency: currency.to_uppercase(),
        }
    }

    pub fn add(&self, other: &Money) -> Result<Money, EmbeddedValueError> {
        if self.currency != other.currency {
            return Err(EmbeddedValueError::ValidationError(
                format!("货币类型不匹配: {} vs {}", self.currency, other.currency)
            ));
        }
        Ok(Money {
            amount: self.amount + other.amount,
            currency: self.currency.clone(),
        })
    }

    pub fn subtract(&self, other: &Money) -> Result<Money, EmbeddedValueError> {
        if self.currency != other.currency {
            return Err(EmbeddedValueError::ValidationError(
                format!("货币类型不匹配: {} vs {}", self.currency, other.currency)
            ));
        }
        if self.amount < other.amount {
            return Err(EmbeddedValueError::ValidationError("余额不足".to_string()));
        }
        Ok(Money {
            amount: self.amount - other.amount,
            currency: self.currency.clone(),
        })
    }

    pub fn multiply(&self, factor: f64) -> Result<Money, EmbeddedValueError> {
        if factor < 0.0 {
            return Err(EmbeddedValueError::ValidationError("乘数不能为负数".to_string()));
        }
        Ok(Money {
            amount: self.amount * factor,
            currency: self.currency.clone(),
        })
    }

    pub fn is_zero(&self) -> bool {
        self.amount == 0.0
    }

    pub fn is_positive(&self) -> bool {
        self.amount > 0.0
    }

    pub fn format(&self) -> String {
        match self.currency.as_str() {
            "CNY" => format!("¥{:.2}", self.amount),
            "USD" => format!("${:.2}", self.amount),
            "EUR" => format!("€{:.2}", self.amount),
            _ => format!("{:.2} {}", self.amount, self.currency),
        }
    }
}

impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format())
    }
}

/// 日期范围值对象（将被嵌入）
#[derive(Debug, Clone, PartialEq)]
pub struct DateRange {
    pub start_date: String,  // 简化为字符串
    pub end_date: String,
}

impl DateRange {
    pub fn new(start_date: String, end_date: String) -> Result<Self, EmbeddedValueError> {
        if start_date.trim().is_empty() {
            return Err(EmbeddedValueError::ValidationError("开始日期不能为空".to_string()));
        }
        if end_date.trim().is_empty() {
            return Err(EmbeddedValueError::ValidationError("结束日期不能为空".to_string()));
        }
        
        // 简单的日期格式验证
        if !Self::is_valid_date_format(&start_date) || !Self::is_valid_date_format(&end_date) {
            return Err(EmbeddedValueError::ValidationError("日期格式无效，应为YYYY-MM-DD".to_string()));
        }
        
        if start_date > end_date {
            return Err(EmbeddedValueError::ValidationError("开始日期不能晚于结束日期".to_string()));
        }
        
        Ok(Self { start_date, end_date })
    }

    fn is_valid_date_format(date: &str) -> bool {
        date.len() == 10 && date.chars().nth(4) == Some('-') && date.chars().nth(7) == Some('-')
    }

    pub fn duration_days(&self) -> u32 {
        // 简化的天数计算
        let start_parts: Vec<&str> = self.start_date.split('-').collect();
        let end_parts: Vec<&str> = self.end_date.split('-').collect();
        
        if start_parts.len() == 3 && end_parts.len() == 3 {
            let start_day: u32 = start_parts[2].parse().unwrap_or(1);
            let end_day: u32 = end_parts[2].parse().unwrap_or(1);
            if end_day >= start_day {
                return end_day - start_day + 1;
            }
        }
        1
    }

    pub fn contains_date(&self, date: &str) -> bool {
        date >= &self.start_date && date <= &self.end_date
    }

    pub fn overlaps_with(&self, other: &DateRange) -> bool {
        self.start_date <= other.end_date && self.end_date >= other.start_date
    }

    pub fn format(&self) -> String {
        format!("{} ~ {}", self.start_date, self.end_date)
    }
}

impl fmt::Display for DateRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format())
    }
}

/// 项目实体（包含嵌入值对象）
#[derive(Debug, Clone)]
pub struct Project {
    pub id: Option<u32>,
    pub name: String,
    pub description: String,
    pub budget: Money,           // 嵌入的金额值对象
    pub actual_cost: Money,      // 嵌入的金额值对象
    pub schedule: DateRange,     // 嵌入的日期范围值对象
    pub status: ProjectStatus,
    pub priority: ProjectPriority,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ProjectStatus {
    Planning,
    InProgress,
    OnHold,
    Completed,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ProjectPriority {
    Low,
    Medium,
    High,
    Critical,
}

impl Project {
    pub fn new(name: String, description: String, budget: Money, schedule: DateRange) -> Self {
        let zero_cost = Money::zero(&budget.currency);
        
        Self {
            id: None,
            name,
            description,
            budget,
            actual_cost: zero_cost,
            schedule,
            status: ProjectStatus::Planning,
            priority: ProjectPriority::Medium,
        }
    }

    pub fn validate(&self) -> Result<(), EmbeddedValueError> {
        if self.name.trim().is_empty() {
            return Err(EmbeddedValueError::ValidationError("项目名称不能为空".to_string()));
        }
        
        if self.budget.currency != self.actual_cost.currency {
            return Err(EmbeddedValueError::ValidationError("预算和实际成本的货币类型必须一致".to_string()));
        }
        
        Ok(())
    }

    pub fn start_project(&mut self) -> Result<(), EmbeddedValueError> {
        match self.status {
            ProjectStatus::Planning => {
                self.status = ProjectStatus::InProgress;
                Ok(())
            }
            _ => Err(EmbeddedValueError::ValidationError("只有计划中的项目才能开始".to_string()))
        }
    }

    pub fn complete_project(&mut self) -> Result<(), EmbeddedValueError> {
        match self.status {
            ProjectStatus::InProgress => {
                self.status = ProjectStatus::Completed;
                Ok(())
            }
            _ => Err(EmbeddedValueError::ValidationError("只有进行中的项目才能完成".to_string()))
        }
    }

    pub fn add_cost(&mut self, cost: Money) -> Result<(), EmbeddedValueError> {
        self.actual_cost = self.actual_cost.add(&cost)?;
        Ok(())
    }

    pub fn budget_variance(&self) -> Result<Money, EmbeddedValueError> {
        self.budget.subtract(&self.actual_cost)
    }

    pub fn is_over_budget(&self) -> bool {
        self.actual_cost.amount > self.budget.amount
    }

    pub fn budget_utilization_percentage(&self) -> f64 {
        if self.budget.is_zero() {
            return 0.0;
        }
        (self.actual_cost.amount / self.budget.amount) * 100.0
    }

    pub fn set_priority(&mut self, priority: ProjectPriority) {
        self.priority = priority;
    }

    pub fn is_active(&self) -> bool {
        matches!(self.status, ProjectStatus::InProgress)
    }

    pub fn duration_days(&self) -> u32 {
        self.schedule.duration_days()
    }
}

/// 嵌入值映射器
pub struct ProjectEmbeddedMapper {
    projects: HashMap<u32, Project>,
    next_id: u32,
}

impl ProjectEmbeddedMapper {
    pub fn new() -> Self {
        Self {
            projects: HashMap::new(),
            next_id: 1,
        }
    }

    /// 保存项目（演示嵌入值的SQL映射）
    pub fn save(&mut self, mut project: Project) -> Result<Project, EmbeddedValueError> {
        project.validate()?;

        let id = match project.id {
            Some(existing_id) => {
                println!("更新项目: {}", project.name);
                existing_id
            }
            None => {
                let new_id = self.next_id;
                self.next_id += 1;
                project.id = Some(new_id);
                println!("创建项目: {}", project.name);
                new_id
            }
        };

        // 生成SQL语句，显示嵌入值如何映射
        self.generate_sql_for_embedded_values(&project)?;
        
        self.projects.insert(id, project.clone());
        Ok(project)
    }

    /// 查找项目
    pub fn find(&self, id: u32) -> Result<Project, EmbeddedValueError> {
        match self.projects.get(&id) {
            Some(project) => {
                println!("从数据库加载项目: {}", project.name);
                Ok(project.clone())
            }
            None => Err(EmbeddedValueError::NotFound(format!("项目ID {} 不存在", id)))
        }
    }

    /// 按状态查找项目
    pub fn find_by_status(&self, status: ProjectStatus) -> Vec<Project> {
        self.projects.values()
            .filter(|project| project.status == status)
            .cloned()
            .collect()
    }

    /// 按优先级查找项目
    pub fn find_by_priority(&self, priority: ProjectPriority) -> Vec<Project> {
        self.projects.values()
            .filter(|project| project.priority == priority)
            .cloned()
            .collect()
    }

    /// 查找超预算的项目
    pub fn find_over_budget_projects(&self) -> Vec<Project> {
        self.projects.values()
            .filter(|project| project.is_over_budget())
            .cloned()
            .collect()
    }

    /// 按货币类型查找项目
    pub fn find_by_currency(&self, currency: &str) -> Vec<Project> {
        self.projects.values()
            .filter(|project| project.budget.currency == currency.to_uppercase())
            .cloned()
            .collect()
    }

    /// 查找在指定日期范围内的项目
    pub fn find_by_date_range(&self, start_date: &str, end_date: &str) -> Vec<Project> {
        self.projects.values()
            .filter(|project| {
                project.schedule.start_date <= end_date.to_string() && project.schedule.end_date >= start_date.to_string()
            })
            .cloned()
            .collect()
    }

    /// 删除项目
    pub fn delete(&mut self, id: u32) -> Result<(), EmbeddedValueError> {
        match self.projects.remove(&id) {
            Some(project) => {
                println!("删除项目: {}", project.name);
                println!("生成SQL: DELETE FROM projects WHERE id = {}", id);
                Ok(())
            }
            None => Err(EmbeddedValueError::NotFound(format!("项目ID {} 不存在", id)))
        }
    }

    /// 获取所有项目
    pub fn find_all(&self) -> Vec<Project> {
        self.projects.values().cloned().collect()
    }

    /// 生成嵌入值的SQL语句
    fn generate_sql_for_embedded_values(&self, project: &Project) -> Result<(), EmbeddedValueError> {
        match project.id {
            Some(id) => {
                println!("生成UPDATE SQL (嵌入值映射):");
                println!("UPDATE projects SET");
                println!("  name = '{}',", project.name);
                println!("  description = '{}',", project.description);
                // 嵌入的Money值对象字段
                println!("  budget_amount = {},", project.budget.amount);
                println!("  budget_currency = '{}',", project.budget.currency);
                println!("  actual_cost_amount = {},", project.actual_cost.amount);
                println!("  actual_cost_currency = '{}',", project.actual_cost.currency);
                // 嵌入的DateRange值对象字段
                println!("  schedule_start_date = '{}',", project.schedule.start_date);
                println!("  schedule_end_date = '{}',", project.schedule.end_date);
                println!("  status = '{:?}',", project.status);
                println!("  priority = '{:?}'", project.priority);
                println!("WHERE id = {};", id);
            }
            None => {
                println!("生成INSERT SQL (嵌入值映射):");
                println!("INSERT INTO projects (");
                println!("  name, description,");
                println!("  budget_amount, budget_currency,");
                println!("  actual_cost_amount, actual_cost_currency,");
                println!("  schedule_start_date, schedule_end_date,");
                println!("  status, priority");
                println!(") VALUES (");
                println!("  '{}', '{}',", project.name, project.description);
                println!("  {}, '{}',", project.budget.amount, project.budget.currency);
                println!("  {}, '{}',", project.actual_cost.amount, project.actual_cost.currency);
                println!("  '{}', '{}',", project.schedule.start_date, project.schedule.end_date);
                println!("  '{:?}', '{:?}'", project.status, project.priority);
                println!(");");
            }
        }
        Ok(())
    }

    /// 获取统计信息
    pub fn get_statistics(&self) -> ProjectStatistics {
        if self.projects.is_empty() {
            return ProjectStatistics::default();
        }

        let projects: Vec<&Project> = self.projects.values().collect();
        let total_projects = projects.len();
        
        // 预算统计
        let mut total_budget_by_currency = HashMap::new();
        let mut total_cost_by_currency = HashMap::new();
        
        for project in &projects {
            let budget_entry = total_budget_by_currency.entry(project.budget.currency.clone()).or_insert(0.0);
            *budget_entry += project.budget.amount;
            
            let cost_entry = total_cost_by_currency.entry(project.actual_cost.currency.clone()).or_insert(0.0);
            *cost_entry += project.actual_cost.amount;
        }

        // 状态统计
        let mut status_counts = HashMap::new();
        for project in &projects {
            *status_counts.entry(format!("{:?}", project.status)).or_insert(0) += 1;
        }

        // 优先级统计
        let mut priority_counts = HashMap::new();
        for project in &projects {
            *priority_counts.entry(format!("{:?}", project.priority)).or_insert(0) += 1;
        }

        let over_budget_count = projects.iter().filter(|p| p.is_over_budget()).count();
        let active_count = projects.iter().filter(|p| p.is_active()).count();

        ProjectStatistics {
            total_projects,
            total_budget_by_currency,
            total_cost_by_currency,
            status_distribution: status_counts,
            priority_distribution: priority_counts,
            over_budget_projects: over_budget_count,
            active_projects: active_count,
        }
    }
}

/// 项目统计信息
#[derive(Debug, Default)]
pub struct ProjectStatistics {
    pub total_projects: usize,
    pub total_budget_by_currency: HashMap<String, f64>,
    pub total_cost_by_currency: HashMap<String, f64>,
    pub status_distribution: HashMap<String, u32>,
    pub priority_distribution: HashMap<String, u32>,
    pub over_budget_projects: usize,
    pub active_projects: usize,
}

impl fmt::Display for ProjectStatistics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "=== 项目统计信息 ===")?;
        writeln!(f, "总项目数: {}", self.total_projects)?;
        writeln!(f, "活跃项目数: {}", self.active_projects)?;
        writeln!(f, "超预算项目数: {}", self.over_budget_projects)?;
        
        writeln!(f, "\n预算统计:")?;
        for (currency, total) in &self.total_budget_by_currency {
            let cost = self.total_cost_by_currency.get(currency).unwrap_or(&0.0);
            let variance = total - cost;
            writeln!(f, "  {}: 预算 {:.2}, 实际 {:.2}, 差异 {:.2}", 
                    currency, total, cost, variance)?;
        }
        
        writeln!(f, "\n状态分布:")?;
        for (status, count) in &self.status_distribution {
            writeln!(f, "  {}: {} 个项目", status, count)?;
        }
        
        writeln!(f, "\n优先级分布:")?;
        for (priority, count) in &self.priority_distribution {
            writeln!(f, "  {}: {} 个项目", priority, count)?;
        }
        
        Ok(())
    }
}

/// 演示嵌入值模式
pub fn demo() {
    println!("=== 嵌入值模式演示 ===\n");
    
    let mut mapper = ProjectEmbeddedMapper::new();
    
    println!("1. 创建包含嵌入值对象的项目:");
    
    // 创建金额值对象
    let budget1 = Money::new(1000000.0, "CNY").unwrap();
    let schedule1 = DateRange::new("2024-01-01".to_string(), "2024-06-30".to_string()).unwrap();
    
    // 创建项目（包含嵌入值）
    let mut project1 = Project::new(
        "电商平台开发".to_string(),
        "构建全新的电商平台系统".to_string(),
        budget1,
        schedule1
    );
    project1.set_priority(ProjectPriority::High);
    
    // 保存项目
    match mapper.save(project1) {
        Ok(saved_project) => {
            println!("✅ 项目保存成功: {}", saved_project.name);
            println!("   预算: {}", saved_project.budget);
            println!("   时间范围: {}", saved_project.schedule);
            println!("   项目天数: {} 天", saved_project.duration_days());
        }
        Err(e) => println!("❌ 保存失败: {}", e),
    }
    
    println!("\n{}", "=".repeat(50));
    
    // 创建更多项目
    println!("2. 创建更多项目:");
    
    let projects_data = vec![
        ("移动应用开发", "开发iOS和Android移动应用", 500000.0, "USD", "2024-02-01", "2024-05-31", ProjectPriority::Medium),
        ("数据分析平台", "构建大数据分析和可视化平台", 800000.0, "CNY", "2024-03-01", "2024-08-31", ProjectPriority::High),
        ("客服系统升级", "升级现有客服系统功能", 200000.0, "CNY", "2024-01-15", "2024-03-15", ProjectPriority::Low),
    ];
    
    for (name, desc, budget_amount, currency, start, end, priority) in projects_data {
        let budget = Money::new(budget_amount, currency).unwrap();
        let schedule = DateRange::new(start.to_string(), end.to_string()).unwrap();
        
        let mut project = Project::new(name.to_string(), desc.to_string(), budget, schedule);
        project.set_priority(priority);
        
        if let Ok(saved) = mapper.save(project) {
            println!("✅ 创建项目: {} (预算: {}, 周期: {} 天)", 
                    saved.name, saved.budget, saved.duration_days());
        }
    }
    
    println!("\n{}", "=".repeat(50));
    
    // 项目操作
    println!("3. 项目操作:");
    
    // 开始项目
    if let Ok(mut project) = mapper.find(1) {
        println!("\n开始项目: {}", project.name);
        match project.start_project() {
            Ok(_) => {
                println!("✅ 项目已开始");
                // 添加成本
                let cost1 = Money::new(150000.0, "CNY").unwrap();
                project.add_cost(cost1).unwrap();
                
                let cost2 = Money::new(200000.0, "CNY").unwrap();
                project.add_cost(cost2).unwrap();
                
                println!("   已发生成本: {}", project.actual_cost);
                println!("   预算利用率: {:.1}%", project.budget_utilization_percentage());
                
                match project.budget_variance() {
                    Ok(variance) => println!("   预算余额: {}", variance),
                    Err(_) => println!("   ⚠️ 项目超预算！超支: {}", 
                                    project.actual_cost.subtract(&project.budget).unwrap()),
                }
                
                mapper.save(project).ok();
            }
            Err(e) => println!("❌ 开始项目失败: {}", e),
        }
    }
    
    println!("\n{}", "=".repeat(50));
    
    // 查询操作
    println!("4. 查询操作:");
    
    // 按状态查询
    println!("\n按状态查询 (进行中):");
    let active_projects = mapper.find_by_status(ProjectStatus::InProgress);
    for project in &active_projects {
        println!("  - {} (状态: {:?}, 优先级: {:?})", 
                project.name, project.status, project.priority);
    }
    
    // 按货币类型查询
    println!("\n按货币类型查询 (CNY):");
    let cny_projects = mapper.find_by_currency("CNY");
    for project in &cny_projects {
        println!("  - {} (预算: {}, 实际: {})", 
                project.name, project.budget, project.actual_cost);
    }
    
    // 查找超预算项目
    println!("\n超预算项目:");
    let over_budget = mapper.find_over_budget_projects();
    if over_budget.is_empty() {
        println!("  无超预算项目");
    } else {
        for project in &over_budget {
            let over_amount = project.actual_cost.subtract(&project.budget).unwrap();
            println!("  - {} (超支: {})", project.name, over_amount);
        }
    }
    
    // 按日期范围查询
    println!("\n2024年上半年项目:");
    let first_half_projects = mapper.find_by_date_range("2024-01-01", "2024-06-30");
    for project in &first_half_projects {
        println!("  - {} (时间: {})", project.name, project.schedule);
    }
    
    println!("\n{}", "=".repeat(50));
    
    // 值对象操作演示
    println!("5. 嵌入值对象操作:");
    
    // Money值对象操作
    println!("\nMoney值对象操作:");
    let money1 = Money::new(1000.0, "CNY").unwrap();
    let money2 = Money::new(500.0, "CNY").unwrap();
    
    match money1.add(&money2) {
        Ok(sum) => println!("  {} + {} = {}", money1, money2, sum),
        Err(e) => println!("  加法错误: {}", e),
    }
    
    match money1.subtract(&money2) {
        Ok(diff) => println!("  {} - {} = {}", money1, money2, diff),
        Err(e) => println!("  减法错误: {}", e),
    }
    
    match money1.multiply(1.5) {
        Ok(product) => println!("  {} × 1.5 = {}", money1, product),
        Err(e) => println!("  乘法错误: {}", e),
    }
    
    // DateRange值对象操作
    println!("\nDateRange值对象操作:");
    let range1 = DateRange::new("2024-01-01".to_string(), "2024-03-31".to_string()).unwrap();
    let range2 = DateRange::new("2024-02-01".to_string(), "2024-04-30".to_string()).unwrap();
    
    println!("  范围1: {}", range1);
    println!("  范围2: {}", range2);
    println!("  范围1持续天数: {} 天", range1.duration_days());
    println!("  是否重叠: {}", range1.overlaps_with(&range2));
    println!("  范围1包含2024-02-15: {}", range1.contains_date("2024-02-15"));
    
    println!("\n{}", "=".repeat(50));
    
    // 统计信息
    println!("6. 统计信息:");
    let stats = mapper.get_statistics();
    println!("{}", stats);
    
    println!("\n{}", "=".repeat(50));
    
    println!("嵌入值模式的特点:");
    println!("✅ 值对象的属性直接映射到拥有者的表中");
    println!("✅ 避免了额外的表和JOIN操作");
    println!("✅ 保持值对象的不可变性和业务逻辑");
    println!("✅ 简化了持久化和查询操作");
    println!("✅ 值对象生命周期与拥有者一致");
    
    println!("\n适用场景:");
    println!("• 金额、货币等值对象");
    println!("• 日期范围、时间段等值对象");
    println!("• 地址、坐标等组合值对象");
    println!("• 不需要独立查询的值对象");
    println!("• 简单的值对象，字段不多的情况");
    
    println!("\n注意事项:");
    println!("• 值对象字段较多时会使主表变宽");
    println!("• 多个相同类型值对象需要前缀区分");
    println!("• 值对象的验证逻辑需要在应用层实现");
    println!("• 数据库层面缺乏值对象的约束");
} 