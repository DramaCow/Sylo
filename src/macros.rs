#[macro_export]
macro_rules! lexer {
    ($($([$lexcmd:ident])? $lexlbl:ident : $regex:expr),+ $(,)?) => {
        $crate::_lexer_def_internal![_ 0_usize ; [] $($([$lexcmd])? $lexlbl : $regex),+]
    };
}

#[macro_export]
macro_rules! parser {
    ({ $($([$lexcmd:ident])? $lexlbl:ident : $regex:expr),+ $(,)? } , $({ $(% $assoc:ident $($token:ident)+)* },)? { $($([$syncmd:ident])? $synlbl:ident : $($($symbol:ident)*)|+),+ $(,)? } $(,)?) => {
        {
            $crate::_lexer_def_internal![__LEX_DEF__ 0_usize ; [] $($([$lexcmd])? $lexlbl : $regex),+];
            let mut __PARSER_DEF__ = $crate::_parser_def_internal![__LEX_DEF__ 0_usize ; [] $($([$syncmd])? $synlbl : $($($symbol)*)|+),+];
            $(
                $crate::_precedence_internal![__PARSER_DEF__ 0_usize ; [] $(% $assoc $($token)+)*];
            )?
            __PARSER_DEF__.build()
        }
    };
}

// =================
// === INTERNALS ===
// =================

#[doc(hidden)]
#[macro_export]
macro_rules! _lexer_rule {
    ($builder:ident $label:ident $regex:expr , emit) => { $builder.rule(stringify!($label).to_string(), $regex); };
    ($builder:ident $label:ident $regex:expr , skip) => { $builder.skip(stringify!($label).to_string(), $regex); };
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
        {
            let mut lexer = $crate::lexer::LexerDefBuilder::new();
            $(
                $crate::_lexer_rule![lexer $label $regex , $command];
            )+
            lexer.build()
        }
    };
    (@ $out:ident $count:expr ; $($id:expr , $command:ident $label:ident $regex:expr);+) => {
        $(
            #[allow(non_upper_case_globals)]
            const $label: lr_parsing_tools::grammar::Symbol = lr_parsing_tools::grammar::Symbol::Terminal($id);
        )+
        const __WORD_COUNT__: usize = $count;
        let $out = _lexer_def_internal![@ _ $count ; $($id , $command $label $regex);+];
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! _precedence_internal {
    ($parser_def:ident $count:expr ; [ $($body:tt)* ] % $assoc:ident $($token:ident)+ % $($tail:tt)+) => {
        $crate::_precedence_internal![$parser_def $count + 1_usize ; [ $($body)* $count , $assoc $($token)+ ; ] % $($tail)+]
    };
    ($parser_def:ident $count:expr ; [ $($body:tt)* ] % $assoc:ident $($token:ident)+) => {
        $crate::_precedence_internal![@ $parser_def $($body)* $count , $assoc $($token)+]
    };
    (@ $parser_def:ident $($id:expr , $assoc:ident $($token:ident)+);+) => {
        {
            $(
                $(
                    if let lr_parsing_tools::grammar::Symbol::Terminal(a) = $token {
                        $parser_def.set_token_precedence(a, $crate::parser::Precedence { level: $id, associativity: $crate::parser::Associativity::Left });
                    }
                )+
            )+
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! _parser_def_internal {
    ($lexer_def:ident $count:expr ; [$($body:tt)*] $label:ident : $($($symbol:ident)*)|+ , $($tail:tt)+) => {
        $crate::_parser_def_internal![$lexer_def $count + 1_usize ; [$($body)* $count , $label &[$(&[$($symbol),*]),*] ;] $($tail)+]
    };
    ($lexer_def:ident $count:expr ; [$($body:tt)*] $label:ident : $($($symbol:ident)*)|+ $(,)?) => {
        $crate::_parser_def_internal![@ $lexer_def $($body)* $count , $label &[$(&[$($symbol),*]),*]]
    };
    (@ $lexer_def:ident $($id:expr , $label:ident $rule:expr);+) => {
        {
            $(
                #[allow(non_upper_case_globals)]
                const $label: lr_parsing_tools::grammar::Symbol = lr_parsing_tools::grammar::Symbol::Variable($id); 
            )+
            $crate::parser::ParserDefBuilder::new(
                $lexer_def,
                vec![$(stringify!($label).to_string()),+],
                lr_parsing_tools::grammar::GrammarBuilder::new()$(.rule($rule))+.build().unwrap(),
            )
        }
    };
}