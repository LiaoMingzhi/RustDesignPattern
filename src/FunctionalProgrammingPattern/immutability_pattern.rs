// 文件路径: /d%3A/workspace/RustLearn/RustDesignPattern/src/FunctionalProgrammingPattern/immutability_pattern.rs

/*
 * 不变性模式 (Immutability Pattern)
 * 
 * 不变性是函数式编程的核心概念，指数据一旦创建就不能被修改。
 * 这种模式提供了线程安全、可预测性和无副作用的保证。
 */

use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

/// 不可变列表
#[derive(Debug, Clone)]
pub enum ImmutableList<T> {
    Empty,
    Cons(T, Rc<ImmutableList<T>>),
}

impl<T> ImmutableList<T> {
    /// 创建空列表
    pub fn empty() -> Self {
        ImmutableList::Empty
    }
    
    /// 在列表头部添加元素（返回新列表）
    pub fn cons(head: T, tail: Self) -> Self {
        ImmutableList::Cons(head, Rc::new(tail))
    }
    
    /// 从向量创建列表
    pub fn from_vec(vec: Vec<T>) -> Self {
        vec.into_iter().rev().fold(ImmutableList::empty(), |acc, x| {
            ImmutableList::cons(x, acc)
        })
    }
    
    /// 获取列表头部
    pub fn head(&self) -> Option<&T> {
        match self {
            ImmutableList::Cons(head, _) => Some(head),
            ImmutableList::Empty => None,
        }
    }
    
    /// 获取列表尾部
    pub fn tail(&self) -> Option<&ImmutableList<T>> {
        match self {
            ImmutableList::Cons(_, tail) => Some(tail),
            ImmutableList::Empty => None,
        }
    }
    
    /// 列表长度
    pub fn len(&self) -> usize {
        match self {
            ImmutableList::Empty => 0,
            ImmutableList::Cons(_, tail) => 1 + tail.len(),
        }
    }
    
    /// 映射操作（返回新列表）
    pub fn map<U, F>(&self, f: F) -> ImmutableList<U>
    where
        F: Fn(&T) -> U + Clone,
    {
        match self {
            ImmutableList::Empty => ImmutableList::Empty,
            ImmutableList::Cons(head, tail) => {
                ImmutableList::cons(f(head), tail.map(f))
            }
        }
    }
    
    /// 过滤操作（返回新列表）
    pub fn filter<F>(&self, predicate: F) -> ImmutableList<T>
    where
        F: Fn(&T) -> bool + Clone,
        T: Clone,
    {
        match self {
            ImmutableList::Empty => ImmutableList::Empty,
            ImmutableList::Cons(head, tail) => {
                let filtered_tail = tail.filter(predicate.clone());
                if predicate(head) {
                    ImmutableList::cons(head.clone(), filtered_tail)
                } else {
                    filtered_tail
                }
            }
        }
    }
}

/// 不可变映射
#[derive(Debug, Clone)]
pub struct ImmutableMap<K, V>
where
    K: Clone + Eq + std::hash::Hash,
    V: Clone,
{
    data: Rc<HashMap<K, V>>,
}

impl<K, V> ImmutableMap<K, V>
where
    K: Clone + Eq + std::hash::Hash,
    V: Clone,
{
    /// 创建空映射
    pub fn new() -> Self {
        Self {
            data: Rc::new(HashMap::new()),
        }
    }
    
    /// 插入键值对（返回新映射）
    pub fn insert(&self, key: K, value: V) -> Self {
        let mut new_data = (*self.data).clone();
        new_data.insert(key, value);
        Self {
            data: Rc::new(new_data),
        }
    }
    
    /// 删除键（返回新映射）
    pub fn remove(&self, key: &K) -> Self {
        let mut new_data = (*self.data).clone();
        new_data.remove(key);
        Self {
            data: Rc::new(new_data),
        }
    }
    
    /// 获取值
    pub fn get(&self, key: &K) -> Option<&V> {
        self.data.get(key)
    }
    
    /// 检查是否包含键
    pub fn contains_key(&self, key: &K) -> bool {
        self.data.contains_key(key)
    }
    
    /// 映射操作（返回新映射）
    pub fn map_values<U, F>(&self, f: F) -> ImmutableMap<K, U>
    where
        U: Clone,
        F: Fn(&V) -> U,
    {
        let mut new_data = HashMap::new();
        for (k, v) in self.data.iter() {
            new_data.insert(k.clone(), f(v));
        }
        ImmutableMap {
            data: Rc::new(new_data),
        }
    }
}

/// 不可变记录
#[derive(Debug, Clone)]
pub struct Person {
    name: String,
    age: u32,
    email: String,
}

impl Person {
    pub fn new(name: String, age: u32, email: String) -> Self {
        Self { name, age, email }
    }
    
    /// 更新姓名（返回新实例）
    pub fn with_name(&self, name: String) -> Self {
        Self {
            name,
            age: self.age,
            email: self.email.clone(),
        }
    }
    
    /// 更新年龄（返回新实例）
    pub fn with_age(&self, age: u32) -> Self {
        Self {
            name: self.name.clone(),
            age,
            email: self.email.clone(),
        }
    }
    
    /// 更新邮箱（返回新实例）
    pub fn with_email(&self, email: String) -> Self {
        Self {
            name: self.name.clone(),
            age: self.age,
            email,
        }
    }
    
    // 访问器方法
    pub fn name(&self) -> &str {
        &self.name
    }
    
    pub fn age(&self) -> u32 {
        self.age
    }
    
