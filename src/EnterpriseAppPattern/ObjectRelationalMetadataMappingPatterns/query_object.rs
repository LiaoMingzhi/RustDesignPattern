// 文件路径: /d%3A/workspace/RustLearn/RustDesignPattern/src/EnterpriseAppPattern/ObjectRelationalMetadataMappingPatterns/query_object.rs

//! # 查询对象模式 (Query Object)
//!
//! ## 概述
//! 查询对象模式将复杂的查询逻辑封装在一个对象中，
//! 提供了一个面向对象的方式来构建和执行数据库查询。
//!
//! ## 优点
//! - 复杂查询逻辑的封装和重用
//! - 类型安全的查询构建
//! - 支持动态查询条件
//! - 便于测试和维护
//! - 提供了流畅的查询API

use std::collections::HashMap;
use std::fmt;

/// 查询操作符
#[derive(Debug, Clone, PartialEq)]
pub enum Operator {
    Equal,
    NotEqual,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
    Like,
    NotLike,
    In,
    NotIn,
    IsNull,
    IsNotNull,
    Between,
}

impl fmt::Display for Operator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let op = match self {
            Operator::Equal => "=",
            Operator::NotEqual => "!=",
            Operator::GreaterThan => ">",
            Operator::GreaterThanOrEqual => ">=",
            Operator::LessThan => "<",
            Operator::LessThanOrEqual => "<=",
            Operator::Like => "LIKE",
            Operator::NotLike => "NOT LIKE",
            Operator::In => "IN",
            Operator::NotIn => "NOT IN",
            Operator::IsNull => "IS NULL",
            Operator::IsNotNull => "IS NOT NULL",
            Operator::Between => "BETWEEN",
        };
        write!(f, "{}", op)
    }
}

/// 查询值
#[derive(Debug, Clone)]
pub enum QueryValue {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Null,
    List(Vec<QueryValue>),
    Range(Box<QueryValue>, Box<QueryValue>),
}

impl fmt::Display for QueryValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QueryValue::String(s) => write!(f, "'{}'", s.replace("'", "''")),
            QueryValue::Integer(i) => write!(f, "{}", i),
            QueryValue::Float(fl) => write!(f, "{}", fl),
            QueryValue::Boolean(b) => write!(f, "{}", if *b { "TRUE" } else { "FALSE" }),
            QueryValue::Null => write!(f, "NULL"),
            QueryValue::List(values) => {
                let formatted: Vec<String> = values.iter().map(|v| v.to_string()).collect();
                write!(f, "({})", formatted.join(", "))
            }
            QueryValue::Range(start, end) => write!(f, "{} AND {}", start, end),
        }
    }
}

/// 查询条件
#[derive(Debug, Clone)]
pub struct Condition {
    pub field: String,
    pub operator: Operator,
    pub value: Option<QueryValue>,
}

impl Condition {
    pub fn new(field: String, operator: Operator, value: Option<QueryValue>) -> Self {
        Self { field, operator, value }
    }

    pub fn to_sql(&self) -> String {
        match &self.value {
            Some(value) => format!("{} {} {}", self.field, self.operator, value),
            None => format!("{} {}", self.field, self.operator),
        }
    }
}

/// 逻辑连接符
#[derive(Debug, Clone, PartialEq)]
pub enum LogicalOperator {
    And,
    Or,
}

impl fmt::Display for LogicalOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LogicalOperator::And => write!(f, "AND"),
            LogicalOperator::Or => write!(f, "OR"),
        }
    }
}

/// 复合条件
#[derive(Debug, Clone)]
pub enum WhereClause {
    Condition(Condition),
    Group(Vec<WhereClause>, LogicalOperator),
    Not(Box<WhereClause>),
}

impl WhereClause {
    pub fn to_sql(&self) -> String {
        match self {
            WhereClause::Condition(condition) => condition.to_sql(),
            WhereClause::Group(clauses, op) => {
                let formatted: Vec<String> = clauses.iter().map(|c| c.to_sql()).collect();
                format!("({})", formatted.join(&format!(" {} ", op)))
            }
            WhereClause::Not(clause) => format!("NOT ({})", clause.to_sql()),
        }
    }
}

/// 排序方向
#[derive(Debug, Clone, PartialEq)]
pub enum SortOrder {
    Asc,
    Desc,
}

impl fmt::Display for SortOrder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SortOrder::Asc => write!(f, "ASC"),
            SortOrder::Desc => write!(f, "DESC"),
        }
    }
}

/// 排序条件
#[derive(Debug, Clone)]
pub struct OrderBy {
    pub field: String,
    pub order: SortOrder,
}

