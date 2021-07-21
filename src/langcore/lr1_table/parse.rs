#![allow(clippy::option_if_let_else)]

use std::mem;
use super::{Action, LR1Table};

#[derive(Debug, PartialEq, Eq)]
pub enum Event<T> {
    Shift(T),
    Reduce { var: usize, child_count: usize, production: usize },
}

pub struct Parse<'a, P, I, T, F> {
    table:         &'a P,
    input:         I,
    f:             F,
    step:          usize,
    next_word:     Option<T>,
    next_action:   Action,
    state_history: Vec<usize>,
}

#[derive(Debug)]
pub enum ParseError<E> {
    InputError(E),
    InvalidAction { step: usize, state: usize, word: Option<usize> },
    InvalidGoto { step: usize, state: usize, var: usize },
}

impl<'a, P, I, T, F> Parse<'a, P, I, T, F>
where
    P: LR1Table,
    F: Fn(&T) -> usize,
{
    #[must_use]
    pub fn new(table: &'a P, input: I, f: F) -> Self {
        Self {
            table,
            input,
            f,
            step:          0, // only really useful for debugging, not strictly necessary
            next_word:     None,
            next_action:   Action::Shift(P::START_STATE),
            state_history: Vec::new(),
        }
    }
}

impl<'a, P, I, T, E, F> Iterator for Parse<'a, P, I, T, F>
where
    P: LR1Table,
    I: Iterator<Item=Result<T, E>>,
    F: Fn(&T) -> usize,
{
    type Item = Result<Event<T>, ParseError<E>>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.next_action {
            Action::Invalid => {
                Some(Err(ParseError::InvalidAction {
                    step: self.step,
                    state: *self.state_history.last().unwrap(),
                    word: self.next_word.as_ref().map(&self.f),
                }))
            },
            Action::Accept => {
                None
            },
            Action::Shift(state) => {
                let curr_word = mem::replace(&mut self.next_word, match self.input.next().transpose() {
                    Ok(val) => val,
                    Err(err) => return Some(Err(ParseError::InputError(err))),
                });
                self.next_action = self.table.action(state, self.next_word.as_ref().map(&self.f));
                self.state_history.push(state);

                if let Some(word) = curr_word {
                    self.step += 1;
                    Some(Ok(Event::Shift(word)))
                } else {
                    // occurs on first and last iterations
                    self.next()
                }
            },
            Action::Reduce(production) => {
                // lookup which variable and how many frontier elements are consumed
                let reduction = self.table.reduction(production);

                // consume part of frontier
                for _ in 0..reduction.count {
                    self.state_history.pop();
                }

                // state is rewinded to before words associated with reduction
                let old_state = *self.state_history.last().unwrap();

                if let Some(state) = self.table.goto(old_state, reduction.var) {
                    self.next_action = self.table.action(state, self.next_word.as_ref().map(&self.f));
                    self.state_history.push(state);
                    Some(Ok(Event::Reduce {
                        var: reduction.var,
                        child_count: reduction.count,
                        production,
                    }))
                } else {
                    Some(Err(ParseError::InvalidGoto {
                        step: self.step,
                        state: old_state,
                        var: reduction.var,
                    }))
                }
            },
        }
    }
}