    pub fn email(&self) -> &str {
        &self.email
    }
}

/// 构建器模式与不变性
pub struct PersonBuilder {
    name: Option<String>,
    age: Option<u32>,
    email: Option<String>,
}

impl PersonBuilder {
    pub fn new() -> Self {
        Self {
            name: None,
            age: None,
            email: None,
        }
    }
    
    pub fn name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }
    
    pub fn age(mut self, age: u32) -> Self {
        self.age = Some(age);
        self
    }
    
    pub fn email(mut self, email: String) -> Self {
        self.email = Some(email);
        self
    }
    
    pub fn build(self) -> Result<Person, String> {
        match (self.name, self.age, self.email) {
            (Some(name), Some(age), Some(email)) => {
                Ok(Person::new(name, age, email))
            }
            _ => Err("缺少必要字段".to_string()),
        }
    }
}

/// 不可变状态机
#[derive(Debug, Clone, PartialEq)]
pub enum State {
    Idle,
    Processing,
    Complete,
    Error(String),
}

#[derive(Debug, Clone)]
pub struct StateMachine {
    current_state: State,
    history: ImmutableList<State>,
}

impl StateMachine {
    pub fn new() -> Self {
        Self {
            current_state: State::Idle,
            history: ImmutableList::empty(),
        }
    }
    
    /// 状态转换（返回新状态机）
    pub fn transition(&self, new_state: State) -> Self {
        Self {
            current_state: new_state.clone(),
            history: ImmutableList::cons(self.current_state.clone(), self.history.clone()),
        }
    }
    
    pub fn current_state(&self) -> &State {
        &self.current_state
    }
    
    pub fn history_length(&self) -> usize {
        self.history.len()
    }
}

/// 函数式数据转换
pub struct DataTransformer;

impl DataTransformer {
    /// 不可变数据管道
    pub fn transform_numbers(numbers: Vec<i32>) -> Vec<String> {
        numbers
            .into_iter()
            .filter(|&x| x > 0)           // 过滤正数
            .map(|x| x * 2)               // 翻倍
            .filter(|&x| x < 100)         // 过滤小于100的数
            .map(|x| format!("值: {}", x)) // 格式化
            .collect()
    }
    
    /// 不可变聚合操作
    pub fn aggregate_data(data: Vec<(String, i32)>) -> HashMap<String, Vec<i32>> {
        data.into_iter()
            .fold(HashMap::new(), |mut acc, (key, value)| {
                acc.entry(key).or_insert_with(Vec::new).push(value);
                acc
            })
    }
}

/// 不变性模式演示
pub fn demo_immutability_pattern() {
    println!("=== 不变性模式演示 ===");
    
    // 1. 不可变列表
    println!("1. 不可变列表:");
    let list1 = ImmutableList::from_vec(vec![1, 2, 3, 4, 5]);
    println!("原始列表长度: {}", list1.len());
    
    let list2 = list1.map(|&x| x * 2);
    println!("映射后列表头部: {:?}", list2.head());
    
    let list3 = list1.filter(|&x| x % 2 == 0);
    println!("过滤后列表长度: {}", list3.len());
    
    // 2. 不可变映射
    println!("\n2. 不可变映射:");
    let map1 = ImmutableMap::new();
    let map2 = map1.insert("键1".to_string(), 10);
    let map3 = map2.insert("键2".to_string(), 20);
    
    println!("map1 包含 '键1': {}", map1.contains_key(&"键1".to_string()));
    println!("map3 包含 '键1': {}", map3.contains_key(&"键1".to_string()));
    println!("map3 的 '键2' 值: {:?}", map3.get(&"键2".to_string()));
    
    // 3. 不可变记录
    println!("\n3. 不可变记录:");
    let person1 = Person::new("张三".to_string(), 25, "zhang@example.com".to_string());
    println!("原始人员: {} - {} 岁", person1.name(), person1.age());
    
    let person2 = person1.with_age(26);
    println!("更新年龄后: {} - {} 岁", person2.name(), person2.age());
    println!("原始人员不变: {} - {} 岁", person1.name(), person1.age());
    
    // 4. 构建器模式
    println!("\n4. 构建器模式:");
    let person3 = PersonBuilder::new()
        .name("李四".to_string())
        .age(30)
        .email("li@example.com".to_string())
        .build()
        .unwrap();
    
    println!("构建的人员: {} - {}", person3.name(), person3.email());
    
    // 5. 不可变状态机
    println!("\n5. 不可变状态机:");
    let sm1 = StateMachine::new();
    println!("初始状态: {:?}", sm1.current_state());
    
    let sm2 = sm1.transition(State::Processing);
    let sm3 = sm2.transition(State::Complete);
    
    println!("最终状态: {:?}", sm3.current_state());
    println!("历史长度: {}", sm3.history_length());
    println!("原始状态机不变: {:?}", sm1.current_state());
    
    // 6. 函数式数据转换
    println!("\n6. 函数式数据转换:");
    let numbers = vec![-1, 2, 50, 3, 100, 4];
    let transformed = DataTransformer::transform_numbers(numbers.clone());
    println!("原始数据: {:?}", numbers);
    println!("转换结果: {:?}", transformed);
    
    println!("\n【不变性模式特点】");
    println!("✓ 线程安全 - 不可变数据天然线程安全");
    println!("✓ 可预测性 - 数据不会意外改变");
    println!("✓ 无副作用 - 函数不会修改输入数据");
    println!("✓ 历史追踪 - 可以保留所有历史版本");
    println!("✓ 函数式编程 - 支持纯函数式操作");
} 