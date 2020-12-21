#[macro_export]
macro_rules! lex_def {
    (@accum $out:tt $count:expr ; [$($body:tt)*] $label:ident : $regex:expr , $($tail:tt)+) => {
        $crate::lex_def![@accum $out $count + 1_usize ; [$($body)* $count , emit $label $regex;] $($tail)*]
    };
    (@accum $out:tt $count:expr ; [$($body:tt)*] [$command:ident] $label:ident : $regex:expr , $($tail:tt)+) => {
        $crate::lex_def![@accum $out $count + 1_usize ; [$($body)* $count , $command $label $regex ;] $($tail)*]
    };
    (@accum $out:tt $count:expr ; [$($body:tt)*] $label:ident : $regex:expr $(,)?) => {
        $crate::lex_def![@fin $out $count + 1_usize ; $($body)* $count , emit $label $regex]
    };
    (@accum $out:tt $count:expr ; [$($body:tt)*] [$command:ident] $label:ident : $regex:expr $(,)?) => {
        $crate::lex_def![@fin $out $count + 1_usize ; $($body)* $count , $command $label $regex]
    };
    (@command emit) => { $crate::lang::lex::LexCommand::Emit };
    (@command skip) => { $crate::lang::lex::LexCommand::Skip };
    (@fin _ $count:expr ; $($id:expr , $command:ident $label:ident $regex:expr);+) => {
        &$crate::lang::lex::LexAnalyzerDef {
            labels: vec![$(stringify!($label).to_string()),+],
            regexes: vec![$($regex),+],
            commands: vec![$($crate::lex_def![@command $command]),+],
        }
    };
    (@fin $out:ident $count:expr ; $($id:expr , $command:ident $label:ident $regex:expr);+) => {
        let $out = $crate::lang::lex::LexAnalyzerDef {
            labels: vec![$(stringify!($label).to_string()),+],
            regexes: vec![$($regex),+],
            commands: vec![$($crate::lex_def![@command $command]),+],
        };
        $(
            #[allow(non_upper_case_globals)]
            const $label: $crate::lang::cfg::Symbol = $crate::lang::cfg::Symbol::Terminal($id);
        )+
        const __WORD_COUNT__: usize = $count;
    };
    ($($body:tt)*) => {
        $crate::lex_def![@accum _ 0_usize ; [] $($body)*]
    };
}

#[macro_export]
macro_rules! syn_def {
    (@count $id:expr ; $word:ident $($tail:tt)+) => {
        #[allow(non_upper_case_globals)]
        const $word: $crate::lang::cfg::Symbol = $crate::lang::cfg::Symbol::Terminal($id);
        $crate::syn_def![@count $id + 1_usize ; $($tail)+]
    };
    (@count $id:expr ; _ $($tail:tt)+) => {
        $crate::syn_def![@count $id + 1_usize ; $($tail)+]
    };
    (@count $id:expr ; $word:ident) => {
        #[allow(non_upper_case_globals)]
        const $word: $crate::lang::cfg::Symbol = $crate::lang::cfg::Symbol::Terminal($id);
        const __WORD_COUNT__: usize = $id + 1_usize;
    };
    (@count $id:expr ; _) => {
        const __WORD_COUNT__: usize = $id + 1_usize;
    };
    (@accum $n:expr ; $count:expr ; [$($body:tt)*] $label:ident : $($($symbol:ident)*)|* , $($tail:tt)+) => {
        $crate::syn_def![@accum $n ; $count + 1_usize ; [$($body)* emit $label $count , &[$(&[$($symbol),*]),*] ;] $($tail)+]
    };
    (@accum $n:expr ; $count:expr ; [$($body:tt)*] [$command:ident] $label:ident : $($($symbol:ident)*)|* , $($tail:tt)+) => {
        $crate::syn_def![@accum $n ; $count + 1_usize ; [$($body)* $command $label $count , &[$(&[$($symbol),*]),*] ;] $($tail)+]
    };
    (@accum $n:expr ; $count:expr ; [$($body:tt)*] $label:ident : $($($symbol:ident)*)|* $(,)?) => {
        $crate::syn_def![@fin $n ; $($body)* emit $label $count , &[$(&[$($symbol),*]),*]]
    };
    (@accum $n:expr ; $count:expr ; [$($body:tt)*] [$command:ident] $label:ident : $($($symbol:ident)*)|* $(,)?) => {
        $crate::syn_def![@fin $n ; $($body)* $command $label $count , &[$(&[$($symbol),*]),*]]
    };
    (@command emit) => { $crate::lang::syn::SynCommand::Emit };
    (@command skip) => { $crate::lang::syn::SynCommand::Skip };
    (@fin $n:expr ; $($command:ident $label:ident $id:expr , $rule:expr);+) => {
        {
            $(
                #[allow(non_upper_case_globals)]
                const $label: $crate::lang::cfg::Symbol = $crate::lang::cfg::Symbol::Variable($id); 
            )+

            $crate::lang::syn::SynAnalyzerDef {
                labels: vec![$(stringify!($label).to_string()),+],
                grammar: $crate::lang::cfg::GrammarBuilder::new($n)$(.rule($rule))+.try_build().unwrap(),
                commands: vec![$($crate::syn_def![@command $command]),+],
            }
        }
    };
    ({ $($word:tt),+ $(,)? } $(;)? $($tail:tt)*) => {
        {
            $crate::syn_def![@count 0_usize ; $($word)+];
            $crate::syn_def![@accum __WORD_COUNT__ ; 0_usize ; [] $($tail)*]
        }
    };
}

#[macro_export]
macro_rules! parser_def {
    ($(lexer :)? { $($lexer:tt)* } , $(parser :)? { $($parser:tt)* } $(,)?) => {
        {
            $crate::lex_def![@accum lex_def 0_usize ; [] $($lexer)*];
            let syn_def = $crate::syn_def![@accum __WORD_COUNT__ ; 0_usize ; [] $($parser)*];
            $crate::lang::parser::ParserDef { lex_def, syn_def }
        }
    };
}