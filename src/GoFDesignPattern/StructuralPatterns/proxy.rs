//! 代理模式 (Proxy Pattern)
//! 
//! 为其他对象提供一种代理以控制对这个对象的访问。
//! 文件路径：/d%3A/workspace/RustLearn/RustDesignPattern/src/GoFDesignPattern/StructuralPatterns/proxy.rs

use std::collections::HashMap;

// 主题接口
trait Image {
    fn display(&self);
    fn get_info(&self) -> String;
}

// 真实主题 - 高分辨率图像
struct RealImage {
    filename: String,
    size: usize,
}

impl RealImage {
    fn new(filename: String) -> Self {
        println!("从磁盘加载图像: {}", filename);
        let size = 1024 * 1024; // 模拟1MB数据
        println!("图像 {} 加载完成 ({}KB)", filename, size / 1024);
        
        Self { filename, size }
    }
}

impl Image for RealImage {
    fn display(&self) {
        println!("显示图像: {} ({}KB)", self.filename, self.size / 1024);
    }

    fn get_info(&self) -> String {
        format!("真实图像: {} ({}KB)", self.filename, self.size / 1024)
    }
}

// 代理 - 图像代理
struct ImageProxy {
    filename: String,
    real_image: Option<RealImage>,
}

impl ImageProxy {
    fn new(filename: String) -> Self {
        Self {
            filename,
            real_image: None,
        }
    }

    fn load_real_image(&mut self) {
        if self.real_image.is_none() {
            self.real_image = Some(RealImage::new(self.filename.clone()));
        }
    }
}

impl Image for ImageProxy {
    fn display(&self) {
        if let Some(ref real_image) = self.real_image {
            real_image.display();
        } else {
            println!("显示占位符图像: {}", self.filename);
        }
    }

    fn get_info(&self) -> String {
        if let Some(ref real_image) = self.real_image {
            real_image.get_info()
        } else {
            format!("图像代理: {} (未加载)", self.filename)
        }
    }
}

// 访问控制代理示例
trait FileSystem {
    fn read_file(&self, filename: &str) -> Result<String, String>;
    fn write_file(&self, filename: &str, content: &str) -> Result<(), String>;
}

struct RealFileSystem {
    files: HashMap<String, String>,
}

impl RealFileSystem {
    fn new() -> Self {
        let mut files = HashMap::new();
        files.insert("public.txt".to_string(), "公开文件内容".to_string());
        files.insert("secret.txt".to_string(), "机密文件内容".to_string());
        
        Self { files }
    }
}

impl FileSystem for RealFileSystem {
    fn read_file(&self, filename: &str) -> Result<String, String> {
        self.files.get(filename)
            .cloned()
            .ok_or_else(|| format!("文件不存在: {}", filename))
    }

    fn write_file(&self, filename: &str, content: &str) -> Result<(), String> {
        println!("写入文件 {}: {}", filename, content);
        Ok(())
    }
}

struct FileSystemProxy {
    real_fs: RealFileSystem,
    user_role: String,
}

impl FileSystemProxy {
    fn new(user_role: String) -> Self {
        Self {
            real_fs: RealFileSystem::new(),
            user_role,
        }
    }

    fn check_access(&self, filename: &str, operation: &str) -> bool {
        if filename.contains("secret") && self.user_role != "admin" {
            println!("访问被拒绝: 用户 '{}' 无权限 {} 文件 '{}'", 
                    self.user_role, operation, filename);
            false
        } else {
            true
        }
    }
}

impl FileSystem for FileSystemProxy {
    fn read_file(&self, filename: &str) -> Result<String, String> {
        if self.check_access(filename, "读取") {
            self.real_fs.read_file(filename)
        } else {
            Err("访问被拒绝".to_string())
        }
    }

    fn write_file(&self, filename: &str, content: &str) -> Result<(), String> {
        if self.check_access(filename, "写入") {
            self.real_fs.write_file(filename, content)
        } else {
            Err("访问被拒绝".to_string())
        }
    }
}

pub fn demo() {
    println!("=== 代理模式演示 ===");

    println!("\n1. 虚拟代理 - 懒加载:");
    let mut proxy = ImageProxy::new("大图片.jpg".to_string());
    println!("创建代理: {}", proxy.get_info());
    
    proxy.load_real_image();
    proxy.display();

    println!("\n2. 保护代理 - 访问控制:");
    let user_fs = FileSystemProxy::new("user".to_string());
    let admin_fs = FileSystemProxy::new("admin".to_string());
    
    // 普通用户尝试访问
    match user_fs.read_file("secret.txt") {
        Ok(content) => println!("读取成功: {}", content),
        Err(e) => println!("读取失败: {}", e),
    }
    
    // 管理员访问
    match admin_fs.read_file("secret.txt") {
        Ok(content) => println!("读取成功: {}", content),
        Err(e) => println!("读取失败: {}", e),
    }
} 