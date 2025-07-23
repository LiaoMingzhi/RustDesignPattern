/*
 * 文件路径: /d%3A/workspace/RustLearn/RustDesignPattern/src/EnterpriseAppPattern/BasePatterns/mod.rs
 * 
 * 基础模式（Base Patterns）模块
 * 
 * 本模块包含企业应用架构中的基础模式，这些模式为其他更复杂的模式提供基础支持。
 * 
 * 包含的模式：
 * 1. Gateway（入口）- 封装对外部系统的访问
 * 2. Registry（注册表）- 提供全局对象访问点
 * 3. Value Object（值对象）- 表示不可变的值类型
 * 4. Mapper（映射器）- 对象与数据源之间的映射转换
 * 5. Money（金钱）- 货币值对象，支持多币种和精确计算
 * 
 * 这些模式是构建企业应用的基础构建块，为上层的业务逻辑模式和架构模式提供支持。
 */

//! # 基础模式 (Base Patterns)
//!
//! ## 模式分类
//! 本目录包含企业应用的基础设计模式：
//!
//! ### 1. 网关 (Gateway)
//! - **文件**: `gateway.rs`
//! - **描述**: 封装对外部系统或资源的访问
//! - **优点**: 简化外部接口，提供统一的访问方式，便于测试和替换
//! - **适用**: 访问外部服务、数据库、文件系统等资源
//!
//! ### 2. 映射器 (Mapper)
//! - **文件**: `mapper.rs`
//! - **描述**: 在两个独立的对象之间设置通信连接
//! - **优点**: 保持对象独立性，支持数据转换，便于系统集成
//! - **适用**: 对象-关系映射，系统间的数据转换
//!
//! ### 3. 注册表 (Registry)
//! - **文件**: `registry.rs`
//! - **描述**: 提供全局访问点来获取对象和服务
//! - **优点**: 避免全局变量，提供统一的对象访问，支持依赖注入
//! - **适用**: 服务定位，配置管理，依赖注入容器
//!
//! ### 4. 值对象 (Value Object)
//! - **文件**: `value_object.rs`
//! - **描述**: 代表简单概念的小型对象，相等性基于对象值而非身份
//! - **优点**: 不可变性，值语义，清晰的领域概念表达
//! - **适用**: 金额、日期范围、地址等值概念
//!
//! ### 5. 金钱 (Money)
//! - **文件**: `money.rs`
//! - **描述**: 用类型安全的方式表示货币金额
//! - **优点**: 避免货币计算错误，支持货币转换，精确的小数计算
//! - **适用**: 财务系统，电商平台，任何涉及金额计算的场景
//!
//! ### 6. 层超类型 (Layer Supertype) - 新增
//! - **描述**: 为某个层的所有类型提供一个共同的超类型
//! - **优点**: 减少代码重复，集中通用功能，强制层规范
//! - **适用**: 需要为同一层提供共同行为的场景
//!
//! ### 7. 分离接口 (Separated Interface) - 新增
//! - **描述**: 将接口定义在独立的包中，与实现分离
//! - **优点**: 减少依赖，提高可测试性，支持多种实现
//! - **适用**: 包之间的解耦，插件架构，测试替身
//!
//! ### 8. 特殊情况 (Special Case) - 新增
//! - **描述**: 为特殊情况创建专门的类，避免条件逻辑
//! - **优点**: 减少条件检查，提高代码可读性，便于扩展
//! - **适用**: 空对象模式，默认行为，异常情况处理
//!
//! ### 9. 插件 (Plugin) - 新增
//! - **描述**: 通过配置而非编程方式链接类
//! - **优点**: 运行时配置，松耦合，支持动态扩展
//! - **适用**: 插件系统，可配置的业务逻辑，扩展点
//!
//! ### 10. 服务存根 (Service Stub) - 新增
//! - **描述**: 在开发和测试时移除对有问题服务的依赖
//! - **优点**: 便于测试，开发时的服务替身，隔离外部依赖
//! - **适用**: 单元测试，集成测试，开发环境配置
//!
//! ### 11. 记录集 (Record Set) - 新增
//! - **描述**: 内存中表格数据的面向对象表示
//! - **优点**: 统一的数据访问接口，支持多种数据源，内存操作高效
//! - **适用**: 数据表示，报表生成，批量数据处理

pub mod gateway;
pub mod registry;
pub mod value_object;
pub mod mapper;
pub mod money;
pub mod layer_supertype;
pub mod separated_interface;
pub mod special_case;
pub mod plugin;

// 重新导出主要的公共接口
pub use gateway::{PaymentGateway, PaymentService, PaymentResponse, GatewayError};
pub use registry::{ServiceRegistry, ConfigRegistry, ServiceFactory, ServiceLifetime, RegistryError, global_registry, global_config};
pub use value_object::{DateRange, EmailAddress, ProductSpecification, ValueObjectError};
pub use mapper::{Mapper, MapperError, DataRow, HashMapDataRow, UserMapper, ProductMapper, OrderMapper, MapperRegistry};
pub use money::{Money, Currency, MoneyError, CurrencyConverter, MoneyBag};
pub use layer_supertype::{DomainObject, DataAccessObject, BusinessService, BusinessContext, TransactionContext, BusinessError, Product, Order, ProductDAO, ProductService};
pub use separated_interface::*;

