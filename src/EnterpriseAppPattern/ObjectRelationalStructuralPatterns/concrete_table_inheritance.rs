//! # 具体表继承模式（Concrete Table Inheritance Pattern）
//!
//! 具体表继承模式为继承层次中的每个具体类创建一个表，
//! 每个表包含该类及其所有超类的所有字段。
//! 这种方式避免了表连接，但可能导致数据冗余。
//!
//! ## 模式特点
//! - **无需连接**: 每个表都是自包含的
//! - **查询简单**: 单表查询性能好
//! - **数据冗余**: 父类字段在每个子表中重复
//! - **模式变更**: 父类变更影响所有子表
//!
//! ## 使用场景
//! - 查询性能优先的场景
//! - 很少进行多态查询时
//! - 继承层次相对稳定时
//! - 表连接代价较高时

use std::collections::HashMap;
use std::fmt::{self, Display, Formatter};
use std::error::Error;

/// 具体表继承错误类型
#[derive(Debug)]
pub enum ConcreteTableInheritanceError {
    RecordNotFound(String),
    ValidationError(String),
    DatabaseError(String),
    TypeMismatch(String),
}

impl Display for ConcreteTableInheritanceError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ConcreteTableInheritanceError::RecordNotFound(msg) => write!(f, "记录未找到: {}", msg),
            ConcreteTableInheritanceError::ValidationError(msg) => write!(f, "验证错误: {}", msg),
            ConcreteTableInheritanceError::DatabaseError(msg) => write!(f, "数据库错误: {}", msg),
            ConcreteTableInheritanceError::TypeMismatch(msg) => write!(f, "类型不匹配: {}", msg),
        }
    }
}

impl Error for ConcreteTableInheritanceError {}

/// 动物基类
#[derive(Debug, Clone)]
pub struct Animal {
    pub id: i64,
    pub name: String,
    pub species: String,
    pub birth_date: String,
    pub weight: f64,
    pub health_status: String,
}

impl Animal {
    pub fn new(id: i64, name: String, species: String) -> Self {
        Self {
            id,
            name,
            species,
            birth_date: "2020-01-01".to_string(),
            weight: 0.0,
            health_status: "健康".to_string(),
        }
    }

    pub fn get_age_in_years(&self) -> i32 {
        // 简化的年龄计算
        2024 - 2020
    }
}

/// 狗类（具体表：dogs）
#[derive(Debug, Clone)]
pub struct Dog {
    // 包含父类的所有字段
    pub id: i64,
    pub name: String,
    pub species: String,
    pub birth_date: String,
    pub weight: f64,
    pub health_status: String,
    // 狗特有的字段
    pub breed: String,
    pub training_level: TrainingLevel,
    pub favorite_toy: String,
    pub is_house_trained: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TrainingLevel {
    Untrained,
    Basic,
    Intermediate,
    Advanced,
}

impl Dog {
    pub fn new(id: i64, name: String, breed: String) -> Self {
        Self {
            id,
            name,
            species: "犬科".to_string(),
            birth_date: "2020-01-01".to_string(),
            weight: 20.0,
            health_status: "健康".to_string(),
            breed,
            training_level: TrainingLevel::Untrained,
            favorite_toy: String::new(),
            is_house_trained: false,
        }
    }

    pub fn train(&mut self, level: TrainingLevel) {
        self.training_level = level;
        println!("🐕 {} 训练到 {:?} 级别", self.name, self.training_level);
    }

    pub fn play_with_toy(&self, toy: &str) {
        println!("🎾 {} 正在玩 {}", self.name, toy);
    }
}

/// 猫类（具体表：cats）
#[derive(Debug, Clone)]
pub struct Cat {
    // 包含父类的所有字段
    pub id: i64,
    pub name: String,
    pub species: String,
    pub birth_date: String,
    pub weight: f64,
    pub health_status: String,
    // 猫特有的字段
    pub breed: String,
    pub coat_pattern: String,
    pub is_indoor: bool,
    pub scratching_post_preference: String,
}

impl Cat {
    pub fn new(id: i64, name: String, breed: String) -> Self {
        Self {
            id,
            name,
            species: "猫科".to_string(),
            birth_date: "2020-01-01".to_string(),
            weight: 4.5,
            health_status: "健康".to_string(),
            breed,
            coat_pattern: "纯色".to_string(),
            is_indoor: true,
            scratching_post_preference: "剑麻".to_string(),
        }
    }

