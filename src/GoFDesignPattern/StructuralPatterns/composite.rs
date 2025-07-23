//! 组合模式 (Composite Pattern)
//! 
//! 将对象组合成树形结构以表示"部分-整体"的层次结构。
//! 文件路径：/d%3A/workspace/RustLearn/RustDesignPattern/src/GoFDesignPattern/StructuralPatterns/composite.rs

// 组件接口
trait Component {
    fn operation(&self);
    fn add(&mut self, component: Box<dyn Component>) -> Result<(), String>;
    fn remove(&mut self, index: usize) -> Result<(), String>;
    fn get_child(&self, index: usize) -> Option<&dyn Component>;
    fn get_name(&self) -> &str;
}

// 叶子节点 - 文件
struct File {
    name: String,
    size: u64,
}

impl File {
    fn new(name: String, size: u64) -> Self {
        Self { name, size }
    }
}

impl Component for File {
    fn operation(&self) {
        println!("文件: {} ({}KB)", self.name, self.size);
    }

    fn add(&mut self, _component: Box<dyn Component>) -> Result<(), String> {
        Err("文件不能添加子组件".to_string())
    }

    fn remove(&mut self, _index: usize) -> Result<(), String> {
        Err("文件不能删除子组件".to_string())
    }

    fn get_child(&self, _index: usize) -> Option<&dyn Component> {
        None
    }

    fn get_name(&self) -> &str {
        &self.name
    }
}

// 复合节点 - 文件夹
struct Folder {
    name: String,
    children: Vec<Box<dyn Component>>,
}

impl Folder {
    fn new(name: String) -> Self {
        Self {
            name,
            children: Vec::new(),
        }
    }
}

impl Component for Folder {
    fn operation(&self) {
        println!("文件夹: {} ({}个项目)", self.name, self.children.len());
        for child in &self.children {
            child.operation();
        }
    }

    fn add(&mut self, component: Box<dyn Component>) -> Result<(), String> {
        println!("添加 {} 到文件夹 {}", component.get_name(), self.name);
        self.children.push(component);
        Ok(())
    }

    fn remove(&mut self, index: usize) -> Result<(), String> {
        if index < self.children.len() {
            let removed = self.children.remove(index);
            println!("从文件夹 {} 删除 {}", self.name, removed.get_name());
            Ok(())
        } else {
            Err("索引超出范围".to_string())
        }
    }

    fn get_child(&self, index: usize) -> Option<&dyn Component> {
        self.children.get(index).map(|child| child.as_ref())
    }

    fn get_name(&self) -> &str {
        &self.name
    }
}

pub fn demo() {
    println!("=== 组合模式演示 ===");

    // 创建文件系统结构
    let mut root = Folder::new("根目录".to_string());
    let mut documents = Folder::new("文档".to_string());
    let mut images = Folder::new("图片".to_string());

    // 添加文件
    documents.add(Box::new(File::new("报告.docx".to_string(), 120))).unwrap();
    documents.add(Box::new(File::new("笔记.txt".to_string(), 25))).unwrap();
    
    images.add(Box::new(File::new("照片1.jpg".to_string(), 2500))).unwrap();
    images.add(Box::new(File::new("照片2.png".to_string(), 1800))).unwrap();

    // 构建层次结构
    root.add(Box::new(documents)).unwrap();
    root.add(Box::new(images)).unwrap();
    root.add(Box::new(File::new("系统文件.sys".to_string(), 500))).unwrap();

    // 统一操作
    root.operation();
} 