impl OrderBy {
    pub fn new(field: String, order: SortOrder) -> Self {
        Self { field, order }
    }

    pub fn to_sql(&self) -> String {
        format!("{} {}", self.field, self.order)
    }
}

/// 查询对象
#[derive(Debug, Clone)]
pub struct QueryObject {
    pub table: String,
    pub fields: Vec<String>,
    pub where_clause: Option<WhereClause>,
    pub order_by: Vec<OrderBy>,
    pub group_by: Vec<String>,
    pub having: Option<WhereClause>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub joins: Vec<Join>,
}

/// 连接类型
#[derive(Debug, Clone)]
pub enum JoinType {
    Inner,
    Left,
    Right,
    Full,
}

impl fmt::Display for JoinType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JoinType::Inner => write!(f, "INNER JOIN"),
            JoinType::Left => write!(f, "LEFT JOIN"),
            JoinType::Right => write!(f, "RIGHT JOIN"),
            JoinType::Full => write!(f, "FULL JOIN"),
        }
    }
}

/// 表连接
#[derive(Debug, Clone)]
pub struct Join {
    pub join_type: JoinType,
    pub table: String,
    pub on_condition: String,
}

impl Join {
    pub fn new(join_type: JoinType, table: String, on_condition: String) -> Self {
        Self { join_type, table, on_condition }
    }

    pub fn to_sql(&self) -> String {
        format!("{} {} ON {}", self.join_type, self.table, self.on_condition)
    }
}

impl QueryObject {
    pub fn new(table: String) -> Self {
        Self {
            table,
            fields: vec!["*".to_string()],
            where_clause: None,
            order_by: Vec::new(),
            group_by: Vec::new(),
            having: None,
            limit: None,
            offset: None,
            joins: Vec::new(),
        }
    }

    /// 选择字段
    pub fn select(mut self, fields: Vec<String>) -> Self {
        self.fields = fields;
        self
    }

    /// 添加WHERE条件
    pub fn where_condition(mut self, condition: WhereClause) -> Self {
        self.where_clause = Some(condition);
        self
    }

    /// 添加ORDER BY
    pub fn order_by(mut self, order: OrderBy) -> Self {
        self.order_by.push(order);
        self
    }

    /// 添加GROUP BY
    pub fn group_by(mut self, fields: Vec<String>) -> Self {
        self.group_by = fields;
        self
    }

    /// 添加HAVING条件
    pub fn having(mut self, condition: WhereClause) -> Self {
        self.having = Some(condition);
        self
    }

    /// 设置LIMIT
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    /// 设置OFFSET
    pub fn offset(mut self, offset: usize) -> Self {
        self.offset = Some(offset);
        self
    }

    /// 添加JOIN
    pub fn join(mut self, join: Join) -> Self {
        self.joins.push(join);
        self
    }

    /// 生成SQL语句
    pub fn to_sql(&self) -> String {
        let mut sql = format!("SELECT {} FROM {}", self.fields.join(", "), self.table);

        // 添加JOIN子句
        for join in &self.joins {
            sql.push_str(&format!(" {}", join.to_sql()));
        }

        // 添加WHERE子句
        if let Some(where_clause) = &self.where_clause {
            sql.push_str(&format!(" WHERE {}", where_clause.to_sql()));
        }

        // 添加GROUP BY子句
        if !self.group_by.is_empty() {
            sql.push_str(&format!(" GROUP BY {}", self.group_by.join(", ")));
        }

        // 添加HAVING子句
        if let Some(having) = &self.having {
            sql.push_str(&format!(" HAVING {}", having.to_sql()));
        }

        // 添加ORDER BY子句
        if !self.order_by.is_empty() {
            let order_clauses: Vec<String> = self.order_by.iter().map(|o| o.to_sql()).collect();
            sql.push_str(&format!(" ORDER BY {}", order_clauses.join(", ")));
        }

        // 添加LIMIT子句
        if let Some(limit) = self.limit {
            sql.push_str(&format!(" LIMIT {}", limit));
        }

        // 添加OFFSET子句
        if let Some(offset) = self.offset {
            sql.push_str(&format!(" OFFSET {}", offset));
        }

        sql
    }
}

/// 查询构建器
pub struct QueryBuilder;

impl QueryBuilder {
    /// 创建等于条件
    pub fn eq(field: &str, value: QueryValue) -> Condition {
        Condition::new(field.to_string(), Operator::Equal, Some(value))
    }

    /// 创建不等于条件
    pub fn ne(field: &str, value: QueryValue) -> Condition {
        Condition::new(field.to_string(), Operator::NotEqual, Some(value))
    }

