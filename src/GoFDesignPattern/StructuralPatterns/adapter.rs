//! 适配器模式 (Adapter Pattern)
//! 
//! 将一个类的接口转换成客户希望的另一个接口。
//! 适配器模式使得原本由于接口不兼容而不能一起工作的那些类可以一起工作。
//! 文件路径：/d%3A/workspace/RustLearn/RustDesignPattern/src/GoFDesignPattern/StructuralPatterns/adapter.rs

// 目标接口 - 媒体播放器
trait MediaPlayer {
    fn play(&self, audio_type: &str, filename: &str);
}

// 适配者接口 - 高级媒体播放器
trait AdvancedMediaPlayer {
    fn play_vlc(&self, filename: &str);
    fn play_mp4(&self, filename: &str);
}

// 具体适配者 - VLC播放器
struct VlcPlayer;

impl AdvancedMediaPlayer for VlcPlayer {
    fn play_vlc(&self, filename: &str) {
        println!("正在播放VLC格式文件: {}", filename);
    }

    fn play_mp4(&self, _filename: &str) {
        // 不支持MP4格式
    }
}

// 具体适配者 - MP4播放器
struct Mp4Player;

impl AdvancedMediaPlayer for Mp4Player {
    fn play_vlc(&self, _filename: &str) {
        // 不支持VLC格式
    }

    fn play_mp4(&self, filename: &str) {
        println!("正在播放MP4格式文件: {}", filename);
    }
}

// 适配器类
struct MediaAdapter {
    advanced_player: Box<dyn AdvancedMediaPlayer>,
}

impl MediaAdapter {
    fn new(audio_type: &str) -> Result<Self, String> {
        let advanced_player: Box<dyn AdvancedMediaPlayer> = match audio_type {
            "vlc" => Box::new(VlcPlayer),
            "mp4" => Box::new(Mp4Player),
            _ => return Err(format!("不支持的音频格式: {}", audio_type)),
        };

        Ok(Self { advanced_player })
    }
}

impl MediaPlayer for MediaAdapter {
    fn play(&self, audio_type: &str, filename: &str) {
        match audio_type {
            "vlc" => self.advanced_player.play_vlc(filename),
            "mp4" => self.advanced_player.play_mp4(filename),
            _ => println!("不支持的格式: {}", audio_type),
        }
    }
}

// 具体目标类 - 音频播放器
struct AudioPlayer {
    media_adapter: Option<MediaAdapter>,
}

impl AudioPlayer {
    fn new() -> Self {
        Self {
            media_adapter: None,
        }
    }
}

impl MediaPlayer for AudioPlayer {
    fn play(&self, audio_type: &str, filename: &str) {
        match audio_type {
            "mp3" => {
                println!("正在播放MP3格式文件: {}", filename);
            }
            "vlc" | "mp4" => {
                // 使用适配器播放其他格式
                if let Ok(adapter) = MediaAdapter::new(audio_type) {
                    adapter.play(audio_type, filename);
                } else {
                    println!("不支持的音频格式: {}", audio_type);
                }
            }
            _ => {
                println!("不支持的音频格式: {}", audio_type);
            }
        }
    }
}

// 另一个适配器示例 - 数据库适配器
// 新的目标接口
trait Database {
    fn connect(&self) -> Result<String, String>;
    fn query(&self, sql: &str) -> Result<Vec<String>, String>;
    fn close(&self);
}

// 遗留的MySQL接口
struct LegacyMySqlDatabase {
    host: String,
    port: u16,
}

impl LegacyMySqlDatabase {
    fn new(host: String, port: u16) -> Self {
        Self { host, port }
    }

    fn mysql_connect(&self) -> bool {
        println!("连接到MySQL数据库 {}:{}", self.host, self.port);
        true
    }

    fn mysql_execute(&self, command: &str) -> Vec<String> {
        println!("执行MySQL命令: {}", command);
        vec!["结果1".to_string(), "结果2".to_string()]
    }

    fn mysql_disconnect(&self) {
        println!("断开MySQL连接");
    }
}

// 遗留的PostgreSQL接口
struct LegacyPostgreSqlDatabase {
    connection_string: String,
}

impl LegacyPostgreSqlDatabase {
    fn new(connection_string: String) -> Self {
        Self { connection_string }
    }

    fn pg_connect(&self) -> i32 {
        println!("连接到PostgreSQL数据库: {}", self.connection_string);
        1 // 返回连接ID
    }

    fn pg_query(&self, sql: &str) -> Vec<String> {
        println!("执行PostgreSQL查询: {}", sql);
        vec!["PG结果1".to_string(), "PG结果2".to_string()]
    }

    fn pg_close(&self) {
        println!("关闭PostgreSQL连接");
    }
}

// MySQL适配器
struct MySqlAdapter {
    legacy_db: LegacyMySqlDatabase,
}

impl MySqlAdapter {
    fn new(host: String, port: u16) -> Self {
        Self {
            legacy_db: LegacyMySqlDatabase::new(host, port),
        }
    }
}

