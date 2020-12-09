use super::parse::ParseStep;

#[derive(Debug)]
pub struct ParseTree {
    pub nodes: Vec<Node>, // first = first leaf, last = tree root
    pub links: Vec<Link>,
}

#[derive(Debug)]
pub enum Node {
    Leaf { word: usize, index: usize },
    Branch { var: usize, head: usize },
}

#[derive(Debug)]
pub struct Link {
    pub index: usize,
    pub next: Option<usize>,
}

// =================
// === INTERNALS ===
// =================

#[derive(Default)]
pub struct ParseTreeBuilder {
    nodes: Vec<Node>,
    links: Vec<Link>,
    frontier: Vec<FrontierElem>,
}

impl ParseTreeBuilder {
    #[must_use]
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            links: Vec::new(),
            frontier: Vec::new(),
        }
    }

    pub fn leaf(&mut self, word: usize, index: usize) {
        self.nodes.push(Node::Leaf { word, index });
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
    pub fn build(self) -> ParseTree {
        // TODO: check tree is valid before returning
        ParseTree {
            nodes: self.nodes,
            links: self.links,
        }
    }
}

enum FrontierElem {
    Node { index: usize },              // Used by non-propagating variables w/ children 
    List { first: usize, last: usize }, // Used by propagating variables
    Empty,                              // Used by variables w/o children
}

impl ParseTreeBuilder {
    /// If a children list was successfully created, returns first and last link indices,
    /// else returns None.
    fn make_children_list(&mut self, num_children: usize) -> Option<(usize, usize)> {
        if num_children == 0 {
            return None;
        }
        
        let mut i = 0_usize;
        
        // tail = last out of non-empty child 
        let tail = loop {
            match self.frontier.last().unwrap() {
                FrontierElem::Node { index: _ }       => break self.links.len(),
                FrontierElem::List { first: _, last } => break *last,
                FrontierElem::Empty => {
                    self.frontier.pop();
                    i += 1;
                    // All children are skip elements
                    if i >= num_children {
                        return None;
                    }
                },
            };
        };

        let mut next: Option<usize> = None;

        for _ in i..num_children {
            let frontier_elem = self.frontier.pop().unwrap();

            match frontier_elem {
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

        Some((next.unwrap(), tail))
    }
}