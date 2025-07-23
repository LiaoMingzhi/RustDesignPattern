//! 外观模式 (Facade Pattern)
//! 
//! 为子系统中的一组接口提供一个一致的界面，外观模式定义了一个高层接口，这个接口使得这一子系统更加容易使用。
//! 文件路径：/d%3A/workspace/RustLearn/RustDesignPattern/src/GoFDesignPattern/StructuralPatterns/facade.rs

// 子系统 - CPU
struct CPU;

impl CPU {
    fn freeze(&self) {
        println!("CPU: 冻结处理器");
    }

    fn jump(&self, position: u64) {
        println!("CPU: 跳转到位置 {}", position);
    }

    fn execute(&self) {
        println!("CPU: 执行指令");
    }
}

// 子系统 - 内存
struct Memory;

impl Memory {
    fn load(&self, position: u64, data: &str) {
        println!("内存: 从位置 {} 加载数据: {}", position, data);
    }
}

// 子系统 - 硬盘
struct HardDrive;

impl HardDrive {
    fn read(&self, lba: u64, size: u32) -> String {
        println!("硬盘: 从LBA {} 读取 {} 字节", lba, size);
        "引导数据".to_string()
    }
}

// 外观类 - 计算机
struct Computer {
    cpu: CPU,
    memory: Memory,
    hard_drive: HardDrive,
}

impl Computer {
    fn new() -> Self {
        Self {
            cpu: CPU,
            memory: Memory,
            hard_drive: HardDrive,
        }
    }

    // 简化的启动接口
    fn start(&self) {
        println!("=== 开始启动计算机 ===");
        self.cpu.freeze();
        let boot_data = self.hard_drive.read(0, 1024);
        self.memory.load(0, &boot_data);
        self.cpu.jump(0);
        self.cpu.execute();
        println!("=== 计算机启动完成 ===");
    }
}

// 另一个例子 - 家庭影院系统
struct Amplifier;
impl Amplifier {
    fn on(&self) { println!("功放: 开启"); }
    fn set_volume(&self, level: u8) { println!("功放: 设置音量到 {}", level); }
}

struct DVDPlayer;
impl DVDPlayer {
    fn on(&self) { println!("DVD播放器: 开启"); }
    fn play(&self, movie: &str) { println!("DVD播放器: 播放 {}", movie); }
}

struct Projector;
impl Projector {
    fn on(&self) { println!("投影仪: 开启"); }
    fn wide_screen_mode(&self) { println!("投影仪: 宽屏模式"); }
}

struct TheaterLights;
impl TheaterLights {
    fn dim(&self, level: u8) { println!("影院灯光: 调暗到 {}%", level); }
}

struct Screen;
impl Screen {
    fn down(&self) { println!("屏幕: 下降"); }
}

// 家庭影院外观
struct HomeTheaterFacade {
    amp: Amplifier,
    dvd: DVDPlayer,
    projector: Projector,
    lights: TheaterLights,
    screen: Screen,
}

impl HomeTheaterFacade {
    fn new() -> Self {
        Self {
            amp: Amplifier,
            dvd: DVDPlayer,
            projector: Projector,
            lights: TheaterLights,
            screen: Screen,
        }
    }

    fn watch_movie(&self, movie: &str) {
        println!("\n=== 准备观看电影: {} ===", movie);
        self.lights.dim(10);
        self.screen.down();
        self.projector.on();
        self.projector.wide_screen_mode();
        self.amp.on();
        self.amp.set_volume(5);
        self.dvd.on();
        self.dvd.play(movie);
        println!("=== 电影开始播放 ===");
    }

    fn end_movie(&self) {
        println!("\n=== 电影结束，关闭设备 ===");
        self.lights.dim(100);
        println!("所有设备已关闭");
    }
}

pub fn demo() {
    println!("=== 外观模式演示 ===");

    println!("\n1. 计算机启动外观:");
    let computer = Computer::new();
    computer.start();

    println!("\n2. 家庭影院外观:");
    let theater = HomeTheaterFacade::new();
    theater.watch_movie("阿凡达");
    theater.end_movie();

    println!("\n外观模式的优点:");
    println!("1. 简化复杂子系统的使用");
    println!("2. 降低客户端与子系统的耦合");
    println!("3. 提供统一的接口");
} 