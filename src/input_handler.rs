use crossterm::{
    cursor::MoveToColumn,
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    style::Print,
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
};
use std::{
    io::{self, StdoutLock, Write},
    process,
};

use crate::command::Trie;

pub fn read_line_with_tab_detection(
    stdout: &mut StdoutLock<'static>,
    trie: &mut Trie,
) -> io::Result<String> {
    enable_raw_mode()?;
    let mut line = String::new();

    print!("$ ");
    io::stdout().flush()?;

    let mut tab_count = 0;

    loop {
        if let Event::Key(key_event) = event::read()? {
            match key_event {
                KeyEvent {
                    modifiers: KeyModifiers::CONTROL,
                    code: KeyCode::Char(c),
                    ..
                } => match c {
                    'j' => {
                        print!("\r\n");
                        io::stdout().flush()?;
                        break;
                    }
                    'c' => {
                        disable_raw_mode()?;
                        process::exit(0);
                    }
                    _ => (),
                },
                _ => (),
            }
            match key_event.code {
                KeyCode::Enter => {
                    print!("\r\n");
                    io::stdout().flush()?;
                    break;
                }
                KeyCode::Tab => {
                    tab_count += 1;
                    let mut words = trie.get_words_with_prefix(&line);
                    if words.len() == 1 {
                        execute!(stdout, MoveToColumn(0), Clear(ClearType::CurrentLine))?;
                        line = format!("{} ", words[0].clone());
                        print!("$ {}", line);
                    } else if words.len() > 1 {
                        if tab_count > 1 {
                            words.sort_unstable();
                            let words_with_prefix = format!("\r\n{}\r\n", words.join("  "));
                            let prompt = format!("$ {}", line);

                            execute!(
                                stdout,
                                Print(words_with_prefix),
                                MoveToColumn(0),
                                Print(prompt)
                            )?;
                            tab_count += 1;
                        } else {
                            // longest common prefix
                            // this challenge is marked as hard, but this is a
                            // leetcode easy
                            let mut idx = 0;
                            let mut longest_common_prefix = String::new();
                            'outer: loop {
                                for i in 1..words.len() {
                                    if idx == words[i].len() || idx == words[i - 1].len() {
                                        break 'outer;
                                    }
                                    if words[i].as_bytes()[idx] != words[i - 1].as_bytes()[idx] {
                                        break 'outer;
                                    }
                                }
                                longest_common_prefix.push(match words[0].chars().nth(idx) {
                                    Some(first_letter) => first_letter,
                                    None => break 'outer,
                                });
                                idx += 1;
                            }
                            if !longest_common_prefix.is_empty() {
                                line = longest_common_prefix;
                                execute!(
                                    stdout,
                                    MoveToColumn(0),
                                    Clear(ClearType::CurrentLine),
                                    Print(format!("$ {}", line))
                                )?;
                            }
                        }
                    }
                    print!("\x07");
                    io::stdout().flush()?;
                }
                KeyCode::Backspace => {
                    if !line.is_empty() {
                        tab_count = 0;
                        line.pop();
                        execute!(stdout, MoveToColumn(0), Clear(ClearType::CurrentLine)).unwrap();
                        print!("$ {}", line);
                        io::stdout().flush()?;
                    }
                }
                KeyCode::Char(c) => {
                    tab_count = 0;
                    line.push(c);
                    print!("{}", c);
                    io::stdout().flush()?;
                }
                _ => {}
            }
        }
    }

    disable_raw_mode()?;
    Ok(line)
}