    /// 创建大于条件
    pub fn gt(field: &str, value: QueryValue) -> Condition {
        Condition::new(field.to_string(), Operator::GreaterThan, Some(value))
    }

    /// 创建大于等于条件
    pub fn gte(field: &str, value: QueryValue) -> Condition {
        Condition::new(field.to_string(), Operator::GreaterThanOrEqual, Some(value))
    }

    /// 创建小于条件
    pub fn lt(field: &str, value: QueryValue) -> Condition {
        Condition::new(field.to_string(), Operator::LessThan, Some(value))
    }

    /// 创建小于等于条件
    pub fn lte(field: &str, value: QueryValue) -> Condition {
        Condition::new(field.to_string(), Operator::LessThanOrEqual, Some(value))
    }

    /// 创建LIKE条件
    pub fn like(field: &str, pattern: &str) -> Condition {
        Condition::new(field.to_string(), Operator::Like, Some(QueryValue::String(pattern.to_string())))
    }

    /// 创建IN条件
    pub fn in_values(field: &str, values: Vec<QueryValue>) -> Condition {
        Condition::new(field.to_string(), Operator::In, Some(QueryValue::List(values)))
    }

    /// 创建BETWEEN条件
    pub fn between(field: &str, start: QueryValue, end: QueryValue) -> Condition {
        Condition::new(field.to_string(), Operator::Between, 
                      Some(QueryValue::Range(Box::new(start), Box::new(end))))
    }

    /// 创建IS NULL条件
    pub fn is_null(field: &str) -> Condition {
        Condition::new(field.to_string(), Operator::IsNull, None)
    }

    /// 创建IS NOT NULL条件
    pub fn is_not_null(field: &str) -> Condition {
        Condition::new(field.to_string(), Operator::IsNotNull, None)
    }

    /// 组合多个条件（AND）
    pub fn and(conditions: Vec<WhereClause>) -> WhereClause {
        WhereClause::Group(conditions, LogicalOperator::And)
    }

    /// 组合多个条件（OR）
    pub fn or(conditions: Vec<WhereClause>) -> WhereClause {
        WhereClause::Group(conditions, LogicalOperator::Or)
    }

    /// 否定条件
    pub fn not(condition: WhereClause) -> WhereClause {
        WhereClause::Not(Box::new(condition))
    }
}

/// 用户查询示例
pub struct UserQueries;

impl UserQueries {
    /// 查找活跃用户
    pub fn find_active_users() -> QueryObject {
        QueryObject::new("users".to_string())
            .where_condition(WhereClause::Condition(
                QueryBuilder::eq("is_active", QueryValue::Boolean(true))
            ))
            .order_by(OrderBy::new("created_at".to_string(), SortOrder::Desc))
    }

    /// 按年龄范围查找用户
    pub fn find_users_by_age_range(min_age: i64, max_age: i64) -> QueryObject {
        QueryObject::new("users".to_string())
            .where_condition(WhereClause::Condition(
                QueryBuilder::between("age", QueryValue::Integer(min_age), QueryValue::Integer(max_age))
            ))
            .order_by(OrderBy::new("age".to_string(), SortOrder::Asc))
    }

    /// 搜索用户（模糊匹配姓名或邮箱）
    pub fn search_users(keyword: &str) -> QueryObject {
        let pattern = format!("%{}%", keyword);
        QueryObject::new("users".to_string())
            .where_condition(QueryBuilder::or(vec![
                WhereClause::Condition(QueryBuilder::like("full_name", &pattern)),
                WhereClause::Condition(QueryBuilder::like("email", &pattern)),
            ]))
            .order_by(OrderBy::new("full_name".to_string(), SortOrder::Asc))
    }

    /// 查找用户及其订单数量
    pub fn find_users_with_order_count() -> QueryObject {
        QueryObject::new("users".to_string())
            .select(vec![
                "users.id".to_string(),
                "users.full_name".to_string(),
                "users.email".to_string(),
                "COUNT(orders.id) as order_count".to_string(),
            ])
            .join(Join::new(
                JoinType::Left,
                "orders".to_string(),
                "orders.user_id = users.id".to_string(),
            ))
            .group_by(vec![
                "users.id".to_string(),
                "users.full_name".to_string(),
                "users.email".to_string(),
            ])
            .order_by(OrderBy::new("order_count".to_string(), SortOrder::Desc))
    }

