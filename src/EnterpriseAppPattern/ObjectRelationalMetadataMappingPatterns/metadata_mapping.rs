// 文件路径: /d%3A/workspace/RustLearn/RustDesignPattern/src/EnterpriseAppPattern/ObjectRelationalMetadataMappingPatterns/metadata_mapping.rs

//! # 元数据映射模式 (Metadata Mapping)
//!
//! ## 概述
//! 元数据映射模式通过配置文件或注解来定义对象和关系数据库之间的映射关系，
//! 而不是在代码中硬编码这些映射。这使得映射关系可以独立于代码进行修改和维护。
//!
//! ## 优点
//! - 映射配置与业务逻辑分离
//! - 支持运行时动态配置
//! - 便于维护和修改映射关系
//! - 支持多种数据库方言
//! - 提供了灵活的字段映射选项
//!
//! ## 适用场景
//! - 需要支持多种数据库的应用
//! - 映射关系复杂的系统
//! - 需要运行时配置的场景
//! - 遗留数据库集成项目

use std::collections::HashMap;
use std::fmt;
use serde::{Deserialize, Serialize};

/// 数据类型映射
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataType {
    String(Option<usize>), // VARCHAR(n)
    Integer,               // INT
    Long,                  // BIGINT
    Float,                 // FLOAT
    Double,                // DOUBLE
    Boolean,               // BOOLEAN
    Date,                  // DATE
    DateTime,              // DATETIME
    Text,                  // TEXT
    Binary(Option<usize>), // BLOB(n)
}

impl fmt::Display for DataType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataType::String(Some(len)) => write!(f, "VARCHAR({})", len),
            DataType::String(None) => write!(f, "VARCHAR"),
            DataType::Integer => write!(f, "INT"),
            DataType::Long => write!(f, "BIGINT"),
            DataType::Float => write!(f, "FLOAT"),
            DataType::Double => write!(f, "DOUBLE"),
            DataType::Boolean => write!(f, "BOOLEAN"),
            DataType::Date => write!(f, "DATE"),
            DataType::DateTime => write!(f, "DATETIME"),
            DataType::Text => write!(f, "TEXT"),
            DataType::Binary(Some(len)) => write!(f, "BLOB({})", len),
            DataType::Binary(None) => write!(f, "BLOB"),
        }
    }
}

/// 字段映射配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldMapping {
    pub property_name: String,    // 对象属性名
    pub column_name: String,      // 数据库列名
    pub data_type: DataType,      // 数据类型
    pub is_primary_key: bool,     // 是否为主键
    pub is_nullable: bool,        // 是否可空
    pub is_auto_increment: bool,  // 是否自增
    pub default_value: Option<String>, // 默认值
}

impl FieldMapping {
    pub fn new(property_name: String, column_name: String, data_type: DataType) -> Self {
        Self {
            property_name,
            column_name,
            data_type,
            is_primary_key: false,
            is_nullable: true,
            is_auto_increment: false,
            default_value: None,
        }
    }

    pub fn primary_key(mut self) -> Self {
        self.is_primary_key = true;
        self.is_nullable = false;
        self
    }

    pub fn not_null(mut self) -> Self {
        self.is_nullable = false;
        self
    }

    pub fn auto_increment(mut self) -> Self {
        self.is_auto_increment = true;
        self
    }

    pub fn default_value(mut self, value: String) -> Self {
        self.default_value = Some(value);
        self
    }
}

/// 关系映射类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RelationType {
    OneToOne,
    OneToMany,
    ManyToOne,
    ManyToMany,
}

/// 关系映射配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationMapping {
    pub property_name: String,          // 对象属性名
    pub target_entity: String,          // 目标实体类型
    pub relation_type: RelationType,     // 关系类型
    pub foreign_key: Option<String>,     // 外键列名
    pub join_table: Option<String>,      // 连接表名（多对多时使用）
    pub join_columns: Vec<String>,       // 连接表列名
    pub lazy_loading: bool,              // 是否延迟加载
}

/// 实体映射配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityMapping {
    pub entity_name: String,                    // 实体名称
    pub table_name: String,                     // 表名
    pub schema_name: Option<String>,            // 模式名
    pub fields: HashMap<String, FieldMapping>, // 字段映射
    pub relations: HashMap<String, RelationMapping>, // 关系映射
}

