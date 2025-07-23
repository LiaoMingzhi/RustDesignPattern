//! 依赖映射模式 (Dependent Mapping)
//! 
//! 将一个对象的某个字段或属性映射到另一个对象中，而不是独立的表
//! 文件路径：/d%3A/workspace/RustLearn/RustDesignPattern/src/EnterpriseAppPattern/ObjectRelationalStructuralPatterns/dependent_mapping.rs

use std::collections::HashMap;
use std::fmt;
use std::error::Error;

/// 依赖映射错误
#[derive(Debug)]
pub enum DependentMappingError {
    MappingError(String),
    ValidationError(String),
    DatabaseError(String),
    NotFound(String),
}

impl fmt::Display for DependentMappingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DependentMappingError::MappingError(msg) => write!(f, "映射错误: {}", msg),
            DependentMappingError::ValidationError(msg) => write!(f, "验证错误: {}", msg),
            DependentMappingError::DatabaseError(msg) => write!(f, "数据库错误: {}", msg),
            DependentMappingError::NotFound(msg) => write!(f, "未找到: {}", msg),
        }
    }
}

impl Error for DependentMappingError {}

/// 地址值对象（依赖对象）
#[derive(Debug, Clone, PartialEq)]
pub struct Address {
    pub street: String,
    pub city: String,
    pub state: String,
    pub zip_code: String,
    pub country: String,
}

impl Address {
    pub fn new(street: String, city: String, state: String, zip_code: String, country: String) -> Self {
        Self { street, city, state, zip_code, country }
    }

    pub fn validate(&self) -> Result<(), DependentMappingError> {
        if self.street.trim().is_empty() {
            return Err(DependentMappingError::ValidationError("街道地址不能为空".to_string()));
        }
        if self.city.trim().is_empty() {
            return Err(DependentMappingError::ValidationError("城市不能为空".to_string()));
        }
        if self.zip_code.trim().is_empty() {
            return Err(DependentMappingError::ValidationError("邮政编码不能为空".to_string()));
        }
        Ok(())
    }

    pub fn is_valid_zip_code(&self) -> bool {
        self.zip_code.chars().all(|c| c.is_ascii_digit()) && self.zip_code.len() == 6
    }

    pub fn format_full(&self) -> String {
        format!("{}, {}, {} {}, {}", 
                self.street, self.city, self.state, self.zip_code, self.country)
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format_full())
    }
}

/// 联系信息值对象（依赖对象）
#[derive(Debug, Clone, PartialEq)]
pub struct ContactInfo {
    pub phone: String,
    pub email: String,
    pub fax: Option<String>,
    pub website: Option<String>,
}

impl ContactInfo {
    pub fn new(phone: String, email: String) -> Self {
        Self {
            phone,
            email,
            fax: None,
            website: None,
        }
    }

    pub fn with_fax(mut self, fax: String) -> Self {
        self.fax = Some(fax);
        self
    }

    pub fn with_website(mut self, website: String) -> Self {
        self.website = Some(website);
        self
    }

    pub fn validate(&self) -> Result<(), DependentMappingError> {
        if self.phone.trim().is_empty() {
            return Err(DependentMappingError::ValidationError("电话号码不能为空".to_string()));
        }
        if self.email.trim().is_empty() || !self.email.contains('@') {
            return Err(DependentMappingError::ValidationError("邮箱格式不正确".to_string()));
        }
        Ok(())
    }

    pub fn has_complete_info(&self) -> bool {
        !self.phone.is_empty() && !self.email.is_empty() && 
        self.fax.is_some() && self.website.is_some()
    }
}

/// 公司实体（主对象，包含依赖映射）
#[derive(Debug, Clone)]
pub struct Company {
    pub id: Option<u32>,
    pub name: String,
    pub address: Address,        // 依赖映射的地址
    pub contact_info: ContactInfo, // 依赖映射的联系信息
    pub founded_year: u32,
    pub employee_count: u32,
    pub industry: String,
}

impl Company {
    pub fn new(name: String, address: Address, contact_info: ContactInfo, 
               founded_year: u32, industry: String) -> Self {
        Self {
            id: None,
            name,
            address,
            contact_info,
            founded_year,
            employee_count: 0,
            industry,
        }
    }

    pub fn validate(&self) -> Result<(), DependentMappingError> {
        if self.name.trim().is_empty() {
            return Err(DependentMappingError::ValidationError("公司名称不能为空".to_string()));
        }
        
        self.address.validate()?;
        self.contact_info.validate()?;
        
        let current_year = 2024; // 简化
        if self.founded_year > current_year {
            return Err(DependentMappingError::ValidationError("成立年份不能是未来".to_string()));
        }
        
        Ok(())
    }

