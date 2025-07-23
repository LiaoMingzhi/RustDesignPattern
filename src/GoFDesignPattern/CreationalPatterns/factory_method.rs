//! 工厂方法模式 (Factory Method Pattern)
//! 
//! 定义一个用于创建对象的接口，让子类决定实例化哪一个类。
//! 工厂方法使一个类的实例化延迟到其子类。
//! 文件路径：/d%3A/workspace/RustLearn/RustDesignPattern/src/GoFDesignPattern/CreationalPatterns/factory_method.rs

use std::fmt::Debug;

// 抽象产品
trait Document: Debug {
    fn open(&self);
    fn save(&self);
    fn close(&self);
    fn get_type(&self) -> &str;
}

// 具体产品 - Word文档
#[derive(Debug)]
struct WordDocument {
    name: String,
}

impl WordDocument {
    fn new(name: String) -> Self {
        Self { name }
    }
}

impl Document for WordDocument {
    fn open(&self) {
        println!("打开Word文档: {}", self.name);
    }

    fn save(&self) {
        println!("保存Word文档: {}", self.name);
    }

    fn close(&self) {
        println!("关闭Word文档: {}", self.name);
    }

    fn get_type(&self) -> &str {
        "Word文档"
    }
}

// 具体产品 - PDF文档
#[derive(Debug)]
struct PdfDocument {
    name: String,
}

impl PdfDocument {
    fn new(name: String) -> Self {
        Self { name }
    }
}

impl Document for PdfDocument {
    fn open(&self) {
        println!("打开PDF文档: {}", self.name);
    }

    fn save(&self) {
        println!("保存PDF文档: {}", self.name);
    }

    fn close(&self) {
        println!("关闭PDF文档: {}", self.name);
    }

    fn get_type(&self) -> &str {
        "PDF文档"
    }
}

// 具体产品 - Excel文档
#[derive(Debug)]
struct ExcelDocument {
    name: String,
}

impl ExcelDocument {
    fn new(name: String) -> Self {
        Self { name }
    }
}

impl Document for ExcelDocument {
    fn open(&self) {
        println!("打开Excel文档: {}", self.name);
    }

    fn save(&self) {
        println!("保存Excel文档: {}", self.name);
    }

    fn close(&self) {
        println!("关闭Excel文档: {}", self.name);
    }

    fn get_type(&self) -> &str {
        "Excel文档"
    }
}

// 抽象创建者
trait DocumentCreator {
    // 工厂方法 - 抽象方法
    fn create_document(&self, name: String) -> Box<dyn Document>;

    // 业务方法 - 使用工厂方法的模板方法
    fn new_document(&self, name: String) -> Box<dyn Document> {
        let document = self.create_document(name);
        document.open();
        document
    }
}

// 具体创建者 - Word文档创建者
struct WordDocumentCreator;

impl DocumentCreator for WordDocumentCreator {
    fn create_document(&self, name: String) -> Box<dyn Document> {
        Box::new(WordDocument::new(name))
    }
}

// 具体创建者 - PDF文档创建者
struct PdfDocumentCreator;

impl DocumentCreator for PdfDocumentCreator {
    fn create_document(&self, name: String) -> Box<dyn Document> {
        Box::new(PdfDocument::new(name))
    }
}

// 具体创建者 - Excel文档创建者
struct ExcelDocumentCreator;

impl DocumentCreator for ExcelDocumentCreator {
    fn create_document(&self, name: String) -> Box<dyn Document> {
        Box::new(ExcelDocument::new(name))
    }
}

// 应用程序类
struct Application {
    documents: Vec<Box<dyn Document>>,
}

impl Application {
    fn new() -> Self {
        Self {
            documents: Vec::new(),
        }
    }

    fn create_document(&mut self, doc_type: &str, name: String) -> Result<(), String> {
        let creator: Box<dyn DocumentCreator> = match doc_type {
            "word" => Box::new(WordDocumentCreator),
            "pdf" => Box::new(PdfDocumentCreator),
            "excel" => Box::new(ExcelDocumentCreator),
            _ => return Err(format!("不支持的文档类型: {}", doc_type)),
        };

        let document = creator.new_document(name);
        self.documents.push(document);
        Ok(())
    }

    fn save_all(&self) {
        println!("\n保存所有文档:");
        for doc in &self.documents {
            doc.save();
        }
    }

    fn close_all(&mut self) {
        println!("\n关闭所有文档:");
        for doc in &self.documents {
            doc.close();
        }
        self.documents.clear();
    }

    fn list_documents(&self) {
        println!("\n当前打开的文档:");
        for (i, doc) in self.documents.iter().enumerate() {
            println!("{}. {} - 类型: {}", i + 1, format!("{:?}", doc), doc.get_type());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_factory_method() {
        let mut app = Application::new();

        // 创建不同类型的文档
        assert!(app.create_document("word", "会议记录.docx".to_string()).is_ok());
        assert!(app.create_document("pdf", "项目报告.pdf".to_string()).is_ok());
        assert!(app.create_document("excel", "销售数据.xlsx".to_string()).is_ok());

        // 测试不支持的类型
        assert!(app.create_document("powerpoint", "演示文稿.pptx".to_string()).is_err());

        // 检查文档数量
        assert_eq!(app.documents.len(), 3);
    }
}

pub fn demo() {
    println!("=== 工厂方法模式演示 ===");

    let mut app = Application::new();

    // 创建不同类型的文档
    println!("创建文档:");
    app.create_document("word", "会议记录.docx".to_string()).unwrap();
    app.create_document("pdf", "技术文档.pdf".to_string()).unwrap();
    app.create_document("excel", "财务报表.xlsx".to_string()).unwrap();

    // 列出所有文档
    app.list_documents();

    // 保存和关闭所有文档
    app.save_all();
    app.close_all();

    // 演示错误处理
    println!("\n尝试创建不支持的文档类型:");
    match app.create_document("powerpoint", "演示文稿.pptx".to_string()) {
        Ok(_) => println!("创建成功"),
        Err(e) => println!("创建失败: {}", e),
    }
} 