impl EntityMapping {
    pub fn new(entity_name: String, table_name: String) -> Self {
        Self {
            entity_name,
            table_name,
            schema_name: None,
            fields: HashMap::new(),
            relations: HashMap::new(),
        }
    }

    pub fn schema(mut self, schema: String) -> Self {
        self.schema_name = Some(schema);
        self
    }

    pub fn add_field(mut self, field: FieldMapping) -> Self {
        self.fields.insert(field.property_name.clone(), field);
        self
    }

    pub fn add_relation(mut self, relation: RelationMapping) -> Self {
        self.relations.insert(relation.property_name.clone(), relation);
        self
    }

    /// 获取完整表名
    pub fn get_full_table_name(&self) -> String {
        match &self.schema_name {
            Some(schema) => format!("{}.{}", schema, self.table_name),
            None => self.table_name.clone(),
        }
    }

    /// 获取主键字段
    pub fn get_primary_key_fields(&self) -> Vec<&FieldMapping> {
        self.fields.values()
            .filter(|field| field.is_primary_key)
            .collect()
    }

    /// 生成CREATE TABLE语句
    pub fn generate_create_table_sql(&self) -> String {
        let mut sql = format!("CREATE TABLE {} (\n", self.get_full_table_name());
        
        let mut field_definitions = Vec::new();
        for field in self.fields.values() {
            let mut def = format!("  {} {}", field.column_name, field.data_type);
            
            if field.is_primary_key {
                def.push_str(" PRIMARY KEY");
            }
            
            if field.is_auto_increment {
                def.push_str(" AUTO_INCREMENT");
            }
            
            if !field.is_nullable {
                def.push_str(" NOT NULL");
            }
            
            if let Some(default) = &field.default_value {
                def.push_str(&format!(" DEFAULT {}", default));
            }
            
            field_definitions.push(def);
        }
        
        sql.push_str(&field_definitions.join(",\n"));
        sql.push_str("\n)");
        sql
    }

    /// 生成SELECT语句
    pub fn generate_select_sql(&self, where_clause: Option<&str>) -> String {
        let columns: Vec<String> = self.fields.values()
            .map(|field| field.column_name.clone())
            .collect();
        
        let mut sql = format!("SELECT {} FROM {}", 
                             columns.join(", "), 
                             self.get_full_table_name());
        
        if let Some(where_clause) = where_clause {
            sql.push_str(&format!(" WHERE {}", where_clause));
        }
        
        sql
    }

    /// 生成INSERT语句
    pub fn generate_insert_sql(&self) -> String {
        let non_auto_fields: Vec<&FieldMapping> = self.fields.values()
            .filter(|field| !field.is_auto_increment)
            .collect();
        
        let columns: Vec<String> = non_auto_fields.iter()
            .map(|field| field.column_name.clone())
            .collect();
        
        let placeholders: Vec<String> = (0..columns.len())
            .map(|_| "?".to_string())
            .collect();
        
        format!("INSERT INTO {} ({}) VALUES ({})",
                self.get_full_table_name(),
                columns.join(", "),
                placeholders.join(", "))
    }

    /// 生成UPDATE语句
    pub fn generate_update_sql(&self) -> String {
        let non_pk_fields: Vec<&FieldMapping> = self.fields.values()
            .filter(|field| !field.is_primary_key && !field.is_auto_increment)
            .collect();
        
        let set_clauses: Vec<String> = non_pk_fields.iter()
            .map(|field| format!("{} = ?", field.column_name))
            .collect();
        
        let pk_conditions: Vec<String> = self.get_primary_key_fields().iter()
            .map(|field| format!("{} = ?", field.column_name))
            .collect();
        
        format!("UPDATE {} SET {} WHERE {}",
                self.get_full_table_name(),
                set_clauses.join(", "),
                pk_conditions.join(" AND "))
    }
}

/// 元数据映射注册表
pub struct MetadataMappingRegistry {
    mappings: HashMap<String, EntityMapping>,
    database_dialect: DatabaseDialect,
}

