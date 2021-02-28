#![allow(clippy::match_same_arms)]

use std::rc::Rc;
use std::iter::once;
use std::fmt::Formatter;
use std::fmt::Error;
use std::fmt::Debug;

use crate::iter::IteratorExtensions;
use crate::debug::StringBuilder;
use super::CharSet;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RegEx {
    pub(super) root: Rc<RENode>,
}

impl RegEx {
    // === canonical constructors ===

    #[must_use]
    pub fn none() -> Self {
        Self::new(RENode::None)
    }

    #[must_use]
    pub fn empty() -> Self {
        Self::new(RENode::Epsilon)
    }

    #[must_use]
    pub fn set(a: CharSet) -> Self {
        fn mk_set(a: CharSet) -> RegEx {
            if a.is_empty() {
                RegEx::new(RENode::None)
            } else {
                RegEx::new(RENode::Set(a))
            }
        }

        mk_set(a)
    }

    #[must_use]
    pub fn then(&self, other: &Self) -> Self {
        fn mk_cat(r: &RegEx, s: &RegEx) -> RegEx {
            fn cat_aux<'a, A, B>(res1: A, res2: B) -> RegEx
            where
                A: IntoIterator<Item=&'a RegEx>,
                B: IntoIterator<Item=&'a RegEx>,
            {
                RegEx::new(RENode::Cat(res1.into_iter().chain(res2).cloned().collect()))
            }
        
            match (&*r.root, &*s.root) {
                (_              , RENode::Epsilon) => r.clone(),
                (RENode::Epsilon, _              ) => s.clone(),
                (_              , RENode::None   ) => RegEx::new(RENode::None),
                (RENode::None   , _              ) => RegEx::new(RENode::None),
                (RENode::Cat(a) , RENode::Cat(b) ) => cat_aux(a, b),
                (_              , RENode::Cat(b) ) => cat_aux(once(r), b),
                (RENode::Cat(a) , _              ) => cat_aux(a, once(s)),
                (_              , _              ) => cat_aux(once(r), once(s)),
            }
        }

        mk_cat(self, other)
    }

    #[must_use]
    pub fn star(&self) -> Self {
        fn mk_star(r: &RegEx) -> RegEx {
            match *r.root {
                RENode::None | RENode::Epsilon => RegEx::new(RENode::Epsilon),
                RENode::Star(_)                => r.clone(),
                _                              => RegEx::new(RENode::Star(r.clone())),
            }
        }

        mk_star(self)
    }

    #[must_use]
    pub fn or(&self, other: &Self) -> Self {
        fn mk_or(r: &RegEx, s: &RegEx) -> RegEx {
            fn or_aux<'a, A, B>(res1: A, res2: B) -> RegEx
            where
                A: IntoIterator<Item=&'a RegEx>,
                B: IntoIterator<Item=&'a RegEx>,
            {
                let refs = merged_sets(res1.into_iter().merge(res2), |a, b| a.union(b));
        
                if refs.is_empty() {
                    RegEx::new(RENode::None)
                } else if refs.len() == 1 {
                    refs[0].clone()
                } else {
                    RegEx::new(RENode::Or(refs))
                }
            }
        
            match (&*r.root, &*s.root) {
                (_             , RENode::None  ) => r.clone(),
                (RENode::None  , _             ) => s.clone(),
                (RENode::Set(x), RENode::Set(y)) => RegEx::set(x.union(&y)),
                (RENode::Or(a) , RENode::Or(b) ) => or_aux(a, b),
                (RENode::Or(a) , _             ) => or_aux(a, once(s)),
                (_             , RENode::Or(b) ) => or_aux(once(r), b),
                (_             , _             ) => or_aux(once(r), once(s)),
            }
        }

        mk_or(self, other)
    }

    #[must_use]
    pub fn and(&self, other: &Self) -> Self {
        fn mk_and(r: &RegEx, s: &RegEx) -> RegEx {
            fn and_aux<'a, A, B>(res1: A, res2: B) -> RegEx
            where
                A: IntoIterator<Item=&'a RegEx>,
                B: IntoIterator<Item=&'a RegEx>,
            {
                let refs = merged_sets(res1.into_iter().merge(res2), |a, b| a.intersection(b));
        
                if refs.is_empty() {
                    RegEx::new(RENode::None)
                } else if refs.len() == 1 {
                    refs[0].clone()
                } else {
                    RegEx::new(RENode::And(refs))
                }
            }
        
            match (&*r.root, &*s.root) {
                (_              , RENode::None   ) => RegEx::new(RENode::None),
                (RENode::None   , _              ) => RegEx::new(RENode::None),
                (_              , RENode::Epsilon) => if r.is_nullable() { RegEx::new(RENode::Epsilon) } else { RegEx::new(RENode::None) }, // TODO: check
                (RENode::Epsilon, _              ) => if s.is_nullable() { RegEx::new(RENode::Epsilon) } else { RegEx::new(RENode::None) }, // TODO: check
                (RENode::Set(x) , RENode::Set(y) ) => RegEx::set(x.intersection(&y)),
                (RENode::And(a) , RENode::And(b) ) => and_aux(a, b),
                (RENode::And(a) , _              ) => and_aux(a, once(s)),
                (_              , RENode::And(b) ) => and_aux(once(r), b),
                (_              , _              ) => and_aux(once(r), once(s)),
            }
        }

        mk_and(self, other)
    }

