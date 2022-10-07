use clap::Parser;
use nom::error::convert_error;
use nom::Finish;
use parser::Address::{Range, Singular};
use parser::AddressPosition::{self, CurrentLine, Default, LastLine, Line};
use parser::AddressRange::{Absolute, Relative};
use parser::Command::{Append, Change, PrintNoLines};
use parser::{Address, SingularAddress};
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
    let mut interactive_mode = false;
    let verbose_errors = false;

    let mut current_line: usize = 1;
    let mut buffer: Vec<String> = Vec::new();
    buffer.push("code on line 1".into());
    buffer.push("code on line 2".into());
    buffer.push("code on line 3".into());
    buffer.push("code on line 4".into());
    buffer.push("code on line 5".into());

    let mut rl = Editor::<()>::new().unwrap();
    loop {
        let prompt = if interactive_mode { "" } else { &args.prompt };
        let readline = rl.readline(prompt);

        match readline {
            Ok(line) => {
                if interactive_mode {
                    if line == "." {
                        interactive_mode = false;
                        continue;
                    }

                    buffer.insert(current_line - 1, line);

                    continue;
                }

                let get_line_address_position =
                    |address_position: AddressPosition| match address_position {
                        Line(n) => n as usize,
                        CurrentLine => current_line,
                        LastLine => buffer.len(),
                        Default => 0,
                    };

                let get_line_singular = |address: SingularAddress| {
                    ((get_line_address_position(address.position) as i64) + address.offset) as usize
                };

                let get_line_range = |address: Address, default: Address|
                 -> (usize, usize) {
                    match address {
                        Singular(singular) => {
                            let default_singular = match default {
                                Singular(s) => s.unwrap(),
                                _ => unreachable!()
                            };

                            let target_index = get_line_singular(singular.unwrap_or(default_singular));
                            (target_index, target_index)
                        }
                        Range(range) => match range {
                            Absolute((addr1, addr2)) => {
                                let (default_singular1, default_singular2) = match default {
                                    Range(r) => match r {
                                        Absolute((x, y)) => (x.unwrap(), y.unwrap()),
                                        _ => unreachable!()
                                    },
                                    _ => unreachable!()
                                };

                                let target_line_addr1 = get_line_singular(addr1.unwrap_or(default_singular1));
                                let target_line_addr2 = get_line_singular(addr2.unwrap_or(default_singular2));

                                (target_line_addr1, target_line_addr2)
                            }
                            Relative((addr1, addr2)) => {
                                let (default_singular1, default_singular2) = match default {
                                    Range(r) => match r {
                                        Absolute((x, y)) => (x.unwrap(), y.unwrap()),
                                        _ => unreachable!()
                                    },
                                    _ => unreachable!()
                                };

                                let target_line_addr1 = get_line_singular(addr1.unwrap_or(default_singular1));
                                let target_line_addr2 = get_line_singular(addr2.unwrap_or(default_singular2));

                                (target_line_addr1, target_line_addr1 + target_line_addr2)
                            }
                        },
                    }
                };

                match parser::command_parser(&line).finish() {
                    Ok((_, command)) => match command {
                        Append(c) => {
                            let default = Address::Singular(Some(SingularAddress {
                                position: CurrentLine,
                                offset: 0,
                            }));

                            let (_, upper_line) = get_line_range(c.addr, default);

                            current_line = upper_line + 1;
                            interactive_mode = true;
                        }
                        PrintNoLines(c) => {
                            let default = Address::Range(Absolute((
                                Some(SingularAddress {
                                    position: Line(1),
                                    offset: 0,
                                }),
                                Some(SingularAddress {
                                    position: LastLine,
                                    offset: 0,
                                }),
                            )));

                            let (lower_line, upper_line) = get_line_range(c.addr, default);

                            current_line = upper_line;

                            for i in lower_line..=upper_line {
                                if i >= buffer.len() {
                                    break;
                                }

                                println!("{}", buffer[i - 1]);
                            }
                        }
                        Change(c) => {
                            let default = Address::Singular(Some(SingularAddress {
                                position: CurrentLine,
                                offset: 0,
                            }));

                            let (lower_line, upper_line) = get_line_range(c.addr, default);

                            current_line = upper_line;
                            interactive_mode = true;

                            for i in lower_line..=upper_line {
                                if i >= buffer.len() {
                                    break;
                                }

                                buffer.remove(i - 1);
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
