use super::{
    Action,
    ParsingTable,
};

#[derive(Debug, PartialEq, Eq)]
pub enum Node {
    Word { word: usize, index: usize },
    Var { var: usize, child_count: usize },
}

pub struct Parse<'a, T, I> {
    table:         &'a T,
    input:         I,
    step:          usize,
    curr_word:     Option<usize>,
    next_action:   Action,
    state_history: Vec<usize>,
}

#[derive(Debug)]
pub struct ParseError {
    pub step: usize,
    pub state: usize,
    pub source: ParseErrorSource,
}
  
#[derive(Debug)]
pub enum ParseErrorSource {
    InvalidAction { word: Option<usize> },
    InvalidGoto   { var: usize },
}

impl<'a, T, I> Parse<'a, T, I>
where
    T: ParsingTable,
{
    #[must_use]
    pub fn new(table: &'a T, input: I) -> Self {
        Self {
            table,
            input,
            step:          0, // only really useful for debugging, not strictly necessary
            curr_word:     None,
            next_action:   Action::Shift(T::START_STATE),
            state_history: Vec::new(),
        }
    }
}

impl<'a, T, I> Iterator for Parse<'a, T, I>
where
    T: ParsingTable,
    I: Iterator<Item=usize>,
{
    type Item = Result<Node, ParseError>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.next_action {
            Action::Invalid => {
                Some(Err(ParseError {
                    step: self.step,
                    state: *self.state_history.last().unwrap(),
                    source: ParseErrorSource::InvalidAction {
                        word: self.curr_word
                    },
                }))
            },
            Action::Accept => {
                None
            },
            Action::Shift(state) => {
                let prev_word = self.curr_word;
                
                // pre-load next action
                self.curr_word = self.input.next();
                self.next_action = self.table.action(state, self.curr_word);
                self.state_history.push(state);

                if let Some(word) = prev_word {
                    self.step += 1;
                    Some(Ok(Node::Word { word, index: self.step - 1 }))   
                } else {
                    // occurs on first and last iterations
                    self.next()
                }
            },
            Action::Reduce(alt) => {
                // lookup which variable and how many frontier elements are consumed
                let reduction = self.table.reduction(alt);

                // consume part of frontier
                for _ in 0..reduction.count {
                    self.state_history.pop();
                }

                // state is rewinded to before words associated with reduction
                let old_state = *self.state_history.last().unwrap();

                if let Some(state) = self.table.goto(old_state, reduction.var) {
                    self.next_action = self.table.action(state, self.curr_word);
                    self.state_history.push(state);
                    Some(Ok(Node::Var { var: reduction.var, child_count: reduction.count }))
                } else {
                    Some(Err(ParseError {
                        step: self.step,
                        state: old_state,
                        source: ParseErrorSource::InvalidGoto {
                            var: reduction.var
                        },
                    }))
                }
            },
        }
    }
}