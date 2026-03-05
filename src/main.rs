use flowscript::lexer::Lexer;
use flowscript::reader::Reader;

fn main() {
    let path = "main.fscc";
    let reader = Reader::new(path);
    let lexer = Lexer::new(reader);
}