impl Database for MySqlAdapter {
    fn connect(&self) -> Result<String, String> {
        if self.legacy_db.mysql_connect() {
            Ok("MySQL连接成功".to_string())
        } else {
            Err("MySQL连接失败".to_string())
        }
    }

    fn query(&self, sql: &str) -> Result<Vec<String>, String> {
        let results = self.legacy_db.mysql_execute(sql);
        Ok(results)
    }

    fn close(&self) {
        self.legacy_db.mysql_disconnect();
    }
}

// PostgreSQL适配器
struct PostgreSqlAdapter {
    legacy_db: LegacyPostgreSqlDatabase,
}

impl PostgreSqlAdapter {
    fn new(connection_string: String) -> Self {
        Self {
            legacy_db: LegacyPostgreSqlDatabase::new(connection_string),
        }
    }
}

impl Database for PostgreSqlAdapter {
    fn connect(&self) -> Result<String, String> {
        let conn_id = self.legacy_db.pg_connect();
        if conn_id > 0 {
            Ok(format!("PostgreSQL连接成功，连接ID: {}", conn_id))
        } else {
            Err("PostgreSQL连接失败".to_string())
        }
    }

    fn query(&self, sql: &str) -> Result<Vec<String>, String> {
        let results = self.legacy_db.pg_query(sql);
        Ok(results)
    }

    fn close(&self) {
        self.legacy_db.pg_close();
    }
}

// 数据库管理器
struct DatabaseManager {
    databases: Vec<Box<dyn Database>>,
}

impl DatabaseManager {
    fn new() -> Self {
        Self {
            databases: Vec::new(),
        }
    }

    fn add_database(&mut self, db: Box<dyn Database>) {
        self.databases.push(db);
    }

    fn connect_all(&self) {
        println!("连接所有数据库:");
        for (i, db) in self.databases.iter().enumerate() {
            match db.connect() {
                Ok(msg) => println!("  数据库{}: {}", i + 1, msg),
                Err(e) => println!("  数据库{}: 错误 - {}", i + 1, e),
            }
        }
    }

    fn query_all(&self, sql: &str) {
        println!("\n在所有数据库执行查询: {}", sql);
        for (i, db) in self.databases.iter().enumerate() {
            match db.query(sql) {
                Ok(results) => {
                    println!("  数据库{} 结果:", i + 1);
                    for result in results {
                        println!("    - {}", result);
                    }
                }
                Err(e) => println!("  数据库{}: 查询错误 - {}", i + 1, e),
            }
        }
    }

    fn close_all(&self) {
        println!("\n关闭所有数据库连接:");
        for (i, db) in self.databases.iter().enumerate() {
            println!("  关闭数据库{}", i + 1);
            db.close();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_media_adapter() {
        let player = AudioPlayer::new();

        // 测试原生支持的格式
        player.play("mp3", "歌曲.mp3");

        // 测试通过适配器支持的格式
        player.play("vlc", "电影.vlc");
        player.play("mp4", "视频.mp4");

        // 测试不支持的格式
        player.play("avi", "文件.avi");
    }

    #[test]
    fn test_database_adapter() {
        let mut manager = DatabaseManager::new();

        // 添加不同的数据库适配器
        manager.add_database(Box::new(MySqlAdapter::new(
            "localhost".to_string(),
            3306,
        )));
        manager.add_database(Box::new(PostgreSqlAdapter::new(
            "postgresql://localhost:5432/mydb".to_string(),
        )));

        manager.connect_all();
        manager.query_all("SELECT * FROM users");
        manager.close_all();
    }
}

pub fn demo() {
    println!("=== 适配器模式演示 ===");

    println!("\n1. 媒体播放器适配器:");
    let audio_player = AudioPlayer::new();
    
    println!("播放不同格式的文件:");
    audio_player.play("mp3", "我的音乐.mp3");
    audio_player.play("vlc", "电影.vlc");
    audio_player.play("mp4", "视频.mp4");
    audio_player.play("avi", "不支持的格式.avi");

    println!("\n2. 数据库适配器:");
    let mut db_manager = DatabaseManager::new();

    // 通过适配器添加不同类型的数据库
    db_manager.add_database(Box::new(MySqlAdapter::new(
        "192.168.1.100".to_string(),
        3306,
    )));
    
    db_manager.add_database(Box::new(PostgreSqlAdapter::new(
        "postgresql://localhost:5432/testdb".to_string(),
    )));

    // 统一接口操作不同的数据库
    db_manager.connect_all();
    db_manager.query_all("SELECT name, email FROM users WHERE active = 1");
    db_manager.close_all();

    println!("\n适配器模式的优点:");
    println!("1. 使不兼容的接口能够协同工作");
    println!("2. 提高代码的复用性");
    println!("3. 将接口转换的逻辑集中在适配器中");
    println!("4. 符合开闭原则，可以在不修改现有代码的情况下引入新的适配器");
} 