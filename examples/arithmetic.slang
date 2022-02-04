ignore r"[ \n\t]+";
$NUM = r"[1-9][0-9]* | 0";
Expr = a:Expr "+" b:Num { a + b } | Num;
Num = a:$NUM { to_string(a) };