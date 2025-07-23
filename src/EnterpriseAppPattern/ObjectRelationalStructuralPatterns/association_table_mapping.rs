// 文件路径: /d%3A/workspace/RustLearn/RustDesignPattern/src/EnterpriseAppPattern/ObjectRelationalStructuralPatterns/association_table_mapping.rs

//! # 关联表映射模式 (Association Table Mapping)
//!
//! ## 概述
//! 关联表映射模式用于处理对象间的多对多关系，通过一个独立的关联表
//! 来保存两个实体间的关联关系。这个模式将复杂的多对多关系分解为
//! 两个一对多的关系。
//!
//! ## 优点
//! - 处理复杂的多对多关系
//! - 支持关联属性（在关联表中存储额外信息）
//! - 保持数据一致性
//! - 便于查询和维护
//! - 符合关系数据库设计规范
//!
//! ## 适用场景
//! - 学生与课程的关系（一个学生可以选多门课，一门课可以被多个学生选择）
//! - 用户与角色的关系
//! - 标签与文章的关系
//! - 项目与开发人员的关系

use std::collections::{HashMap, HashSet};
use std::fmt;

/// 关联表映射错误
#[derive(Debug)]
pub enum AssociationMappingError {
    EntityNotFound(String),
    AssociationExists(String),
    AssociationNotFound(String),
    DatabaseError(String),
    ValidationError(String),
}

impl fmt::Display for AssociationMappingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AssociationMappingError::EntityNotFound(msg) => write!(f, "实体未找到: {}", msg),
            AssociationMappingError::AssociationExists(msg) => write!(f, "关联已存在: {}", msg),
            AssociationMappingError::AssociationNotFound(msg) => write!(f, "关联未找到: {}", msg),
            AssociationMappingError::DatabaseError(msg) => write!(f, "数据库错误: {}", msg),
            AssociationMappingError::ValidationError(msg) => write!(f, "验证错误: {}", msg),
        }
    }
}

impl std::error::Error for AssociationMappingError {}

/// 学生实体
#[derive(Debug, Clone, PartialEq)]
pub struct Student {
    pub id: u32,
    pub name: String,
    pub email: String,
    pub major: String,
    pub enrollment_year: u32,
}

impl Student {
    pub fn new(id: u32, name: String, email: String, major: String, enrollment_year: u32) -> Self {
        Self {
            id,
            name,
            email,
            major,
            enrollment_year,
        }
    }
}

impl fmt::Display for Student {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Student[{}]: {} ({}) - {} 级 {}", 
               self.id, self.name, self.email, self.enrollment_year, self.major)
    }
}

/// 课程实体
#[derive(Debug, Clone, PartialEq)]
pub struct Course {
    pub id: u32,
    pub code: String,
    pub name: String,
    pub credits: u32,
    pub department: String,
    pub instructor: String,
}

impl Course {
    pub fn new(id: u32, code: String, name: String, credits: u32, department: String, instructor: String) -> Self {
        Self {
            id,
            code,
            name,
            credits,
            department,
            instructor,
        }
    }
}

impl fmt::Display for Course {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Course[{}]: {} - {} ({} 学分) by {}", 
               self.id, self.code, self.name, self.credits, self.instructor)
    }
}

/// 选课记录（关联表实体，包含关联属性）
#[derive(Debug, Clone, PartialEq)]
pub struct Enrollment {
    pub student_id: u32,
    pub course_id: u32,
    pub enrollment_date: String,
    pub grade: Option<String>,
    pub status: EnrollmentStatus,
    pub semester: String,
}

impl Enrollment {
    pub fn new(student_id: u32, course_id: u32, semester: String) -> Self {
        Self {
            student_id,
            course_id,
            enrollment_date: "2024-01-01".to_string(),
            grade: None,
            status: EnrollmentStatus::Enrolled,
            semester,
        }
    }

    pub fn with_grade(mut self, grade: String) -> Self {
        self.grade = Some(grade);
        self.status = EnrollmentStatus::Completed;
        self
    }
}

