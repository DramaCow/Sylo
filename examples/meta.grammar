ignore r"[ \n\t]+";
$IDENT = "something";
$CODE = "something else";
$REGEX = r#""something""#;

Grammar = Rule+;
Rule = $IDENT "=" Expr ";" {};
Expr = Production ("|" Production {})* {};
Production = ProductionTerm+ $CODE {};
ProductionTerm = $IDENT ":" Term {}
               | Term;
Term = Factor "?" {}
     | Factor "*" {}
     | Factor "+" {}
     | Factor;
Factor = "(" Expr ")" {}
       | $IDENT;