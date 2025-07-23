//! 迭代器模式 (Iterator Pattern)
//! 
//! 提供一种方法顺序访问一个聚合对象中各个元素，而又不需暴露该对象的内部表示。
//! 文件路径：/d%3A/workspace/RustLearn/RustDesignPattern/src/GoFDesignPattern/BehavioralPatterns/iterator.rs

// 图书数据结构
#[derive(Debug, Clone)]
struct Book {
    title: String,
    author: String,
    isbn: String,
}

impl Book {
    fn new(title: String, author: String, isbn: String) -> Self {
        Self { title, author, isbn }
    }
}

// 图书集合
struct BookShelf {
    books: Vec<Book>,
}

impl BookShelf {
    fn new() -> Self {
        Self { books: Vec::new() }
    }

    fn add_book(&mut self, book: Book) {
        self.books.push(book);
    }

    fn get_book_at(&self, index: usize) -> Option<&Book> {
        self.books.get(index)
    }

    fn length(&self) -> usize {
        self.books.len()
    }

    // 创建不同类型的迭代器
    fn iter(&self) -> BookIterator {
        BookIterator::new(self)
    }

    fn reverse_iter(&self) -> ReverseBookIterator {
        ReverseBookIterator::new(self)
    }

    fn author_filter_iter(&self, author: &str) -> AuthorFilterIterator {
        AuthorFilterIterator::new(self, author.to_string())
    }
}

// 基本迭代器
struct BookIterator<'a> {
    bookshelf: &'a BookShelf,
    index: usize,
}

impl<'a> BookIterator<'a> {
    fn new(bookshelf: &'a BookShelf) -> Self {
        Self { bookshelf, index: 0 }
    }
}

impl<'a> Iterator for BookIterator<'a> {
    type Item = &'a Book;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.bookshelf.length() {
            let book = self.bookshelf.get_book_at(self.index);
            self.index += 1;
            book
        } else {
            None
        }
    }
}

// 反向迭代器
struct ReverseBookIterator<'a> {
    bookshelf: &'a BookShelf,
    index: isize,
}

impl<'a> ReverseBookIterator<'a> {
    fn new(bookshelf: &'a BookShelf) -> Self {
        Self {
            bookshelf,
            index: bookshelf.length() as isize - 1,
        }
    }
}

impl<'a> Iterator for ReverseBookIterator<'a> {
    type Item = &'a Book;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= 0 {
            let book = self.bookshelf.get_book_at(self.index as usize);
            self.index -= 1;
            book
        } else {
            None
        }
    }
}

// 作者过滤迭代器
struct AuthorFilterIterator<'a> {
    bookshelf: &'a BookShelf,
    author: String,
    index: usize,
}

impl<'a> AuthorFilterIterator<'a> {
    fn new(bookshelf: &'a BookShelf, author: String) -> Self {
        Self {
            bookshelf,
            author,
            index: 0,
        }
    }
}

impl<'a> Iterator for AuthorFilterIterator<'a> {
    type Item = &'a Book;

    fn next(&mut self) -> Option<Self::Item> {
        while self.index < self.bookshelf.length() {
            if let Some(book) = self.bookshelf.get_book_at(self.index) {
                self.index += 1;
                if book.author == self.author {
                    return Some(book);
                }
            } else {
                break;
            }
        }
        None
    }
}

// 另一个例子 - 树节点迭代器
#[derive(Debug)]
struct TreeNode {
    value: i32,
    children: Vec<TreeNode>,
}

impl TreeNode {
    fn new(value: i32) -> Self {
        Self {
            value,
            children: Vec::new(),
        }
    }

    fn add_child(&mut self, child: TreeNode) {
        self.children.push(child);
    }

    // 深度优先遍历迭代器
    fn dfs_iter(&self) -> DFSIterator {
        DFSIterator::new(self)
    }
}

struct DFSIterator<'a> {
    stack: Vec<&'a TreeNode>,
}

impl<'a> DFSIterator<'a> {
    fn new(root: &'a TreeNode) -> Self {
        Self {
            stack: vec![root],
        }
    }
}

impl<'a> Iterator for DFSIterator<'a> {
    type Item = i32;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(node) = self.stack.pop() {
            // 将子节点按相反顺序压入栈，确保左边的子节点先被访问
            for child in node.children.iter().rev() {
                self.stack.push(child);
            }
            Some(node.value)
        } else {
            None
        }
    }
}

pub fn demo() {
    println!("=== 迭代器模式演示 ===");

    // 创建图书集合
    let mut bookshelf = BookShelf::new();
    bookshelf.add_book(Book::new("Rust编程".to_string(), "张三".to_string(), "978-1111111111".to_string()));
    bookshelf.add_book(Book::new("设计模式".to_string(), "李四".to_string(), "978-2222222222".to_string()));
    bookshelf.add_book(Book::new("算法导论".to_string(), "张三".to_string(), "978-3333333333".to_string()));
    bookshelf.add_book(Book::new("计算机网络".to_string(), "王五".to_string(), "978-4444444444".to_string()));

    println!("\n1. 正向迭代:");
    for book in bookshelf.iter() {
        println!("  《{}》 - 作者: {}", book.title, book.author);
    }

    println!("\n2. 反向迭代:");
    for book in bookshelf.reverse_iter() {
        println!("  《{}》 - 作者: {}", book.title, book.author);
    }

    println!("\n3. 按作者过滤 (张三):");
    for book in bookshelf.author_filter_iter("张三") {
        println!("  《{}》 - ISBN: {}", book.title, book.isbn);
    }

    println!("\n4. 树节点深度优先遍历:");
    let mut root = TreeNode::new(1);
    let mut child1 = TreeNode::new(2);
    child1.add_child(TreeNode::new(4));
    child1.add_child(TreeNode::new(5));
    
    let mut child2 = TreeNode::new(3);
    child2.add_child(TreeNode::new(6));
    
    root.add_child(child1);
    root.add_child(child2);

    print!("  DFS遍历结果: ");
    for value in root.dfs_iter() {
        print!("{} ", value);
    }
    println!();

    println!("\n5. 使用Rust标准库Iterator trait:");
    let book_titles: Vec<String> = bookshelf
        .iter()
        .map(|book| book.title.clone())
        .filter(|title| title.contains("算法"))
        .collect();
    
    println!("  包含'算法'的书籍: {:?}", book_titles);
} 