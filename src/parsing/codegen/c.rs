use std::fmt::Write;
use crate::utils::IndentWriter;
use crate::langcore::re::{LexTable, Command};
use crate::lexer::Lexer;
use super::rep;

/// # Errors
#[allow(clippy::too_many_lines)]
pub fn lexer<W: Write>(fmt: W, name: &str, lexer: &Lexer) -> Result<W, std::fmt::Error> {
    let mut fmt = IndentWriter::new(fmt);
    let ttypes: Vec<_> = lexer.vocab.iter().map(|s| s.to_ascii_uppercase()).collect();

    writeln!(fmt, "#include <stddef.h>")?;
    writeln!(fmt, "#include <stdint.h>")?;
    writeln!(fmt)?;
    writeln!(fmt, "struct {name} {{", name=name)?;
    writeln!(fmt, "    const uint8_t *const input;")?;
    writeln!(fmt, "    const size_t length;")?;
    writeln!(fmt, "    size_t index;")?;
    writeln!(fmt, "}};")?;
    writeln!(fmt)?;
    writeln!(fmt, "enum {name}_TokenType {{", name=name)?;
    for (_, ttype) in ttypes.iter().enumerate().filter(|(i, _)| if let Command::Emit = lexer.table.commands[*i] { true } else { false }) {
        writeln!(fmt, "    {ttype},", ttype=ttype)?;
    }
    writeln!(fmt, "    TT_ERROR = -1,")?;
    writeln!(fmt, "    TT_SKIP = -2,")?;
    writeln!(fmt, "}};")?;
    writeln!(fmt)?;
    writeln!(fmt, "struct {name}_Token {{", name=name)?;
    writeln!(fmt, "    enum {name}_TokenType type;", name=name)?;
    writeln!(fmt, "    size_t span_start;")?;
    writeln!(fmt, "    size_t span_end;")?;
    writeln!(fmt, "}};")?;
    writeln!(fmt)?;
    writeln!(fmt, "struct {name}_Error {{", name=name)?;
    writeln!(fmt, "    size_t pos;")?;
    writeln!(fmt, "}};")?;
    writeln!(fmt)?;
    writeln!(fmt, "struct {}_Item {{", name)?;
    writeln!(fmt, "    enum {{ OK = 0, ERR = 1, NONE = -1 }} tag;")?;
    writeln!(fmt, "    union {{")?;
    writeln!(fmt, "        struct {name}_Token token;", name=name)?;
    writeln!(fmt, "        struct {name}_Error error;", name=name)?;
    writeln!(fmt, "    }};")?;
    writeln!(fmt, "}};")?;
    writeln!(fmt)?;
    writeln!(fmt, "static inline struct {name}_Item {name}_Item_newToken(enum {name}_TokenType type, size_t span_start, size_t span_end) {{", name=name)?;
    writeln!(fmt, "    return (struct {}_Item) {{ .tag = OK, .token = {{ type, span_start, span_end }} }};", name)?;
    writeln!(fmt, "}}")?;
    writeln!(fmt)?;
    writeln!(fmt, "static inline struct {name}_Item {name}_Item_newError(size_t pos) {{", name=name)?;
    writeln!(fmt, "    return (struct {name}_Item) {{ .tag = ERR, .error = pos }};", name=name)?;
    writeln!(fmt, "}}")?;
    writeln!(fmt)?;
    writeln!(fmt, "static inline struct {name}_Item {name}_Item_newNone() {{", name=name)?;
    writeln!(fmt, "    return (struct {name}_Item) {{ .tag = NONE }};", name=name)?;
    writeln!(fmt, "}}")?;
    writeln!(fmt)?;
    writeln!(fmt, "struct {name} {name}_new(const uint8_t *input, size_t length) {{", name=name)?;
    writeln!(fmt, "    return (struct {name}) {{ .input = input, .length = length, .index = 0 }};", name=name)?;
    writeln!(fmt, "}}")?;
    writeln!(fmt)?;
    writeln!(fmt, "struct {name}_Item {name}_next(struct {name} *const this) {{", name=name)?;
    writeln!(fmt, "    if (this->index >= this->length) {{")?;
    writeln!(fmt, "        return {name}_Item_newNone();", name=name)?;
    writeln!(fmt, "    }}")?;
    writeln!(fmt)?;
    writeln!(fmt, "    size_t start_index = this->index;")?;
    writeln!(fmt, "    enum {name}_TokenType last_accept_ttype = TT_ERROR;", name=name)?;
    writeln!(fmt, "    size_t last_accept_index;")?;
    writeln!(fmt)?;
    writeln!(fmt, "    // *** LEXER TABLE START ***")?;
    fmt.indent();        
    for (i, state) in rep::Lexer::new(name, lexer).states.iter().enumerate() {
        writeln!(fmt, "s{}: {{", i)?;
        fmt.indent();

        if state.transitions.is_empty() {
            // `i` is a labelled state
            if let Some(class) = state.class {
                if let Command::Skip = lexer.table.command(class) {
                    writeln!(fmt, "return {name}_next(this);", name=name)?;
                } else {
                    writeln!(fmt, "return {name}_Item_newToken({ttype}, start_index, this->index);", name=name, ttype=ttypes[class])?;
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
                    &ttypes[class]
                };

                // If `i` can only transition to labelled states, then either
                // its corresponding token will be returned immediately or the
                // token of any destination state will take priority (due to
                // maximal munch). As such, there would be no need for the
                // last accept token type or index to be stored. 
                if state.can_transition_to_unlabelled_state {
                    writeln!(fmt, "last_accept_ttype = {ttype};", ttype=ttype)?;
                    writeln!(fmt, "last_accept_index = this->index;")?;
                }
            }

            // Transitions to non-sink states. Semantically speaking, if any no
            // transition is taken, then we transition to the sink state.
            for rep::Transition { intervals, dst } in &state.transitions {
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

                writeln!(fmt, ") {{ ++this->index; goto s{dst}; }}", dst=dst)?;
                // writeln!(fmt, ") GOTO({});", dst)?;
            }
            
            // `i` is a labelled state
            if let Some(class) = state.class {
                if let Command::Skip = lexer.table.command(class) {
                    writeln!(fmt, "return {}_next(this);", name)?;
                } else {
                    writeln!(fmt, "return {}_Item_newToken({}, start_index, this->index);", name, ttypes[class])?;
                }
            } else {
                writeln!(fmt, "goto sink;")?;
            }
        }

        fmt.unindent();
        writeln!(fmt, "}}")?;
    }
    fmt.unindent();
    writeln!(fmt, "    // ***  LEXER TABLE END  ***")?;
    writeln!(fmt)?;
    writeln!(fmt, "    sink:")?;
    writeln!(fmt, "        if (last_accept_ttype == TT_ERROR) {{")?;
    writeln!(fmt, "            size_t pos = this->index;")?;
    writeln!(fmt, "            this->index = -1; // forces next iteration to return None")?;
    writeln!(fmt, "            return {name}_Item_newError(pos);", name=name)?;
    writeln!(fmt, "        }} else if (last_accept_ttype == TT_SKIP) {{")?;
    writeln!(fmt, "            this->index = last_accept_index;")?;
    writeln!(fmt, "            return {name}_next(this);", name=name)?;
    writeln!(fmt, "        }} else {{")?;
    writeln!(fmt, "            this->index = last_accept_index;")?;
    writeln!(fmt, "            return {name}_Item_newToken(last_accept_ttype, start_index, last_accept_index);", name=name)?;
    writeln!(fmt, "        }}")?;
    writeln!(fmt, "}}")?;

    Ok(fmt.build())
}