//! TODO

use crate::lang::lex::Token;
use crate::debug::StringBuilder;
use crate::lang::Parser;

/// Concrete Syntax Tree.
#[derive(Debug)]
pub struct CST<'a> {
    tokens: Vec<Token<'a, str>>,
    nodes: Vec<CSTNode>, // first = first leaf, last = tree root
    links: Vec<Link>,
}

#[derive(Clone, Copy)]
pub struct CSTNodeId(usize);

#[derive(Debug)]
pub enum CSTNode {
    Leaf(CSTLeaf),
    Branch(CSTBranch),
}

#[derive(Debug)]
pub struct CSTLeaf {
    #[deprecated] pub word: usize,
    index: usize,
}

#[derive(Debug)]
pub struct CSTBranch {
    pub var: usize,
    head: usize,
}

pub struct CSTChildren<'a> {
    cst: &'a CST<'a>,
    next_link_index: Option<usize>,
}

impl CST<'_> {
    #[must_use]
    pub fn root(&self) -> CSTNodeId {
        CSTNodeId(self.nodes.len() - 1)
    }

    #[must_use]
    pub fn dot(&self, parser: &Parser, text: &str) -> String {
        dot_with_labelling_internal(self, |word| &text[self.tokens[word].span.clone()], |var| &parser.var_names[var])
    }
}

impl CSTNodeId {
    #[must_use]
    pub fn to_node<'a>(&self, cst: &'a CST) -> &'a CSTNode {
        &cst.nodes[self.0]
    }
}

impl CSTBranch {
    #[must_use]
    pub fn children<'a>(&self, cst: &'a CST) -> CSTChildren<'a> {
        CSTChildren { cst, next_link_index: Some(self.head) }
    }
}

impl Iterator for CSTChildren<'_> {
    type Item = CSTNodeId;
    
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(i) = self.next_link_index {
            self.next_link_index = self.cst.links[i].next;
            Some(CSTNodeId(self.cst.links[i].index))
        } else {
            None
        }
    }
}

// =================
// === INTERNALS ===
// =================

#[derive(Debug)]
struct Link {
    index: usize,
    next: Option<usize>,
}

pub(crate) struct CSTBuilder<'a> {
    cst: CST<'a>,
    frontier: Vec<FrontierElem>,
}

enum FrontierElem {
    Node { index: usize },              // Used by non-skip variables w/ children 
    List { first: usize, last: usize }, // Used by skip variables
    Empty,                              // Any variables that contain no children are ignored
}

impl<'a> CSTBuilder<'a> {
    #[must_use]
    pub fn new() -> Self {
        Self {
            cst: CST { tokens: Vec::new(), nodes: Vec::new(), links: Vec::new() },
            frontier: Vec::new(),
        }
    }

    pub fn leaf(&mut self, token: Token<'a, str>) {
        self.cst.nodes.push(CSTNode::Leaf(CSTLeaf { word: token.class, index: self.cst.tokens.len() }));
        self.cst.tokens.push(token);
        self.frontier.push(FrontierElem::Node { index: self.cst.nodes.len() - 1 });
    }

    pub fn branch(&mut self, var: usize, child_count: usize) {
        if let Some((first, _)) = self.make_children_list(child_count) {
            self.cst.nodes.push(CSTNode::Branch(CSTBranch { var, head: first }));
            self.frontier.push(FrontierElem::Node { index: self.cst.nodes.len() - 1 });
        } else {
            self.frontier.push(FrontierElem::Empty);
        }
    }

    pub fn list(&mut self, child_count: usize) {
        if let Some((first, last)) = self.make_children_list(child_count) {
            self.frontier.push(FrontierElem::List { first, last });
        } else {
            self.frontier.push(FrontierElem::Empty);
        }
    }

    #[must_use]
    pub fn build(self) -> CST<'a> {
        // TODO: check tree is valid before returning
        self.cst
    }

    /// If a non-empty children list was created, returns first and last link indices,
    /// else returns None.
    fn make_children_list(&mut self, child_count: usize) -> Option<(usize, usize)> {
        if child_count == 0 {
            return None;
        }
        
        let mut count = child_count;
        
        // get output link index of last non-empty child 
        let last = loop {
            match self.frontier.last().unwrap() {
                FrontierElem::Node { index: _ }       => break self.cst.links.len(), // final link will be the next created link
                FrontierElem::List { first: _, last } => break *last,            // final link will be poached from list
                FrontierElem::Empty => {
                    self.frontier.pop();
                    count -= 1;
                },
            };

            // All children are empty elements
            if count == 0 {
                return None;
            }
        };

        // construct linked list of children in reverse order
        let mut next: Option<usize> = None;
        for _ in 0..count {
            match self.frontier.pop().unwrap() {
                FrontierElem::Node { index } => {
                    self.cst.links.push(Link { index, next });
                    next = Some(self.cst.links.len() - 1);
                },
                FrontierElem::List { first, last } => {
                    self.cst.links[last].next = next;
                    next = Some(first);
                },
                FrontierElem::Empty => (),
            }
        }

        Some((next.unwrap(), last))
    }
}

fn dot_with_labelling_internal<F, G, T, U>(cst: &CST, word_labelling: F, var_labelling: G) -> String
    where F: Fn(usize) -> T,
          G: Fn(usize) -> U,
          T: std::fmt::Display,
          U: std::fmt::Display,
{
    let mut dot = StringBuilder::new();

    dot.writeln("digraph CC {");
    dot.indent();

    // nodes
    for (id, node) in cst.nodes.iter().enumerate() {
        match node {
            CSTNode::Leaf(leaf) => dot.writeln(&format!("s{}[label=\"{}\", shape=none];", id, word_labelling(leaf.index))),
            CSTNode::Branch(branch)  => dot.writeln(&format!("s{}[label=\"{}\", shape=oval];", id, var_labelling(branch.var))),
        }
    }
    dot.newline();

    // edges
    let mut stack = vec![cst.nodes.len() - 1];
    while let Some(id) = stack.pop() {
        if let CSTNode::Branch(branch) = &cst.nodes[id] {
            let mut index = branch.head;
            loop {
                let link = &cst.links[index];
                dot.writeln(&format!("s{}->s{};", id, link.index));
                stack.push(link.index);
                if let Some(next) = link.next {
                    index = next;
                } else {
                    break;
                }
            }
        }
    }
    dot.newline();

    // place leaves on same level
    dot.writeln("{");
    dot.indent();
    dot.writeln("rank=max;");
    dot.newline();
    for (id, node) in cst.nodes.iter().enumerate() {
        if let CSTNode::Leaf(_) = node {
            dot.writeln(&format!("s{}; ", id))
        }
    }
    dot.unindent();
    dot.writeln("}");

    dot.unindent();
    dot.write("}");

    dot.build()
}