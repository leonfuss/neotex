use super::{definition::LexerState, infra::Tokenizer};

fn check(input: &str) -> Result<(), ()> {
    use tracing_subscriber::FmtSubscriber;
    let subscriber = FmtSubscriber::builder().with_max_level(tracing::Level::TRACE).finish();

    let _ = tracing::subscriber::set_global_default(subscriber)
        .map_err(|_err| eprintln!("Unable to set global default subscriber"));

    let tokenizer: Tokenizer<LexerState> = Tokenizer::new(input);

    for token in tokenizer {
        println!("{:?} - '{}'", token, &input[token.span.span()]);
    }

    Err(())
}

#[test]
fn simple_lex() {
    let input = include_str!(
        "/Users/leon/Documents/02_university/05_semester/numerical_calculus/notes/main.tex"
    );
    check(input).unwrap();
}
