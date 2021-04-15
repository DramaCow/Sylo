#[macro_export]
macro_rules! lexer_def {
    ($($([$lexcmd:ident])? $lexlbl:ident : $regex:expr),+ $(,)?) => {
        $crate::_lexer_def_internal![_ 0_usize ; [] $($([$lexcmd])? $lexlbl : $regex),+]
    };
}

#[macro_export]
macro_rules! parser_def {
    ({ $($([$lexcmd:ident])? $lexlbl:ident : $regex:expr),+ $(,)? } , $({ $(% $assoc:ident $($token:ident)+)* },)? { $($([$syncmd:ident])? $synlbl:ident : $($($symbol:ident)*)|*),+ $(,)? } $(,)?) => {
        {
            $crate::_lexer_def_internal![__LEX_DEF__ 0_usize ; [] $($([$lexcmd])? $lexlbl : $regex),+];
            let mut parser_def = $crate::_parser_def_internal![__LEX_DEF__ 0_usize ; [] $($([$syncmd])? $synlbl : $($($symbol)*)|*),+];
            $(
                let mut __TOKEN_PRECEDENCE__: Vec<Option<$crate::lang::lr::Precedence>> = vec![None; __WORD_COUNT__];
                $crate::_precedence_internal![__TOKEN_PRECEDENCE__ 0_usize ; [] $(% $assoc $($token)+)*];
                parser_def.attach_precedence(__TOKEN_PRECEDENCE__);
            )?
            parser_def
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
macro_rules! _precedence_internal {
    ($token_precedence:ident $count:expr ; [ $($body:tt)* ] % $assoc:ident $($token:ident)+ % $($tail:tt)+) => {
        $crate::_precedence_internal![$token_precedence $count + 1_usize ; [ $($body)* $count , $assoc $($token)+ ; ] % $($tail)+]
    };
    ($token_precedence:ident $count:expr ; [ $($body:tt)* ] % $assoc:ident $($token:ident)+) => {
        $crate::_precedence_internal![@ $token_precedence $($body)* $count , $assoc $($token)+]
    };
    (@ $token_precedence:ident $($id:expr , $assoc:ident $($token:ident)+);+) => {
        {
            $(
                $(
                    if let $crate::lang::cfg::Symbol::Terminal(a) = $token {
                        $token_precedence[a] = Some($crate::lang::lr::Precedence::left($id));
                    }
                )+
            )+
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! _parser_def_internal {
    ($lexer_def:ident $count:expr ; [$($body:tt)*] $label:ident : $($($symbol:ident)*)|* , $($tail:tt)+) => {
        $crate::_parser_def_internal![$lexer_def $count + 1_usize ; [$($body)* $count , $label &[$(&[$($symbol),*]),*] ;] $($tail)+]
    };
    ($lexer_def:ident $count:expr ; [$($body:tt)*] $label:ident : $($($symbol:ident)*)|* $(,)?) => {
        $crate::_parser_def_internal![@ $lexer_def $($body)* $count , $label &[$(&[$($symbol),*]),*]]
    };
    (@ $lexer_def:ident $($id:expr , $label:ident $rule:expr);+) => {
        {
            $(
                #[allow(non_upper_case_globals)]
                const $label: $crate::lang::cfg::Symbol = $crate::lang::cfg::Symbol::Variable($id); 
            )+
            $crate::lang::ParserDef::new(
                $lexer_def,
                vec![$(stringify!($label).to_string()),+],
                $crate::lang::cfg::GrammarBuilder::new()$(.rule($rule))+.try_build().unwrap(),
            )
        }
    };
}