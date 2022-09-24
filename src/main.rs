use clap::Parser;
use rustyline::error::ReadlineError;
use rustyline::Editor;
mod parser;

/// Red is the standard rust text editor
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// String to use for interactive prompt
    #[clap(short, long, value_parser, default_value = "")]
    prompt: String,

    /// File to edit
    #[clap(value_parser)]
    file: Option<String>,
}

fn main() {
    let args = Args::parse();
    let interactive_mode = true;

    let mut rl = Editor::<()>::new().unwrap();
    loop {
        let readline = rl.readline(&args.prompt);

        match readline {
            Ok(line) => {
                println!("Line: {}", line);
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
}
