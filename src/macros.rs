#[macro_export]
macro_rules! words {
    (@accum $count:expr ; [$($body:tt)*] $label:ident , $($tail:tt)+) => {
        $crate::words![@accum $count + 1_usize ; [$($body)* $label $count ;] $($tail)*]
    };
    (@accum $count:expr ; [$($body:tt)*] $label:ident $(,)?) => {
        $crate::words![@fin $count + 1_usize ; $($body)* $label $count]
    };
    (@fin $count:expr ; $($label:ident $id:expr);+) => {
        $(
            #[allow(non_upper_case_globals)]
            const $label: $crate::lang::cfg::Symbol = $crate::lang::cfg::Symbol::Terminal($id);
        )+
        const __WORD_COUNT__: usize = $count;
    };
    ($($labels:ident),*) => {
        $crate::words![@accum 0_usize ; [] $($labels),*]
    };
}

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
    (@command emit) => { $crate::lang::lex::Command::Emit };
    (@command skip) => { $crate::lang::lex::Command::Skip };
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
    (@accum $n:expr ; $count:expr ; [$($body:tt)*] $label:ident : $($($symbol:ident)*)|* , $($tail:tt)+) => {
        $crate::syn_def![@accum $n ; $count + 1_usize ; [$($body)* $label $count , &[$(&[$($symbol),*]),*] ;] $($tail)+]
    };
    (@accum $n:expr ; $count:expr ; [$($body:tt)*] $label:ident : $($($symbol:ident)*)|* $(,)?) => {
        $crate::syn_def![@fin $n ; $($body)* $label $count , &[$(&[$($symbol),*]),*]]
    };
    (@fin $n:expr ; $($label:ident $id:expr , $rule:expr);+) => {
        {
            $(
                #[allow(non_upper_case_globals)]
                const $label: $crate::lang::cfg::Symbol = $crate::lang::cfg::Symbol::Variable($id); 
            )+
    
            $crate::lang::syn::SynAnalyzerDef {
                labels: vec![$(stringify!($label).to_string()),+],
                grammar: $crate::lang::cfg::GrammarBuilder::new($n)$(.rule($rule))+.try_build().unwrap(),
            }
        }
    };
    (@internal $n:expr ; $($grammar:tt)*) => {
        $crate::syn_def![@accum $n ; 0_usize ; [] $($grammar)*]
    };
    ({ $($labels:ident),* $(,)? } $(,)? $($grammar:tt)*) => {
        {
            $crate::words![$($labels),*];
            $crate::syn_def![@internal __WORD_COUNT__ ; $($grammar)*]
        }
    };
}

#[macro_export]
macro_rules! parser_cmd {
    (emit) => { $crate::lang::parser::Command::Emit };
    (skip) => { $crate::lang::parser::Command::Skip };
}

#[macro_export]
macro_rules! parser_def {
    (@accum $lex_def:ident {$($grammar:tt)*} {$($commands:tt)*} $label:ident : $($($symbol:ident)*)|* , $($tail:tt)+) => {
        $crate::parser_def![@accum $lex_def {$($grammar)* $label : $($($symbol)*)|* ,} {$($commands)* parser_cmd![emit],} $($tail)+]
    };
    (@accum $lex_def:ident {$($grammar:tt)*} {$($commands:tt)*} [$command:ident] $label:ident : $($($symbol:ident)*)|* , $($tail:tt)+) => {
        $crate::parser_def![@accum $lex_def {$($grammar)* $label : $($($symbol)*)|* ,} {$($commands)* parser_cmd![$command] ,} $($tail)+]
    };
    (@accum $lex_def:ident {$($grammar:tt)*} {$($commands:tt)*} $label:ident : $($($symbol:ident)*)|* $(,)?) => {
        $crate::parser_def![@fin   $lex_def {$($grammar)* $label : $($($symbol)*)|*}   {$($commands)* parser_cmd![emit]}]
    };
    (@accum $lex_def:ident {$($grammar:tt)*} {$($commands:tt)*} [$command:ident] $label:ident : $($($symbol:ident)*)|* $(,)?) => {
        $crate::parser_def![@fin   $lex_def {$($grammar)* $label : $($($symbol)*)|*}   {$($commands)* parser_cmd![$command]}]
    };
    (@fin $lex_def:ident {$($grammar:tt)*} {$($commands:tt)*}) => {
        {
            let __SYN_DEF__ = syn_def![@internal __WORD_COUNT__ ; $($grammar)*];
            $crate::lang::parser::ParserDef { lex_def: $lex_def, syn_def: __SYN_DEF__, commands: vec![$($commands)*] }
        }
    };
    (lexer : { $($lexer:tt)* } , parser : { $($parser:tt)* } $(,)?) => {
        {
            $crate::lex_def![@accum __LEX_DEF__ 0_usize ; [] $($lexer)*];
            $crate::parser_def![@accum __LEX_DEF__ {} {} $($parser)*]
        }
    };
}