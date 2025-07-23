//! 访问者模式 (Visitor Pattern)
//! 
//! 表示一个作用于某对象结构中的各元素的操作。它使你可以在不改变各元素的类的前提下定义作用于这些元素的新操作。
//! 文件路径：/d%3A/workspace/RustLearn/RustDesignPattern/src/GoFDesignPattern/BehavioralPatterns/visitor.rs

// 访问者trait
trait ShapeVisitor {
    fn visit_circle(&mut self, circle: &Circle);
    fn visit_rectangle(&mut self, rectangle: &Rectangle);
    fn visit_triangle(&mut self, triangle: &Triangle);
}

// 元素trait
trait Shape {
    fn accept(&self, visitor: &mut dyn ShapeVisitor);
    fn get_info(&self) -> String;
}

// 具体元素 - 圆形
struct Circle {
    radius: f64,
    x: f64,
    y: f64,
}

impl Circle {
    fn new(radius: f64, x: f64, y: f64) -> Self {
        Self { radius, x, y }
    }

    fn get_radius(&self) -> f64 {
        self.radius
    }

    fn get_position(&self) -> (f64, f64) {
        (self.x, self.y)
    }
}

impl Shape for Circle {
    fn accept(&self, visitor: &mut dyn ShapeVisitor) {
        visitor.visit_circle(self);
    }

    fn get_info(&self) -> String {
        format!("圆形(半径: {}, 位置: ({}, {}))", self.radius, self.x, self.y)
    }
}

// 具体元素 - 矩形
struct Rectangle {
    width: f64,
    height: f64,
    x: f64,
    y: f64,
}

impl Rectangle {
    fn new(width: f64, height: f64, x: f64, y: f64) -> Self {
        Self { width, height, x, y }
    }

    fn get_dimensions(&self) -> (f64, f64) {
        (self.width, self.height)
    }

    fn get_position(&self) -> (f64, f64) {
        (self.x, self.y)
    }
}

impl Shape for Rectangle {
    fn accept(&self, visitor: &mut dyn ShapeVisitor) {
        visitor.visit_rectangle(self);
    }

    fn get_info(&self) -> String {
        format!("矩形(宽: {}, 高: {}, 位置: ({}, {}))", 
                self.width, self.height, self.x, self.y)
    }
}

// 具体元素 - 三角形
struct Triangle {
    base: f64,
    height: f64,
    x: f64,
    y: f64,
}

impl Triangle {
    fn new(base: f64, height: f64, x: f64, y: f64) -> Self {
        Self { base, height, x, y }
    }

    fn get_base(&self) -> f64 {
        self.base
    }

    fn get_height(&self) -> f64 {
        self.height
    }

    fn get_position(&self) -> (f64, f64) {
        (self.x, self.y)
    }
}

impl Shape for Triangle {
    fn accept(&self, visitor: &mut dyn ShapeVisitor) {
        visitor.visit_triangle(self);
    }

    fn get_info(&self) -> String {
        format!("三角形(底: {}, 高: {}, 位置: ({}, {}))", 
                self.base, self.height, self.x, self.y)
    }
}

// 具体访问者 - 面积计算器
struct AreaCalculator {
    total_area: f64,
}

impl AreaCalculator {
    fn new() -> Self {
        Self { total_area: 0.0 }
    }

    fn get_total_area(&self) -> f64 {
        self.total_area
    }

    fn reset(&mut self) {
        self.total_area = 0.0;
    }
}

impl ShapeVisitor for AreaCalculator {
    fn visit_circle(&mut self, circle: &Circle) {
        let area = std::f64::consts::PI * circle.get_radius().powi(2);
        self.total_area += area;
        println!("计算圆形面积: {:.2}", area);
    }

    fn visit_rectangle(&mut self, rectangle: &Rectangle) {
        let (width, height) = rectangle.get_dimensions();
        let area = width * height;
        self.total_area += area;
        println!("计算矩形面积: {:.2}", area);
    }

    fn visit_triangle(&mut self, triangle: &Triangle) {
        let area = 0.5 * triangle.get_base() * triangle.get_height();
        self.total_area += area;
        println!("计算三角形面积: {:.2}", area);
    }
}

// 具体访问者 - 周长计算器
struct PerimeterCalculator {
    total_perimeter: f64,
}

impl PerimeterCalculator {
    fn new() -> Self {
        Self { total_perimeter: 0.0 }
    }

    fn get_total_perimeter(&self) -> f64 {
        self.total_perimeter
    }
}