impl fmt::Display for Enrollment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let grade_str = self.grade.as_ref().map(|g| g.as_str()).unwrap_or("未评分");
        write!(f, "Enrollment[学生:{}, 课程:{}, 学期:{}, 状态:{:?}, 成绩:{}]", 
               self.student_id, self.course_id, self.semester, self.status, grade_str)
    }
}

/// 选课状态
#[derive(Debug, Clone, PartialEq)]
pub enum EnrollmentStatus {
    Enrolled,    // 已选课
    Dropped,     // 已退课
    Completed,   // 已完成
    InProgress,  // 进行中
}

/// 用户实体
#[derive(Debug, Clone, PartialEq)]
pub struct User {
    pub id: u32,
    pub username: String,
    pub email: String,
    pub full_name: String,
}

impl User {
    pub fn new(id: u32, username: String, email: String, full_name: String) -> Self {
        Self { id, username, email, full_name }
    }
}

impl fmt::Display for User {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "User[{}]: {} - {}", self.id, self.username, self.full_name)
    }
}

/// 角色实体
#[derive(Debug, Clone, PartialEq)]
pub struct Role {
    pub id: u32,
    pub name: String,
    pub description: String,
    pub permissions: Vec<String>,
}

impl Role {
    pub fn new(id: u32, name: String, description: String, permissions: Vec<String>) -> Self {
        Self { id, name, description, permissions }
    }
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Role[{}]: {} - {} (权限: {})", 
               self.id, self.name, self.description, self.permissions.len())
    }
}

/// 用户角色关联
#[derive(Debug, Clone, PartialEq)]
pub struct UserRole {
    pub user_id: u32,
    pub role_id: u32,
    pub assigned_date: String,
    pub assigned_by: u32,
    pub expires_at: Option<String>,
}

impl UserRole {
    pub fn new(user_id: u32, role_id: u32, assigned_by: u32) -> Self {
        Self {
            user_id,
            role_id,
            assigned_date: "2024-01-01".to_string(),
            assigned_by,
            expires_at: None,
        }
    }

    pub fn with_expiry(mut self, expires_at: String) -> Self {
        self.expires_at = Some(expires_at);
        self
    }
}

/// 学生选课系统的关联表映射器
pub struct StudentCourseMapper {
    students: HashMap<u32, Student>,
    courses: HashMap<u32, Course>,
    enrollments: Vec<Enrollment>,
    next_student_id: u32,
    next_course_id: u32,
}

impl StudentCourseMapper {
    pub fn new() -> Self {
        let mut mapper = Self {
            students: HashMap::new(),
            courses: HashMap::new(),
            enrollments: Vec::new(),
            next_student_id: 1,
            next_course_id: 1,
        };
        
        mapper.init_test_data();
        mapper
    }

    fn init_test_data(&mut self) {
        // 添加学生
        self.add_student(Student::new(1, "张三".to_string(), "zhangsan@example.com".to_string(), "计算机科学".to_string(), 2023));
        self.add_student(Student::new(2, "李四".to_string(), "lisi@example.com".to_string(), "软件工程".to_string(), 2023));
        self.add_student(Student::new(3, "王五".to_string(), "wangwu@example.com".to_string(), "数据科学".to_string(), 2022));
        
        // 添加课程
        self.add_course(Course::new(1, "CS101".to_string(), "计算机科学导论".to_string(), 3, "计算机学院".to_string(), "张教授".to_string()));
        self.add_course(Course::new(2, "CS102".to_string(), "数据结构与算法".to_string(), 4, "计算机学院".to_string(), "李教授".to_string()));
        self.add_course(Course::new(3, "MATH201".to_string(), "高等数学".to_string(), 4, "数学学院".to_string(), "王教授".to_string()));
        self.add_course(Course::new(4, "ENG101".to_string(), "大学英语".to_string(), 2, "外语学院".to_string(), "Smith教授".to_string()));
        
        // 添加选课记录
        self.enroll_student(1, 1, "2024春季".to_string()).unwrap();
        self.enroll_student(1, 2, "2024春季".to_string()).unwrap();
        self.enroll_student(1, 3, "2024春季".to_string()).unwrap();
        
        self.enroll_student(2, 1, "2024春季".to_string()).unwrap();
        self.enroll_student(2, 4, "2024春季".to_string()).unwrap();
        
        self.enroll_student(3, 2, "2024春季".to_string()).unwrap();
        self.enroll_student(3, 3, "2024春季".to_string()).unwrap();
        self.enroll_student(3, 4, "2024春季".to_string()).unwrap();
        
        // 设置一些成绩
        self.set_grade(1, 1, "A".to_string()).unwrap();
        self.set_grade(1, 3, "B+".to_string()).unwrap();
        self.set_grade(2, 1, "A-".to_string()).unwrap();
        self.set_grade(3, 2, "B".to_string()).unwrap();
        
        self.next_student_id = 4;
        self.next_course_id = 5;
    }