    pub fn scratch(&self) {
        println!("🐱 {} 正在用 {} 磨爪", self.name, self.scratching_post_preference);
    }

    pub fn purr(&self) {
        println!("😸 {} 发出了呼噜声", self.name);
    }
}

/// 鸟类（具体表：birds）
#[derive(Debug, Clone)]
pub struct Bird {
    // 包含父类的所有字段
    pub id: i64,
    pub name: String,
    pub species: String,
    pub birth_date: String,
    pub weight: f64,
    pub health_status: String,
    // 鸟特有的字段
    pub wing_span: f64,
    pub can_fly: bool,
    pub song_complexity: SongComplexity,
    pub migration_pattern: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SongComplexity {
    Silent,
    Simple,
    Complex,
    Melodic,
}

impl Bird {
    pub fn new(id: i64, name: String, species: String) -> Self {
        Self {
            id,
            name,
            species,
            birth_date: "2020-01-01".to_string(),
            weight: 0.5,
            health_status: "健康".to_string(),
            wing_span: 20.0,
            can_fly: true,
            song_complexity: SongComplexity::Simple,
            migration_pattern: "无迁徙".to_string(),
        }
    }

    pub fn sing(&self) {
        match self.song_complexity {
            SongComplexity::Silent => println!("🦅 {} 保持沉默", self.name),
            SongComplexity::Simple => println!("🐦 {} 发出简单的叫声", self.name),
            SongComplexity::Complex => println!("🎵 {} 唱出复杂的歌曲", self.name),
            SongComplexity::Melodic => println!("🎶 {} 演奏美妙的旋律", self.name),
        }
    }

    pub fn fly(&self) {
        if self.can_fly {
            println!("🕊️ {} 展翅飞翔，翼展 {:.1}cm", self.name, self.wing_span);
        } else {
            println!("🐧 {} 不能飞行", self.name);
        }
    }
}

/// 具体表继承数据访问对象
pub struct ConcreteTableInheritanceDAO {
    dogs: HashMap<i64, Dog>,
    cats: HashMap<i64, Cat>,
    birds: HashMap<i64, Bird>,
    next_id: i64,
}

impl ConcreteTableInheritanceDAO {
    pub fn new() -> Self {
        Self {
            dogs: HashMap::new(),
            cats: HashMap::new(),
            birds: HashMap::new(),
            next_id: 1,
        }
    }

    fn generate_id(&mut self) -> i64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    /// 保存狗
    pub fn save_dog(&mut self, mut dog: Dog) -> Result<i64, ConcreteTableInheritanceError> {
        if dog.id == 0 {
            dog.id = self.generate_id();
        }
        
        if dog.name.trim().is_empty() {
            return Err(ConcreteTableInheritanceError::ValidationError("狗的名字不能为空".to_string()));
        }

        let id = dog.id;
        self.dogs.insert(id, dog);
        println!("🐕 保存狗记录: ID {}", id);
        Ok(id)
    }

    /// 保存猫
    pub fn save_cat(&mut self, mut cat: Cat) -> Result<i64, ConcreteTableInheritanceError> {
        if cat.id == 0 {
            cat.id = self.generate_id();
        }
        
        if cat.name.trim().is_empty() {
            return Err(ConcreteTableInheritanceError::ValidationError("猫的名字不能为空".to_string()));
        }

        let id = cat.id;
        self.cats.insert(id, cat);
        println!("🐱 保存猫记录: ID {}", id);
        Ok(id)
    }

    /// 保存鸟
    pub fn save_bird(&mut self, mut bird: Bird) -> Result<i64, ConcreteTableInheritanceError> {
        if bird.id == 0 {
            bird.id = self.generate_id();
        }
        
        if bird.name.trim().is_empty() {
            return Err(ConcreteTableInheritanceError::ValidationError("鸟的名字不能为空".to_string()));
        }

        let id = bird.id;
        self.birds.insert(id, bird);
        println!("🐦 保存鸟记录: ID {}", id);
        Ok(id)
    }

    /// 查找狗
    pub fn find_dog(&self, id: i64) -> Result<&Dog, ConcreteTableInheritanceError> {
        self.dogs.get(&id)
            .ok_or_else(|| ConcreteTableInheritanceError::RecordNotFound(format!("狗 {} 不存在", id)))
    }