impl ShapeVisitor for PerimeterCalculator {
    fn visit_circle(&mut self, circle: &Circle) {
        let perimeter = 2.0 * std::f64::consts::PI * circle.get_radius();
        self.total_perimeter += perimeter;
        println!("计算圆形周长: {:.2}", perimeter);
    }

    fn visit_rectangle(&mut self, rectangle: &Rectangle) {
        let (width, height) = rectangle.get_dimensions();
        let perimeter = 2.0 * (width + height);
        self.total_perimeter += perimeter;
        println!("计算矩形周长: {:.2}", perimeter);
    }

    fn visit_triangle(&mut self, triangle: &Triangle) {
        // 简化计算，假设是等腰三角形
        let base = triangle.get_base();
        let height = triangle.get_height();
        let side = (height.powi(2) + (base / 2.0).powi(2)).sqrt();
        let perimeter = base + 2.0 * side;
        self.total_perimeter += perimeter;
        println!("计算三角形周长: {:.2}", perimeter);
    }
}

// 具体访问者 - 渲染器
struct RenderVisitor {
    rendered_count: usize,
}

impl RenderVisitor {
    fn new() -> Self {
        Self { rendered_count: 0 }
    }

    fn get_rendered_count(&self) -> usize {
        self.rendered_count
    }
}

impl ShapeVisitor for RenderVisitor {
    fn visit_circle(&mut self, circle: &Circle) {
        let (x, y) = circle.get_position();
        println!("渲染圆形: 半径={}, 中心=({}, {})", circle.get_radius(), x, y);
        self.rendered_count += 1;
    }

    fn visit_rectangle(&mut self, rectangle: &Rectangle) {
        let (x, y) = rectangle.get_position();
        let (w, h) = rectangle.get_dimensions();
        println!("渲染矩形: 尺寸={}x{}, 位置=({}, {})", w, h, x, y);
        self.rendered_count += 1;
    }

    fn visit_triangle(&mut self, triangle: &Triangle) {
        let (x, y) = triangle.get_position();
        println!("渲染三角形: 底={}, 高={}, 位置=({}, {})", 
                triangle.get_base(), triangle.get_height(), x, y);
        self.rendered_count += 1;
    }
}

// 对象结构 - 绘图板
struct Drawing {
    shapes: Vec<Box<dyn Shape>>,
}

impl Drawing {
    fn new() -> Self {
        Self {
            shapes: Vec::new(),
        }
    }

    fn add_shape(&mut self, shape: Box<dyn Shape>) {
        println!("添加形状: {}", shape.get_info());
        self.shapes.push(shape);
    }

    fn accept(&self, visitor: &mut dyn ShapeVisitor) {
        for shape in &self.shapes {
            shape.accept(visitor);
        }
    }

    fn list_shapes(&self) {
        println!("\n绘图板中的形状:");
        for (i, shape) in self.shapes.iter().enumerate() {
            println!("  {}. {}", i + 1, shape.get_info());
        }
    }
}

pub fn demo() {
    println!("=== 访问者模式演示 ===");

    // 创建绘图板和形状
    let mut drawing = Drawing::new();
    
    drawing.add_shape(Box::new(Circle::new(5.0, 10.0, 10.0)));
    drawing.add_shape(Box::new(Rectangle::new(8.0, 6.0, 0.0, 0.0)));
    drawing.add_shape(Box::new(Triangle::new(6.0, 4.0, 5.0, 5.0)));
    drawing.add_shape(Box::new(Circle::new(3.0, 20.0, 15.0)));

    drawing.list_shapes();

    // 使用面积计算访问者
    println!("\n1. 计算总面积:");
    let mut area_calculator = AreaCalculator::new();
    drawing.accept(&mut area_calculator);
    println!("总面积: {:.2}", area_calculator.get_total_area());

    // 使用周长计算访问者
    println!("\n2. 计算总周长:");
    let mut perimeter_calculator = PerimeterCalculator::new();
    drawing.accept(&mut perimeter_calculator);
    println!("总周长: {:.2}", perimeter_calculator.get_total_perimeter());

    // 使用渲染访问者
    println!("\n3. 渲染所有形状:");
    let mut renderer = RenderVisitor::new();
    drawing.accept(&mut renderer);
    println!("已渲染 {} 个形状", renderer.get_rendered_count());

    println!("\n访问者模式的优点:");
    println!("1. 增加新的访问者很容易，符合开闭原则");
    println!("2. 将有关的行为集中到一个访问者对象中");
    println!("3. 使得增加新的操作变得容易");
    println!("4. 访问者可以跨越类的等级结构访问属于不同等级结构的成员对象");
} 