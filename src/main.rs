mod parser;
use parser::Parser;


fn main() {
    let raw = r#"
    [dict]
    first = "first"
    # comment
    second ="another"
    bool = true

    [table]

    |abc|def|
    |---|---|
    |one|two|
    # comment
    |  1| 2 |
    |  2| 3 |

    [three]
    a=1
    B=2
    "#;

    let mut parser = Parser::new(raw);
    let map = parser.read();


    println!("{:?}", map);

}

