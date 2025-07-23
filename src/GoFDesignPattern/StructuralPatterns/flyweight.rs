//! 享元模式 (Flyweight Pattern)
//! 
//! 运用共享技术有效地支持大量细粒度的对象。
//! 文件路径：/d%3A/workspace/RustLearn/RustDesignPattern/src/GoFDesignPattern/StructuralPatterns/flyweight.rs

use std::collections::HashMap;

// 享元接口
trait TreeType {
    fn render(&self, canvas: &str, x: i32, y: i32);
    fn get_info(&self) -> String;
}

// 具体享元 - 树的类型
#[derive(Debug, Clone)]
struct ConcreteTreeType {
    name: String,
    color: String,
    sprite: String, // 图像数据
}

impl ConcreteTreeType {
    fn new(name: String, color: String, sprite: String) -> Self {
        Self { name, color, sprite }
    }
}

impl TreeType for ConcreteTreeType {
    fn render(&self, canvas: &str, x: i32, y: i32) {
        println!(
            "在画布 {} 的位置 ({}, {}) 渲染 {} 的 {} 树",
            canvas, x, y, self.color, self.name
        );
    }

    fn get_info(&self) -> String {
        format!("{} {} (精灵: {})", self.color, self.name, self.sprite)
    }
}

// 享元工厂
struct TreeTypeFactory {
    tree_types: HashMap<String, Box<dyn TreeType>>,
}

impl TreeTypeFactory {
    fn new() -> Self {
        Self {
            tree_types: HashMap::new(),
        }
    }

    fn get_tree_type(&mut self, name: &str, color: &str, sprite: &str) -> &dyn TreeType {
        let key = format!("{}_{}", name, color);
        
        if !self.tree_types.contains_key(&key) {
            println!("创建新的树类型享元: {}", key);
            let tree_type = Box::new(ConcreteTreeType::new(
                name.to_string(),
                color.to_string(),
                sprite.to_string(),
            ));
            self.tree_types.insert(key.clone(), tree_type);
        } else {
            println!("复用现有的树类型享元: {}", key);
        }

        self.tree_types.get(&key).unwrap().as_ref()
    }

    fn get_created_flyweights(&self) -> usize {
        self.tree_types.len()
    }

    fn list_flyweights(&self) {
        println!("已创建的享元对象:");
        for (key, tree_type) in &self.tree_types {
            println!("  {}: {}", key, tree_type.get_info());
        }
    }
}

// 上下文 - 树对象（包含外部状态）
struct Tree {
    x: i32,
    y: i32,
    tree_type_key: String, // 存储享元的键而不是引用
}

impl Tree {
    fn new(x: i32, y: i32, tree_type_key: String) -> Self {
        Self {
            x,
            y,
            tree_type_key,
        }
    }

    fn render(&self, canvas: &str, factory: &TreeTypeFactory) {
        if let Some(tree_type) = factory.tree_types.get(&self.tree_type_key) {
            tree_type.render(canvas, self.x, self.y);
        }
    }
}

// 森林 - 上下文的集合
struct Forest {
    trees: Vec<Tree>,
    factory: TreeTypeFactory,
}

impl Forest {
    fn new() -> Self {
        Self {
            trees: Vec::new(),
            factory: TreeTypeFactory::new(),
        }
    }

    fn plant_tree(&mut self, x: i32, y: i32, name: &str, color: &str, sprite: &str) {
        let key = format!("{}_{}", name, color);
        self.factory.get_tree_type(name, color, sprite);
        let tree = Tree::new(x, y, key);
        self.trees.push(tree);
    }

    fn render(&self, canvas: &str) {
        println!("\n渲染森林 ({}棵树):", self.trees.len());
        for tree in &self.trees {
            tree.render(canvas, &self.factory);
        }
    }

    fn get_stats(&self) -> (usize, usize) {
        (self.trees.len(), self.factory.get_created_flyweights())
    }

    fn list_flyweights(&self) {
        self.factory.list_flyweights();
    }
}

pub fn demo() {
    println!("=== 享元模式演示 ===");

    let mut forest = Forest::new();

    // 种植大量树木
    println!("\n种植树木:");
    
    // 种植多棵相同类型的树
    for i in 0..5 {
        forest.plant_tree(i * 10, i * 5, "松树", "绿色", "pine_sprite.png");
    }
    
    for i in 0..3 {
        forest.plant_tree(i * 15 + 50, i * 8, "橡树", "深绿色", "oak_sprite.png");
    }
    
    for i in 0..4 {
        forest.plant_tree(i * 12 + 100, i * 6, "松树", "绿色", "pine_sprite.png");
    }

    let (total_trees, flyweights) = forest.get_stats();
    println!("\n统计信息:");
    println!("总树木数量: {}", total_trees);
    println!("享元对象数量: {}", flyweights);
    println!("内存节约: {}棵树只用了{}个享元对象!", total_trees, flyweights);

    // 显示所有享元
    forest.list_flyweights();

    // 渲染森林
    forest.render("主画布");

    println!("\n享元模式的优点:");
    println!("1. 减少内存使用，通过共享相同的内部状态");
    println!("2. 适用于需要创建大量相似对象的场景");
    println!("3. 将内部状态与外部状态分离");
} 