    /// 添加学生
    pub fn add_student(&mut self, student: Student) {
        self.students.insert(student.id, student);
    }

    /// 添加课程
    pub fn add_course(&mut self, course: Course) {
        self.courses.insert(course.id, course);
    }

    /// 学生选课（建立关联）
    pub fn enroll_student(&mut self, student_id: u32, course_id: u32, semester: String) -> Result<(), AssociationMappingError> {
        // 验证学生和课程是否存在
        if !self.students.contains_key(&student_id) {
            return Err(AssociationMappingError::EntityNotFound(format!("学生不存在: {}", student_id)));
        }
        
        if !self.courses.contains_key(&course_id) {
            return Err(AssociationMappingError::EntityNotFound(format!("课程不存在: {}", course_id)));
        }
        
        // 检查是否已经选课
        if self.enrollments.iter().any(|e| e.student_id == student_id && e.course_id == course_id && e.semester == semester) {
            return Err(AssociationMappingError::AssociationExists(
                format!("学生 {} 已经选择了课程 {} (学期: {})", student_id, course_id, semester)
            ));
        }
        
        // 创建选课记录
        let enrollment = Enrollment::new(student_id, course_id, semester);
        self.enrollments.push(enrollment);
        
        println!("  ✅ 学生 {} 成功选择课程 {}", student_id, course_id);
        Ok(())
    }

    /// 学生退课（删除关联）
    pub fn drop_course(&mut self, student_id: u32, course_id: u32, semester: String) -> Result<(), AssociationMappingError> {
        let index = self.enrollments.iter().position(|e| {
            e.student_id == student_id && e.course_id == course_id && e.semester == semester
        });
        
        match index {
            Some(idx) => {
                let mut enrollment = self.enrollments[idx].clone();
                enrollment.status = EnrollmentStatus::Dropped;
                self.enrollments[idx] = enrollment;
                println!("  🚫 学生 {} 退选课程 {}", student_id, course_id);
                Ok(())
            },
            None => Err(AssociationMappingError::AssociationNotFound(
                format!("未找到学生 {} 的课程 {} 选课记录", student_id, course_id)
            )),
        }
    }

    /// 设置成绩
    pub fn set_grade(&mut self, student_id: u32, course_id: u32, grade: String) -> Result<(), AssociationMappingError> {
        let enrollment = self.enrollments.iter_mut().find(|e| {
            e.student_id == student_id && e.course_id == course_id && e.status == EnrollmentStatus::Enrolled
        });
        
        match enrollment {
            Some(e) => {
                e.grade = Some(grade.clone());
                e.status = EnrollmentStatus::Completed;
                println!("  📊 为学生 {} 的课程 {} 设置成绩: {}", student_id, course_id, grade);
                Ok(())
            },
            None => Err(AssociationMappingError::AssociationNotFound(
                format!("未找到学生 {} 的课程 {} 有效选课记录", student_id, course_id)
            )),
        }
    }

