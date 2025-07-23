/*
 * 文件路径: /d%3A/workspace/RustLearn/RustDesignPattern/src/FunctionalProgrammingPattern/lazy_evaluation.rs
 * 
 * 惰性求值模式 (Lazy Evaluation Pattern)
 * 
 * 惰性求值是一种计算策略，其中表达式的求值被延迟到实际需要其值的时候。
 * 这种模式可以提高程序的性能，特别是在处理大型数据集或昂贵的计算时，
 * 因为它避免了不必要的计算，并支持无限数据结构的处理。
 * 
 * 主要特点：
 * 1. 按需计算 - 只在需要时才进行实际计算
 * 2. 性能优化 - 避免不必要的计算开销
 * 3. 内存效率 - 不会一次性加载所有数据到内存
 * 4. 无限数据结构 - 可以处理理论上无限的序列
 * 5. 缓存机制 - 计算结果被缓存，避免重复计算
 * 
 * 使用场景：
 * - 昂贵的计算：数据库查询、复杂算法等
 * - 大数据处理：流式处理大型数据集
 * - 无限序列：生成器、迭代器等
 * - 条件计算：只在特定条件下才需要的计算
 * 
 * 实现说明：
 * - Lazy<T>：通用的惰性值容器，支持任意类型
 * - LazyRange：惰性范围生成器，演示无限序列概念
 * - 使用RefCell和Option实现内部可变性
 * - 提供强制求值和状态检查接口
 * 
 * 注意事项：
 * - 惰性求值可能导致难以预测的性能特征
 * - 在多线程环境中需要考虑线程安全问题
 * - 调试惰性计算可能比较困难，因为执行顺序不确定
 * - 在Rust中使用了unsafe代码，实际项目中应考虑更安全的替代方案
 */

use std::cell::RefCell;

/// 惰性值容器
pub struct Lazy<T> {
    value: RefCell<Option<T>>,
    computation: RefCell<Option<Box<dyn FnOnce() -> T>>>,
}

impl<T> Lazy<T> {
    /// 创建新的惰性值
    pub fn new<F>(computation: F) -> Self
    where
        F: FnOnce() -> T + 'static,
    {
        Self {
            value: RefCell::new(None),
            computation: RefCell::new(Some(Box::new(computation))),
        }
    }
    
    /// 强制求值
    pub fn force(&self) -> &T {
        if self.value.borrow().is_none() {
            if let Some(computation) = self.computation.borrow_mut().take() {
                let result = computation();
                *self.value.borrow_mut() = Some(result);
            }
        }
        
        // 这里使用unsafe是为了返回引用，实际使用中可以考虑其他方案
        unsafe {
            &*self.value.as_ptr().cast::<T>()
        }
    }
    
    /// 检查是否已计算
    pub fn is_computed(&self) -> bool {
        self.value.borrow().is_some()
    }
}

/// 惰性范围生成器
pub struct LazyRange {
    start: i32,
    step: i32,
    count: usize,
}

impl LazyRange {
    pub fn new(start: i32, step: i32, count: usize) -> Self {
        Self { start, step, count }
    }
    
    /// 获取指定索引的值
    pub fn get(&self, index: usize) -> Option<i32> {
        if index < self.count {
            Some(self.start + (index as i32) * self.step)
        } else {
            None
        }
    }
    
    /// 转换为向量
    pub fn to_vec(&self) -> Vec<i32> {
        (0..self.count)
            .map(|i| self.get(i).unwrap())
            .collect()
    }
}

/// 惰性求值演示
pub fn demo_lazy_evaluation() {
    println!("=== 惰性求值模式演示 ===");
    
    // 基础惰性值
    let lazy_value = Lazy::new(|| {
        println!("计算昂贵操作...");
        std::thread::sleep(std::time::Duration::from_millis(100));
        42
    });
    
    println!("惰性值已创建，但尚未计算");
    println!("是否已计算: {}", lazy_value.is_computed());
    
    println!("强制求值: {}", lazy_value.force());
    println!("是否已计算: {}", lazy_value.is_computed());
    
    println!("再次获取值: {}", lazy_value.force()); // 不会重新计算
    
    // 惰性范围
    let range = LazyRange::new(1, 2, 10);
    println!("\n惰性范围 (1, 步长2, 10个):");
    println!("第5个元素: {:?}", range.get(4));
    println!("前5个元素: {:?}", range.to_vec().into_iter().take(5).collect::<Vec<_>>());
    
    println!("\n【惰性求值模式特点】");
    println!("✓ 按需计算 - 只在需要时才进行计算");
    println!("✓ 性能优化 - 避免不必要的计算开销");
    println!("✓ 无限数据结构 - 可以处理无限序列");
} 