    pub fn age_years(&self) -> u32 {
        2024 - self.founded_year // 简化
    }

    pub fn update_employee_count(&mut self, count: u32) {
        self.employee_count = count;
    }

    pub fn update_address(&mut self, address: Address) -> Result<(), DependentMappingError> {
        address.validate()?;
        self.address = address;
        Ok(())
    }

    pub fn update_contact_info(&mut self, contact_info: ContactInfo) -> Result<(), DependentMappingError> {
        contact_info.validate()?;
        self.contact_info = contact_info;
        Ok(())
    }
}

/// 依赖映射器 - 处理公司及其依赖对象的映射
pub struct CompanyDependentMapper {
    companies: HashMap<u32, Company>,
    next_id: u32,
}

impl CompanyDependentMapper {
    pub fn new() -> Self {
        Self {
            companies: HashMap::new(),
            next_id: 1,
        }
    }

    /// 保存公司（包括依赖对象）
    pub fn save(&mut self, mut company: Company) -> Result<Company, DependentMappingError> {
        company.validate()?;

        let id = match company.id {
            Some(existing_id) => {
                println!("更新公司: {}", company.name);
                existing_id
            }
            None => {
                let new_id = self.next_id;
                self.next_id += 1;
                company.id = Some(new_id);
                println!("创建公司: {}", company.name);
                new_id
            }
        };

        // 模拟SQL语句生成
        self.generate_sql_statements(&company)?;
        
        self.companies.insert(id, company.clone());
        Ok(company)
    }

    /// 查找公司
    pub fn find(&self, id: u32) -> Result<Company, DependentMappingError> {
        match self.companies.get(&id) {
            Some(company) => {
                println!("从数据库加载公司: {}", company.name);
                Ok(company.clone())
            }
            None => Err(DependentMappingError::NotFound(format!("公司ID {} 不存在", id)))
        }
    }

    /// 按名称查找公司
    pub fn find_by_name(&self, name: &str) -> Vec<Company> {
        self.companies.values()
            .filter(|company| company.name.contains(name))
            .cloned()
            .collect()
    }

    /// 按城市查找公司
    pub fn find_by_city(&self, city: &str) -> Vec<Company> {
        self.companies.values()
            .filter(|company| company.address.city.contains(city))
            .cloned()
            .collect()
    }

    /// 按行业查找公司
    pub fn find_by_industry(&self, industry: &str) -> Vec<Company> {
        self.companies.values()
            .filter(|company| company.industry.contains(industry))
            .cloned()
            .collect()
    }

    /// 删除公司
    pub fn delete(&mut self, id: u32) -> Result<(), DependentMappingError> {
        match self.companies.remove(&id) {
            Some(company) => {
                println!("删除公司: {}", company.name);
                println!("生成SQL: DELETE FROM companies WHERE id = {}", id);
                Ok(())
            }
            None => Err(DependentMappingError::NotFound(format!("公司ID {} 不存在", id)))
        }
    }

    /// 获取所有公司
    pub fn find_all(&self) -> Vec<Company> {
        self.companies.values().cloned().collect()
    }

    /// 生成SQL语句（模拟依赖映射的SQL生成）
    fn generate_sql_statements(&self, company: &Company) -> Result<(), DependentMappingError> {
        match company.id {
            Some(id) => {
                // 更新SQL - 注意依赖对象的字段被映射到主表中
                println!("生成UPDATE SQL:");
                println!("UPDATE companies SET");
                println!("  name = '{}',", company.name);
                println!("  address_street = '{}',", company.address.street);
                println!("  address_city = '{}',", company.address.city);
                println!("  address_state = '{}',", company.address.state);
                println!("  address_zip_code = '{}',", company.address.zip_code);
                println!("  address_country = '{}',", company.address.country);
                println!("  contact_phone = '{}',", company.contact_info.phone);
                println!("  contact_email = '{}',", company.contact_info.email);
                if let Some(ref fax) = company.contact_info.fax {
                    println!("  contact_fax = '{}',", fax);
                }
                if let Some(ref website) = company.contact_info.website {
                    println!("  contact_website = '{}',", website);
                }
                println!("  founded_year = {},", company.founded_year);
                println!("  employee_count = {},", company.employee_count);
                println!("  industry = '{}'", company.industry);
                println!("WHERE id = {};", id);
            }
            None => {
                // 插入SQL
                println!("生成INSERT SQL:");
                println!("INSERT INTO companies (");
                println!("  name, address_street, address_city, address_state, address_zip_code, address_country,");
                println!("  contact_phone, contact_email, contact_fax, contact_website,");
                println!("  founded_year, employee_count, industry");
                println!(") VALUES (");
                println!("  '{}', '{}', '{}', '{}', '{}', '{}',", 
                        company.name, company.address.street, company.address.city, 
                        company.address.state, company.address.zip_code, company.address.country);
                print!("  '{}', '{}'", company.contact_info.phone, company.contact_info.email);
                if let Some(ref fax) = company.contact_info.fax {
                    print!(", '{}'", fax);
                } else {
                    print!(", NULL");
                }
                if let Some(ref website) = company.contact_info.website {
                    print!(", '{}'", website);
                } else {
                    print!(", NULL");
                }
                println!(",");
                println!("  {}, {}, '{}'", company.founded_year, company.employee_count, company.industry);
                println!(");");
            }
        }
        Ok(())
    }

