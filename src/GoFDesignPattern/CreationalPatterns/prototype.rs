//! 原型模式 (Prototype Pattern)
//! 
//! 用原型实例指定创建对象的种类，并且通过拷贝这些原型创建新的对象。
//! 文件路径：/d%3A/workspace/RustLearn/RustDesignPattern/src/GoFDesignPattern/CreationalPatterns/prototype.rs

use std::collections::HashMap;

// 原型trait
trait Prototype: std::fmt::Debug {
    fn clone_prototype(&self) -> Box<dyn Prototype>;
    fn get_type(&self) -> &str;
    fn get_info(&self) -> String;
}

// 具体原型 - 图形基类
#[derive(Debug, Clone)]
struct Shape {
    id: String,
    shape_type: String,
    x: i32,
    y: i32,
    color: String,
}

impl Shape {
    fn new(id: String, shape_type: String, x: i32, y: i32, color: String) -> Self {
        Self {
            id,
            shape_type,
            x,
            y,
            color,
        }
    }

    fn set_position(&mut self, x: i32, y: i32) {
        self.x = x;
        self.y = y;
    }

    fn set_color(&mut self, color: String) {
        self.color = color;
    }
}

impl Prototype for Shape {
    fn clone_prototype(&self) -> Box<dyn Prototype> {
        Box::new(self.clone())
    }

    fn get_type(&self) -> &str {
        &self.shape_type
    }

    fn get_info(&self) -> String {
        format!(
            "形状ID: {}, 类型: {}, 位置: ({}, {}), 颜色: {}",
            self.id, self.shape_type, self.x, self.y, self.color
        )
    }
}

// 具体原型 - 圆形
#[derive(Debug, Clone)]
struct Circle {
    shape: Shape,
    radius: f32,
}

impl Circle {
    fn new(id: String, x: i32, y: i32, color: String, radius: f32) -> Self {
        Self {
            shape: Shape::new(id, "圆形".to_string(), x, y, color),
            radius,
        }
    }

    fn set_radius(&mut self, radius: f32) {
        self.radius = radius;
    }
}

impl Prototype for Circle {
    fn clone_prototype(&self) -> Box<dyn Prototype> {
        Box::new(self.clone())
    }

    fn get_type(&self) -> &str {
        &self.shape.shape_type
    }

    fn get_info(&self) -> String {
        format!(
            "{}, 半径: {}",
            self.shape.get_info(),
            self.radius
        )
    }
}

// 具体原型 - 矩形
#[derive(Debug, Clone)]
struct Rectangle {
    shape: Shape,
    width: f32,
    height: f32,
}

impl Rectangle {
    fn new(id: String, x: i32, y: i32, color: String, width: f32, height: f32) -> Self {
        Self {
            shape: Shape::new(id, "矩形".to_string(), x, y, color),
            width,
            height,
        }
    }

    fn set_size(&mut self, width: f32, height: f32) {
        self.width = width;
        self.height = height;
    }
}

impl Prototype for Rectangle {
    fn clone_prototype(&self) -> Box<dyn Prototype> {
        Box::new(self.clone())
    }

    fn get_type(&self) -> &str {
        &self.shape.shape_type
    }

    fn get_info(&self) -> String {
        format!(
            "{}, 尺寸: {}x{}",
            self.shape.get_info(),
            self.width,
            self.height
        )
    }
}

// 原型管理器
struct PrototypeManager {
    prototypes: HashMap<String, Box<dyn Prototype>>,
}

impl PrototypeManager {
    fn new() -> Self {
        Self {
            prototypes: HashMap::new(),
        }
    }

    fn register_prototype(&mut self, key: String, prototype: Box<dyn Prototype>) {
        self.prototypes.insert(key, prototype);
    }

    fn create_clone(&self, key: &str) -> Option<Box<dyn Prototype>> {
        self.prototypes.get(key).map(|prototype| prototype.clone_prototype())
    }

    fn list_prototypes(&self) {
        println!("已注册的原型:");
        for (key, prototype) in &self.prototypes {
            println!("  {}: {}", key, prototype.get_info());
        }
    }
}

