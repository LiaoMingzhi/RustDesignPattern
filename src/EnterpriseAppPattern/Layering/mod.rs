/*
 * 文件路径: /d%3A/workspace/RustLearn/RustDesignPattern/src/EnterpriseAppPattern/Layering/mod.rs
 * 
 * 分层架构（Layering）模块
 * 
 * 本模块包含企业应用分层架构的核心模式，展示了如何将应用程序组织成不同的层次，
 * 每一层都有明确的职责和依赖关系。
 * 
 * 包含的模式：
 * 1. Presentation Layer（表现层）- 处理用户交互和数据展示
 * 2. Business Layer（业务层）- 实现业务逻辑和业务规则
 * 
 * 分层架构的核心原则：
 * - 每一层只能依赖于它下面的层
 * - 上层通过接口访问下层
 * - 每一层都有明确的职责边界
 * - 层间通过数据传输对象（DTO）进行通信
 */

pub mod presentation_layer;
pub mod business_layer;

// 重新导出主要的公共接口
pub use presentation_layer::{
    HttpRequest, HttpResponse,
    UserController, PresentationError
};
pub use business_layer::{
    User, Order, Product, OrderItem, UserStatus, OrderStatus,
    UserBusinessService, OrderBusinessService, BusinessProcessCoordinator,
    BusinessError, BusinessResult, BusinessRuleEngine
};

/// 演示所有分层架构模式
pub fn demo_all() {
    println!("=== 企业应用分层架构模式演示 ===\n");
    
    println!("【分层架构说明】");
    println!("分层架构是企业应用中最常用的架构模式，它将应用程序组织成不同的层次：");
    println!("1. 表现层（Presentation Layer）：负责用户交互和数据展示");
    println!("2. 业务层（Business Layer）：实现业务逻辑和业务规则");
    println!("3. 数据访问层（Data Access Layer）：负责数据持久化和访问");
    println!();
    
    println!("分层架构的优势：");
    println!("- 职责分离：每一层都有明确的职责");
    println!("- 可维护性：变化通常局限在一个层内");
    println!("- 可测试性：可以独立测试每一层");
    println!("- 可重用性：下层可以被多个上层使用");
    println!();
    
    // 演示表现层
    println!("=== 表现层演示 ===");
    presentation_layer::demo();
    
    println!("\n{}", "=".repeat(80));
    
    // 演示业务层
    println!("=== 业务层演示 ===");
    business_layer::demo();
    
    println!("\n=== 分层架构演示完成 ===");
    
    println!("\n【分层架构最佳实践】");
    println!("1. 依赖原则：");
    println!("   - 上层可以依赖下层，但下层不能依赖上层");
    println!("   - 使用依赖倒置原则，依赖于抽象而不是具体实现");
    
    println!("\n2. 数据传递：");
    println!("   - 使用DTO（数据传输对象）在层间传递数据");
    println!("   - 避免在层间传递领域对象");
    println!("   - 每一层都有自己的数据模型");
    
    println!("\n3. 错误处理：");
    println!("   - 每一层都有自己的错误类型");
    println!("   - 底层错误需要转换为上层理解的错误");
    println!("   - 不要让底层的技术异常泄露到上层");
    
    println!("\n4. 事务管理：");
    println!("   - 业务层负责定义事务边界");
    println!("   - 表现层不应该直接管理事务");
    println!("   - 使用声明式事务管理");
    
    println!("\n5. 性能考虑：");
    println!("   - 避免过度的层间调用");
    println!("   - 合理使用缓存");
    println!("   - 考虑批量操作以减少层间交互");
    
    println!("\n【应用场景】");
    println!("分层架构适用于：");
    println!("- 传统的企业级应用");
    println!("- 具有复杂业务逻辑的系统");
    println!("- 需要多种客户端接口的应用");
    println!("- 团队规模较大的项目");
    println!("- 需要长期维护的系统");
} 