    #[must_use]
    pub fn not(&self) -> Self {
        fn mk_not(r: &RegEx) -> RegEx {
            match &*r.root {
                RENode::None   => RegEx::set(CharSet::universe()),
                RENode::Set(s) => RegEx::set(s.complement()),
                RENode::Not(a) => a.clone(),
                _              => RegEx::new(RENode::Not(r.clone())),
            }
        }

        mk_not(self)
    }

    // === non-canonical constructors ===

    #[must_use]
    pub fn opt(&self) -> Self {
        self.or(&RegEx::empty())
    }

    #[must_use]
    pub fn plus(&self) -> Self {
        self.then(&self.star())
    }

    #[must_use]
    pub fn diff(&self, other: &Self) -> Self {
        self.and(&other.not())
    }

    // === other functions ===

    #[must_use]
    pub fn deriv(&self, a: u8) -> Self {
        fn deriv_cat(children: &[RegEx], a: u8) -> RegEx {
            fn aux(r: &RegEx, s: &RegEx, a: u8) -> RegEx {
                let nu_r_da_s = if r.is_nullable() {
                    s.deriv(a)
                } else {
                    RegEx::new(RENode::None)
                };
                (r.deriv(a).then(s)).or(&nu_r_da_s)
            }
    
            match children {
                [] | [_] => {
                    panic!("Should be impossible for Cat node to have <2 children.")
                },
                [r, s] => {
                    aux(r, s, a)
                },
                [r, ..] => {
                    // Tail of children still form a valid Cat node.
                    let s = &RegEx::new(RENode::Cat(children[1..].to_vec()));
                    aux(r, s, a)
                },
            }
        }
        
        fn deriv_or(children: &[RegEx], a: u8) -> RegEx {
            match children {
                [] | [_] => {
                    panic!("Should be impossible for Or node to have <2 children.")
                },
                [r, s] => {
                    r.deriv(a).or(&s.deriv(a))
                },
                [r, ..] => {
                    let s = &RegEx::new(RENode::Or(children[1..].to_vec()));
                    r.deriv(a).or(&s.deriv(a))
                },
            }
        }
        
        fn deriv_and(children: &[RegEx], a: u8) -> RegEx {
            match children {
                [] | [_] => {
                    panic!("Should be impossible for And node to have <2 children.")
                },
                [r, s] => {
                    r.deriv(a).and(&s.deriv(a))
                },
                [r, ..] => {
                    let s = &RegEx::new(RENode::And(children[1..].to_vec()));
                    r.deriv(a).and(&s.deriv(a))
                },
            }
        }
    
        match &*self.root {
            RENode::None
            | RENode::Epsilon => RegEx::new(RENode::None),
            RENode::Set(s)    => if s.contains(a) { RegEx::new(RENode::Epsilon) } else { RegEx::new(RENode::None) },
            RENode::Cat(res)  => deriv_cat(res, a),
            RENode::Star(re)  => re.deriv(a).then(self),
            RENode::Or(res)   => deriv_or(res, a),
            RENode::And(res)  => deriv_and(res, a),
            RENode::Not(re)   => re.deriv(a).not(),
        }
    }

    #[must_use]
    pub fn is_nullable(&self) -> bool {
        self.root.is_nullable()
    }

