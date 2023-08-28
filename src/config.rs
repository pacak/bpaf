//! The idea of the `Cursors` struct is to be able to keep a state for reading from multiple fields
//! that is relatively cheap to clone

use std::{cell::RefCell, collections::BTreeMap, rc::Rc};

pub trait Config {
    fn get(&self, path: &[(&'static str, usize)], name: &'static str, num: usize)
        -> Option<String>;
}

impl Config for BTreeMap<String, String> {
    fn get(
        &self,
        path: &[(&'static str, usize)],
        name: &'static str,
        num: usize,
    ) -> Option<String> {
        if path.is_empty() && num == 0 {
            self.get(name).cloned()
        } else {
            None
        }
    }
}

#[derive(Clone)]
pub(crate) struct ConfigReader {
    config: Rc<dyn Config>,
    cursors: Cursors,
    unconsumed: usize,
    path: Vec<(&'static str, usize)>,
}

impl ConfigReader {
    pub(crate) fn new(config: Rc<dyn Config + 'static>) -> Self {
        Self {
            config,
            cursors: Cursors::default(),
            unconsumed: 1000000000,
            path: Vec::new(),
        }
    }

    pub(crate) fn enter(&mut self, name: &'static str) {
        let pos = self.cursors.enter(&self.path, name);
        self.path.push((name, pos));
    }

    pub(crate) fn exit(&mut self) {
        self.path.pop();
    }
    pub(crate) fn get(&mut self, name: &'static str) -> Option<String> {
        let pos = self.cursors.get(&self.path, name);

        let v = self.config.get(&self.path, name, pos)?;
        self.unconsumed -= 1;
        Some(v)
    }
    pub(crate) fn unconsumed(&self) -> usize {
        self.unconsumed
    }
}

impl std::fmt::Debug for ConfigReader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConfigReader").finish()
    }
}

#[derive(Debug, Clone, Default)]
pub(crate) struct Cursors {
    tree: Rc<RefCell<Trie>>,
    items: Vec<usize>,
}

impl Cursors {
    pub(crate) fn enter(&mut self, path: &[(&'static str, usize)], name: &'static str) -> usize {
        let mut t = (*self.tree).borrow_mut();
        let (pos, children) = t.enter(path, name, self.items.len());
        for child in children {
            if let Some(v) = self.items.get_mut(child) {
                *v = 0;
            }
        }

        loop {
            match self.items.get_mut(pos) {
                Some(v) => {
                    let res = *v;
                    *v += 1;
                    return res;
                }
                None => self.items.push(0),
            }
        }
    }

    pub(crate) fn get(&mut self, path: &[(&'static str, usize)], name: &'static str) -> usize {
        let pos = (*self.tree)
            .borrow_mut()
            .enter(path, name, self.items.len())
            .0;

        loop {
            match self.items.get_mut(pos) {
                Some(v) => {
                    let res = *v;
                    *v += 1;
                    return res;
                }
                None => self.items.push(0),
            }
        }
    }
}

#[derive(Debug, Default)]
struct Trie {
    leaf: Option<usize>,
    children: BTreeMap<&'static str, Trie>,
}

impl Trie {
    fn enter(
        &mut self,
        path: &[(&'static str, usize)],
        name: &'static str,
        fallback: usize,
    ) -> (usize, impl Iterator<Item = usize> + '_) {
        let mut cur = self;

        for (p, _) in path {
            cur = cur.children.entry(p).or_default();
        }
        cur = cur.children.entry(name).or_default();
        if cur.leaf.is_none() {
            cur.leaf = Some(fallback);
        }
        (
            cur.leaf.unwrap(),
            cur.children.values().filter_map(|t| t.leaf),
        )
    }
}

#[test]
fn basic_cursor_logic() {
    let mut cursors = Cursors::default();
    assert_eq!(0, cursors.get(&[], "hello"));
    assert_eq!(0, cursors.get(&[], "bob"));
    assert_eq!(1, cursors.get(&[], "hello"));
    assert_eq!(2, cursors.get(&[], "hello"));
    cursors.enter(&[], "nest");
    assert_eq!(0, cursors.get(&[("nest", 0)], "hello"));
    assert_eq!(1, cursors.get(&[("nest", 0)], "hello"));
    assert_eq!(2, cursors.get(&[("nest", 0)], "hello"));
    cursors.enter(&[], "nest");
    assert_eq!(0, cursors.get(&[("nest", 1)], "hello"));
    assert_eq!(1, cursors.get(&[("nest", 1)], "hello"));
    assert_eq!(2, cursors.get(&[("nest", 1)], "hello"));
    assert_eq!(3, cursors.get(&[], "hello"));
}
