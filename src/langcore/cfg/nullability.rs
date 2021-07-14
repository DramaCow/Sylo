use super::{Grammar, Symbol};

#[must_use]
pub fn nullability(grammar: &Grammar) -> Vec<bool> {
    let mut nullable = vec![false; grammar.var_count()];

    let mut done = false;

    while !done {
        done = true;

        for (A, rule) in grammar.rules().enumerate() {
            if !nullable[A] {
                nullable[A] = rule.alts().any(|alt| {
                    alt.iter().all(|&symbol| {
                        match symbol {
                            Symbol::Terminal(_) => false,
                            Symbol::Variable(B) => nullable[B],
                        }
                    })
                });
    
                if nullable[A] {
                    done = false;
                }
            }
        }
    }

    nullable
}