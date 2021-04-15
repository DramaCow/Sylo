#[derive(Debug, Clone)]
pub struct Precedence {
    level: usize,
    associativity: Associativity,
}

#[derive(Debug, Clone)]
pub enum Associativity {
    Left,
    Right,
    Nonassoc,
}

impl Precedence {
    #[must_use]
    pub fn left(level: usize) -> Self {
        Self { level, associativity: Associativity::Left }
    }

    #[must_use]
    pub fn right(level: usize) -> Self {
        Self { level, associativity: Associativity::Right }
    }

    #[must_use]
    pub fn nonassoc(level: usize) -> Self {
        Self { level, associativity: Associativity::Nonassoc }
    }
}