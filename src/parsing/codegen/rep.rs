use std::collections::{BTreeMap, btree_map::Entry::{Occupied, Vacant}};
use crate::langcore::re::{LexTable, Command};
use crate::langcore::cfg::Grammar;
use crate::langcore::lr1_table::{self, Action, Reduction, LR1TableBuilder};
use crate::lexer;
use crate::parser;
use parser::strategy;

pub struct LR1Parser {
    pub name: String,
    pub lexer: Lexer,
    pub varnames: Vec<String>,
    pub reductions: Vec<Reduction>,
    pub grammar: Grammar,
    pub states: Vec<ParserState>,
}

pub struct ParserState {
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

pub struct Lexer {
    pub name: String,
    pub ttypes: Vec<String>,
    pub commands: Vec<Command>,
    pub ttype_count: usize, // excluding Skip words
    pub states: Vec<LexerState>,
}

pub struct LexerState {
    pub transitions: Vec<Transition>,
    pub can_transition_to_unlabelled_state: bool,
    pub class: Option<usize>,
}

pub struct Transition {
    pub intervals: Vec<(u8, u8)>,
    pub dst: usize,
}

impl LR1Parser {
    /// # Errors
    pub fn new<'a, S>(name: &str, def: &'a parser::ParserDef, strategy: &S) -> Result<Self, lr1_table::ConstructionError>
    where
        S::Builder: LR1TableBuilder,
        S: strategy::Strategy<'a>,
    {
        let builder = strategy.builder(&def.grammar);

        let parsing_table = S::build(&builder, &def.grammar, def.conflict_resolution())?;

        let action_rows = parsing_table.actions.chunks_exact(parsing_table.word_count);
        let goto_rows = parsing_table.gotos.chunks_exact(parsing_table.var_count);

        Ok(Self {
            name: name.to_string(),
            lexer: Lexer::new(name, &def.lexer_def),
            varnames: def.var_names.clone(),
            reductions: parsing_table.reductions,
            grammar: def.grammar.clone(),
            states: action_rows.zip(goto_rows).map(|(action_row, goto_row)| ParserState::new(action_row, goto_row)).collect(),
        })
    }
}

impl ParserState {
    fn new(action_row: &[Action], goto_row: &[Option<usize>]) -> Self {
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

        let has_shift_transitions = ttrans.iter().any(|ttran| matches!(ttran.action, Action::Shift(_)));

        Self { ttrans, nttrans, has_shift_transitions }
    }
}

impl Lexer {
    #[must_use]
    pub fn new(name: &str, def: &lexer::LexerDef) -> Self {
        let lexer = def.build();
        let ttype_count = lexer.table.commands.iter().filter(|command| !matches!(command, Command::Skip)).count() + 1; // +1 for EoF
        let states = lexer.table.next.chunks_exact(256).enumerate().map(|(i, row)| LexerState::new(&lexer, i, row)).collect();

        Self {
            name: name.to_string(),
            ttypes: lexer.vocab,
            commands: lexer.table.commands,
            ttype_count,
            states,
        }
    }
}

impl LexerState {
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