    /// 获取学生选择的所有课程
    pub fn get_student_courses(&self, student_id: u32) -> Result<Vec<(Course, Enrollment)>, AssociationMappingError> {
        if !self.students.contains_key(&student_id) {
            return Err(AssociationMappingError::EntityNotFound(format!("学生不存在: {}", student_id)));
        }
        
        let courses: Vec<(Course, Enrollment)> = self.enrollments.iter()
            .filter(|e| e.student_id == student_id)
            .filter_map(|e| {
                self.courses.get(&e.course_id).map(|course| (course.clone(), e.clone()))
            })
            .collect();
        
        Ok(courses)
    }

    /// 获取课程的所有学生
    pub fn get_course_students(&self, course_id: u32) -> Result<Vec<(Student, Enrollment)>, AssociationMappingError> {
        if !self.courses.contains_key(&course_id) {
            return Err(AssociationMappingError::EntityNotFound(format!("课程不存在: {}", course_id)));
        }
        
        let students: Vec<(Student, Enrollment)> = self.enrollments.iter()
            .filter(|e| e.course_id == course_id)
            .filter_map(|e| {
                self.students.get(&e.student_id).map(|student| (student.clone(), e.clone()))
            })
            .collect();
        
        Ok(students)
    }

    /// 获取学生的成绩单
    pub fn get_transcript(&self, student_id: u32) -> Result<Vec<(Course, String)>, AssociationMappingError> {
        let student_courses = self.get_student_courses(student_id)?;
        
        let transcript: Vec<(Course, String)> = student_courses.into_iter()
            .filter_map(|(course, enrollment)| {
                enrollment.grade.map(|grade| (course, grade))
            })
            .collect();
        
        Ok(transcript)
    }

    /// 获取课程成绩分布
    pub fn get_course_grades(&self, course_id: u32) -> Result<HashMap<String, u32>, AssociationMappingError> {
        let course_students = self.get_course_students(course_id)?;
        let mut grade_distribution = HashMap::new();
        
        for (_, enrollment) in course_students {
            if let Some(grade) = enrollment.grade {
                *grade_distribution.entry(grade).or_insert(0) += 1;
            }
        }
        
        Ok(grade_distribution)
    }

    /// 获取所有学生
    pub fn get_all_students(&self) -> Vec<&Student> {
        self.students.values().collect()
    }

    /// 获取所有课程
    pub fn get_all_courses(&self) -> Vec<&Course> {
        self.courses.values().collect()
    }

    /// 获取所有选课记录
    pub fn get_all_enrollments(&self) -> &Vec<Enrollment> {
        &self.enrollments
    }
}

/// 用户角色系统的关联表映射器
pub struct UserRoleMapper {
    users: HashMap<u32, User>,
    roles: HashMap<u32, Role>,
    user_roles: Vec<UserRole>,
    next_user_id: u32,
    next_role_id: u32,
}

impl UserRoleMapper {
    pub fn new() -> Self {
        let mut mapper = Self {
            users: HashMap::new(),
            roles: HashMap::new(),
            user_roles: Vec::new(),
            next_user_id: 1,
            next_role_id: 1,
        };
        
        mapper.init_test_data();
        mapper
    }

    fn init_test_data(&mut self) {
        // 添加用户
        self.add_user(User::new(1, "admin".to_string(), "admin@example.com".to_string(), "系统管理员".to_string()));
        self.add_user(User::new(2, "editor".to_string(), "editor@example.com".to_string(), "编辑员".to_string()));
        self.add_user(User::new(3, "viewer".to_string(), "viewer@example.com".to_string(), "普通用户".to_string()));
        
        // 添加角色
        self.add_role(Role::new(1, "Administrator".to_string(), "系统管理员".to_string(), 
                               vec!["create".to_string(), "read".to_string(), "update".to_string(), "delete".to_string(), "manage_users".to_string()]));
        self.add_role(Role::new(2, "Editor".to_string(), "编辑员".to_string(), 
                               vec!["create".to_string(), "read".to_string(), "update".to_string()]));
        self.add_role(Role::new(3, "Viewer".to_string(), "查看者".to_string(), 
                               vec!["read".to_string()]));
        self.add_role(Role::new(4, "Moderator".to_string(), "版主".to_string(), 
                               vec!["read".to_string(), "update".to_string(), "moderate".to_string()]));
        
        // 分配角色
        self.assign_role(1, 1, 1).unwrap(); // admin -> Administrator
        self.assign_role(2, 2, 1).unwrap(); // editor -> Editor
        self.assign_role(2, 4, 1).unwrap(); // editor -> Moderator (多角色)
        self.assign_role(3, 3, 1).unwrap(); // viewer -> Viewer
        
        self.next_user_id = 4;
        self.next_role_id = 5;
    }

