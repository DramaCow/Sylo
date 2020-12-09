use super::{Command, SynAnalyzer, Action, Reduction};

#[derive(Debug)]
pub enum ParseStep {
    Leaf { word: usize, index: usize },
    Branch { var: usize, num_children: usize },
    List { num_children: usize },
    // Variable(Reduction),
}

enum FrontierElem {
    Node { index: usize },              // Used by non-propagating variables w/ children 
    List { first: usize, last: usize }, // Used by propagating variables
    Empty,                              // Used by variables w/o children
}

pub struct Parse<'a, I: Iterator<Item=usize>> {
    syn: &'a SynAnalyzer,
    words:  I,
    // === internals ===
    step:          usize,
    next_word:     Option<usize>,
    next_action:   Action,
    state_history: Vec<usize>,
}

#[derive(Debug)]
pub struct ParseError {}

impl<'a, I: Iterator<Item=usize>> Parse<'a, I> {
    #[must_use]
    pub fn new(syn: &'a SynAnalyzer, words: I) -> Self {
        Self {
            syn,
            words,
            // === internals ===
            step:          0,
            next_word:     None,
            next_action:   Action::Shift(0),
            state_history: Vec::new(),
        }
    }
}

impl<'a, I: Iterator<Item=usize>> Iterator for Parse<'a, I> {
    type Item = Result<ParseStep, ParseError>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.next_action {
            Action::Invalid => {
                Some(Err(ParseError {}))
            },
            Action::Accept => {
                None
            },
            Action::Shift(next_state) => {
                self.state_history.push(next_state);

                let curr_word = self.next_word;

                self.next_word  = self.words.next();
                self.next_action = *self.syn.action(next_state, self.next_word);
                
                curr_word.map_or(self.next(), |word| {
                    self.step += 1;
                    Some(Ok(ParseStep::Leaf { word, index: self.step - 1 }))
                })
            },
            Action::Reduce(alt) => {
                let reduction = &self.syn.reductions[alt];
                (0..reduction.count).for_each(|_| { self.state_history.pop(); });
                let old_state = *self.state_history.last().unwrap();

                self.syn.goto(old_state, reduction.var).map_or(Some(Err(ParseError {})), |next_state| {
                    self.state_history.push(next_state);
                    self.next_action = *self.syn.action(next_state, self.next_word);
                    // Some(Ok(ParseStep::Variable(*reduction)));
                    Some(Ok(match self.syn.commands.get(reduction.var).unwrap() {
                        Command::Emit => ParseStep::Branch { var: reduction.var, num_children: reduction.count },
                        Command::Skip => ParseStep::List { num_children: reduction.count },
                    }))
                })
            },
        }
    }
}