    #[must_use]
    pub fn dot(&self) -> String {
        let mut stack: Vec<(usize, &RegEx)> = vec![(0, self)];
        let mut next_id = 0_usize;

        let mut obj = StringBuilder::new();

        obj.writeln("digraph RegEx {");
        obj.indent();
        obj.writeln("node[shape=plain];");
        
        while let Some((parent_id, parent)) = stack.pop() {
            match &*parent.root {
                RENode::None => {
                    obj.writeln(&format!("s{}[label=\"\u{2205}\"]", parent_id));
                },
                RENode::Epsilon => {
                    obj.writeln(&format!("s{}[label=\"\u{03B5}\"]", parent_id));
                },
                RENode::Set(set) => {
                    obj.writeln(&format!("s{}[label=\"{:?}\"]", parent_id, set));
                },
                RENode::Cat(children) => {
                    obj.writeln(&format!("s{}[label=\"cat\"]", parent_id));
                    for child in children {
                        next_id += 1;
                        obj.writeln(&format!("s{}->s{}", parent_id, next_id));
                        stack.push((next_id, child));
                    }
                },
                RENode::Star(child) => {
                    obj.writeln(&format!("s{}[label=\"star\"]", parent_id));
                    next_id += 1;
                    obj.writeln(&format!("s{}->s{}", parent_id, next_id));
                    stack.push((next_id, child));
                },
                RENode::Or(children) => {
                    obj.writeln(&format!("s{}[label=\"or\"]", parent_id));
                    for child in children {
                        next_id += 1;
                        obj.writeln(&format!("s{}->s{}", parent_id, next_id));
                        stack.push((next_id, child));
                    }
                },
                RENode::And(children) => {
                    obj.writeln(&format!("s{}[label=\"and\"]", parent_id));
                    for child in children {
                        next_id += 1;
                        obj.writeln(&format!("s{}->s{}", parent_id, next_id));
                        stack.push((next_id, child));
                    }
                },
                RENode::Not(child) => {
                    obj.writeln(&format!("s{}[label=\"not\"]", parent_id));
                    next_id += 1;
                    obj.writeln(&format!("s{}->s{}", parent_id, next_id));
                    stack.push((next_id, child));
                },
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

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum RENode {
    None,
    Epsilon,

    /// # Invariants
    /// * Set is not empty
    Set(CharSet),

    /// # Invariants
    /// * At least 2 children
    /// * No child is None
    /// * No child is Epsilon
    /// * No child is Cat
    Cat(Vec<RegEx>),

    /// # Invariants
    /// * Child is not None
    /// * Child is not Epsilon
    /// * Child is not Star
    Star(RegEx),

    /// # Invariants
    /// * At least 2 children
    /// * No child is None
    /// * No child is Or
    /// * At most 1 child is a Set
    Or(Vec<RegEx>),

    /// # Invariants
    /// * At least 2 children
    /// * No child is None
    /// * No child is Epsilon
    /// * No child is And
    /// * At most 1 child is a Set
    And(Vec<RegEx>),

    /// # Invariants
    /// * Child is not None
    /// * Child is not a Set
    /// * Child is not Not
    Not(RegEx),
}

impl RegEx {
    fn new(node: RENode) -> RegEx {
        RegEx { root: Rc::new(node) }
    }
}

impl RENode {
    fn is_nullable(&self) -> bool {
        match self {
            RENode::None     => false,
            RENode::Epsilon  => true,
            RENode::Set(_)   => false,
            RENode::Cat(res) => res.iter().all(RegEx::is_nullable),
            RENode::Star(_)  => true,
            RENode::Or(res)  => res.iter().any(RegEx::is_nullable),
            RENode::And(res) => res.iter().all(RegEx::is_nullable),
            RENode::Not(re)  => !re.is_nullable(),
        }
    }
}

fn merged_sets<'a, T, F>(res: T, f: F) -> Vec<RegEx>
where
    T: IntoIterator<Item=&'a RegEx>,
    F: Fn(CharSet, &CharSet) -> CharSet,
{
    let mut sets: Vec<&CharSet> = Vec::new();
    let mut refs: Vec<RegEx> = Vec::new();

    for re in res {
        if let RENode::Set(a) = &*re.root {
            sets.push(a);
        } else {
            refs.push(re.clone());
        }
    }

    if let Some(first) = sets.pop() {
        let re = RegEx::new(RENode::Set(sets.into_iter().fold(first.clone(), |acc, x| f(acc, x))));
        refs.into_iter().merge(once(re)).collect()
    } else {
        refs
    }
}

impl Debug for RENode {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match self {
            RENode::None => {
                f.write_str("\u{2205}")
            },
            RENode::Epsilon => {
                f.write_str("\u{03B5}")
            },
            RENode::Set(set) => {
                f.write_str(&format!("{:?}", set))
            },
            RENode::Cat(children) => {
                f.write_str(&format!("({})", children.iter().map(|child| format!("{:?}", child)).collect::<String>()))
            },
            RENode::Star(child) => {
                f.write_str(&format!("({:?})*", child))
            },
            RENode::Or(children) => {
                f.write_str(&format!("({})", children.iter().map(|child| format!("{:?}", child)).collect::<Vec<_>>().join("|")))

            },
            RENode::And(children) => {
                f.write_str(&format!("({})", children.iter().map(|child| format!("{:?}", child)).collect::<Vec<_>>().join("&")))

            },
            RENode::Not(child) => {
                f.write_str(&format!("!({:?})", child))
            },
        }
    }
}

impl Debug for RegEx {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        f.write_str(&format!("{:?}", *self.root))
    }
}

#[cfg(test)]
mod tests;