    /// 添加用户
    pub fn add_user(&mut self, user: User) {
        self.users.insert(user.id, user);
    }

    /// 添加角色
    pub fn add_role(&mut self, role: Role) {
        self.roles.insert(role.id, role);
    }

    /// 为用户分配角色
    pub fn assign_role(&mut self, user_id: u32, role_id: u32, assigned_by: u32) -> Result<(), AssociationMappingError> {
        // 验证用户和角色是否存在
        if !self.users.contains_key(&user_id) {
            return Err(AssociationMappingError::EntityNotFound(format!("用户不存在: {}", user_id)));
        }
        
        if !self.roles.contains_key(&role_id) {
            return Err(AssociationMappingError::EntityNotFound(format!("角色不存在: {}", role_id)));
        }
        
        // 检查是否已经分配了这个角色
        if self.user_roles.iter().any(|ur| ur.user_id == user_id && ur.role_id == role_id) {
            return Err(AssociationMappingError::AssociationExists(
                format!("用户 {} 已经拥有角色 {}", user_id, role_id)
            ));
        }
        
        // 创建用户角色关联
        let user_role = UserRole::new(user_id, role_id, assigned_by);
        self.user_roles.push(user_role);
        
        println!("  ✅ 为用户 {} 分配角色 {}", user_id, role_id);
        Ok(())
    }

    /// 移除用户角色
    pub fn revoke_role(&mut self, user_id: u32, role_id: u32) -> Result<(), AssociationMappingError> {
        let index = self.user_roles.iter().position(|ur| ur.user_id == user_id && ur.role_id == role_id);
        
        match index {
            Some(idx) => {
                self.user_roles.remove(idx);
                println!("  🚫 移除用户 {} 的角色 {}", user_id, role_id);
                Ok(())
            },
            None => Err(AssociationMappingError::AssociationNotFound(
                format!("用户 {} 没有角色 {}", user_id, role_id)
            )),
        }
    }

    /// 获取用户的所有角色
    pub fn get_user_roles(&self, user_id: u32) -> Result<Vec<Role>, AssociationMappingError> {
        if !self.users.contains_key(&user_id) {
            return Err(AssociationMappingError::EntityNotFound(format!("用户不存在: {}", user_id)));
        }
        
        let roles: Vec<Role> = self.user_roles.iter()
            .filter(|ur| ur.user_id == user_id)
            .filter_map(|ur| self.roles.get(&ur.role_id).cloned())
            .collect();
        
        Ok(roles)
    }

    /// 获取角色的所有用户
    pub fn get_role_users(&self, role_id: u32) -> Result<Vec<User>, AssociationMappingError> {
        if !self.roles.contains_key(&role_id) {
            return Err(AssociationMappingError::EntityNotFound(format!("角色不存在: {}", role_id)));
        }
        
        let users: Vec<User> = self.user_roles.iter()
            .filter(|ur| ur.role_id == role_id)
            .filter_map(|ur| self.users.get(&ur.user_id).cloned())
            .collect();
        
        Ok(users)
    }

    /// 获取用户的所有权限
    pub fn get_user_permissions(&self, user_id: u32) -> Result<HashSet<String>, AssociationMappingError> {
        let roles = self.get_user_roles(user_id)?;
        let mut permissions = HashSet::new();
        
        for role in roles {
            for permission in role.permissions {
                permissions.insert(permission);
            }
        }
        
        Ok(permissions)
    }

    /// 检查用户是否有特定权限
    pub fn has_permission(&self, user_id: u32, permission: &str) -> Result<bool, AssociationMappingError> {
        let permissions = self.get_user_permissions(user_id)?;
        Ok(permissions.contains(permission))
    }

