#[macro_export]
macro_rules! lexer_def {
    ($($([$lexcmd:ident])? $lexlbl:ident : $regex:expr),+ $(,)?) => {
        $crate::_lexer_def_internal![_ 0_usize ; [] $($([$lexcmd])? $lexlbl : $regex),+]
    };
}

#[macro_export]
macro_rules! parser_def {
    ({ $($([$lexcmd:ident])? $lexlbl:ident : $regex:expr),+ $(,)? } , { $($([$syncmd:ident])? $synlbl:ident : $($($symbol:ident)*)|*),+ $(,)? } $(,)?) => {
        {
            $crate::_lexer_def_internal![__LEX_DEF__ 0_usize ; [] $($([$lexcmd])? $lexlbl : $regex),+];
            $crate::_parser_def_internal![__LEX_DEF__ 0_usize ; [] $($([$syncmd])? $synlbl : $($($symbol)*)|*),+]
        }
    };
}

// =================
// === INTERNALS ===
// =================

#[doc(hidden)]
#[macro_export]
macro_rules! _lexer_command {
    (emit) => { $crate::lang::lex::Command::Emit };
    (skip) => { $crate::lang::lex::Command::Skip };
}

#[doc(hidden)]
#[macro_export]
macro_rules! _lexer_def_internal {
    ($out:tt $count:expr ; [$($body:tt)*] $label:ident : $regex:expr , $($tail:tt)+) => {
        $crate::_lexer_def_internal![$out $count + 1_usize ; [$($body)* $count , emit $label $regex ;] $($tail)*]
    };
    ($out:tt $count:expr ; [$($body:tt)*] [$command:ident] $label:ident : $regex:expr , $($tail:tt)+) => {
        $crate::_lexer_def_internal![$out $count + 1_usize ; [$($body)* $count , $command $label $regex ;] $($tail)*]
    };
    ($out:tt $count:expr ; [$($body:tt)*] $label:ident : $regex:expr $(,)?) => {
        $crate::_lexer_def_internal![@ $out $count + 1_usize ; $($body)* $count , emit $label $regex]
    };
    ($out:tt $count:expr ; [$($body:tt)*] [$command:ident] $label:ident : $regex:expr $(,)?) => {
        $crate::_lexer_def_internal![@ $out $count + 1_usize ; $($body)* $count , $command $label $regex]
    };
    (@ _ $count:expr ; $($id:expr , $command:ident $label:ident $regex:expr);+) => {
        $crate::lang::LexerDef {
            vocab: $crate::lang::Vocabulary::new(vec![$(stringify!($label).to_string()),+]),
            regexes: vec![$($regex),+],
            commands: vec![$($crate::_lexer_command![$command]),+]
        }
    };
    (@ $out:ident $count:expr ; $($id:expr , $command:ident $label:ident $regex:expr);+) => {
        $(
            #[allow(non_upper_case_globals)]
            const $label: $crate::lang::cfg::Symbol = $crate::lang::cfg::Symbol::Terminal($id);
        )+
        const __WORD_COUNT__: usize = $count;
        let $out = _lexer_def_internal![@ _ $count ; $($id , $command $label $regex);+];
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! _parser_command {
    (emit) => { $crate::lang::Command::Emit };
    (skip) => { $crate::lang::Command::Skip };
}

#[doc(hidden)]
#[macro_export]
macro_rules! _parser_def_internal {
    ($lexer_def:ident $count:expr ; [$($body:tt)*] $label:ident : $($($symbol:ident)*)|* , $($tail:tt)+) => {
        $crate::_parser_def_internal![$lexer_def $count + 1_usize ; [$($body)* $count , emit $label &[$(&[$($symbol),*]),*] ;] $($tail)+]
    };
    ($lexer_def:ident $count:expr ; [$($body:tt)*] [$command:ident] $label:ident : $($($symbol:ident)*)|* , $($tail:tt)+) => {
        $crate::_parser_def_internal![$lexer_def $count + 1_usize ; [$($body)* $count , $command $label &[$(&[$($symbol),*]),*] ;] $($tail)+]
    };
    ($lexer_def:ident $count:expr ; [$($body:tt)*] $label:ident : $($($symbol:ident)*)|* $(,)?) => {
        $crate::_parser_def_internal![@ $lexer_def $($body)* $count , emit $label &[$(&[$($symbol),*]),*]]
    };
    ($lexer_def:ident $count:expr ; [$($body:tt)*] [$command:ident] $label:ident : $($($symbol:ident)*)|* $(,)?) => {
        $crate::_parser_def_internal![@ $lexer_def $($body)* $count , $command $label &[$(&[$($symbol),*]),*]]
    };
    (@ $lexer_def:ident $($id:expr , $command:ident $label:ident $rule:expr);+) => {
        {
            $(
                #[allow(non_upper_case_globals)]
                const $label: $crate::lang::cfg::Symbol = $crate::lang::cfg::Symbol::Variable($id); 
            )+
            $crate::lang::ParserDef {
                lexer_def: $lexer_def,
                var_names: vec![$(stringify!($label).to_string()),+],
                grammar: $crate::lang::cfg::GrammarBuilder::new()$(.rule($rule))+.try_build().unwrap(),
                commands: vec![$($crate::_parser_command![$command]),+],
            }
        }
    };
}