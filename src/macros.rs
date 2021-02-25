#[macro_export]
macro_rules! lexer_def {
    ($($([$command:ident])? $label:ident : $regex:expr),+ $(,)?) => {
        $crate::_lexer_def_internal![@accum _ 0_usize ; [] $($([$command])? $label : $regex),+]
    };
}

// =================
// === INTERNALS ===
// =================

#[doc(hidden)]
#[macro_export]
macro_rules! _lexer_command {
    (emit) => { $crate::lang::Command::Emit };
    (skip) => { $crate::lang::Command::Skip };
}

#[doc(hidden)]
#[macro_export]
macro_rules! _lexer_def_internal {
    (@accum $out:tt $count:expr ; [$($body:tt)*] $label:ident : $regex:expr , $($tail:tt)+) => {
        $crate::_lexer_def_internal![@accum $out $count + 1_usize ; [$($body)* $count , emit $label $regex;] $($tail)*]
    };
    (@accum $out:tt $count:expr ; [$($body:tt)*] [$command:ident] $label:ident : $regex:expr , $($tail:tt)+) => {
        $crate::_lexer_def_internal![@accum $out $count + 1_usize ; [$($body)* $count , $command $label $regex ;] $($tail)*]
    };
    (@accum $out:tt $count:expr ; [$($body:tt)*] $label:ident : $regex:expr $(,)?) => {
        $crate::_lexer_def_internal![@fin $out $count + 1_usize ; $($body)* $count , emit $label $regex]
    };
    (@accum $out:tt $count:expr ; [$($body:tt)*] [$command:ident] $label:ident : $regex:expr $(,)?) => {
        $crate::_lexer_def_internal![@fin $out $count + 1_usize ; $($body)* $count , $command $label $regex]
    };
    (@fin _ $count:expr ; $($id:expr , $command:ident $label:ident $regex:expr);+) => {
        $crate::lang::LexerDef {
            labels: vec![$(stringify!($label).to_string()),+],
            lex_def: $crate::lang::lex::LexDef {
                regexes: vec![$($regex),+],
                commands: vec![$($crate::_lexer_command![$command]),+],
            },
            commands: vec![$($crate::_lexer_command![$command]),+]
        }
    };
    (@fin $out:ident $count:expr ; $($id:expr , $command:ident $label:ident $regex:expr);+) => {
        $(
            #[allow(non_upper_case_globals)]
            const $label: $crate::lang::cfg::Symbol = $crate::lang::cfg::Symbol::Terminal($id);
        )+
        const __WORD_COUNT__: usize = $count;
        let $out = _lexer_def_internal![@fin _ $count ; $($id , $command $label $regex);+];
    };
}

#[doc(hidden)]
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
    
            let syn_def = $crate::lang::syn::SynDef {
                grammar: $crate::lang::cfg::GrammarBuilder::new($n)$(.rule($rule))+.try_build().unwrap(),
                word_count: $n,
            };

            (vec![$(stringify!($label).to_string()),+], syn_def)
        }
    };
    (@internal $n:expr ; $($grammar:tt)*) => {
        $crate::syn_def![@accum $n ; 0_usize ; [] $($grammar)*]
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! _parser_command {
    (emit) => { $crate::lang::Command::Emit };
    (skip) => { $crate::lang::Command::Skip };
}

#[macro_export]
macro_rules! _parser_def_internal {
    (@accum $lexer_def:ident {$($grammar:tt)*} {$($commands:tt)*} $label:ident : $($($symbol:ident)*)|* , $($tail:tt)+) => {
        $crate::_parser_def_internal![@accum $lexer_def {$($grammar)* $label : $($($symbol)*)|* ,} {$($commands)* $crate::_parser_command![emit],} $($tail)+]
    };
    (@accum $lexer_def:ident {$($grammar:tt)*} {$($commands:tt)*} [$command:ident] $label:ident : $($($symbol:ident)*)|* , $($tail:tt)+) => {
        $crate::_parser_def_internal![@accum $lexer_def {$($grammar)* $label : $($($symbol)*)|* ,} {$($commands)* $crate::_parser_command![$command] ,} $($tail)+]
    };
    (@accum $lexer_def:ident {$($grammar:tt)*} {$($commands:tt)*} $label:ident : $($($symbol:ident)*)|* $(,)?) => {
        $crate::_parser_def_internal![@fin   $lexer_def {$($grammar)* $label : $($($symbol)*)|*}   {$($commands)* $crate::_parser_command![emit]}]
    };
    (@accum $lexer_def:ident {$($grammar:tt)*} {$($commands:tt)*} [$command:ident] $label:ident : $($($symbol:ident)*)|* $(,)?) => {
        $crate::_parser_def_internal![@fin   $lexer_def {$($grammar)* $label : $($($symbol)*)|*}   {$($commands)* $crate::_parser_command![$command]}]
    };
    (@fin $lexer_def:ident {$($grammar:tt)*} {$($commands:tt)*}) => {
        {
            let (syn_labels, __SYN_DEF__) = syn_def![@internal __WORD_COUNT__ ; $($grammar)*];
            $crate::lang::ParserDef {
                lexer_def: $lexer_def,
                syn_labels,
                syn_def: __SYN_DEF__,
                commands: vec![$($commands)*]
            }
        }
    };
}

#[macro_export]
macro_rules! parser_def {
    (lexer : { $($lexer:tt)* } , parser : { $($parser:tt)* } $(,)?) => {
        {
            $crate::_lexer_def_internal![@accum __LEX_DEF__ 0_usize ; [] $($lexer)*];
            $crate::_parser_def_internal![@accum __LEX_DEF__ {} {} $($parser)*]
        }
    };
}