/// 演示所有基础模式
pub fn demo_all() {
    println!("=== 基础模式总览 ===\n");
    
    println!("📋 基础模式分类表:");
    println!("┌──────────────────┬──────────────────┬──────────────────┬──────────────────┐");
    println!("│ 模式类型         │ 主要职责         │ 核心优势         │ 典型应用         │");
    println!("├──────────────────┼──────────────────┼──────────────────┼──────────────────┤");
    println!("│ 网关 (Gateway)   │ 外部系统访问     │ 接口统一         │ API调用, 数据库  │");
    println!("│ 映射器 (Mapper)  │ 对象转换映射     │ 对象独立         │ ORM, 数据转换    │");
    println!("│ 注册表 (Registry)│ 全局对象访问     │ 服务定位         │ IoC容器, 配置    │");
    println!("│ 值对象 (Value)   │ 值概念表示       │ 不可变性         │ 金额, 日期范围   │");
    println!("│ 金钱 (Money)     │ 货币安全计算     │ 类型安全         │ 财务, 电商系统   │");
    println!("└──────────────────┴──────────────────┴──────────────────┴──────────────────┘");
    
    println!("\n📋 新增模式分类表:");
    println!("┌──────────────────┬──────────────────┬──────────────────┬──────────────────┐");
    println!("│ 模式类型         │ 主要职责         │ 核心优势         │ 典型应用         │");
    println!("├──────────────────┼──────────────────┼──────────────────┼──────────────────┤");
    println!("│ 层超类型         │ 层内共同行为     │ 减少重复代码     │ 实体基类, DAO    │");
    println!("│ 分离接口         │ 接口与实现分离   │ 降低依赖         │ 插件架构, 测试   │");
    println!("│ 特殊情况         │ 特殊情况处理     │ 减少条件逻辑     │ 空对象, 默认值   │");
    println!("│ 插件 (Plugin)    │ 动态功能扩展     │ 运行时配置       │ 插件系统, 扩展   │");
    println!("│ 服务存根         │ 测试服务替身     │ 隔离外部依赖     │ 单元测试, Mock   │");
    println!("│ 记录集           │ 内存数据表示     │ 统一数据接口     │ 报表, 批处理     │");
    println!("└──────────────────┴──────────────────┴──────────────────┴──────────────────┘");
    
    println!("\n🎯 模式选择指南:");
    
    println!("\n• 外部系统集成:");
    println!("  ✅ 网关模式 - 统一外部接口访问");
    println!("  ✅ 映射器模式 - 处理数据格式转换");
    println!("  ✅ 服务存根 - 开发测试时的服务替身");
    
    println!("\n• 对象管理:");
    println!("  ✅ 注册表模式 - 全局对象访问和服务定位");
    println!("  ✅ 层超类型 - 为同一层提供共同行为");
    println!("  ✅ 分离接口 - 解耦接口定义和实现");
    
    println!("\n• 值和数据表示:");
    println!("  ✅ 值对象模式 - 表示领域中的值概念");
    println!("  ✅ 金钱模式 - 安全的货币计算");
    println!("  ✅ 记录集模式 - 内存中的表格数据");
    
    println!("\n• 特殊情况处理:");
    println!("  ✅ 特殊情况模式 - 避免大量条件判断");
    println!("  ✅ 插件模式 - 支持动态功能扩展");
    
    println!("\n💡 设计原则:");
    println!("  1. 单一职责原则 - 每个模式专注解决特定问题");
    println!("  2. 开闭原则 - 对扩展开放，对修改关闭");
    println!("  3. 依赖倒置原则 - 依赖抽象而非具体");
    println!("  4. 接口隔离原则 - 客户端不应依赖不需要的接口");
    
    println!("\n🔗 模式组合使用:");
    println!("  • 网关 + 映射器 - 外部系统集成的完整解决方案");
    println!("  • 注册表 + 分离接口 - 依赖注入和解耦");
    println!("  • 值对象 + 特殊情况 - 领域模型的完善表达");
    println!("  • 层超类型 + 插件 - 可扩展的分层架构");
    
    println!("\n{}", "=".repeat(80));
    
    // 演示网关模式
    gateway::demo();
    
    println!("\n{}", "=".repeat(80));
    
    // 演示注册表模式
    registry::demo();
    
    println!("\n{}", "=".repeat(80));
    
    // 演示值对象模式
    value_object::demo();
    
    println!("\n{}", "=".repeat(80));
    
    // 演示层超类型模式
    layer_supertype::demo();
    
    println!("\n=== 基础模式演示全部完成 ===");
    
    println!("\n🎓 学习建议:");
    println!("  1. 先掌握基础模式:");
    println!("     - 值对象：理解值语义和不可变性");
    println!("     - 网关：学习外部系统集成");
    println!("     - 映射器：掌握对象转换技术");
    
    println!("\n  2. 进阶到架构模式:");
    println!("     - 注册表：服务定位和依赖注入");
    println!("     - 层超类型：分层架构设计");
    println!("     - 分离接口：模块化和解耦");
    
    println!("\n  3. 专业化应用:");
    println!("     - 特殊情况：高质量代码设计");
    println!("     - 插件：可扩展系统架构");
    println!("     - 服务存根：测试驱动开发");
    
    println!("\n📚 相关资源:");
    println!("  • Martin Fowler《企业应用架构模式》");
    println!("  • Eric Evans《领域驱动设计》");
    println!("  • Clean Architecture principles");
    println!("  • SOLID design principles");
    
    println!("\n🚀 实践建议:");
    println!("  • 从简单项目开始应用这些模式");
    println!("  • 重点关注模式解决的问题");
    println!("  • 避免过度设计和模式滥用");
    println!("  • 结合具体场景选择合适的模式");
    println!("  • 持续重构和改进代码质量");
}