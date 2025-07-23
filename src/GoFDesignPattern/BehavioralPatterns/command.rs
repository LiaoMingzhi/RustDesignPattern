//! 命令模式 (Command Pattern)
//! 
//! 将一个请求封装为一个对象，从而使你可用不同的请求对客户进行参数化，对请求排队或记录请求日志，以及支持可撤销的操作。
//! 文件路径：/d%3A/workspace/RustLearn/RustDesignPattern/src/GoFDesignPattern/BehavioralPatterns/command.rs

// 命令接口
trait Command {
    fn execute(&mut self);
    fn undo(&mut self);
    fn get_description(&self) -> String;
}

// 接收者 - 文本编辑器
struct TextEditor {
    content: String,
}

impl TextEditor {
    fn new() -> Self {
        Self {
            content: String::new(),
        }
    }

    fn add_text(&mut self, text: &str) {
        self.content.push_str(text);
        println!("添加文本: '{}'", text);
    }

    fn delete_text(&mut self, length: usize) {
        let new_len = self.content.len().saturating_sub(length);
        self.content.truncate(new_len);
        println!("删除了 {} 个字符", length);
    }

    fn get_content(&self) -> &str {
        &self.content
    }
}

// 具体命令 - 添加文本命令
struct AddTextCommand {
    editor: *mut TextEditor,
    text: String,
}

impl AddTextCommand {
    fn new(editor: &mut TextEditor, text: String) -> Self {
        Self {
            editor: editor as *mut TextEditor,
            text,
        }
    }
}

impl Command for AddTextCommand {
    fn execute(&mut self) {
        unsafe {
            (*self.editor).add_text(&self.text);
        }
    }

    fn undo(&mut self) {
        unsafe {
            (*self.editor).delete_text(self.text.len());
        }
    }

    fn get_description(&self) -> String {
        format!("添加文本: '{}'", self.text)
    }
}

// 具体命令 - 删除文本命令
struct DeleteTextCommand {
    editor: *mut TextEditor,
    deleted_text: String,
    delete_length: usize,
}

impl DeleteTextCommand {
    fn new(editor: &mut TextEditor, length: usize) -> Self {
        let deleted_text = {
            let content = &unsafe { &(*editor) }.content;
            let start = content.len().saturating_sub(length);
            content[start..].to_string()
        };
        
        Self {
            editor: editor as *mut TextEditor,
            deleted_text,
            delete_length: length,
        }
    }
}

impl Command for DeleteTextCommand {
    fn execute(&mut self) {
        unsafe {
            (*self.editor).delete_text(self.delete_length);
        }
    }

    fn undo(&mut self) {
        unsafe {
            (*self.editor).add_text(&self.deleted_text);
        }
    }

    fn get_description(&self) -> String {
        format!("删除 {} 个字符", self.delete_length)
    }
}

// 调用者 - 编辑器控制器
struct EditorController {
    history: Vec<Box<dyn Command>>,
    current_position: usize,
}

impl EditorController {
    fn new() -> Self {
        Self {
            history: Vec::new(),
            current_position: 0,
        }
    }

    fn execute_command(&mut self, mut command: Box<dyn Command>) {
        // 如果当前位置不在历史末尾，清除后面的命令
        if self.current_position < self.history.len() {
            self.history.truncate(self.current_position);
        }

        command.execute();
        self.history.push(command);
        self.current_position = self.history.len();
    }

    fn undo(&mut self) -> bool {
        if self.current_position > 0 {
            self.current_position -= 1;
            self.history[self.current_position].undo();
            println!("撤销: {}", self.history[self.current_position].get_description());
            true
        } else {
            println!("没有可撤销的操作");
            false
        }
    }

    fn redo(&mut self) -> bool {
        if self.current_position < self.history.len() {
            self.history[self.current_position].execute();
            println!("重做: {}", self.history[self.current_position].get_description());
            self.current_position += 1;
            true
        } else {
            println!("没有可重做的操作");
            false
        }
    }

    fn show_history(&self) {
        println!("命令历史 (当前位置: {}):", self.current_position);
        for (i, cmd) in self.history.iter().enumerate() {
            let marker = if i < self.current_position { "✓" } else { "○" };
            println!("  {} {}. {}", marker, i + 1, cmd.get_description());
        }
    }
}

pub fn demo() {
    println!("=== 命令模式演示 ===");

    let mut editor = TextEditor::new();
    let mut controller = EditorController::new();

    // 执行一系列命令
    println!("\n执行命令:");
    controller.execute_command(Box::new(AddTextCommand::new(&mut editor, "Hello ".to_string())));
    println!("当前内容: '{}'", editor.get_content());

    controller.execute_command(Box::new(AddTextCommand::new(&mut editor, "World!".to_string())));
    println!("当前内容: '{}'", editor.get_content());

    controller.execute_command(Box::new(DeleteTextCommand::new(&mut editor, 6)));
    println!("当前内容: '{}'", editor.get_content());

    controller.execute_command(Box::new(AddTextCommand::new(&mut editor, "Rust!".to_string())));
    println!("当前内容: '{}'", editor.get_content());

    // 显示历史
    controller.show_history();

    // 撤销操作
    println!("\n撤销操作:");
    controller.undo();
    println!("当前内容: '{}'", editor.get_content());

    controller.undo();
    println!("当前内容: '{}'", editor.get_content());

    // 重做操作
    println!("\n重做操作:");
    controller.redo();
    println!("当前内容: '{}'", editor.get_content());

    // 显示最终历史
    controller.show_history();
} 