/// 数据库方言
#[derive(Debug, Clone)]
pub enum DatabaseDialect {
    MySQL,
    PostgreSQL,
    SQLite,
    Oracle,
    SQLServer,
}

impl MetadataMappingRegistry {
    pub fn new(dialect: DatabaseDialect) -> Self {
        Self {
            mappings: HashMap::new(),
            database_dialect: dialect,
        }
    }

    /// 注册实体映射
    pub fn register_mapping(&mut self, mapping: EntityMapping) {
        self.mappings.insert(mapping.entity_name.clone(), mapping);
    }

    /// 获取实体映射
    pub fn get_mapping(&self, entity_name: &str) -> Option<&EntityMapping> {
        self.mappings.get(entity_name)
    }

    /// 获取所有映射
    pub fn get_all_mappings(&self) -> Vec<&EntityMapping> {
        self.mappings.values().collect()
    }

    /// 验证映射配置
    pub fn validate_mappings(&self) -> Vec<String> {
        let mut errors = Vec::new();
        
        for mapping in self.mappings.values() {
            // 检查是否有主键
            if mapping.get_primary_key_fields().is_empty() {
                errors.push(format!("实体 {} 没有定义主键", mapping.entity_name));
            }
            
            // 检查关系映射的目标实体是否存在
            for relation in mapping.relations.values() {
                if !self.mappings.contains_key(&relation.target_entity) {
                    errors.push(format!("实体 {} 的关系 {} 引用了不存在的目标实体 {}", 
                                       mapping.entity_name, relation.property_name, relation.target_entity));
                }
            }
        }
        
        errors
    }

    /// 生成数据库架构DDL
    pub fn generate_schema_ddl(&self) -> Vec<String> {
        let mut ddl_statements = Vec::new();
        
        // 按依赖关系排序（简化版：按字母顺序）
        let mut sorted_mappings: Vec<&EntityMapping> = self.mappings.values().collect();
        sorted_mappings.sort_by(|a, b| a.entity_name.cmp(&b.entity_name));
        
        for mapping in sorted_mappings {
            ddl_statements.push(mapping.generate_create_table_sql());
        }
        
        ddl_statements
    }
}

/// 配置构建器
pub struct MappingBuilder;

impl MappingBuilder {
    /// 构建用户实体映射
    pub fn build_user_mapping() -> EntityMapping {
        EntityMapping::new("User".to_string(), "users".to_string())
            .add_field(
                FieldMapping::new("id".to_string(), "id".to_string(), DataType::Long)
                    .primary_key()
                    .auto_increment()
            )
            .add_field(
                FieldMapping::new("name".to_string(), "name".to_string(), DataType::String(Some(100)))
                    .not_null()
            )
            .add_field(
                FieldMapping::new("email".to_string(), "email".to_string(), DataType::String(Some(255)))
                    .not_null()
            )
            .add_field(
                FieldMapping::new("age".to_string(), "age".to_string(), DataType::Integer)
            )
            .add_field(
                FieldMapping::new("created_at".to_string(), "created_at".to_string(), DataType::DateTime)
                    .not_null()
                    .default_value("CURRENT_TIMESTAMP".to_string())
            )
    }

    /// 构建订单实体映射
    pub fn build_order_mapping() -> EntityMapping {
        EntityMapping::new("Order".to_string(), "orders".to_string())
            .add_field(
                FieldMapping::new("id".to_string(), "id".to_string(), DataType::Long)
                    .primary_key()
                    .auto_increment()
            )
            .add_field(
                FieldMapping::new("user_id".to_string(), "user_id".to_string(), DataType::Long)
                    .not_null()
            )
            .add_field(
                FieldMapping::new("total_amount".to_string(), "total_amount".to_string(), DataType::Double)
                    .not_null()
            )
            .add_field(
                FieldMapping::new("status".to_string(), "status".to_string(), DataType::String(Some(50)))
                    .not_null()
                    .default_value("'PENDING'".to_string())
            )
            .add_field(
                FieldMapping::new("created_at".to_string(), "created_at".to_string(), DataType::DateTime)
                    .not_null()
                    .default_value("CURRENT_TIMESTAMP".to_string())
            )
    }

