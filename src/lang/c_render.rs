use std::collections::{
    HashMap,
    BTreeSet,
};
use crate::debug::StringBuilder;
use super::{
    Lexer,
    Vocabulary,
    re::ScanningTable,
};

#[allow(clippy::too_many_lines)]
#[must_use]
pub fn render_lexer(lexer: &Lexer, name: &str) -> String {
    let mut fmt = StringBuilder::new();

    let scan_type      = &format!("struct {}Scan", name);
    let ttype_type     = &format!("enum {}TokenType", name);
    let token_type     = &format!("struct {}Token", name);
    let error_type     = &format!("struct {}Error", name);
    let scan_item_type = &format!("struct {}Scan_Item", name);

    #[allow(clippy::useless_format)]
    let code = format!("\
#include <stddef.h>
#include <stdint.h>

#define NEXT_CHAR(CHAR) \\
    do {{ \\
        uint8_t CHAR = this->input[index]; \\
        if (++index == this->length) goto done; \\
    }} while (0);

{scan_type} {{
    const uint8_t *const input;
    const size_t length;
    size_t index;
}};

{ttype_type} {{
    // todo
}};

{token_type} {{
    {ttype_type} type;
    size_t span_start;
    size_t span_end;
}};

{error_type} {{
    size_t pos;
}};

{scan_item_type} {{
    enum {{ OK = 0, ERR = 1, NONE = -1 }} tag;
    union {{
        {token_type} token;
        {error_type} error;
    }};
}};

{scan_item_type} {name}Scan_next({scan_type} *this) {{
    size_t index = this->index;
    {ttype_type} last_accept_ttype;
    size_t last_accept_index = 0;

    // todo
}}

int main() {{
    return 0;
}}
",
    name = name,
    scan_type = scan_type,
    ttype_type = ttype_type,
    token_type = token_type,
    error_type = error_type,
    scan_item_type = scan_item_type,
);

    fmt.writeln(&code);

    // fmt.write(ttype_type).writeln(" {").indent();
    // for ttype in &lexer.vocab.symbolic_names {
    //     fmt.write(&ttype.to_uppercase()).writeln(",");
    // }
    // fmt.unindent().writeln("};");
    // fmt.newline();

    // ===

    let next_char = "NEXT_CHAR(ch)";

    for (i, row) in lexer.table.next.chunks_exact(256).enumerate() {
        //
        let mut state2chars: HashMap<usize, BTreeSet<u8>> = HashMap::new();
        let mut can_transition_to_unlabelled_state = false; // NOTE: I consider the sink a "labelled" state
        for (ch, state) in (0..=255_u8).zip(row.iter().copied()) {
            if state != lexer.table.sink() {
                state2chars.entry(state).or_default().insert(ch);

                if lexer.table.class(state).is_none() {
                    can_transition_to_unlabelled_state = true;
                }
            }
        }

        fmt.write("s").write(&i.to_string()).write(": { ").writeln(next_char).indent();

        // `i` is a labelled state
        if let Some(class) = lexer.table.class(i) {
            let ttype = &lexer.vocab.symbolic_names[class].to_uppercase(); // TODO:

            // If `i` can only transition to labelled states, then either
            // its corresponding token will be returned immediately or the
            // token of any destination state will take priority (due to
            // maximal munch). As such, there would be no need for the
            // last accept token type or index to be stored. 
            if can_transition_to_unlabelled_state {
                fmt.write("last_accept_ttype = ").write(ttype).writeln(";");
                fmt.writeln("last_accept_index = index;");
            }
        }
        
        for (&state, chars) in &state2chars {
            let mut iter = chars.iter().copied();

            if let Some(ch0) = iter.next() {
                fmt.write("if (");
                
                let mut start = ch0;
                let mut end = ch0;

                for ch in iter {
                    if end + 1 == ch {
                        end = ch;
                    } else {
                        if start + 1 < end {
                            fmt.write("(").write(&hex(start)).write(" <= ch && ch <= ").write(&hex(end)).writeln(") ||").write("    ");
                        } else if start == end {
                            fmt.write("ch == ").write(&hex(start)).writeln(" ||").write("    ");
                        } else {
                            fmt.write("ch == ").write(&hex(start)).writeln(" ||").write("    ").write("ch == ").write(&hex(end)).writeln(" ||").write("    ");
                        }

                        fmt.write("(").write(&hex(start)).write(" <= ch && ch <= ").write(&hex(end)).writeln(") ||").write("    ");
                        start = ch;
                        end = ch;
                    }
                }

                if start + 1 < end {
                    fmt.write("(").write(&hex(start)).write(" <= ch && ch <= ").write(&hex(end)).write(")");
                } else if start == end {
                    fmt.write("ch == ").write(&hex(start));
                } else {
                    fmt.write("ch == ").write(&hex(start)).writeln(" ||").write("    ").write("ch == ").write(&hex(end));
                }
                
                fmt.write(") goto s").write(&state.to_string()).writeln(";");                
            }
        }

        // `i` is a labelled state
        if let Some(class) = lexer.table.class(i) {
            let ttype = &lexer.vocab.symbolic_names[class].to_uppercase(); // TODO:
            fmt.unindent().write("} return (").write(token_type).writeln(") { OK };").newline();
        } else {
            fmt.unindent().writeln("} goto sink;").newline();
        }

    }

    fmt.build()
}

fn hex(x: u8) -> String {
    format!("0x{:02x?}", x)
}