    /// 查找猫
    pub fn find_cat(&self, id: i64) -> Result<&Cat, ConcreteTableInheritanceError> {
        self.cats.get(&id)
            .ok_or_else(|| ConcreteTableInheritanceError::RecordNotFound(format!("猫 {} 不存在", id)))
    }

    /// 查找鸟
    pub fn find_bird(&self, id: i64) -> Result<&Bird, ConcreteTableInheritanceError> {
        self.birds.get(&id)
            .ok_or_else(|| ConcreteTableInheritanceError::RecordNotFound(format!("鸟 {} 不存在", id)))
    }

    /// 获取所有动物的基本信息（需要联合查询）
    pub fn get_all_animals_summary(&self) -> Vec<AnimalSummary> {
        let mut summaries = Vec::new();

        // 从狗表收集
        for dog in self.dogs.values() {
            summaries.push(AnimalSummary {
                id: dog.id,
                name: dog.name.clone(),
                species: dog.species.clone(),
                animal_type: "狗".to_string(),
                weight: dog.weight,
            });
        }

        // 从猫表收集
        for cat in self.cats.values() {
            summaries.push(AnimalSummary {
                id: cat.id,
                name: cat.name.clone(),
                species: cat.species.clone(),
                animal_type: "猫".to_string(),
                weight: cat.weight,
            });
        }

        // 从鸟表收集
        for bird in self.birds.values() {
            summaries.push(AnimalSummary {
                id: bird.id,
                name: bird.name.clone(),
                species: bird.species.clone(),
                animal_type: "鸟".to_string(),
                weight: bird.weight,
            });
        }

        summaries
    }

    /// 按重量范围查找动物（需要查询所有表）
    pub fn find_animals_by_weight_range(&self, min_weight: f64, max_weight: f64) -> Vec<AnimalSummary> {
        self.get_all_animals_summary()
            .into_iter()
            .filter(|animal| animal.weight >= min_weight && animal.weight <= max_weight)
            .collect()
    }

    /// 获取统计信息
    pub fn get_statistics(&self) -> AnimalStatistics {
        AnimalStatistics {
            total_dogs: self.dogs.len(),
            total_cats: self.cats.len(),
            total_birds: self.birds.len(),
            total_animals: self.dogs.len() + self.cats.len() + self.birds.len(),
        }
    }
}

/// 动物摘要信息
#[derive(Debug, Clone)]
pub struct AnimalSummary {
    pub id: i64,
    pub name: String,
    pub species: String,
    pub animal_type: String,
    pub weight: f64,
}

impl Display for AnimalSummary {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}[{}] - {}, {}, 重量: {:.1}kg", 
               self.animal_type, self.id, self.name, self.species, self.weight)
    }
}

/// 动物统计信息
#[derive(Debug)]
pub struct AnimalStatistics {
    pub total_dogs: usize,
    pub total_cats: usize,
    pub total_birds: usize,
    pub total_animals: usize,
}

impl Display for AnimalStatistics {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "动物统计 - 狗: {}, 猫: {}, 鸟: {}, 总数: {}", 
               self.total_dogs, self.total_cats, self.total_birds, self.total_animals)
    }
}