// 图形编辑器
struct GraphicEditor {
    shapes: Vec<Box<dyn Prototype>>,
    prototype_manager: PrototypeManager,
}

impl GraphicEditor {
    fn new() -> Self {
        let mut editor = Self {
            shapes: Vec::new(),
            prototype_manager: PrototypeManager::new(),
        };

        // 注册默认原型
        editor.register_default_prototypes();
        editor
    }

    fn register_default_prototypes(&mut self) {
        // 注册圆形原型
        let circle_prototype = Box::new(Circle::new(
            "circle_prototype".to_string(),
            0, 0,
            "红色".to_string(),
            10.0,
        ));
        self.prototype_manager.register_prototype("circle".to_string(), circle_prototype);

        // 注册矩形原型
        let rectangle_prototype = Box::new(Rectangle::new(
            "rectangle_prototype".to_string(),
            0, 0,
            "蓝色".to_string(),
            20.0,
            15.0,
        ));
        self.prototype_manager.register_prototype("rectangle".to_string(), rectangle_prototype);
    }

    fn create_shape(&mut self, prototype_key: &str, id: String) -> Result<(), String> {
        if let Some(new_shape) = self.prototype_manager.create_clone(prototype_key) {
            // 这里可以对克隆的对象进行自定义修改
            println!("基于原型 '{}' 创建新形状: {}", prototype_key, id);
            self.shapes.push(new_shape);
            Ok(())
        } else {
            Err(format!("未找到原型: {}", prototype_key))
        }
    }

    fn list_shapes(&self) {
        println!("\n当前画布上的形状:");
        for (i, shape) in self.shapes.iter().enumerate() {
            println!("  {}. {}", i + 1, shape.get_info());
        }
    }

    fn list_prototypes(&self) {
        self.prototype_manager.list_prototypes();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prototype_pattern() {
        let mut editor = GraphicEditor::new();

        // 测试创建形状
        assert!(editor.create_shape("circle", "circle_1".to_string()).is_ok());
        assert!(editor.create_shape("rectangle", "rect_1".to_string()).is_ok());

        // 测试不存在的原型
        assert!(editor.create_shape("triangle", "tri_1".to_string()).is_err());

        // 检查形状数量
        assert_eq!(editor.shapes.len(), 2);
    }

    #[test]
    fn test_clone_independence() {
        let original = Circle::new(
            "original".to_string(),
            10, 20,
            "红色".to_string(),
            5.0,
        );

        let cloned = original.clone();

        // 验证克隆的独立性
        assert_eq!(original.shape.x, cloned.shape.x);
        assert_eq!(original.radius, cloned.radius);
    }
}

pub fn demo() {
    println!("=== 原型模式演示 ===");

    let mut editor = GraphicEditor::new();

    // 显示已注册的原型
    println!("\n1. 显示已注册的原型:");
    editor.list_prototypes();

    // 使用原型创建新对象
    println!("\n2. 基于原型创建新形状:");
    editor.create_shape("circle", "红色圆形1".to_string()).unwrap();
    editor.create_shape("circle", "红色圆形2".to_string()).unwrap();
    editor.create_shape("rectangle", "蓝色矩形1".to_string()).unwrap();

    // 显示创建的形状
    editor.list_shapes();

    // 尝试创建不存在的原型
    println!("\n3. 尝试创建不存在的原型:");
    match editor.create_shape("triangle", "三角形1".to_string()) {
        Ok(_) => println!("创建成功"),
        Err(e) => println!("创建失败: {}", e),
    }

    // 演示原型的独立性
    println!("\n4. 演示克隆对象的独立性:");
    let original_circle = Circle::new(
        "原始圆形".to_string(),
        100, 200,
        "绿色".to_string(),
        25.0,
    );

    let mut cloned_circle = original_circle.clone();
    cloned_circle.shape.set_position(300, 400);
    cloned_circle.shape.set_color("黄色".to_string());
    cloned_circle.set_radius(50.0);

    println!("原始圆形: {}", original_circle.get_info());
    println!("克隆圆形: {}", cloned_circle.get_info());
} 