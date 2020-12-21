use super::{SynAnalyzer, SynAction};

#[derive(Debug, PartialEq, Eq)]
pub enum Action {
    Shift { word: usize, index: usize },
    Reduce { var: usize, count: usize },
}

pub struct SynParse<'a, I: Iterator<Item=usize>> {
    syn:           &'a SynAnalyzer,
    words:         I,
    step:          usize,
    next_word:     Option<usize>,
    next_action:   SynAction,
    state_history: Vec<usize>,
}

#[derive(Debug)]
pub struct SynParseError {}

impl<'a, I: Iterator<Item=usize>> SynParse<'a, I> {
    #[must_use]
    pub fn new(syn: &'a SynAnalyzer, words: I) -> Self {
        Self {
            syn,
            words,
            step:          0,
            next_word:     None,
            next_action:   SynAction::Shift(0),
            state_history: Vec::new(),
        }
    }
}

impl<'a, I: Iterator<Item=usize>> Iterator for SynParse<'a, I> {
    type Item = Result<Action, SynParseError>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.next_action {
            SynAction::Invalid => {
                Some(Err(SynParseError {}))
            },
            SynAction::Accept => {
                None
            },
            SynAction::Shift(next_state) => {
                let curr_word = self.next_word;
                
                // pre-load next state
                self.next_word = self.words.next();
                self.next_action = self.syn.action(next_state, self.next_word);
                self.state_history.push(next_state);

                if let Some(word) = curr_word {
                    self.step += 1;
                    Some(Ok(Action::Shift { word, index: self.step - 1 }))   
                } else {
                    // occurs on first and last iterations
                    self.next()
                }
            },
            SynAction::Reduce(alt) => {
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
                    Some(Ok(Action::Reduce { var: reduction.var, count: reduction.count }))
                } else {
                    Some(Err(SynParseError {}))
                }
            },
        }
    }
}