use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader};
use std::collections::vec_deque::VecDeque;
use std::error::Error;
use std::ops::Deref;

trait Node {
    fn name(&self) -> &String;
    fn size(&self) -> usize;
}

struct FileNode {
    filename: String,
    filesize: usize
}

impl Node for FileNode {
    fn name(&self) -> &String {
        &self.filename
    }
    fn size(&self) -> usize {
        self.filesize
    }
}

struct DirectoryNode {
    dirname: String,
    entries: VecDeque<Box<dyn Node>>
}

impl Node for DirectoryNode {
    fn name(&self) -> &String {
        &self.dirname
    }
    fn size(&self) -> usize {
        self.entries.iter().map(|x| x.size()).sum()
    }
}

impl DirectoryNode {
    fn new (name: String) -> DirectoryNode {
        DirectoryNode {
            dirname: name,
            entries: VecDeque::new()
        }
    }
}

struct DirectoryTree {
    root: DirectoryNode
}

impl Default for DirectoryTree {
    fn default() -> Self {
        DirectoryTree {
            root: DirectoryNode::new(String::from("/"))
        }
    }
}

fn load_data(mut reader: BufReader<File>) -> Result<DirectoryTree, Box<dyn Error>> {
    let mut line = String::new();
    let tree: DirectoryTree = Default::default();
    while reader.read_line(&mut line).is_ok() {

    }
    Ok(DirectoryTree{
        root: DirectoryNode{
            dirname: "/".parse()?,
            entries: VecDeque::new()
        }
    })
}

fn main() -> Result<(), Box<dyn Error>> {
    let f = File::open("input.txt")?;
    let reader = BufReader::new(f);
    let data = load_data(reader);
    Ok(())
}
