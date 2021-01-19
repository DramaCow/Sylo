use crate::lang::lex::Token;
use crate::debug::StringBuilder;

#[derive(Debug)]
pub struct CST<'a> {
    tokens: Vec<Token<'a>>,
    nodes: Vec<Node>, // first = first leaf, last = tree root
    links: Vec<Link>,
}

#[derive(Debug)]
pub enum Node {
    Leaf { index: usize },
    Branch { var: usize, head: usize },
}

#[derive(Debug)]
pub struct Link {
    index: usize,
    next: Option<usize>,
}

impl CST<'_> {
    #[must_use]
    pub fn dot(&self) -> String {
        self.dot_with_labelling(|var| var)
    }

    #[must_use]
    pub fn dot_with_labelling<F, T>(&self, labelling: F) -> String
        where F: Fn(usize) -> T,
              T: std::fmt::Display,
    {
        let mut dot = StringBuilder::new();

        dot.writeln("digraph CC {");
        dot.indent();

        // nodes
        for (id, node) in self.nodes.iter().enumerate() {
            match node {
                Node::Leaf { index } => dot.writeln(&format!("s{}[label=\"{}\", shape=none];", id, self.tokens[*index].lexeme)),
                Node::Branch { var, .. }  => dot.writeln(&format!("s{}[label=\"{}\", shape=oval];", id, labelling(*var))),
            }
        }
        dot.newline();

        // edges
        let mut stack = vec![self.nodes.len() - 1];
        while let Some(id) = stack.pop() {
            if let Node::Branch { var: _ , head } = &self.nodes[id] {
                let mut index = *head;
                loop {
                    let link = &self.links[index];
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
        for (id, node) in self.nodes.iter().enumerate() {
            if let Node::Leaf { .. } = node {
                dot.writeln(&format!("s{}; ", id))
            }
        }
        dot.unindent();
        dot.writeln("}");

        dot.unindent();
        dot.write("}");

        dot.build()
    }
}

// =================
// === INTERNALS ===
// =================

pub(crate) struct CSTBuilder {
    nodes: Vec<Node>,
    links: Vec<Link>,
    frontier: Vec<FrontierElem>,
}

enum FrontierElem {
    Node { index: usize },              // Used by non-skip variables w/ children 
    List { first: usize, last: usize }, // Used by skip variables
    Empty,                              // Any variables that contain no children are ignored
}

impl CSTBuilder {
    #[must_use]
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            links: Vec::new(),
            frontier: Vec::new(),
        }
    }

    pub fn leaf(&mut self, index: usize) {
        self.nodes.push(Node::Leaf { index });
        self.frontier.push(FrontierElem::Node { index: self.nodes.len() - 1 });
    }

    pub fn branch(&mut self, var: usize, num_children: usize) {
        if let Some((first, _)) = self.make_children_list(num_children) {
            self.nodes.push(Node::Branch { var, head: first });
            self.frontier.push(FrontierElem::Node { index: self.nodes.len() - 1 });
        } else {
            self.frontier.push(FrontierElem::Empty);
        }
    }

    pub fn list(&mut self, num_children: usize) {
        if let Some((first, last)) = self.make_children_list(num_children) {
            self.frontier.push(FrontierElem::List { first, last });
        } else {
            self.frontier.push(FrontierElem::Empty);
        }
    }

    #[must_use]
    pub fn build<'a>(self, tokens: Vec<Token<'a>>) -> CST<'a> {
        // TODO: check tree is valid before returning
        CST {
            tokens,
            nodes: self.nodes,
            links: self.links,
        }
    }

    /// If a non-empty children list was created, returns first and last link indices,
    /// else returns None.
    fn make_children_list(&mut self, num_children: usize) -> Option<(usize, usize)> {
        if num_children == 0 {
            return None;
        }
        
        let mut count = num_children;
        
        // get output link index of last non-empty child 
        let last = loop {
            match self.frontier.last().unwrap() {
                FrontierElem::Node { index: _ }       => break self.links.len(), // final link will be the next created link
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
                    self.links.push(Link { index, next });
                    next = Some(self.links.len() - 1);
                },
                FrontierElem::List { first, last } => {
                    self.links[last].next = next;
                    next = Some(first);
                },
                FrontierElem::Empty => (),
            }
        }

        Some((next.unwrap(), last))
    }
}