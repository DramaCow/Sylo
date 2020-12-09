#![allow(clippy::match_same_arms)]

use std::rc::Rc;
use std::ops::Deref;
use std::iter::once;
use std::fmt::Formatter;
use std::fmt::Error;
use std::fmt::Debug;

use crate::iter::IteratorExtensions;
use crate::debug::StringBuilder;
use super::CharSet;

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct RegEx {
    pub(super) root: RENodeRef,
}

impl RegEx {
    // === canonical constructors ===

    #[must_use]
    pub fn none() -> Self {
        Self { root: RENodeRef::new(RENode::None) }
    }

    #[must_use]
    pub fn empty() -> Self {
        Self { root: RENodeRef::new(RENode::Epsilon) }
    }

    #[must_use]
    pub fn set(a: CharSet) -> Self {
        Self { root: mk_set(a) }
    }

    #[must_use]
    pub fn then(&self, other: &Self) -> Self {
        Self { root: mk_cat(&self.root, &other.root) }
    }

    #[must_use]
    pub fn star(&self) -> Self {
        Self { root: mk_star(&self.root) }
    }

    #[must_use]
    pub fn or(&self, other: &Self) -> Self {
        Self { root: mk_or(&self.root, &other.root) }
    }

    #[must_use]
    pub fn and(&self, other: &Self) -> Self {
        Self { root: mk_and(&self.root, &other.root) }
    }

    #[must_use]
    pub fn not(&self) -> Self {
        Self { root: mk_not(&self.root) }
    }

    // === non-canonical constructors ===

    #[must_use]
    pub fn opt(&self) -> Self {
        Self { root: mk_or(&self.root, &RENodeRef::new(RENode::Epsilon)) }
    }

    #[must_use]
    pub fn plus(&self) -> Self {
        Self { root: mk_cat(&self.root, &mk_star(&self.root)) }
    }

    // === other functions ===

    #[must_use]
    pub fn deriv(&self, a: u8) -> Self {
        Self { root: self.root.deriv(a) }
    }

