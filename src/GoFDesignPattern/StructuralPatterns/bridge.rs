//! 桥接模式 (Bridge Pattern)
//! 
//! 将抽象部分与它的实现部分分离，使它们都可以独立地变化。
//! 文件路径：/d%3A/workspace/RustLearn/RustDesignPattern/src/GoFDesignPattern/StructuralPatterns/bridge.rs

// 实现接口 - 绘图API
trait DrawingAPI {
    fn draw_circle(&self, x: f64, y: f64, radius: f64);
    fn draw_rectangle(&self, x: f64, y: f64, width: f64, height: f64);
    fn set_color(&mut self, color: &str);
    fn get_api_name(&self) -> &str;
}

// 具体实现 - OpenGL绘图API
struct OpenGLAPI {
    color: String,
}

impl OpenGLAPI {
    fn new() -> Self {
        Self {
            color: "白色".to_string(),
        }
    }
}

impl DrawingAPI for OpenGLAPI {
    fn draw_circle(&self, x: f64, y: f64, radius: f64) {
        println!(
            "[OpenGL] 绘制{}圆形: 中心({:.1}, {:.1}), 半径: {:.1}",
            self.color, x, y, radius
        );
    }

    fn draw_rectangle(&self, x: f64, y: f64, width: f64, height: f64) {
        println!(
            "[OpenGL] 绘制{}矩形: 位置({:.1}, {:.1}), 尺寸: {:.1}x{:.1}",
            self.color, x, y, width, height
        );
    }

    fn set_color(&mut self, color: &str) {
        self.color = color.to_string();
        println!("[OpenGL] 设置颜色为: {}", color);
    }

    fn get_api_name(&self) -> &str {
        "OpenGL"
    }
}

// 具体实现 - DirectX绘图API
struct DirectXAPI {
    color: String,
}

impl DirectXAPI {
    fn new() -> Self {
        Self {
            color: "白色".to_string(),
        }
    }
}

impl DrawingAPI for DirectXAPI {
    fn draw_circle(&self, x: f64, y: f64, radius: f64) {
        println!(
            "[DirectX] 渲染{}圆形: 坐标({:.1}, {:.1}), 半径: {:.1}",
            self.color, x, y, radius
        );
    }

    fn draw_rectangle(&self, x: f64, y: f64, width: f64, height: f64) {
        println!(
            "[DirectX] 渲染{}矩形: 坐标({:.1}, {:.1}), 大小: {:.1}x{:.1}",
            self.color, x, y, width, height
        );
    }

    fn set_color(&mut self, color: &str) {
        self.color = color.to_string();
        println!("[DirectX] 颜色设置为: {}", color);
    }

    fn get_api_name(&self) -> &str {
        "DirectX"
    }
}

// 具体实现 - SVG绘图API
struct SVGAPI {
    color: String,
    svg_content: Vec<String>,
}

impl SVGAPI {
    fn new() -> Self {
        Self {
            color: "black".to_string(),
            svg_content: Vec::new(),
        }
    }

    fn export_svg(&self) -> String {
        let mut svg = String::from("<svg xmlns=\"http://www.w3.org/2000/svg\">\n");
        for content in &self.svg_content {
            svg.push_str(&format!("  {}\n", content));
        }
        svg.push_str("</svg>");
        svg
    }
}

impl DrawingAPI for SVGAPI {
    fn draw_circle(&self, x: f64, y: f64, radius: f64) {
        let svg_element = format!(
            "<circle cx=\"{:.1}\" cy=\"{:.1}\" r=\"{:.1}\" fill=\"{}\" />",
            x, y, radius, self.color
        );
        println!("[SVG] 添加圆形元素: {}", svg_element);
    }

    fn draw_rectangle(&self, x: f64, y: f64, width: f64, height: f64) {
        let svg_element = format!(
            "<rect x=\"{:.1}\" y=\"{:.1}\" width=\"{:.1}\" height=\"{:.1}\" fill=\"{}\" />",
            x, y, width, height, self.color
        );
        println!("[SVG] 添加矩形元素: {}", svg_element);
    }

