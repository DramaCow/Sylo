#[derive(Debug)]
pub struct CST {
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
pub struct CSTBuilder {
    nodes: Vec<Node>,
    links: Vec<Link>,
    frontier: Vec<FrontierElem>,
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
    pub fn build(self) -> CST {
        // TODO: check tree is valid before returning
        CST {
            nodes: self.nodes,
            links: self.links,
        }
    }
}

enum FrontierElem {
    Node { index: usize },              // Used by non-skip variables w/ children 
    List { first: usize, last: usize }, // Used by skip variables
    Empty,                              // Any variables that contain no children are ignored
}

impl CSTBuilder {
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