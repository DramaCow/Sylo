use std::collections::{BTreeMap, btree_map::Entry::{Occupied, Vacant}};
use crate::lang::re::LexTable;
use crate::lexer;

pub struct Lexer {
    pub name: String,
    pub ttypes: Vec<String>,
    pub states: Vec<State>,
}

pub struct State {
    pub transitions: Vec<Transition>,
    pub can_transition_to_unlabelled_state: bool,
    pub class: Option<usize>,
}

pub struct Transition {
    pub intervals: Vec<(u8, u8)>,
    pub dst: usize,
}

impl Lexer {
    #[must_use]
    pub fn new(name: &str, lexer: &lexer::Lexer) -> Self {
        Self {
            name: name.to_string(),
            ttypes: lexer.vocab.to_vec(),
            states: lexer.table.next.chunks_exact(256).enumerate().map(|(i, row)| State::new(lexer, i, row)).collect(),
        }
    }
}

impl State {
    fn new(lexer: &lexer::Lexer, index: usize, row: &[usize]) -> Self {
        // NOTE: I consider the sink a "labelled" state
        let mut transitions: BTreeMap<usize, Vec<(u8, u8)>> = BTreeMap::new();
        let mut can_transition_to_unlabelled_state = false;

        for (ch, state) in (0..=255_u8).zip(row.iter().copied()) {
            if state != lexer.table.sink() {
                match transitions.entry(state) {
                    Occupied(mut entry) => {
                        let intervals = entry.get_mut();
                        let last = intervals.last_mut().unwrap();
                        if last.1 + 1 == ch {
                            last.1 = ch;
                        } else {
                            intervals.push((ch, ch));
                        }
                    },
                    Vacant(entry) => { entry.insert(vec![(ch, ch)]); }
                }
   
                if lexer.table.class(state).is_none() {
                    can_transition_to_unlabelled_state = true;
                }
            }
        }

        Self {
            transitions: transitions.into_iter().map(|(dst, intervals)| Transition { intervals, dst }).collect(),
            can_transition_to_unlabelled_state,
            class: lexer.table.class(index),
        }
    }
}