/// 演示具体表继承模式
pub fn demo() {
    println!("=== 具体表继承模式演示 ===\n");

    let mut dao = ConcreteTableInheritanceDAO::new();

    // 创建并保存狗
    println!("1. 创建并保存狗");
    let mut dog1 = Dog::new(0, "旺财".to_string(), "金毛寻回犬".to_string());
    dog1.weight = 30.0;
    dog1.favorite_toy = "网球".to_string();
    dog1.is_house_trained = true;

    let mut dog2 = Dog::new(0, "小黑".to_string(), "拉布拉多".to_string());
    dog2.weight = 25.0;
    dog2.training_level = TrainingLevel::Advanced;

    let _ = dao.save_dog(dog1);
    let _ = dao.save_dog(dog2);

    // 创建并保存猫
    println!("\n2. 创建并保存猫");
    let mut cat1 = Cat::new(0, "咪咪".to_string(), "英国短毛猫".to_string());
    cat1.weight = 4.2;
    cat1.coat_pattern = "银渐层".to_string();

    let mut cat2 = Cat::new(0, "橘子".to_string(), "橘猫".to_string());
    cat2.weight = 5.8;
    cat2.coat_pattern = "橘色条纹".to_string();
    cat2.is_indoor = false;

    let _ = dao.save_cat(cat1);
    let _ = dao.save_cat(cat2);

    // 创建并保存鸟
    println!("\n3. 创建并保存鸟");
    let mut bird1 = Bird::new(0, "小红".to_string(), "红腹锦鸡".to_string());
    bird1.weight = 0.8;
    bird1.wing_span = 40.0;
    bird1.song_complexity = SongComplexity::Melodic;

    let mut bird2 = Bird::new(0, "企鹅".to_string(), "帝企鹅".to_string());
    bird2.weight = 30.0;
    bird2.wing_span = 100.0;
    bird2.can_fly = false;

    let _ = dao.save_bird(bird1);
    let _ = dao.save_bird(bird2);

    // 演示具体类型的特有行为
    println!("\n4. 演示动物特有行为");
    
    if let Ok(dog) = dao.find_dog(1) {
        dog.play_with_toy("飞盘");
        // 训练狗
        let mut dog_clone = dog.clone();
        dog_clone.train(TrainingLevel::Intermediate);
    }

    if let Ok(cat) = dao.find_cat(3) {
        cat.scratch();
        cat.purr();
    }

    if let Ok(bird) = dao.find_bird(5) {
        bird.sing();
        bird.fly();
    }

    // 演示多态查询（需要联合所有表）
    println!("\n5. 多态查询演示");
    let all_animals = dao.get_all_animals_summary();
    println!("   所有动物:");
    for animal in &all_animals {
        println!("     {}", animal);
    }

    // 按重量范围查询
    println!("\n6. 按重量范围查询（5-35kg）");
    let medium_animals = dao.find_animals_by_weight_range(5.0, 35.0);
    for animal in medium_animals {
        println!("     {}", animal);
    }

    // 显示统计信息
    println!("\n7. 统计信息");
    let stats = dao.get_statistics();
    println!("   {}", stats);

    // 演示单表查询的优势
    println!("\n8. 单表查询性能演示");
    println!("   查询所有狗（单表查询，性能优秀）:");
    for dog in dao.dogs.values() {
        println!("     狗[{}] - {}, 品种: {}, 训练级别: {:?}", 
                 dog.id, dog.name, dog.breed, dog.training_level);
    }

    println!("\n=== 具体表继承模式演示完成 ===");

    println!("\n💡 具体表继承模式的优势:");
    println!("1. 查询性能 - 每个表都是自包含的，无需表连接");
    println!("2. 简单查询 - 单表查询逻辑简单，容易优化");
    println!("3. 模式清晰 - 每个具体类对应一个明确的表结构");
    println!("4. 并发友好 - 不同类型的操作不会互相影响");

    println!("\n⚠️ 设计考虑:");
    println!("1. 数据冗余 - 父类字段在每个子表中重复存储");
    println!("2. 模式变更 - 父类结构变更需要修改所有子表");
    println!("3. 多态查询 - 需要联合查询多个表，增加复杂性");
    println!("4. 数据一致性 - 跨表的业务规则较难维护");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dog_creation_and_training() {
        let mut dog = Dog::new(1, "测试犬".to_string(), "测试品种".to_string());
        assert_eq!(dog.training_level, TrainingLevel::Untrained);
        
        dog.train(TrainingLevel::Basic);
        assert_eq!(dog.training_level, TrainingLevel::Basic);
    }

    #[test]
    fn test_dao_operations() {
        let mut dao = ConcreteTableInheritanceDAO::new();
        
        let dog = Dog::new(0, "测试狗".to_string(), "测试品种".to_string());
        let id = dao.save_dog(dog).unwrap();
        
        let found_dog = dao.find_dog(id).unwrap();
        assert_eq!(found_dog.name, "测试狗");
    }

    #[test]
    fn test_animal_summary() {
        let mut dao = ConcreteTableInheritanceDAO::new();
        
        let dog = Dog::new(0, "狗狗".to_string(), "品种".to_string());
        let cat = Cat::new(0, "猫咪".to_string(), "品种".to_string());
        
        dao.save_dog(dog).unwrap();
        dao.save_cat(cat).unwrap();
        
        let summary = dao.get_all_animals_summary();
        assert_eq!(summary.len(), 2);
    }
} 