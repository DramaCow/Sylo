use std::ops::Deref;
use std::collections::{HashSet, HashMap, BTreeMap};
use std::iter::once;

use super::{CharSet, RegEx, RENode, RENodeRef};
use crate::debug::StringBuilder;

pub struct DFA {
    states: Vec<DFAState>,
}

pub struct DFAState {
    pub class: Option<usize>,
    pub next: HashMap<u8, usize>,
    _private: (),
}

impl From<&RegEx> for DFA {
    fn from(regex: &RegEx) -> Self {
        DFABuilder::build(&RENodeRefVec::new(vec![regex.root.clone()]))
    }
}

impl<'a, T> From<T> for DFA
where
    T: IntoIterator<Item = &'a RegEx>,
{
    fn from(regexes: T) -> Self {
        DFABuilder::build(&RENodeRefVec::new(regexes.into_iter().map(|regex| regex.root.clone()).collect()))
     }
}

impl DFA {
    /// Constructs the equivalent, minimized DFA via Hopcroft's algorithm.
    #[must_use]
    pub fn minimize(&self) -> Self {
        hopcroft::minimize(self)
    }

    #[must_use]
    pub fn matches(&self, text: &str) -> bool {
        // Note: start index is always 1.
        self.class(text.bytes().fold(1, |id, byte| { self.step(id, byte) })).is_some()
    }

    #[must_use]
    pub fn step(&self, id: usize, symbol: u8) -> usize {
        match self.states[id].next.get(&symbol) {
            Some(&next_id) => next_id,
            None => 0,
        }
    }

    #[must_use]
    pub fn class(&self, id: usize) -> Option<usize> {
        self.states[id].class
    }

    #[must_use]
    pub fn states(&self) -> &[DFAState] {
        &self.states
    }

    #[must_use]
    pub fn dot(&self) -> String {
        let mut obj = StringBuilder::new();

        obj.writeln("digraph DFA {");
        obj.indent();

        obj.writeln("rankdir=LR;");
        obj.newline();

        obj.writeln("node[shape=point]; q;");
        obj.writeln("node[shape=invhouse]; s0[label=\"\"];");
        obj.newline();

        obj.writeln("node[shape=doublecircle];");
        for (a, state) in self.states().iter().enumerate().skip(1) {
            if let Some(class) = state.class {
                obj.writeln(&format!("s{}[label=\"{}\"];", a, class));
            }
        }
        obj.newline();

        obj.writeln("node[shape=circle];");
        for (a, state) in self.states().iter().enumerate().skip(1) {
            if state.class.is_none() {
                obj.writeln(&format!("s{}[label=\"\"];", a));
            }
        }
        
        obj.newline();
        obj.writeln(&format!("s0->s0[label=\"{:?}\"];", CharSet::universe()));
        obj.writeln("q->s1;");

        for (a, state) in self.states().iter().enumerate().skip(1) {
            let mut inv: HashMap<usize, Vec<u8>>= HashMap::new();
            for (&symbol, &b) in &state.next {
                inv.entry(b).or_default().push(symbol);
            }

            for (b, symbols) in &mut inv {
                let mut set = CharSet::empty();
                for symbol in symbols {
                    set = set.union(&CharSet::point(*symbol));
                }
                obj.writeln(&format!("s{}->s{}[label=\"{:?}\"];", a, b, set));
            }
        }

        obj.unindent();
        obj.writeln("}");

        obj.build()
    }
}

// =================
// === INTERNALS ===
// =================

impl DFAState {
    fn new(next: HashMap<u8, usize>, class: Option<usize>) -> Self {
        Self {
            class,
            next,
            _private: (),
        }
    }

