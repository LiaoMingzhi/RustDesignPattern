//! 建造者模式 (Builder Pattern)
//! 
//! 将一个复杂对象的构建与它的表示分离，使得同样的构建过程可以创建不同的表示。
//! 文件路径：/d%3A/workspace/RustLearn/RustDesignPattern/src/GoFDesignPattern/CreationalPatterns/builder.rs

#[derive(Debug, Clone)]
pub struct Computer {
    cpu: String,
    memory: u32,
    storage: String,
    gpu: Option<String>,
    wifi: bool,
    bluetooth: bool,
}

impl Computer {
    fn new() -> ComputerBuilder {
        ComputerBuilder::new()
    }
}

pub struct ComputerBuilder {
    cpu: Option<String>,
    memory: Option<u32>,
    storage: Option<String>,
    gpu: Option<String>,
    wifi: bool,
    bluetooth: bool,
}

impl ComputerBuilder {
    fn new() -> Self {
        Self {
            cpu: None,
            memory: None,
            storage: None,
            gpu: None,
            wifi: false,
            bluetooth: false,
        }
    }

    pub fn cpu(mut self, cpu: &str) -> Self {
        self.cpu = Some(cpu.to_string());
        self
    }

    pub fn memory(mut self, memory: u32) -> Self {
        self.memory = Some(memory);
        self
    }

    pub fn storage(mut self, storage: &str) -> Self {
        self.storage = Some(storage.to_string());
        self
    }

    pub fn gpu(mut self, gpu: &str) -> Self {
        self.gpu = Some(gpu.to_string());
        self
    }

    pub fn wifi(mut self, wifi: bool) -> Self {
        self.wifi = wifi;
        self
    }

    pub fn bluetooth(mut self, bluetooth: bool) -> Self {
        self.bluetooth = bluetooth;
        self
    }

    pub fn build(self) -> Result<Computer, String> {
        let cpu = self.cpu.ok_or("CPU是必需的")?;
        let memory = self.memory.ok_or("内存是必需的")?;
        let storage = self.storage.ok_or("存储是必需的")?;

        if memory < 4 {
            return Err("内存至少需要4GB".to_string());
        }

        Ok(Computer {
            cpu,
            memory,
            storage,
            gpu: self.gpu,
            wifi: self.wifi,
            bluetooth: self.bluetooth,
        })
    }
}

// 导演类 - 负责具体的构建步骤
pub struct ComputerDirector;

impl ComputerDirector {
    pub fn build_gaming_computer() -> Result<Computer, String> {
        Computer::new()
            .cpu("Intel i9-12900K")
            .memory(32)
            .storage("1TB NVMe SSD")
            .gpu("RTX 4080")
            .wifi(true)
            .bluetooth(true)
            .build()
    }

    pub fn build_office_computer() -> Result<Computer, String> {
        Computer::new()
            .cpu("Intel i5-12400")
            .memory(16)
            .storage("512GB SSD")
            .wifi(true)
            .bluetooth(false)
            .build()
    }

    pub fn build_budget_computer() -> Result<Computer, String> {
        Computer::new()
            .cpu("AMD Ryzen 5 5600")
            .memory(8)
            .storage("256GB SSD")
            .wifi(false)
            .bluetooth(false)
            .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_pattern() {
        // 测试自定义构建
        let custom_computer = Computer::new()
            .cpu("AMD Ryzen 7 5800X")
            .memory(16)
            .storage("1TB SSD")
            .gpu("RTX 3070")
            .wifi(true)
            .bluetooth(true)
            .build();

        assert!(custom_computer.is_ok());
        let computer = custom_computer.unwrap();
        println!("自定义电脑配置: {:?}", computer);

        // 测试预设配置
        let gaming_pc = ComputerDirector::build_gaming_computer();
        assert!(gaming_pc.is_ok());
        println!("游戏电脑配置: {:?}", gaming_pc.unwrap());

        // 测试验证失败的情况
        let invalid_computer = Computer::new()
            .cpu("Intel i3")
            .memory(2) // 内存不足
            .storage("128GB SSD")
            .build();

        assert!(invalid_computer.is_err());
        println!("构建失败: {}", invalid_computer.unwrap_err());
    }
}

pub fn demo() {
    println!("=== 建造者模式演示 ===");

    // 使用导演类构建预设配置
    println!("\n1. 构建游戏电脑:");
    match ComputerDirector::build_gaming_computer() {
        Ok(computer) => println!("{:?}", computer),
        Err(e) => println!("构建失败: {}", e),
    }

    println!("\n2. 构建办公电脑:");
    match ComputerDirector::build_office_computer() {
        Ok(computer) => println!("{:?}", computer),
        Err(e) => println!("构建失败: {}", e),
    }

    // 自定义构建
    println!("\n3. 自定义构建:");
    match Computer::new()
        .cpu("M2 Pro")
        .memory(16)
        .storage("512GB SSD")
        .wifi(true)
        .bluetooth(true)
        .build()
    {
        Ok(computer) => println!("{:?}", computer),
        Err(e) => println!("构建失败: {}", e),
    }
} 