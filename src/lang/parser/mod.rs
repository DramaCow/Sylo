use crate::lang::lex;
use crate::lang::syn;

#[derive(Clone)]
pub enum Command {
    Skip,
    Emit,
}

pub struct ParserDef {
    pub lex_def: lex::LexAnalyzerDef,
    pub syn_def: syn::SynAnalyzerDef,
    pub commands: Vec<Command>,
}

pub struct Parser {
    pub(crate) lex: lex::LexAnalyzer,
    pub(crate) syn: syn::SynAnalyzer,
    pub(crate) commands: Vec<Command>
}

impl Parser {
    /// # Errors
    pub fn try_compile(def: &ParserDef) -> Result<Self, syn::CompileError> {
        Ok(Self {
            lex: lex::LexAnalyzer::compile(&def.lex_def),
            syn: syn::SynAnalyzer::try_compile(&def.syn_def)?,
            commands: def.commands.to_vec(),
        })
    }

    // /// # Errors
    // pub fn cst<'a>(&'a self, text: &'a str) -> Result<CST, syn::ParseError> {
    //     let tokens = self.lex.parse(text).collect::<Result<Vec<_>, _>>().unwrap();
    //     let tokens2 = self.lex.parse(text).map(|res| res.and_then(|token| Ok(token.class)));

    //     let mut builder = CSTBuilder::new();

    //     for step in self.syn.parse(tokens.iter().map(|token| token.class)) {
    //         match step? {
    //             syn::Instruction::Shift { word, index } => builder.leaf(word, index),
    //             syn::Instruction::Reduce { var, count } => {
    //                 match self.syn.commands.get(var).unwrap() {
    //                     syn::Command::Emit => builder.branch(var, count),
    //                     syn::Command::Skip => builder.list(count),
    //                 };
    //             },
    //         }
    //     }

    //     Ok(builder.build())
    // }
}