    fn sink() -> Self {
        Self::new(HashMap::new(), None)
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
struct RENodeRefVec {
    vec: Vec<RENodeRef>,
}

impl Deref for RENodeRefVec {
    type Target = [RENodeRef];
    fn deref(&self) -> &Self::Target {
        &*self.vec
    }
}

impl RENodeRefVec {
    fn new(nodes: Vec<RENodeRef>) -> Self {
        Self { vec: nodes }
    }

    fn sink(size: usize) -> Self {
        let none = RENodeRef::new(RENode::None);
        Self { vec: vec![none; size] }
    }

    fn deriv(&self, a: u8) -> RENodeRefVec {
        Self { vec: self.vec.iter().map(|node| node.deriv(a)).collect() }
    }

    fn class(&self) -> Option<usize> {
        self.vec.iter().position(|node| node.is_nullable())
    }
}

struct DFABuilder {
    states: Vec<DFAState>,
    re2idx: BTreeMap<RENodeRefVec, usize>,
}

impl DFABuilder {
    fn build(start: &RENodeRefVec) -> DFA {
        // s0 = sink state
        let states = vec![DFAState::sink()];
        let re2idx = once((RENodeRefVec::sink(start.len()), 0_usize)).collect();
        
        let mut builder = Self { states, re2idx };
        
        // s1 = start state
        builder.add_state(start);
        
        builder.explore(start, 1);

        DFA {
            states: builder.states,
        }
    }

    fn add_state(&mut self, q: &RENodeRefVec) -> usize {
        let idx = self.states.len();
        self.re2idx.insert(q.clone(), idx);
        self.states.push(DFAState::new(HashMap::new(), q.class()));
        idx
    }

    fn explore(&mut self, q: &RENodeRefVec, i: usize) {
        for set in approx_deriv_classes_vec(q) {
            self.goto(q, i, &set);
        }
    }

    fn goto(&mut self, q: &RENodeRefVec, i: usize, set: &CharSet) {
        let c = set.min().unwrap();
        let qc = &q.deriv(c);

        if let Some(&j) = self.re2idx.get(qc) {
            for a in set.chars() {
                self.states[i].next.insert(a, j);
            }
        } else {
            let j = self.add_state(qc);
            for a in set.chars() {
                self.states[i].next.insert(a, j);
            }
            self.explore(qc, j);
        }
    }
}

fn cross<'a, B: IntoIterator<Item = &'a CharSet>>(set1: &HashSet<CharSet>, set2: B) -> HashSet<CharSet> {
    set2.into_iter().flat_map(|t| {
        set1.iter().filter_map(move |s| {
            let u = t.intersection(&s);
            if u.is_empty() { None } else { Some(u) }
        })
    }).collect()
}

// TODO: memoize
fn approx_deriv_classes(root: &RENodeRef) -> HashSet<CharSet> {
    let mut stack = vec![root];
    let mut charsets: HashSet<CharSet> = once(CharSet::universe()).collect();
    
    while let Some(node) = stack.pop() {
        match node.as_ref() {
            RENode::None | RENode::Epsilon => {
                // Do nothing. C(eps) = {universe}, so C(r) ^ C(eps) = C(r).
            },
            RENode::Set(set) => {
                // TODO: set cannot be empty?
                if !set.is_empty() && !set.is_universe() {
                    let cset = &set.complement();
                    charsets = cross(&charsets, vec![set, cset].into_iter());
                }
            },
            RENode::Cat(children) => {
                let mut tail = &children[..];
                while let Some(head) = tail.first() {
                    stack.push(head);
                    if !head.is_nullable() {
                        break;
                    }
                    tail = &tail[1..];
                }
            },
            RENode::Star(child) | RENode::Not(child) => {
                stack.push(child);
            },
            RENode::Or(children) | RENode::And(children) => {
                for child in children {
                    stack.push(child);
                }
            },
        }
    }
    
    charsets
}

fn approx_deriv_classes_vec(root: &RENodeRefVec) -> HashSet<CharSet> {
    root.iter().fold(once(CharSet::universe()).collect(), |acc, x| {
        cross(&acc, &approx_deriv_classes(x))
    })
}

mod hopcroft;

#[cfg(test)]
mod tests;