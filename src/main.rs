extern crate core;

use std::borrow::{Borrow, BorrowMut};
use std::cell::RefCell;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::vec_deque::VecDeque;
use std::error::Error;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;

use lazy_regex::*;
use downcast_rs::*;

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

    match RefCell::borrow(&current).deref() {
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
    }

    // unsure what behavior we need. will err for now
    Err("Not found")
    // let newdir = DirectoryNode::new(dirname.into_string());
    // current.entries.push_back(newdir);
    // return Ok(&newdir);
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

fn load_data(mut reader: BufReader<File>) -> Result<DirectoryTree, Box<dyn Error>> {
    let mut t: DirectoryTree = Default::default();
    let mut dir_stack: VecDeque<NodeRef> = VecDeque::new();
    dir_stack.push_back( t.root.clone());
    let mut line = String::new();
    while reader.read_line(&mut line).is_ok() {
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
    let data = load_data(reader);
    Ok(())
}