    /// 查找高价值用户（订单总额超过指定金额）
    pub fn find_high_value_users(min_total: f64) -> QueryObject {
        QueryObject::new("users".to_string())
            .select(vec![
                "users.id".to_string(),
                "users.full_name".to_string(),
                "SUM(orders.total_amount) as total_spent".to_string(),
            ])
            .join(Join::new(
                JoinType::Inner,
                "orders".to_string(),
                "orders.user_id = users.id".to_string(),
            ))
            .group_by(vec![
                "users.id".to_string(),
                "users.full_name".to_string(),
            ])
            .having(WhereClause::Condition(
                QueryBuilder::gte("SUM(orders.total_amount)", QueryValue::Float(min_total))
            ))
            .order_by(OrderBy::new("total_spent".to_string(), SortOrder::Desc))
    }
}

/// 演示查询对象模式
pub fn demo() {
    println!("=== 查询对象模式演示 ===\n");

    println!("1. 基本查询构建");
    let basic_query = QueryObject::new("users".to_string())
        .select(vec!["id".to_string(), "name".to_string(), "email".to_string()])
        .where_condition(WhereClause::Condition(
            QueryBuilder::eq("is_active", QueryValue::Boolean(true))
        ))
        .order_by(OrderBy::new("name".to_string(), SortOrder::Asc))
        .limit(10);

    println!("   SQL: {}\n", basic_query.to_sql());

    println!("2. 复杂条件查询");
    let complex_query = QueryObject::new("users".to_string())
        .where_condition(QueryBuilder::and(vec![
            WhereClause::Condition(QueryBuilder::gte("age", QueryValue::Integer(18))),
            QueryBuilder::or(vec![
                WhereClause::Condition(QueryBuilder::like("email", "%@company.com")),
                WhereClause::Condition(QueryBuilder::like("email", "%@example.com")),
            ]),
            QueryBuilder::not(WhereClause::Condition(QueryBuilder::is_null("phone"))),
        ]))
        .order_by(OrderBy::new("created_at".to_string(), SortOrder::Desc));

    println!("   SQL: {}\n", complex_query.to_sql());

    println!("3. 带JOIN的查询");
    let join_query = UserQueries::find_users_with_order_count();
    println!("   用户及订单数量查询:");
    println!("   SQL: {}\n", join_query.to_sql());

    println!("4. 聚合查询");
    let aggregate_query = UserQueries::find_high_value_users(1000.0);
    println!("   高价值用户查询 (消费>1000):");
    println!("   SQL: {}\n", aggregate_query.to_sql());

    println!("5. 预定义查询示例");
    
    // 活跃用户查询
    let active_users_query = UserQueries::find_active_users();
    println!("   活跃用户查询:");
    println!("   SQL: {}\n", active_users_query.to_sql());

    // 年龄范围查询
    let age_range_query = UserQueries::find_users_by_age_range(25, 35);
    println!("   年龄25-35岁用户查询:");
    println!("   SQL: {}\n", age_range_query.to_sql());

    // 搜索查询
    let search_query = UserQueries::search_users("john");
    println!("   搜索用户'john':");
    println!("   SQL: {}\n", search_query.to_sql());

    println!("6. 分页查询");
    let paginated_query = QueryObject::new("users".to_string())
        .where_condition(WhereClause::Condition(
            QueryBuilder::eq("is_active", QueryValue::Boolean(true))
        ))
        .order_by(OrderBy::new("id".to_string(), SortOrder::Asc))
        .limit(20)
        .offset(40); // 第3页，每页20条

    println!("   分页查询 (第3页，每页20条):");
    println!("   SQL: {}\n", paginated_query.to_sql());

    println!("=== 查询对象模式演示完成 ===");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_query_building() {
        let query = QueryObject::new("users".to_string())
            .select(vec!["id".to_string(), "name".to_string()])
            .where_condition(WhereClause::Condition(
                QueryBuilder::eq("active", QueryValue::Boolean(true))
            ));

        let sql = query.to_sql();
        assert!(sql.contains("SELECT id, name"));
        assert!(sql.contains("FROM users"));
        assert!(sql.contains("WHERE active = TRUE"));
    }

    #[test]
    fn test_complex_conditions() {
        let condition = QueryBuilder::and(vec![
            WhereClause::Condition(QueryBuilder::gt("age", QueryValue::Integer(18))),
            WhereClause::Condition(QueryBuilder::like("name", "John%")),
        ]);

        let sql = condition.to_sql();
        assert!(sql.contains("(age > 18 AND name LIKE 'John%')"));
    }

    #[test]
    fn test_join_query() {
        let query = QueryObject::new("users".to_string())
            .join(Join::new(
                JoinType::Inner,
                "orders".to_string(),
                "orders.user_id = users.id".to_string(),
            ));

        let sql = query.to_sql();
        assert!(sql.contains("INNER JOIN orders ON orders.user_id = users.id"));
    }
} 