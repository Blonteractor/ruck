use std::env;
use std::fs::File;
use std::io::Read;
use std::iter::Enumerate;
use std::path::Path;
use std::str::Chars;

const STRIPLENGTH: usize = 30000;

type TuringStrip = [u8; STRIPLENGTH];
type Position = usize;
fn main() {
    let mut tape: TuringStrip = [0 as u8; 30000];
    let mut pointer: Position = 0;

    let mut input = String::new();

    if let Some(file_path) = env::args().nth(1) {
        let path = Path::new(&file_path);
        if !path.exists() {
            println!("Path does not exist.");
            return;
        }

        let mut file = match File::open(&path) {
            Ok(file) => file,
            Err(why) => {
                println!("Couldnt open file: {}", why);
                return;
            }
        };

        match file.read_to_string(&mut input) {
            Ok(_) => {}
            Err(why) => {
                println!("Couldn't read contents of file: {:?}", why);
                return;
            }
        }
    } else {
        println!("No input.");
        return;
    };

    let mut iter = input.chars().enumerate();

    parse_input_stream(&mut iter, &input, &mut tape, &mut pointer, false);
}
fn parse_input_stream(
    iter: &mut Enumerate<Chars>,
    whole: &str,
    tape: &mut TuringStrip,
    pointer: &mut Position,
    is_loop: bool,
) {
    let mut loop_body = if is_loop { whole } else { "" };
    loop {
        let (i, token) = match iter.next() {
            Some((i, t)) => (i, t),
            None => {
                break;
            }
        };
        match parse_token(token, tape, pointer) {
            ExecutionState::AllGood => {
                continue;
            }
            ExecutionState::Error(msg) => {
                println!("ruck: warning on token no. {} ({}): {}", i, token, msg);
            }
            ExecutionState::LoopOpen => {
                if let Some((pos, _)) = iter.find(|f| f.1 == ']') {
                    loop_body = unsafe { whole.get_unchecked(i + 1..pos + 1) };
                    //call this entire shit with loop_body as the input, recursively
                    parse_input_stream(
                        &mut loop_body.chars().enumerate(),
                        loop_body,
                        tape,
                        pointer,
                        true,
                    );
                } else {
                    println!(
                        "ruck: fatal error on token no. {} ([): Loop was not closed",
                        i
                    );
                }
            }
            ExecutionState::LoopSkip => {
                if let Some((pos, _)) = iter.find(|f| f.1 == ']') {
                    iter.nth(i + pos);
                } else {
                    println!(
                        "ruck: fatal error on token no. {} ([): Loop was not closed",
                        i
                    );
                }
            }
            ExecutionState::LoopStop => {
                return; // we were in a loop, now we aint, so return
            }
            ExecutionState::LoopAgain => {
                parse_input_stream(
                    &mut loop_body.chars().enumerate(),
                    loop_body,
                    tape,
                    pointer,
                    true,
                );
                return;
            }
        }
    }
}

fn parse_token<'a>(
    token: char,
    turing_strip: &mut TuringStrip,
    pointer_position: &mut Position,
) -> ExecutionState<'a> {
    let current_cell_value = &mut turing_strip[*pointer_position];

    match token {
        // Increment count of current cell
        '+' => {
            *current_cell_value += 1;
            ExecutionState::AllGood
        }

        // Decrement count of current cell
        '-' => {
            *current_cell_value -= 1;
            ExecutionState::AllGood
        }

        // Move pointer to the right
        '>' => {
            if *pointer_position == STRIPLENGTH - 1 {
                *pointer_position = 0;
                ExecutionState::Error("Pointer overflowed.")
            } else {
                *pointer_position += 1;
                ExecutionState::AllGood
            }
        }

        // Move pointer to the left
        '<' => {
            if *pointer_position == 0 {
                *pointer_position = STRIPLENGTH - 1;
                ExecutionState::Error("Pointer underflowed.")
            } else {
                *pointer_position -= 1;
                ExecutionState::AllGood
            }
        }

        // Print current value to stdout as ascii
        '.' => {
            print!("{}", *current_cell_value as u8 as char);
            ExecutionState::AllGood
        }

        // Take input of singke character and set its ascii to value of current cell
        ',' => {
            let mut io_buff: [u8; 1] = [0; 1];

            std::io::stdin().read(&mut io_buff).unwrap();
            *current_cell_value = io_buff[0];

            ExecutionState::AllGood
        }

        /* Do the thing between square brackets till value of current cell is 0 */
        // Loop open
        '[' => {
            if *current_cell_value != 0 {
                // need to jump to loop close
                ExecutionState::LoopOpen
            } else {
                ExecutionState::LoopSkip
            }
        }

        // Loop close
        ']' => {
            if *current_cell_value != 0 {
                ExecutionState::LoopAgain
            } else {
                ExecutionState::LoopStop
            }
        }

        // Comment
        _ => ExecutionState::AllGood,
    }
}

enum ExecutionState<'a> {
    AllGood,
    Error(&'a str),
    LoopOpen,
    LoopSkip,
    LoopStop,
    LoopAgain,
}