    fn set_color(&mut self, color: &str) {
        self.color = color.to_string();
        println!("[SVG] 设置填充颜色: {}", color);
    }

    fn get_api_name(&self) -> &str {
        "SVG"
    }
}

// 抽象类 - 形状
trait Shape {
    fn draw(&self);
    fn resize(&mut self, factor: f64);
    fn move_to(&mut self, x: f64, y: f64);
    fn set_color(&mut self, color: &str);
    fn get_info(&self) -> String;
}

// 具体抽象类 - 圆形
struct Circle {
    x: f64,
    y: f64,
    radius: f64,
    drawing_api: Box<dyn DrawingAPI>,
}

impl Circle {
    fn new(x: f64, y: f64, radius: f64, drawing_api: Box<dyn DrawingAPI>) -> Self {
        Self {
            x,
            y,
            radius,
            drawing_api,
        }
    }
}

impl Shape for Circle {
    fn draw(&self) {
        self.drawing_api.draw_circle(self.x, self.y, self.radius);
    }

    fn resize(&mut self, factor: f64) {
        self.radius *= factor;
        println!("圆形缩放因子: {:.2}, 新半径: {:.1}", factor, self.radius);
    }

    fn move_to(&mut self, x: f64, y: f64) {
        println!("圆形从({:.1}, {:.1})移动到({:.1}, {:.1})", self.x, self.y, x, y);
        self.x = x;
        self.y = y;
    }

    fn set_color(&mut self, color: &str) {
        self.drawing_api.set_color(color);
    }

    fn get_info(&self) -> String {
        format!(
            "圆形[API: {}, 位置: ({:.1}, {:.1}), 半径: {:.1}]",
            self.drawing_api.get_api_name(),
            self.x,
            self.y,
            self.radius
        )
    }
}

// 具体抽象类 - 矩形
struct Rectangle {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    drawing_api: Box<dyn DrawingAPI>,
}

impl Rectangle {
    fn new(x: f64, y: f64, width: f64, height: f64, drawing_api: Box<dyn DrawingAPI>) -> Self {
        Self {
            x,
            y,
            width,
            height,
            drawing_api,
        }
    }
}

impl Shape for Rectangle {
    fn draw(&self) {
        self.drawing_api
            .draw_rectangle(self.x, self.y, self.width, self.height);
    }

    fn resize(&mut self, factor: f64) {
        self.width *= factor;
        self.height *= factor;
        println!(
            "矩形缩放因子: {:.2}, 新尺寸: {:.1}x{:.1}",
            factor, self.width, self.height
        );
    }

    fn move_to(&mut self, x: f64, y: f64) {
        println!("矩形从({:.1}, {:.1})移动到({:.1}, {:.1})", self.x, self.y, x, y);
        self.x = x;
        self.y = y;
    }

    fn set_color(&mut self, color: &str) {
        self.drawing_api.set_color(color);
    }

    fn get_info(&self) -> String {
        format!(
            "矩形[API: {}, 位置: ({:.1}, {:.1}), 尺寸: {:.1}x{:.1}]",
            self.drawing_api.get_api_name(),
            self.x,
            self.y,
            self.width,
            self.height
        )
    }
}

// 图形编辑器 - 演示桥接模式的使用
struct GraphicsEditor {
    shapes: Vec<Box<dyn Shape>>,
}

impl GraphicsEditor {
    fn new() -> Self {
        Self {
            shapes: Vec::new(),
        }
    }

    fn add_shape(&mut self, shape: Box<dyn Shape>) {
        println!("添加形状: {}", shape.get_info());
        self.shapes.push(shape);
    }

    fn draw_all(&self) {
        println!("\n绘制所有形状:");
        for (i, shape) in self.shapes.iter().enumerate() {
            println!("形状{}: {}", i + 1, shape.get_info());
            shape.draw();
        }
    }

    fn resize_all(&mut self, factor: f64) {
        println!("\n缩放所有形状，因子: {:.2}", factor);
        for shape in &mut self.shapes {
            shape.resize(factor);
        }
    }

