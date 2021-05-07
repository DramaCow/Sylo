use std::collections::{
    BTreeMap,
    btree_map::Entry::{Occupied, Vacant},
};
use crate::utils::StringBuilder;
use crate::lang::re::{LexTable, Command};
use crate::syntax::Lexer;
use tinytemplate::TinyTemplate;
use serde::Serialize;

/// # Errors
pub fn render_lexer(lexer: &Lexer, name: &str) -> Result<String, std::fmt::Error> {
    #[derive(Serialize)]
    struct Context {
        name: String,
        ttype_labels: Vec<String>,
    }

    let lexdata = LexerSer::new(lexer);
    
    let table_code = {
        let mut fmt = StringBuilder::new();
        fmt.indent();
            
        for (i, state) in lexdata.states.iter().enumerate() {
            writeln!(fmt, "s{}: {{", i)?;
            fmt.indent();

            if state.transitions.is_empty() {
                // `i` is a labelled state
                if let Some(class) = state.class {
                    if let Command::Skip = lexer.table.command(class) {
                        writeln!(fmt, "return {}_next(this);", name)?;
                    } else {
                        writeln!(fmt, "return {}_Item_newToken({}, start_index, this->index);", name, lexer.vocab[class])?;
                    }
                } else {
                    writeln!(fmt, "goto sink;")?;
                }
            } else {
                writeln!(fmt, "if (this->index >= this->length) goto sink;")?;
                writeln!(fmt, "uint8_t ch = this->input[this->index];")?;
                // writeln!(fmt, "NEXT_CHAR(ch)")?;

                // `i` is a labelled state
                if let Some(class) = state.class {
                    let ttype = if let Command::Skip = lexer.table.command(class) {
                        "TT_SKIP"
                    } else {
                        &lexer.vocab[class]
                    };

                    // If `i` can only transition to labelled states, then either
                    // its corresponding token will be returned immediately or the
                    // token of any destination state will take priority (due to
                    // maximal munch). As such, there would be no need for the
                    // last accept token type or index to be stored. 
                    if state.can_transition_to_unlabelled_state {
                        writeln!(fmt, "last_accept_ttype = {};", ttype)?;
                        writeln!(fmt, "last_accept_index = this->index;")?;
                    }
                }

                // Transitions to non-sink states. Semantically speaking, if any no
                // transition is taken, then we transition to the sink state.
                for Transition { intervals, dest } in &state.transitions {
                    write!(fmt, "if (")?;

                    for (i, &(start, end)) in intervals.iter().enumerate() {
                        if i > 0 {
                            write!(fmt, "    ")?;
                        }

                        #[allow(clippy::comparison_chain)]
                        if start + 1 < end {
                            write!(fmt, "(0x{:02x?} <= ch && ch <= 0x{:02x?})", start, end)?;
                        } else if start + 1 == end {
                            writeln!(fmt, "ch == 0x{:02x?} ||", start)?;
                            write!(fmt, "    ch == 0x{:02x?}", end)?;
                        } else {
                            write!(fmt, "ch == 0x{:02x?}", start)?;
                        }

                        if i < intervals.len() - 1 {
                            writeln!(fmt, " ||")?;
                        }
                    }

                    writeln!(fmt, ") {{ ++this->index; goto s{}; }}", dest)?;
                    // writeln!(fmt, ") GOTO({});", dest)?;
                }
                
                // `i` is a labelled state
                if let Some(class) = state.class {
                    if let Command::Skip = lexer.table.command(class) {
                        writeln!(fmt, "return {}_next(this);", name)?;
                    } else {
                        writeln!(fmt, "return {}_Item_newToken({}, start_index, this->index);", name, lexer.vocab[class])?;
                    }
                } else {
                    writeln!(fmt, "goto sink;")?;
                }
            }

            fmt.unindent();
            writeln!(fmt, "}}")?;
        }

        fmt.build()
    };

    let context = Context {
        name: name.to_string(),
        ttype_labels: lexer.vocab.to_vec(),
    };

    let mut tt = TinyTemplate::new();
    tt.add_template("lexer0", include_str!("templates/lexer0.c.tt")).unwrap();
    tt.add_template("lexer1", include_str!("templates/lexer1.c.tt")).unwrap();

    Ok(format!("{}\n\n{}\n{}", tt.render("lexer0", &context).unwrap(), table_code, tt.render("lexer1", &context).unwrap()))
}

// fn hex(x: u8) -> String {
//     format!("0x{:02x?}", x)
// }

// ================
// === INTERNAL ===
// ================

struct LexerSer {
    states: Vec<State>,
}

struct State {
    transitions: Vec<Transition>,
    can_transition_to_unlabelled_state: bool,
    class: Option<usize>,
}

struct Transition {
    intervals: Vec<(u8, u8)>,
    dest: usize,
}

impl LexerSer {
    fn new(lexer: &Lexer) -> Self {
        Self {
            states: lexer.table.next.chunks_exact(256).enumerate().map(|(i, row)| State::new(lexer, i, row)).collect(),
        }
    }
}

impl State {
    fn new(lexer: &Lexer, index: usize, row: &[usize]) -> Self {
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
            transitions: transitions.into_iter().map(|(dest, intervals)| Transition { intervals, dest }).collect(),
            can_transition_to_unlabelled_state,
            class: lexer.table.class(index),
        }
    }
}

// {{ for state in states -}}
//     s{ @index }: \{
//         NEXT_CHAR(ch)
//         {{ for transition in state.transitions -}}
//         if ({{ for interval in transition.intervals -}}
//             ({ interval.0 } <= ch && ch <= { interval.1 })
//             {{- if not @last }} ||
//             {{ endif }}
//             {{- endfor }}) goto s{ transition.dest };
//         {{- if not @last }}
//         {{ else }}{{ endif }}
//         {{- endfor }}
//     }

//     {{ endfor }}