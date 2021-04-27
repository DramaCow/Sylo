//! TODO

use crate::debug::StringBuilder;
use crate::lang::Parser;

/// Concrete Syntax Tree.
#[derive(Debug)]
pub struct CST<'a> {
    lexemes: Vec<&'a str>,
    nodes: Vec<Node>, // first = first leaf, last = tree root
    links: Vec<Link>,
}

pub struct CSTBuilder<'a> {
    cst: CST<'a>,
    frontier: Vec<FrontierElem>,
}

impl Default for CSTBuilder<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> CSTBuilder<'a> {
    #[must_use]
    pub fn new() -> Self {
        Self {
            cst: CST { lexemes: Vec::new(), nodes: Vec::new(), links: Vec::new() },
            frontier: Vec::new(),
        }
    }

    pub fn leaf(&mut self, word: usize, lexeme: &'a str) {
        self.cst.nodes.push(Node::Leaf { word, index: self.cst.lexemes.len() });
        self.cst.lexemes.push(lexeme);
        self.frontier.push(FrontierElem::Node { index: self.cst.nodes.len() - 1 });
    }

    pub fn branch(&mut self, var: usize, child_count: usize) {
        if let Some((first, _)) = self.make_children_list(child_count) {
            self.cst.nodes.push(Node::Branch { var, head: first });
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
}

impl CST<'_> {
    /// # Errors
    pub fn dot(&self, parser: &Parser) -> Result<String, std::fmt::Error> {
        dot_with_labelling_internal(self, |word| self.lexemes[word], |var| &parser.var_names[var])
    }
}

// =================
// === INTERNALS ===
// =================

#[derive(Debug)]
enum Node {
    Leaf { word: usize, index: usize },
    Branch { var: usize, head: usize },
}

#[derive(Debug)]
struct Link {
    index: usize,
    next: Option<usize>,
}

enum FrontierElem {
    Node { index: usize },              // Used by non-skip variables w/ children 
    List { first: usize, last: usize }, // Used by skip variables
    Empty,                              // Any variables that contain no children are ignored
}

impl CSTBuilder<'_> {
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

/// # Errors
fn dot_with_labelling_internal<F, G, T, U>(cst: &CST, word_labelling: F, var_labelling: G) -> Result<String, std::fmt::Error>
    where F: Fn(usize) -> T,
          G: Fn(usize) -> U,
          T: std::fmt::Display,
          U: std::fmt::Display,
{
    let mut dot = StringBuilder::new();

    writeln!(dot, "digraph CC {{")?;
    dot.indent();

    // nodes
    for (id, node) in cst.nodes.iter().enumerate() {
        match *node {
            Node::Leaf { word: _, index } => { writeln!(dot, "s{}[label=\"{}\", shape=none];", id, word_labelling(index))?; }
            Node::Branch { var, .. } =>  { writeln!(dot, "s{}[label=\"{}\", shape=oval];", id, var_labelling(var))?; }
        };
    }
    dot.newline();

    // edges
    let mut stack = vec![cst.nodes.len() - 1];
    while let Some(id) = stack.pop() {
        if let Node::Branch { var: _, head } = cst.nodes[id] {
            let mut index = head;
            loop {
                let link = &cst.links[index];
                writeln!(dot, "s{}->s{};", id, link.index)?;
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
    writeln!(dot, "{{")?;
    dot.indent();
    writeln!(dot, "rank=max;")?;
    dot.newline();
    for (id, node) in cst.nodes.iter().enumerate() {
        if let Node::Leaf { .. } = node {
            writeln!(dot, "s{}; ", id)?;
        }
    }

    dot.unindent();
    writeln!(dot, "}}")?;
    dot.unindent();
    write!(dot, "}}")?;

    Ok(dot.build())
}