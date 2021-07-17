use crate::langcore::re::Command;
use crate::langcore::lr1_table::{Action, Reduction};
use crate::parser;

pub struct LR1Parser {
    pub name: String,
    pub ttypes: Vec<String>,
    pub varnames: Vec<String>,
    pub states: Vec<State>,
    pub reductions: Vec<Reduction>,
    pub ttype_count: usize,
}

pub struct State {
    pub ttrans: Vec<TTransition>,
    pub nttrans: Vec<NTTransition>,
    pub has_shift_transitions: bool,
}

pub struct TTransition {
    pub word: Option<usize>, // lookup ttype
    pub action: Action,
}

pub struct NTTransition {
    pub var: usize, // lookup varname
    pub dst: usize,
}

impl LR1Parser {
    #[must_use]
    pub fn new(name: &str, parser: &parser::Parser) -> Self {
        let action_rows = parser.parsing_table.actions.chunks_exact(parser.parsing_table.word_count);
        let goto_rows = parser.parsing_table.gotos.chunks_exact(parser.parsing_table.var_count);

        Self {
            name: name.to_string(),
            ttypes: parser.lexer.vocab.to_vec(),
            varnames: parser.var_names.to_vec(),
            states: action_rows.zip(goto_rows).enumerate().map(|(i, (action_row, goto_row))| State::new(parser, i, action_row, goto_row)).collect(),
            reductions: parser.parsing_table.reductions.to_vec(),
            ttype_count: parser.lexer.table.commands.iter().filter(|command| if let Command::Skip = command { false } else { true }).count() + 1 // +1 for EoF
        }
    }
}

impl State {
    fn new(parser: &parser::Parser, index: usize, action_row: &[Action], goto_row: &[Option<usize>]) -> Self {
        let ttrans: Vec<_> = action_row.iter().enumerate().filter_map(|(word, &action)| {
            if let Action::Invalid = action {
                None
            } else {
                Some(TTransition { word: if word > 0 { Some(word - 1) } else { None }, action })
            }
        }).collect();
        
        let nttrans: Vec<_> = goto_row.iter().enumerate().filter_map(|(var, &dst)| {
            Some(NTTransition { var, dst: dst? })
        }).collect();

        let mut has_shift_transitions = ttrans.iter().any(|ttran| if let Action::Shift(_) = ttran.action { true } else { false });

        Self { ttrans, nttrans, has_shift_transitions }
    }
}