    /// 获取统计信息
    pub fn get_statistics(&self) -> CompanyStatistics {
        if self.companies.is_empty() {
            return CompanyStatistics::default();
        }

        let companies: Vec<&Company> = self.companies.values().collect();
        let total_companies = companies.len();
        let total_employees: u32 = companies.iter().map(|c| c.employee_count).sum();
        let avg_employees = total_employees as f64 / total_companies as f64;
        
        let oldest_year = companies.iter().map(|c| c.founded_year).min().unwrap_or(2024);
        let newest_year = companies.iter().map(|c| c.founded_year).max().unwrap_or(2024);
        
        // 按行业统计
        let mut industry_counts = HashMap::new();
        for company in &companies {
            *industry_counts.entry(company.industry.clone()).or_insert(0) += 1;
        }

        // 按城市统计
        let mut city_counts = HashMap::new();
        for company in &companies {
            *city_counts.entry(company.address.city.clone()).or_insert(0) += 1;
        }

        CompanyStatistics {
            total_companies,
            total_employees,
            average_employees: avg_employees,
            oldest_company_year: oldest_year,
            newest_company_year: newest_year,
            industry_distribution: industry_counts,
            city_distribution: city_counts,
        }
    }
}

/// 公司统计信息
#[derive(Debug, Default)]
pub struct CompanyStatistics {
    pub total_companies: usize,
    pub total_employees: u32,
    pub average_employees: f64,
    pub oldest_company_year: u32,
    pub newest_company_year: u32,
    pub industry_distribution: HashMap<String, u32>,
    pub city_distribution: HashMap<String, u32>,
}

impl fmt::Display for CompanyStatistics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "=== 公司统计信息 ===")?;
        writeln!(f, "总公司数: {}", self.total_companies)?;
        writeln!(f, "总员工数: {}", self.total_employees)?;
        writeln!(f, "平均员工数: {:.1}", self.average_employees)?;
        writeln!(f, "最老公司成立年份: {}", self.oldest_company_year)?;
        writeln!(f, "最新公司成立年份: {}", self.newest_company_year)?;
        
        writeln!(f, "\n行业分布:")?;
        for (industry, count) in &self.industry_distribution {
            writeln!(f, "  {}: {} 家公司", industry, count)?;
        }
        
        writeln!(f, "\n城市分布:")?;
        for (city, count) in &self.city_distribution {
            writeln!(f, "  {}: {} 家公司", city, count)?;
        }
        
        Ok(())
    }
}

