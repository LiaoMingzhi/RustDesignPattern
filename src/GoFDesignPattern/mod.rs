//! GoF设计模式模块
//! 
//! 包含《设计模式：可复用面向对象软件的基础》（Gang of Four）一书中定义的23种设计模式的Rust实现。
//! 
//! 分为三大类：
//! - 创建型模式（5种）：关注对象的创建
//! - 结构型模式（7种）：关注类和对象的组合
//! - 行为型模式（11种）：关注对象间的通信

pub mod CreationalPatterns;
pub mod StructuralPatterns;
pub mod BehavioralPatterns;

pub fn run_all_patterns() {
    println!("🦀 Rust设计模式演示程序");
    println!("本程序演示了GoF 23种设计模式在Rust中的实现\n");

    // 运行所有创建型模式
    CreationalPatterns::run_all_demos();

    // 运行所有结构型模式
    StructuralPatterns::run_all_demos();

    // 运行所有行为型模式
    BehavioralPatterns::run_all_demos();

    print_summary();
}

fn print_summary() {
    println!("🎉 所有23种GoF设计模式演示完成！");
    println!("\n📋 模式总结：");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    println!("\n🏗️  创建型模式（5种）：");
    println!("   1. 抽象工厂模式 - 创建相关对象族");
    println!("   2. 建造者模式 - 分步骤构建复杂对象");
    println!("   3. 工厂方法模式 - 创建对象的统一接口");
    println!("   4. 原型模式 - 通过复制创建对象");
    println!("   5. 单例模式 - 确保只有一个实例");

    println!("\n🏛️  结构型模式（7种）：");
    println!("   6. 适配器模式 - 接口转换");
    println!("   7. 桥接模式 - 抽象与实现分离");
    println!("   8. 组合模式 - 部分-整体层次结构");
    println!("   9. 装饰器模式 - 动态添加功能");
    println!("  10. 外观模式 - 简化复杂子系统");
    println!("  11. 享元模式 - 共享细粒度对象");
    println!("  12. 代理模式 - 控制对象访问");

    println!("\n🎭 行为型模式（11种）：");
    println!("  13. 责任链模式 - 请求沿链传递");
    println!("  14. 命令模式 - 请求封装为对象");
    println!("  15. 解释器模式 - 语言解释器");
    println!("  16. 迭代器模式 - 顺序访问元素");
    println!("  17. 中介者模式 - 对象交互的中介");
    println!("  18. 备忘录模式 - 保存和恢复状态");
    println!("  19. 观察者模式 - 一对多依赖通知");
    println!("  20. 状态模式 - 状态改变行为");
    println!("  21. 策略模式 - 算法可替换");
    println!("  22. 模板方法模式 - 算法骨架定义");
    println!("  23. 访问者模式 - 新操作与结构分离");

    println!("\n💡 设计模式的核心原则：");
    println!("   • 开闭原则：对扩展开放，对修改关闭");
    println!("   • 单一职责原则：一个类只有一个引起变化的原因");
    println!("   • 依赖倒置原则：依赖抽象而不是具体实现");
    println!("   • 接口隔离原则：接口应该细化和专一");
    println!("   • 里氏替换原则：子类可以替换父类");
    println!("   • 迪米特法则：最少知识原则");

    println!("\n🚀 Rust特色：");
    println!("   • 内存安全：无垃圾回收的内存安全");
    println!("   • 线程安全：编译时防止数据竞争");
    println!("   • 零成本抽象：抽象不带来运行时开销");
    println!("   • Trait系统：灵活的类型系统");
    
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("感谢学习Rust设计模式！🦀✨");
} 