    /// 构建完整的映射注册表
    pub fn build_complete_registry() -> MetadataMappingRegistry {
        let mut registry = MetadataMappingRegistry::new(DatabaseDialect::MySQL);
        
        registry.register_mapping(Self::build_user_mapping());
        registry.register_mapping(Self::build_order_mapping());
        
        registry
    }
}

/// 演示元数据映射模式
pub fn demo() {
    println!("=== 元数据映射模式演示 ===\n");
    
    // 构建映射注册表
    println!("1. 构建元数据映射注册表");
    let registry = MappingBuilder::build_complete_registry();
    
    println!("   已注册 {} 个实体映射", registry.mappings.len());
    for entity_name in registry.mappings.keys() {
        println!("   - {}", entity_name);
    }
    
    // 验证映射配置
    println!("\n2. 验证映射配置");
    let validation_errors = registry.validate_mappings();
    if validation_errors.is_empty() {
        println!("   ✅ 所有映射配置都有效");
    } else {
        println!("   ❌ 发现配置错误:");
        for error in validation_errors {
            println!("     - {}", error);
        }
    }
    
    // 展示用户实体映射详情
    println!("\n3. 用户实体映射详情");
    if let Some(user_mapping) = registry.get_mapping("User") {
        println!("   实体名: {}", user_mapping.entity_name);
        println!("   表名: {}", user_mapping.get_full_table_name());
        println!("   字段映射:");
        for field in user_mapping.fields.values() {
            println!("     - {} -> {} ({}{}{})",
                     field.property_name,
                     field.column_name,
                     field.data_type,
                     if field.is_primary_key { ", PK" } else { "" },
                     if field.is_nullable { "" } else { ", NOT NULL" });
        }
    }
    
    // 生成SQL语句
    println!("\n4. 生成SQL语句");
    if let Some(user_mapping) = registry.get_mapping("User") {
        println!("   CREATE TABLE语句:");
        println!("   {}\n", user_mapping.generate_create_table_sql());
        
        println!("   SELECT语句:");
        println!("   {}\n", user_mapping.generate_select_sql(Some("id = ?")));
        
        println!("   INSERT语句:");
        println!("   {}\n", user_mapping.generate_insert_sql());
        
        println!("   UPDATE语句:");
        println!("   {}\n", user_mapping.generate_update_sql());
    }
    
    // 生成完整的数据库架构
    println!("5. 生成数据库架构DDL");
    let ddl_statements = registry.generate_schema_ddl();
    for (i, ddl) in ddl_statements.iter().enumerate() {
        println!("   表 {}: \n{}\n", i + 1, ddl);
    }
    
    // 序列化映射配置（演示配置的持久化）
    println!("6. 序列化映射配置");
    if let Some(user_mapping) = registry.get_mapping("User") {
        match serde_json::to_string_pretty(user_mapping) {
            Ok(json) => {
                println!("   用户映射的JSON配置:");
                println!("   {}", json);
            }
            Err(e) => println!("   序列化失败: {}", e),
        }
    }
    
    println!("\n=== 元数据映射模式演示完成 ===");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_field_mapping_creation() {
        let field = FieldMapping::new(
            "id".to_string(),
            "id".to_string(),
            DataType::Long
        ).primary_key().auto_increment();
        
        assert_eq!(field.property_name, "id");
        assert_eq!(field.column_name, "id");
        assert!(field.is_primary_key);
        assert!(field.is_auto_increment);
        assert!(!field.is_nullable);
    }

    #[test]
    fn test_entity_mapping_sql_generation() {
        let mapping = MappingBuilder::build_user_mapping();
        
        let create_sql = mapping.generate_create_table_sql();
        assert!(create_sql.contains("CREATE TABLE users"));
        assert!(create_sql.contains("id BIGINT PRIMARY KEY AUTO_INCREMENT NOT NULL"));
        
        let select_sql = mapping.generate_select_sql(Some("id = ?"));
        assert!(select_sql.contains("SELECT"));
        assert!(select_sql.contains("FROM users"));
        assert!(select_sql.contains("WHERE id = ?"));
    }

    #[test]
    fn test_mapping_registry_validation() {
        let registry = MappingBuilder::build_complete_registry();
        let errors = registry.validate_mappings();
        assert!(errors.is_empty(), "映射配置应该是有效的");
    }
} 