//! 备忘录模式 (Memento Pattern)
//! 
//! 在不破坏封装性的前提下，捕获一个对象的内部状态，并在该对象之外保存这个状态。
//! 文件路径：/d%3A/workspace/RustLearn/RustDesignPattern/src/GoFDesignPattern/BehavioralPatterns/memento.rs

// 备忘录 - 保存文档状态
#[derive(Debug, Clone)]
struct DocumentMemento {
    content: String,
    cursor_position: usize,
    selection_start: usize,
    selection_end: usize,
    timestamp: String,
}

impl DocumentMemento {
    fn new(content: String, cursor_position: usize, selection_start: usize, selection_end: usize) -> Self {
        Self {
            content,
            cursor_position,
            selection_start,
            selection_end,
            timestamp: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        }
    }

    fn get_content(&self) -> &str {
        &self.content
    }

    fn get_cursor_position(&self) -> usize {
        self.cursor_position
    }

    fn get_selection(&self) -> (usize, usize) {
        (self.selection_start, self.selection_end)
    }

    fn get_timestamp(&self) -> &str {
        &self.timestamp
    }
}

// 发起人 - 文档编辑器
struct DocumentEditor {
    content: String,
    cursor_position: usize,
    selection_start: usize,
    selection_end: usize,
}

impl DocumentEditor {
    fn new() -> Self {
        Self {
            content: String::new(),
            cursor_position: 0,
            selection_start: 0,
            selection_end: 0,
        }
    }

    // 创建备忘录
    fn create_memento(&self) -> DocumentMemento {
        DocumentMemento::new(
            self.content.clone(),
            self.cursor_position,
            self.selection_start,
            self.selection_end,
        )
    }

    // 从备忘录恢复状态
    fn restore_from_memento(&mut self, memento: &DocumentMemento) {
        self.content = memento.content.clone();
        self.cursor_position = memento.cursor_position;
        self.selection_start = memento.selection_start;
        self.selection_end = memento.selection_end;
        println!("从备忘录恢复状态 (时间: {})", memento.timestamp);
    }

    // 编辑操作
    fn insert_text(&mut self, text: &str) {
        self.content.insert_str(self.cursor_position, text);
        self.cursor_position += text.len();
        println!("插入文本: '{}'", text);
    }

    fn delete_text(&mut self, length: usize) {
        let start = self.cursor_position.saturating_sub(length);
        let end = self.cursor_position;
        if start < self.content.len() && end <= self.content.len() {
            self.content.drain(start..end);
            self.cursor_position = start;
            println!("删除了 {} 个字符", length);
        }
    }

    fn set_cursor_position(&mut self, position: usize) {
        self.cursor_position = position.min(self.content.len());
        println!("光标移动到位置: {}", self.cursor_position);
    }

    fn select_text(&mut self, start: usize, end: usize) {
        self.selection_start = start.min(self.content.len());
        self.selection_end = end.min(self.content.len());
        println!("选择文本: {}..{}", self.selection_start, self.selection_end);
    }

    fn get_content(&self) -> &str {
        &self.content
    }

    fn show_status(&self) {
        println!("文档状态: 内容='{}'，光标位置={}，选择={}..{}", 
                self.content, self.cursor_position, self.selection_start, self.selection_end);
    }
}

// 管理者 - 历史记录管理器
struct HistoryManager {
    mementos: Vec<DocumentMemento>,
    current_index: Option<usize>,
    max_history: usize,
}

impl HistoryManager {
    fn new(max_history: usize) -> Self {
        Self {
            mementos: Vec::new(),
            current_index: None,
            max_history,
        }
    }

    fn save_state(&mut self, memento: DocumentMemento) {
        // 如果当前不在历史末尾，删除后面的历史
        if let Some(index) = self.current_index {
            self.mementos.truncate(index + 1);
        }

        self.mementos.push(memento);

        // 限制历史记录数量
        if self.mementos.len() > self.max_history {
            self.mementos.remove(0);
        }

        self.current_index = Some(self.mementos.len() - 1);
        println!("保存状态到历史记录 (索引: {})", self.current_index.unwrap());
    }

    fn undo(&mut self) -> Option<&DocumentMemento> {
        if let Some(index) = self.current_index {
            if index > 0 {
                self.current_index = Some(index - 1);
                println!("撤销到历史记录 (索引: {})", self.current_index.unwrap());
                return self.mementos.get(self.current_index.unwrap());
            }
        }
        println!("没有可撤销的历史记录");
        None
    }

    fn redo(&mut self) -> Option<&DocumentMemento> {
        if let Some(index) = self.current_index {
            if index + 1 < self.mementos.len() {
                self.current_index = Some(index + 1);
                println!("重做到历史记录 (索引: {})", self.current_index.unwrap());
                return self.mementos.get(self.current_index.unwrap());
            }
        }
        println!("没有可重做的历史记录");
        None
    }

    fn show_history(&self) {
        println!("历史记录:");
        for (i, memento) in self.mementos.iter().enumerate() {
            let marker = if Some(i) == self.current_index { ">>>" } else { "   " };
            println!("{} {}: '{}' (时间: {})", 
                    marker, i, memento.content, memento.timestamp);
        }
    }

    fn get_current_memento(&self) -> Option<&DocumentMemento> {
        if let Some(index) = self.current_index {
            self.mementos.get(index)
        } else {
            None
        }
    }
}

pub fn demo() {
    println!("=== 备忘录模式演示 ===");

    let mut editor = DocumentEditor::new();
    let mut history = HistoryManager::new(5);

    // 保存初始状态
    history.save_state(editor.create_memento());
    editor.show_status();

    // 执行一些编辑操作
    println!("\n执行编辑操作:");
    editor.insert_text("Hello");
    history.save_state(editor.create_memento());
    editor.show_status();

    editor.insert_text(" World");
    history.save_state(editor.create_memento());
    editor.show_status();

    editor.set_cursor_position(5);
    editor.insert_text(",");
    history.save_state(editor.create_memento());
    editor.show_status();

    editor.select_text(0, 5);
    history.save_state(editor.create_memento());
    editor.show_status();

    // 显示历史记录
    println!("\n历史记录:");
    history.show_history();

    // 撤销操作
    println!("\n撤销操作:");
    if let Some(memento) = history.undo() {
        editor.restore_from_memento(memento);
        editor.show_status();
    }

    if let Some(memento) = history.undo() {
        editor.restore_from_memento(memento);
        editor.show_status();
    }

    // 重做操作
    println!("\n重做操作:");
    if let Some(memento) = history.redo() {
        editor.restore_from_memento(memento);
        editor.show_status();
    }

    // 显示最终历史状态
    println!("\n最终历史记录:");
    history.show_history();

    println!("\n备忘录模式的优点:");
    println!("1. 提供了一种可以恢复状态的机制");
    println!("2. 实现了信息的封装，用户不需要关心状态的保存细节");
    println!("3. 简化了发起人，发起人不需要管理和保存其内部状态");
} 