/// 演示依赖映射模式
pub fn demo() {
    println!("=== 依赖映射模式演示 ===\n");
    
    let mut mapper = CompanyDependentMapper::new();
    
    println!("1. 创建公司及其依赖对象:");
    
    // 创建地址对象
    let address1 = Address::new(
        "北京市朝阳区建国门外大街1号".to_string(),
        "北京".to_string(),
        "北京市".to_string(),
        "100001".to_string(),
        "中国".to_string()
    );
    
    // 创建联系信息对象
    let contact1 = ContactInfo::new(
        "010-85120000".to_string(),
        "info@company1.com".to_string()
    ).with_fax("010-85120001".to_string())
     .with_website("https://www.company1.com".to_string());
    
    // 创建公司对象（包含依赖映射）
    let mut company1 = Company::new(
        "北京科技有限公司".to_string(),
        address1,
        contact1,
        2010,
        "科技".to_string()
    );
    company1.update_employee_count(500);
    
    // 保存公司
    match mapper.save(company1) {
        Ok(saved_company) => {
            println!("✅ 公司保存成功: {}", saved_company.name);
            println!("   地址: {}", saved_company.address);
            println!("   联系电话: {}", saved_company.contact_info.phone);
            println!("   邮箱: {}", saved_company.contact_info.email);
        }
        Err(e) => println!("❌ 保存失败: {}", e),
    }
    
    println!("\n{}", "=".repeat(50));
    
    // 创建更多公司
    println!("2. 创建更多公司:");
    
    let companies_data = vec![
        ("上海金融集团", "上海市浦东新区陆家嘴环路1000号", "上海", "上海市", "200120", "021-58888888", "contact@shanghai-finance.com", 2005, "金融", 1200),
        ("深圳创新科技", "深圳市南山区科技园南区深南大道9999号", "深圳", "广东省", "518057", "0755-12345678", "info@shenzhen-tech.com", 2015, "科技", 800),
        ("广州制造企业", "广州市天河区珠江新城花城大道123号", "广州", "广东省", "510623", "020-87654321", "service@guangzhou-mfg.com", 2000, "制造", 2000),
    ];
    
    for (name, street, city, state, zip, phone, email, year, industry, employees) in companies_data {
        let address = Address::new(
            street.to_string(),
            city.to_string(), 
            state.to_string(),
            zip.to_string(),
            "中国".to_string()
        );
        
        let contact = ContactInfo::new(phone.to_string(), email.to_string());
        
        let mut company = Company::new(
            name.to_string(),
            address,
            contact,
            year,
            industry.to_string()
        );
        company.update_employee_count(employees);
        
        if let Ok(saved) = mapper.save(company) {
            println!("✅ 创建公司: {} ({}年成立, {}人)", saved.name, saved.founded_year, saved.employee_count);
        }
    }
    
    println!("\n{}", "=".repeat(50));
    
    // 查询操作
    println!("3. 查询操作:");
    
    // 按城市查询
    println!("\n按城市查询 (北京):");
    let beijing_companies = mapper.find_by_city("北京");
    for company in &beijing_companies {
        println!("  - {} (地址: {})", company.name, company.address.city);
    }
    
    // 按行业查询
    println!("\n按行业查询 (科技):");
    let tech_companies = mapper.find_by_industry("科技");
    for company in &tech_companies {
        println!("  - {} ({}人, {}年成立)", company.name, company.employee_count, company.founded_year);
    }
    
    // 查找特定公司
    println!("\n查找特定公司 (ID: 1):");
    match mapper.find(1) {
        Ok(company) => {
            println!("  公司名称: {}", company.name);
            println!("  完整地址: {}", company.address);
            println!("  联系信息: 电话: {}, 邮箱: {}", company.contact_info.phone, company.contact_info.email);
            println!("  公司年龄: {} 年", company.age_years());
            if let Some(ref website) = company.contact_info.website {
                println!("  网站: {}", website);
            }
        }
        Err(e) => println!("  ❌ {}", e),
    }
    
    println!("\n{}", "=".repeat(50));
    
    // 更新操作
    println!("4. 更新依赖对象:");
    
    if let Ok(mut company) = mapper.find(2) {
        println!("更新前地址: {}", company.address);
        
        // 更新地址
        let new_address = Address::new(
            "上海市浦东新区陆家嘴环路2000号".to_string(),
            "上海".to_string(),
            "上海市".to_string(),
            "200121".to_string(),
            "中国".to_string()
        );
        
        match company.update_address(new_address) {
            Ok(_) => {
                if let Ok(updated) = mapper.save(company) {
                    println!("✅ 地址更新成功");
                    println!("更新后地址: {}", updated.address);
                }
            }
            Err(e) => println!("❌ 地址更新失败: {}", e),
        }
    }
    
    println!("\n{}", "=".repeat(50));
    
    // 统计信息
    println!("5. 统计信息:");
    let stats = mapper.get_statistics();
    println!("{}", stats);
    
    println!("\n{}", "=".repeat(50));
    
    println!("依赖映射模式的特点:");
    println!("✅ 将依赖对象的属性映射到主对象的表中");
    println!("✅ 减少了表的数量和JOIN操作");
    println!("✅ 适用于值对象和简单的依赖关系");
    println!("✅ 提高查询性能，简化数据访问");
    println!("✅ 保持对象的完整性和一致性");
    
    println!("\n适用场景:");
    println!("• 值对象的映射（如地址、联系信息）");
    println!("• 简单的组合关系");
    println!("• 不需要独立查询依赖对象的场景");
    println!("• 性能要求较高的查询场景");
    println!("• 依赖对象生命周期与主对象一致的情况");
} 