    fn set_all_color(&mut self, color: &str) {
        println!("\n设置所有形状颜色为: {}", color);
        for shape in &mut self.shapes {
            shape.set_color(color);
        }
    }

    fn list_shapes(&self) {
        println!("\n当前形状列表:");
        for (i, shape) in self.shapes.iter().enumerate() {
            println!("{}. {}", i + 1, shape.get_info());
        }
    }
}

// API工厂 - 根据平台创建不同的绘图API
struct APIFactory;

impl APIFactory {
    fn create_api(api_type: &str) -> Result<Box<dyn DrawingAPI>, String> {
        match api_type.to_lowercase().as_str() {
            "opengl" => Ok(Box::new(OpenGLAPI::new())),
            "directx" => Ok(Box::new(DirectXAPI::new())),
            "svg" => Ok(Box::new(SVGAPI::new())),
            _ => Err(format!("不支持的API类型: {}", api_type)),
        }
    }

    fn get_supported_apis() -> Vec<&'static str> {
        vec!["OpenGL", "DirectX", "SVG"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bridge_pattern() {
        // 测试不同API的圆形
        let opengl_api = Box::new(OpenGLAPI::new());
        let mut circle = Circle::new(10.0, 20.0, 5.0, opengl_api);
        
        circle.draw();
        circle.set_color("红色");
        circle.resize(1.5);

        // 测试不同API的矩形
        let directx_api = Box::new(DirectXAPI::new());
        let mut rectangle = Rectangle::new(0.0, 0.0, 100.0, 50.0, directx_api);
        
        rectangle.draw();
        rectangle.move_to(10.0, 10.0);
        rectangle.draw();
    }

    #[test]
    fn test_api_independence() {
        // 验证形状可以独立使用不同的API
        let apis = vec![
            APIFactory::create_api("opengl").unwrap(),
            APIFactory::create_api("directx").unwrap(),
            APIFactory::create_api("svg").unwrap(),
        ];

        for (i, api) in apis.into_iter().enumerate() {
            let circle = Circle::new(i as f64 * 10.0, 0.0, 5.0, api);
            println!("测试API {}: {}", i + 1, circle.get_info());
        }
    }
}

pub fn demo() {
    println!("=== 桥接模式演示 ===");

    println!("\n1. 支持的绘图API:");
    for api in APIFactory::get_supported_apis() {
        println!("  - {}", api);
    }

    // 创建图形编辑器
    let mut editor = GraphicsEditor::new();

    println!("\n2. 使用不同API创建形状:");

    // 使用OpenGL API创建圆形
    if let Ok(opengl_api) = APIFactory::create_api("opengl") {
        let mut circle = Circle::new(50.0, 50.0, 25.0, opengl_api);
        circle.set_color("红色");
        editor.add_shape(Box::new(circle));
    }

    // 使用DirectX API创建矩形
    if let Ok(directx_api) = APIFactory::create_api("directx") {
        let mut rectangle = Rectangle::new(100.0, 100.0, 80.0, 60.0, directx_api);
        rectangle.set_color("蓝色");
        editor.add_shape(Box::new(rectangle));
    }

    // 使用SVG API创建另一个圆形
    if let Ok(svg_api) = APIFactory::create_api("svg") {
        let mut svg_circle = Circle::new(200.0, 150.0, 30.0, svg_api);
        svg_circle.set_color("绿色");
        editor.add_shape(Box::new(svg_circle));
    }

    // 显示所有形状
    editor.list_shapes();

    // 绘制所有形状
    editor.draw_all();

    // 统一操作所有形状
    println!("\n3. 统一操作演示:");
    editor.resize_all(1.2);
    editor.set_all_color("黄色");
    editor.draw_all();

    println!("\n桥接模式的优点:");
    println!("1. 分离抽象接口和实现部分，两者可以独立变化");
    println!("2. 提高了可扩展性，可以独立扩展抽象部分和实现部分");
    println!("3. 实现细节对客户端透明");
    println!("4. 符合开闭原则，可以在不修改现有代码的情况下添加新的抽象和实现");
} 