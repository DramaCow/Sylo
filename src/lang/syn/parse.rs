use super::{SynAnalyzer, Action};

#[derive(Debug, PartialEq, Eq)]
pub enum Instruction {
    Shift { word: usize, index: usize },
    Reduce { var: usize, count: usize },
}

pub struct Parse<'a, I: Iterator<Item=usize>> {
    syn:           &'a SynAnalyzer,
    words:         I,
    count:         usize,
    next_word:     Option<usize>,
    next_action:   Action,
    state_history: Vec<usize>,
}

#[derive(Debug)]
pub struct ParseError {
    pub count: usize,
    pub source: ParseErrorType,
}

#[derive(Debug)]
pub enum ParseErrorType {
    InvalidAction { state: usize, word: Option<usize> },
    InvalidGoto   { state: usize, var: usize },
}

impl<'a, I: Iterator<Item=usize>> Parse<'a, I> {
    #[must_use]
    pub fn new(syn: &'a SynAnalyzer, words: I) -> Self {
        Self {
            syn,
            words,
            count:         0,
            next_word:     None,
            next_action:   Action::Shift(0),
            state_history: Vec::new(),
        }
    }
}

impl<'a, I: Iterator<Item=usize>> Iterator for Parse<'a, I> {
    type Item = Result<Instruction, ParseError>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.next_action {
            Action::Invalid => {
                Some(Err(ParseError {
                    count: self.count,
                    source: ParseErrorType::InvalidAction {
                        state: *self.state_history.last().unwrap(),
                        word: self.next_word
                    },
                }))
            },
            Action::Accept => {
                None
            },
            Action::Shift(next_state) => {
                let curr_word = self.next_word;
                
                // pre-load next state
                self.next_word = self.words.next();
                self.next_action = self.syn.action(next_state, self.next_word);
                self.state_history.push(next_state);

                if let Some(word) = curr_word {
                    self.count += 1;
                    Some(Ok(Instruction::Shift { word, index: self.count - 1 }))   
                } else {
                    // occurs on first and last iterations
                    self.next()
                }
            },
            Action::Reduce(alt) => {
                // lookup which variable and how many frontier elements are consumed
                let reduction = &self.syn.reductions[alt];

                // consume part of frontier
                for _ in 0..reduction.count {
                    self.state_history.pop();
                }

                // state is rewinded to before words associated with reduction
                let old_state = *self.state_history.last().unwrap();

                if let Some(next_state) = self.syn.goto(old_state, reduction.var) {
                    self.next_action = self.syn.action(next_state, self.next_word);
                    self.state_history.push(next_state);
                    Some(Ok(Instruction::Reduce { var: reduction.var, count: reduction.count }))
                } else {
                    Some(Err(ParseError {
                        count: self.count,
                        source: ParseErrorType::InvalidGoto {
                            state: old_state,
                            var: reduction.var
                        },
                    }))
                }
            },
        }
    }
}