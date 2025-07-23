//! 解释器模式 (Interpreter Pattern)
//! 
//! 给定一个语言，定义它的文法的一种表示，并定义一个解释器，这个解释器使用该表示来解释语言中的句子。
//! 文件路径：/d%3A/workspace/RustLearn/RustDesignPattern/src/GoFDesignPattern/BehavioralPatterns/interpreter.rs

use std::collections::HashMap;

// 表达式接口
trait Expression {
    fn interpret(&self, context: &Context) -> i32;
}

// 上下文类
struct Context {
    variables: HashMap<String, i32>,
}

impl Context {
    fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }

    fn set_variable(&mut self, name: String, value: i32) {
        self.variables.insert(name, value);
    }

    fn get_variable(&self, name: &str) -> Option<i32> {
        self.variables.get(name).copied()
    }
}

// 终结符表达式 - 数字
struct NumberExpression {
    number: i32,
}

impl NumberExpression {
    fn new(number: i32) -> Self {
        Self { number }
    }
}

impl Expression for NumberExpression {
    fn interpret(&self, _context: &Context) -> i32 {
        self.number
    }
}

// 终结符表达式 - 变量
struct VariableExpression {
    name: String,
}

impl VariableExpression {
    fn new(name: String) -> Self {
        Self { name }
    }
}

impl Expression for VariableExpression {
    fn interpret(&self, context: &Context) -> i32 {
        context.get_variable(&self.name).unwrap_or(0)
    }
}

// 非终结符表达式 - 加法
struct AddExpression {
    left: Box<dyn Expression>,
    right: Box<dyn Expression>,
}

impl AddExpression {
    fn new(left: Box<dyn Expression>, right: Box<dyn Expression>) -> Self {
        Self { left, right }
    }
}

impl Expression for AddExpression {
    fn interpret(&self, context: &Context) -> i32 {
        self.left.interpret(context) + self.right.interpret(context)
    }
}

// 非终结符表达式 - 减法
struct SubtractExpression {
    left: Box<dyn Expression>,
    right: Box<dyn Expression>,
}

impl SubtractExpression {
    fn new(left: Box<dyn Expression>, right: Box<dyn Expression>) -> Self {
        Self { left, right }
    }
}

impl Expression for SubtractExpression {
    fn interpret(&self, context: &Context) -> i32 {
        self.left.interpret(context) - self.right.interpret(context)
    }
}

// 简单的表达式解析器
struct ExpressionParser;

impl ExpressionParser {
    fn parse(expression: &str) -> Result<Box<dyn Expression>, String> {
        let tokens: Vec<&str> = expression.split_whitespace().collect();
        if tokens.len() == 3 {
            let left = Self::parse_token(tokens[0])?;
            let operator = tokens[1];
            let right = Self::parse_token(tokens[2])?;

            match operator {
                "+" => Ok(Box::new(AddExpression::new(left, right))),
                "-" => Ok(Box::new(SubtractExpression::new(left, right))),
                _ => Err(format!("不支持的操作符: {}", operator)),
            }
        } else if tokens.len() == 1 {
            Self::parse_token(tokens[0])
        } else {
            Err("无效的表达式格式".to_string())
        }
    }

    fn parse_token(token: &str) -> Result<Box<dyn Expression>, String> {
        if let Ok(number) = token.parse::<i32>() {
            Ok(Box::new(NumberExpression::new(number)))
        } else if token.chars().all(|c| c.is_alphabetic()) {
            Ok(Box::new(VariableExpression::new(token.to_string())))
        } else {
            Err(format!("无效的标记: {}", token))
        }
    }
}

pub fn demo() {
    println!("=== 解释器模式演示 ===");

    let mut context = Context::new();
    context.set_variable("x".to_string(), 10);
    context.set_variable("y".to_string(), 5);
    context.set_variable("z".to_string(), 3);

    let expressions = vec![
        "10",
        "x",
        "x + y",
        "x - y",
        "y + z",
        "x - z",
    ];

    for expr_str in expressions {
        println!("\n表达式: {}", expr_str);
        match ExpressionParser::parse(expr_str) {
            Ok(expression) => {
                let result = expression.interpret(&context);
                println!("结果: {}", result);
            }
            Err(e) => {
                println!("解析错误: {}", e);
            }
        }
    }

    println!("\n上下文变量:");
    for (var, value) in &context.variables {
        println!("  {} = {}", var, value);
    }
} 