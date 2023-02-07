extern crate core;

use std::borrow::{Borrow, BorrowMut};
use std::cell::RefCell;
use std::fs::File;
use std::io::{BufRead, BufReader, Result as IoResult};
use std::collections::vec_deque::VecDeque;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::ops::{Deref, DerefMut};
use std::rc::Rc;

use lazy_regex::*;

enum Node{
    FileNode {
        filename: String,
        filesize: usize,
    },
    DirectoryNode {
        dirname: String,
        entries: VecDeque<NodeRef>,
    }
}

type NodeRef = Rc<RefCell<Node>>;

fn get_name(n: &Node) -> &String {
    match n {
        Node::FileNode{filename: f, .. } => f,
        Node::DirectoryNode{dirname: d, ..} => d,
    }
}

fn get_size(n: NodeRef) -> usize {
    let node = RefCell::borrow(&n);
    match node.deref() {
        Node::FileNode {filesize: s, ..} => *s,
        Node::DirectoryNode {entries, ..} =>
            entries.iter().map(|x| get_size(x.clone())).sum(),
    }
}

struct DirectoryTree {
    root: NodeRef
}

fn make_node_ref(n: Node) -> NodeRef {
    Rc::new(RefCell::new(n))
}

impl Default for DirectoryTree {
    fn default() -> Self {
        DirectoryTree {
            root: make_node_ref(
                Node::DirectoryNode {
                    dirname: String::from("/"),
                    entries: VecDeque::new()
                }
            )
        }
    }
}

impl Node {
    fn _fmt_rec(&self, f: &mut Formatter<'_>, depth: usize) -> std::fmt::Result {
        let padding = " ".repeat(depth);
        match self {
            Node::FileNode { filesize, filename } => {
                return write!(f, "{padding}{filesize}  {filename}");
            }
            Node::DirectoryNode { dirname, entries } => {
                write!(f, "{padding}{dirname} {}\n", entries.len())?;
                for entry in entries.iter() {
                    RefCell::borrow(entry)._fmt_rec(f, depth+2)?;
                }
                return Ok(());
            }
        }
    }
}

impl Display for Node {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        return self._fmt_rec(f, 0);
    }
}

fn execute_cd<'a> (
    dir_stack: &'a mut VecDeque<NodeRef>,
    dirname: &str
) -> Result<(), &'static str> {
    if "/" == dirname {
        let root = dir_stack.pop_front().unwrap();
        dir_stack.clear();
        dir_stack.push_back(root);
        return Ok(())
    }
    let current = dir_stack[dir_stack.len()-1].clone();

    let x = match RefCell::borrow(&current).deref() {
        Node::DirectoryNode { entries, ..} => {
            for entry in entries.iter() {
                match RefCell::borrow(entry).deref() {
                    Node::FileNode { .. } => continue,
                    Node::DirectoryNode { dirname: name, .. } => {
                        if name == dirname {
                            dir_stack.push_back(entry.clone());
                            return Ok(())
                        }
                    }
                }
            };
            return Err("Couldn't find directory")
        }
        _ => panic!()
    }; x
}

fn execute_dirent(mut current: NodeRef, fst: &str, snd: &str) -> Result<(), &'static str> {
    match RefCell::borrow_mut(&current).deref_mut() {
        Node::FileNode {..} => Err("Can't ls a file"),
        Node::DirectoryNode {
            entries, ..
        } => if fst == "dir" {
            entries.push_back(make_node_ref(
                Node::DirectoryNode {
                    dirname: String::from(snd),
                    entries: VecDeque::new()
                }
            ));
            Ok(())
        }
        else {
            let p: usize = fst.parse::<usize>().or(Err("Invalid looking string"))?;
            entries.push_back(make_node_ref(
                Node::FileNode {
                    filename: String::from(snd),
                    filesize: p
                }
            ));
            Ok(())
        }
    }

}

fn load_data(mut reader: BufReader<File>) -> Result<DirectoryTree, &'static str> {
    let mut t: DirectoryTree = Default::default();
    let mut dir_stack: VecDeque<NodeRef> = VecDeque::new();
    dir_stack.push_back( t.root.clone());
    let mut line = String::new();
    let mut i = 0;
    loop {
        let res = reader.read_line(&mut line);
        if res.is_err() {
            return Err("Things went south");
        }
        if res.unwrap() == 0 { // EOF
            break;
        }
        let cd_cap = regex_captures!(r#"^\$ cd (.+)$"#, &line);
        if cd_cap.is_some() {
            let (_, dirname) = cd_cap.unwrap();
            execute_cd(&mut dir_stack, dirname)?;
            continue
        }
        let dir_cap = regex_captures!(r#"^\$ ls$"#, &line);
        if dir_cap.is_some() {
            continue
        }
        let dirent_cap = regex_captures!(r#"^(dir|\d+) (.*)$"#, &line);
        if dirent_cap.is_some() {
            let (_, fst, snd) = dirent_cap.unwrap();
            let current = dir_stack.get(0).unwrap();
            execute_dirent(current.clone(), fst, snd)?;
            continue
        }
    }
    Ok(t)
}

fn main() -> Result<(), Box<dyn Error>> {
    let f = File::open("input.txt")?;
    let reader = BufReader::new(f);
    let data = load_data(reader)?;
    println!("{}", RefCell::borrow(&data.root));
    Ok(())
}
