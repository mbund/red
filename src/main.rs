use clap::Parser;
use nom::error::convert_error;
use nom::Finish;
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
    // let interactive_mode = true;
    let verbose_errors = false;

    let current_line: usize = 0;
    let mut buffer = Vec::new();
    buffer.push("code on line 1");
    buffer.push("code on line 2");
    buffer.push("code on line 3");
    buffer.push("code on line 4");

    let mut rl = Editor::<()>::new().unwrap();
    loop {
        let readline = rl.readline(&args.prompt);

        match readline {
            Ok(line) => {
                // let e = parser::command_parser(&line).finish().err().unwrap();
                // println!("{}", convert_error(line.as_str(), e));

                match parser::command_parser(&line).finish() {
                    Ok((_, command)) => match command {
                        parser::Command::Append(x) => {
                            println!("{:#?}", x.addr);
                        }
                        parser::Command::PrintNoLines(x) => {
                            let (lower_index, upper_index) = match x.addr {
                                parser::Address::Singular(singular) => {
                                    let target_line = (((match singular.position {
                                        parser::AddressPosition::Line(n) => n as usize,
                                        parser::AddressPosition::CurrentLine => current_line,
                                        parser::AddressPosition::LastLine => buffer.len(),
                                        parser::AddressPosition::Default => current_line,
                                    })
                                        as i64)
                                        + singular.offset)
                                        as usize;

                                    (target_line - 1, target_line - 1)
                                }
                                parser::Address::Range(_) => (1337, 1337),
                            };

                            for i in lower_index..=upper_index {
                                if i >= buffer.len() {
                                    break;
                                }

                                println!("{}", buffer[i]);
                            }
                        }
                        _ => {}
                    },
                    Err(error) => {
                        if verbose_errors {
                            println!("{}", convert_error(line.as_str(), error));
                        } else {
                            println!("?");
                        }
                    }
                }
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