    /// 获取所有用户
    pub fn get_all_users(&self) -> Vec<&User> {
        self.users.values().collect()
    }

    /// 获取所有角色
    pub fn get_all_roles(&self) -> Vec<&Role> {
        self.roles.values().collect()
    }
}

/// 演示关联表映射模式
pub fn demo() {
    println!("=== 关联表映射模式演示 ===\n");

    // 演示学生-课程关联映射
    println!("📚 学生选课系统 - 关联表映射");
    println!("处理学生与课程之间的多对多关系\n");

    let mut student_course_mapper = StudentCourseMapper::new();

    println!("1. 初始数据展示");
    println!("   学生列表:");
    for student in student_course_mapper.get_all_students() {
        println!("     - {}", student);
    }

    println!("\n   课程列表:");
    for course in student_course_mapper.get_all_courses() {
        println!("     - {}", course);
    }

    println!("\n2. 学生选课记录");
    for enrollment in student_course_mapper.get_all_enrollments() {
        println!("     - {}", enrollment);
    }

    println!("\n3. 查询学生的课程");
    for student_id in 1..=3 {
        match student_course_mapper.get_student_courses(student_id) {
            Ok(courses) => {
                println!("   学生 {} 的课程:", student_id);
                for (course, enrollment) in courses {
                    let grade_info = enrollment.grade.as_ref().map(|g| format!(" (成绩: {})", g)).unwrap_or_else(|| " (未评分)".to_string());
                    println!("     - {} - 状态: {:?}{}", course.name, enrollment.status, grade_info);
                }
            },
            Err(e) => println!("   查询失败: {}", e),
        }
        println!();
    }

    println!("4. 查询课程的学生");
    for course_id in 1..=4 {
        match student_course_mapper.get_course_students(course_id) {
            Ok(students) => {
                let course_name = student_course_mapper.courses.get(&course_id).map(|c| c.name.as_str()).unwrap_or("未知课程");
                println!("   课程 \"{}\" 的学生:", course_name);
                for (student, enrollment) in students {
                    let grade_info = enrollment.grade.as_ref().map(|g| format!(" (成绩: {})", g)).unwrap_or_else(|| " (未评分)".to_string());
                    println!("     - {} - 状态: {:?}{}", student.name, enrollment.status, grade_info);
                }
            },
            Err(e) => println!("   查询失败: {}", e),
        }
        println!();
    }

    println!("5. 学生成绩单");
    for student_id in 1..=3 {
        match student_course_mapper.get_transcript(student_id) {
            Ok(transcript) => {
                let student_name = student_course_mapper.students.get(&student_id).map(|s| s.name.as_str()).unwrap_or("未知学生");
                println!("   {} 的成绩单:", student_name);
                if transcript.is_empty() {
                    println!("     - 暂无已评分课程");
                } else {
                    for (course, grade) in transcript {
                        println!("     - {}: {}", course.name, grade);
                    }
                }
            },
            Err(e) => println!("   查询失败: {}", e),
        }
        println!();
    }

    println!("6. 课程成绩分布");
    for course_id in 1..=3 {
        match student_course_mapper.get_course_grades(course_id) {
            Ok(distribution) => {
                let course_name = student_course_mapper.courses.get(&course_id).map(|c| c.name.as_str()).unwrap_or("未知课程");
                println!("   课程 \"{}\" 成绩分布:", course_name);
                if distribution.is_empty() {
                    println!("     - 暂无成绩");
                } else {
                    for (grade, count) in distribution {
                        println!("     - {}: {} 人", grade, count);
                    }
                }
            },
            Err(e) => println!("   查询失败: {}", e),
        }
        println!();
    }

    println!("7. 动态操作演示");
    
    // 新增选课
    println!("   新增选课:");
    match student_course_mapper.enroll_student(2, 3, "2024春季".to_string()) {
        Ok(_) => println!("     选课成功"),
        Err(e) => println!("     选课失败: {}", e),
    }

    // 设置成绩
    println!("\n   设置成绩:");
    match student_course_mapper.set_grade(2, 3, "A".to_string()) {
        Ok(_) => println!("     成绩设置成功"),
        Err(e) => println!("     成绩设置失败: {}", e),
    }

    // 退课
    println!("\n   退课操作:");
    match student_course_mapper.drop_course(3, 4, "2024春季".to_string()) {
        Ok(_) => println!("     退课成功"),
        Err(e) => println!("     退课失败: {}", e),
    }

    println!("\n{}", "=".repeat(80));

    // 演示用户-角色关联映射
    println!("\n👥 用户角色系统 - 关联表映射");
    println!("处理用户与角色之间的多对多关系\n");

    let mut user_role_mapper = UserRoleMapper::new();

    println!("1. 初始数据展示");
    println!("   用户列表:");
    for user in user_role_mapper.get_all_users() {
        println!("     - {}", user);
    }

    println!("\n   角色列表:");
    for role in user_role_mapper.get_all_roles() {
        println!("     - {}", role);
    }

    println!("\n2. 用户角色分配");
    for user_id in 1..=3 {
        match user_role_mapper.get_user_roles(user_id) {
            Ok(roles) => {
                let user_name = user_role_mapper.users.get(&user_id).map(|u| u.username.as_str()).unwrap_or("未知用户");
                println!("   用户 \"{}\" 的角色:", user_name);
                for role in roles {
                    println!("     - {} ({})", role.name, role.description);
                }
            },
            Err(e) => println!("   查询失败: {}", e),
        }
        println!();
    }

    println!("3. 角色用户列表");
    for role_id in 1..=4 {
        match user_role_mapper.get_role_users(role_id) {
            Ok(users) => {
                let role_name = user_role_mapper.roles.get(&role_id).map(|r| r.name.as_str()).unwrap_or("未知角色");
                println!("   角色 \"{}\" 的用户:", role_name);
                for user in users {
                    println!("     - {} ({})", user.username, user.full_name);
                }
            },
            Err(e) => println!("   查询失败: {}", e),
        }
        println!();
    }

    println!("4. 用户权限检查");
    for user_id in 1..=3 {
        match user_role_mapper.get_user_permissions(user_id) {
            Ok(permissions) => {
                let user_name = user_role_mapper.users.get(&user_id).map(|u| u.username.as_str()).unwrap_or("未知用户");
                println!("   用户 \"{}\" 的权限:", user_name);
                for permission in &permissions {
                    println!("     - {}", permission);
                }
                println!("     总权限数: {}", permissions.len());
            },
            Err(e) => println!("   查询失败: {}", e),
        }
        println!();
    }

    println!("5. 权限验证");
    let test_permissions = vec!["read", "create", "delete", "manage_users"];
    for user_id in 1..=3 {
        let user_name = user_role_mapper.users.get(&user_id).map(|u| u.username.as_str()).unwrap_or("未知用户");
        println!("   用户 \"{}\" 权限验证:", user_name);
        for permission in &test_permissions {
            match user_role_mapper.has_permission(user_id, permission) {
                Ok(has_perm) => {
                    let status = if has_perm { "✅" } else { "❌" };
                    println!("     {} {}", status, permission);
                },
                Err(e) => println!("     验证失败: {}", e),
            }
        }
        println!();
    }

    println!("6. 动态角色管理");
    
    // 分配新角色
    println!("   为用户3分配Editor角色:");
    match user_role_mapper.assign_role(3, 2, 1) {
        Ok(_) => println!("     角色分配成功"),
        Err(e) => println!("     角色分配失败: {}", e),
    }

    // 验证新权限
    println!("\n   验证用户3的新权限:");
    match user_role_mapper.has_permission(3, "create") {
        Ok(has_perm) => {
            println!("     create权限: {}", if has_perm { "✅ 有" } else { "❌ 无" });
        },
        Err(e) => println!("     验证失败: {}", e),
    }

    // 移除角色
    println!("\n   移除用户2的Moderator角色:");
    match user_role_mapper.revoke_role(2, 4) {
        Ok(_) => println!("     角色移除成功"),
        Err(e) => println!("     角色移除失败: {}", e),
    }

    println!("\n=== 关联表映射模式演示完成 ===");

    println!("\n💡 关联表映射模式的优势:");
    println!("1. 处理复杂关系 - 完美解决多对多关系映射问题");
    println!("2. 关联属性 - 可以在关联表中存储额外的业务信息");
    println!("3. 数据一致性 - 通过外键约束保证引用完整性");
    println!("4. 查询灵活性 - 支持双向查询和复杂的关联查询");
    println!("5. 可扩展性 - 容易添加新的关联属性");

    println!("\n🏗️ 实现的关联映射:");
    println!("• 学生-课程关联 (Enrollment表)");
    println!("  - 支持选课、退课、成绩管理");
    println!("  - 包含学期、状态等关联属性");
    println!("• 用户-角色关联 (UserRole表)");
    println!("  - 支持动态角色分配和权限管理");
    println!("  - 包含分配时间、过期时间等关联属性");

    println!("\n⚠️ 设计考虑:");
    println!("1. 性能优化 - 合理设置索引，避免N+1查询问题");
    println!("2. 数据一致性 - 使用事务确保关联操作的原子性");
    println!("3. 级联操作 - 考虑删除主实体时如何处理关联记录");
    println!("4. 权限控制 - 确保关联操作的安全性和权限验证");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_student_course_mapping() {
        let mut mapper = StudentCourseMapper::new();
        
        // 测试添加学生和课程
        mapper.add_student(Student::new(10, "测试学生".to_string(), "test@example.com".to_string(), "测试专业".to_string(), 2024));
        mapper.add_course(Course::new(10, "TEST101".to_string(), "测试课程".to_string(), 3, "测试学院".to_string(), "测试教授".to_string()));
        
        // 测试选课
        assert!(mapper.enroll_student(10, 10, "2024春季".to_string()).is_ok());
        
        // 测试重复选课
        assert!(mapper.enroll_student(10, 10, "2024春季".to_string()).is_err());
        
        // 测试查询学生课程
        let courses = mapper.get_student_courses(10).unwrap();
        assert_eq!(courses.len(), 1);
        
        // 测试设置成绩
        assert!(mapper.set_grade(10, 10, "A+".to_string()).is_ok());
        
        // 测试成绩单
        let transcript = mapper.get_transcript(10).unwrap();
        assert_eq!(transcript.len(), 1);
        assert_eq!(transcript[0].1, "A+");
    }

    #[test]
    fn test_user_role_mapping() {
        let mut mapper = UserRoleMapper::new();
        
        // 测试添加用户和角色
        mapper.add_user(User::new(10, "testuser".to_string(), "test@example.com".to_string(), "测试用户".to_string()));
        mapper.add_role(Role::new(10, "TestRole".to_string(), "测试角色".to_string(), vec!["test".to_string()]));
        
        // 测试分配角色
        assert!(mapper.assign_role(10, 10, 1).is_ok());
        
        // 测试重复分配
        assert!(mapper.assign_role(10, 10, 1).is_err());
        
        // 测试查询用户角色
        let roles = mapper.get_user_roles(10).unwrap();
        assert_eq!(roles.len(), 1);
        assert_eq!(roles[0].name, "TestRole");
        
        // 测试权限检查
        assert!(mapper.has_permission(10, "test").unwrap());
        assert!(!mapper.has_permission(10, "nonexistent").unwrap());
        
        // 测试移除角色
        assert!(mapper.revoke_role(10, 10).is_ok());
        let roles_after = mapper.get_user_roles(10).unwrap();
        assert_eq!(roles_after.len(), 0);
    }

    #[test]
    fn test_error_handling() {
        let mut mapper = StudentCourseMapper::new();
        
        // 测试不存在的学生
        assert!(mapper.enroll_student(999, 1, "2024春季".to_string()).is_err());
        
        // 测试不存在的课程
        assert!(mapper.enroll_student(1, 999, "2024春季".to_string()).is_err());
        
        // 测试查询不存在的学生
        assert!(mapper.get_student_courses(999).is_err());
    }
} 