    #[must_use]
    pub fn dot(&self) -> String {
        let mut stack: Vec<(usize, &RENodeRef)> = vec![(0, &self.root)];
        let mut next_id = 0_usize;

        let mut obj = StringBuilder::new();

        obj.writeln("digraph RegEx {");
        obj.indent();
        obj.writeln("node[shape=plain];");
        
        while let Some((parent_id, parent)) = stack.pop() {
            match parent.as_ref() {
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

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct RENodeRef { ptr: Rc<RENode> }

impl Deref for RENodeRef {
    type Target = RENode;
    fn deref(&self) -> &Self::Target {
        &*self.ptr
    }
}

impl AsRef<RENode> for RENodeRef {
    fn as_ref(&self) -> &RENode {
        self.ptr.as_ref()
    }
}

impl RENodeRef {
    pub(super) fn new(node: RENode) -> RENodeRef {
        // RENodeRef::new(node)
        RENodeRef { ptr: Rc::new(node) }
    }

    pub(super) fn deriv(&self, a: u8) -> Self {
        fn deriv_cat(children: &[RENodeRef], a: u8) -> RENodeRef {
            fn aux(r: &RENodeRef, s: &RENodeRef, a: u8) -> RENodeRef {
                let nu_r_da_s = if r.is_nullable() {
                    s.deriv(a)
                } else {
                    RENodeRef::new(RENode::None)
                };
                mk_or(&mk_cat(&r.deriv(a), s), &nu_r_da_s)
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
                    let s = &RENodeRef::new(RENode::Cat(children[1..].to_vec()));
                    aux(r, s, a)
                },
            }
        }
        
        fn deriv_or(children: &[RENodeRef], a: u8) -> RENodeRef {
            match children {
                [] | [_] => {
                    panic!("Should be impossible for Or node to have <2 children.")
                },
                [r, s] => {
                    mk_or(&r.deriv(a), &s.deriv(a))
                },
                [r, ..] => {
                    let s = &RENodeRef::new(RENode::Or(children[1..].to_vec()));
                    mk_or(&r.deriv(a), &s.deriv(a))
                },
            }
        }
        
        fn deriv_and(children: &[RENodeRef], a: u8) -> RENodeRef {
            match children {
                [] | [_] => {
                    panic!("Should be impossible for And node to have <2 children.")
                },
                [r, s] => {
                    mk_and(&r.deriv(a), &s.deriv(a))
                },
                [r, ..] => {
                    let s = &RENodeRef::new(RENode::And(children[1..].to_vec()));
                    mk_and(&r.deriv(a), &s.deriv(a))
                },
            }
        }
    
        match self.as_ref() {
            RENode::None
            | RENode::Epsilon => RENodeRef::new(RENode::None),
            RENode::Set(s)    => if s.contains(a) { RENodeRef::new(RENode::Epsilon) } else { RENodeRef::new(RENode::None) },
            RENode::Cat(res)  => deriv_cat(res, a),
            RENode::Star(re)  => mk_cat(&re.deriv(a), self),
            RENode::Or(res)   => deriv_or(res, a),
            RENode::And(res)  => deriv_and(res, a),
            RENode::Not(re)   => mk_not(&re.deriv(a)),
        }
    }
}

fn mk_set(a: CharSet) -> RENodeRef {
    if a.is_empty() {
        RENodeRef::new(RENode::None)
    } else {
        RENodeRef::new(RENode::Set(a))
    }
}

fn mk_cat(r: &RENodeRef, s: &RENodeRef) -> RENodeRef {
    fn cat_aux<'a, A, B>(res1: A, res2: B) -> RENodeRef
    where
        A: IntoIterator<Item=&'a RENodeRef>,
        B: IntoIterator<Item=&'a RENodeRef>,
    {
        RENodeRef::new(RENode::Cat(res1.into_iter().chain(res2).cloned().collect()))
    }

    match (r.as_ref(), s.as_ref()) {
        (_              , RENode::Epsilon) => r.clone(),
        (RENode::Epsilon, _              ) => s.clone(),
        (_              , RENode::None   ) => RENodeRef::new(RENode::None),
        (RENode::None   , _              ) => RENodeRef::new(RENode::None),
        (RENode::Cat(a) , RENode::Cat(b) ) => cat_aux(a, b),
        (_              , RENode::Cat(b) ) => cat_aux(once(r), b),
        (RENode::Cat(a) , _              ) => cat_aux(a, once(s)),
        (_              , _              ) => cat_aux(once(r), once(s)),
    }
}

fn mk_star(r: &RENodeRef) -> RENodeRef {
    match r.as_ref() {
        RENode::None | RENode::Epsilon => RENodeRef::new(RENode::Epsilon),
        RENode::Star(_)                => r.clone(),
        _                              => RENodeRef::new(RENode::Star(r.clone())),
    }
}

fn mk_or(r: &RENodeRef, s: &RENodeRef) -> RENodeRef {
    fn or_aux<'a, A, B>(res1: A, res2: B) -> RENodeRef
    where
        A: IntoIterator<Item=&'a RENodeRef>,
        B: IntoIterator<Item=&'a RENodeRef>,
    {
        let refs = merged_sets(res1.into_iter().merge(res2), |a, b| a.union(b));

        if refs.is_empty() {
            RENodeRef::new(RENode::None)
        } else if refs.len() == 1 {
            refs[0].clone()
        } else {
            RENodeRef::new(RENode::Or(refs))
        }
    }

    match (r.as_ref(), s.as_ref()) {
        (_             , RENode::None  ) => r.clone(),
        (RENode::None  , _             ) => s.clone(),
        (RENode::Set(x), RENode::Set(y)) => mk_set(x.union(y)),
        (RENode::Or(a) , RENode::Or(b) ) => or_aux(a, b),
        (RENode::Or(a) , _             ) => or_aux(a, once(s)),
        (_             , RENode::Or(b) ) => or_aux(once(r), b),
        (_             , _             ) => or_aux(once(r), once(s)),
    }
}

fn mk_and(r: &RENodeRef, s: &RENodeRef) -> RENodeRef {
    fn and_aux<'a, A, B>(res1: A, res2: B) -> RENodeRef
    where
        A: IntoIterator<Item=&'a RENodeRef>,
        B: IntoIterator<Item=&'a RENodeRef>,
    {
        let refs = merged_sets(res1.into_iter().merge(res2), |a, b| a.intersection(b));

        if refs.is_empty() {
            RENodeRef::new(RENode::None)
        } else if refs.len() == 1 {
            refs[0].clone()
        } else {
            RENodeRef::new(RENode::And(refs))
        }
    }

    match (r.as_ref(), s.as_ref()) {
        (_              , RENode::None   ) => RENodeRef::new(RENode::None),
        (RENode::None   , _              ) => RENodeRef::new(RENode::None),
        (_              , RENode::Epsilon) => if r.is_nullable() { RENodeRef::new(RENode::Epsilon) } else { RENodeRef::new(RENode::None) }, // TODO: check
        (RENode::Epsilon, _              ) => if s.is_nullable() { RENodeRef::new(RENode::Epsilon) } else { RENodeRef::new(RENode::None) }, // TODO: check
        (RENode::Set(x) , RENode::Set(y) ) => mk_set(x.intersection(y)),
        (RENode::And(a) , RENode::And(b) ) => and_aux(a, b),
        (RENode::And(a) , _              ) => and_aux(a, once(s)),
        (_              , RENode::And(b) ) => and_aux(once(r), b),
        (_              , _              ) => and_aux(once(r), once(s)),
    }
}

fn mk_not(r: &RENodeRef) -> RENodeRef {
    match r.as_ref() {
        RENode::None   => mk_set(CharSet::universe()),
        RENode::Set(s) => mk_set(s.complement()),
        RENode::Not(a) => a.clone(),
        _              => RENodeRef::new(RENode::Not(r.clone())),
    }
}

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
    Cat(Vec<RENodeRef>),

    /// # Invariants
    /// * Child is not None
    /// * Child is not Epsilon
    /// * Child is not Star
    Star(RENodeRef),

    /// # Invariants
    /// * At least 2 children
    /// * No child is None
    /// * No child is Or
    /// * At most 1 child is a Set
    Or(Vec<RENodeRef>),

    /// # Invariants
    /// * At least 2 children
    /// * No child is None
    /// * No child is Epsilon
    /// * No child is And
    /// * At most 1 child is a Set
    And(Vec<RENodeRef>),

    /// # Invariants
    /// * Child is not None
    /// * Child is not a Set
    /// * Child is not Not
    Not(RENodeRef),
}

impl RENode {
    pub(super) fn is_nullable(&self) -> bool {
        match self {
            RENode::None     => false,
            RENode::Epsilon  => true,
            RENode::Set(_)   => false,
            RENode::Cat(res) => res.iter().all(|re| re.is_nullable()),
            RENode::Star(_)  => true,
            RENode::Or(res)  => res.iter().any(|re| re.is_nullable()),
            RENode::And(res) => res.iter().all(|re| re.is_nullable()),
            RENode::Not(re)  => !re.is_nullable(),
        }
    }
}

fn merged_sets<'a, T, F>(res: T, f: F) -> Vec<RENodeRef>
where
    T: IntoIterator<Item=&'a RENodeRef>,
    F: Fn(CharSet, &CharSet) -> CharSet,
{
    let mut sets: Vec<&CharSet> = Vec::new();
    let mut refs: Vec<RENodeRef> = Vec::new();

    for re in res {
        if let RENode::Set(a) = re.as_ref() {
            sets.push(a);
        } else {
            refs.push(re.clone());
        }
    }

    if let Some(first) = sets.pop() {
        let re = RENodeRef::new(RENode::Set(sets.into_iter().fold(first.clone(), |acc, x| f(acc, x))));
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

impl Debug for RENodeRef {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        f.write_str(&format!("{:?}", self.as_ref()))
    }
}

impl Debug for RegEx {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        f.write_str(&format!("{:?}", &self.root))
    }
}

#[cfg(test)]
mod tests;