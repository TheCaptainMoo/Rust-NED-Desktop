use std::{io::{self, Write, Read}, fs::File};
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tui::{
    style::Style,
    backend::{CrosstermBackend, Backend}, 
    widgets::{Block, Borders, Paragraph, Wrap},
    layout::{Layout, Constraint, Direction},
    Terminal,
    Frame
};
use tinyfiledialogs::{
    open_file_dialog,
    save_file_dialog_with_filter
};

mod encryption;

fn main() -> Result<(), io::Error> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut output: String = String::new();
    let mut input_text: String = String::new();
    let mut selection: InputMode = InputMode::Input;

    loop {
        let mut decider: String = String::new();
        let mut mode: String = String::new();
        let mut key_text: String = String::new();
        let mut recursion: String = String::new();    


        let mut cursor_index: usize = input_text.len();

        loop {
            terminal.draw(|f| ui(f, &selection, cursor_index, input_text.clone(), decider.clone(), mode.clone(), key_text.clone(), recursion.clone(), output.clone()))?;

            if let Ok(event) = crossterm::event::read() {
                match event {
                    Event::Key(key) => {
                        match selection {
                            InputMode::Input => {
                                match key.code {
                                    KeyCode::Tab | KeyCode::Enter => {
                                        selection = InputMode::Decider;
                                    }
                                    KeyCode::BackTab => {
                                        selection = InputMode::Load;
                                    }
                                    KeyCode::Left => {
                                        if cursor_index > 0 {
                                            cursor_index -= 1;
                                        }
                                    }
                                    KeyCode::Right => {
                                        if cursor_index < input_text.len() {
                                            cursor_index += 1;
                                        }
                                    }
                                    KeyCode::Char(c) => {
                                        input_text.insert(cursor_index, c);
                                        cursor_index += 1;
                                    }
                                    KeyCode::Backspace => {
                                        if cursor_index > 0 {
                                            input_text.remove(cursor_index-1);
                                            cursor_index -= 1;
                                        }
                            
                                        //text.pop();
                                    }
                                    _ => {}
                                }
                            }
                            InputMode::Decider => {
                                match key.code {
                                    KeyCode::Tab | KeyCode::Enter => {
                                        selection = InputMode::Mode;
                                    }
                                    KeyCode::BackTab => {
                                        selection = InputMode::Input;
                                    }
                                    _ => {
                                        cursor_index = decider.len();
                                        handle_input(key.code, cursor_index,&mut decider);
                                    }
                                }
                            }
                            InputMode::Mode => {
                                match key.code {
                                    KeyCode::Tab | KeyCode::Enter => {
                                        selection = InputMode::Key;
                                    }
                                    KeyCode::BackTab => {
                                        selection = InputMode::Decider;
                                    }
                                    _ => {
                                        cursor_index = mode.len();
                                        handle_input(key.code, cursor_index, &mut mode);
                                    }
                                }
                            }
                            InputMode::Key => {
                                match key.code {
                                    KeyCode::Tab | KeyCode::Enter => {
                                        selection = InputMode::Recursion;
                                    }
                                    KeyCode::BackTab => {
                                        selection = InputMode::Mode;
                                    }
                                    _ => {
                                        cursor_index = key_text.len();
                                        handle_input(key.code, cursor_index, &mut key_text);
                                    }
                                }
                            }
                            InputMode::Recursion => {
                                match key.code {
                                    KeyCode::Tab | KeyCode::Enter => {
                                        selection = InputMode::Submit;
                                    }
                                    KeyCode::BackTab => {
                                        selection = InputMode::Key;
                                    }
                                    _ => {
                                        cursor_index = recursion.len();
                                        handle_input(key.code, cursor_index, &mut recursion);
                                    }
                                }
                            }
                            InputMode::Submit => {
                                match key.code {
                                    KeyCode::Enter => {
                                        selection = InputMode::Save;
                                        output.clear();
                                        break;
                                    }
                                    KeyCode::Tab => {
                                        selection = InputMode::Save;
                                    }
                                    KeyCode::BackTab => {
                                        selection = InputMode::Recursion;
                                    }
                                    _ => {}
                                }
                            }
                            InputMode::Save => {
                                match key.code {
                                    KeyCode::BackTab => {
                                        selection = InputMode::Submit;
                                    }
                                    KeyCode::Tab => {
                                        selection = InputMode::RawText;
                                    }
                                    KeyCode::Enter => {
                                        if let Some(file_path) = save_file_dialog_with_filter("Save Output", "", &[&"*.txt"], &"(*.txt) File"){
                                            let mut file = File::create(file_path)?;

                                            file.write_all(output.as_bytes())?;
                                        }
                                    }
                                    _ => {}
                                }
                            }
                            InputMode::RawText => {
                                match key.code {
                                    KeyCode::BackTab => {
                                        selection = InputMode::Save;
                                    }
                                    KeyCode::Tab => {
                                        selection = InputMode::Load;
                                    }
                                    KeyCode::Enter => {
                                        disable_raw_mode()?;
                                        execute!(
                                            terminal.backend_mut(),
                                            LeaveAlternateScreen,
                                            DisableMouseCapture
                                        )?;
                                        terminal.show_cursor()?;

                                        println!("Output: {}", output);
                                    }
                                    _ => {}
                                }
                            }
                            InputMode::Load => {
                                match key.code {
                                    KeyCode::BackTab => {
                                        selection = InputMode::RawText;
                                    }
                                    KeyCode::Tab => {
                                        selection = InputMode::Input;
                                    }
                                    KeyCode::Enter => {
                                        if let Some(file_path) = open_file_dialog("Save Output", "", Some((&["*.txt"], "(*.txt) File"))){
                                            let mut file = File::open(file_path)?;

                                            file.read_to_string(&mut input_text)?;
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        let key: i32 = match key_text.trim().parse(){
            Ok(num) => num,
            Err(_) => {
                //println!("Invalid Key");
                0
            },
        };

        let recursion: i32 = match recursion.trim().parse(){
            Ok(num) => num,
            Err(_) => {
                //println!("Invalid Recursion");
                0
            },
        };

        output = match mode.trim().to_lowercase().as_str(){
            "default" => match decider.trim().to_lowercase().as_str() {
                "encrypt" => encryption::encrypt(&input_text.trim().to_string().to_uppercase(), key, recursion),
                "decrypt" => encryption::decrypt(&input_text.trim().to_string().to_uppercase(), key, recursion),
                _ => {
                    //println!("Please use either 'Encrypt' or 'Decrypt'");
                    continue;
                }
            },
            "ascii" => match decider.trim().to_lowercase().as_str() {
                "encrypt" => encryption::ascii_encrypt(&input_text.trim().to_string(), key.try_into().unwrap(), recursion),
                "decrypt" => encryption::ascii_decrypt(&input_text.trim().to_string(), key.try_into().unwrap(), recursion),
                _ => {
                    //println!("Please use either 'Encrypt' or 'Decrypt'");
                    continue;
                }
            }
            _ => {
                //println!("Please use either 'Default' or 'ASCII'");
                continue;
            }
        };

        input_text.clear();
    }

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

fn ui<B: Backend>(f: &mut Frame<B>, state: &InputMode, cursor_pos: usize, input_text: String, decider_text: String, mode_text: String, key_text: String, recursion_text: String, output_text: String){
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(30),
            Constraint::Percentage(70),
        ])
        .split(f.size());

    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(10),
            Constraint::Percentage(20),
            Constraint::Percentage(10),
            Constraint::Percentage(10),
            Constraint::Percentage(10),
            Constraint::Percentage(10),
            Constraint::Percentage(10),
            Constraint::Percentage(10),
            Constraint::Percentage(10),
        ])
        .split(chunks[0]);

    let left_sub_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50),
            Constraint::Percentage(50)
        ])
        .split(left_chunks[8]);
        

    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(80),
            Constraint::Percentage(20)
        ])
        .split(chunks[1]);

    let mut input_style = Style::default().bg(tui::style::Color::Black).fg(tui::style::Color::White);
    let mut decider_style = Style::default().bg(tui::style::Color::Black).fg(tui::style::Color::White);
    let mut mode_style = Style::default().bg(tui::style::Color::Black).fg(tui::style::Color::White);
    let mut key_style = Style::default().bg(tui::style::Color::Black).fg(tui::style::Color::White);
    let mut recursion_style = Style::default().bg(tui::style::Color::Black).fg(tui::style::Color::White);
    let mut submit_style = Style::default().bg(tui::style::Color::Black).fg(tui::style::Color::White);
    
    let mut save_style = Style::default().bg(tui::style::Color::Black).fg(tui::style::Color::White);
    let mut load_style = Style::default().bg(tui::style::Color::Black).fg(tui::style::Color::White);
    let mut rawtext_style = Style::default().bg(tui::style::Color::Black).fg(tui::style::Color::White);

    let mut inp_text = input_text;

    match state {
        InputMode::Input => {
            inp_text = inp_text[0..cursor_pos].to_string() + "|" + &inp_text[cursor_pos..inp_text.len()];
            input_style = Style::default().fg(tui::style::Color::Green);
        }
        InputMode::Decider => {
            decider_style = Style::default().fg(tui::style::Color::Green);
        }
        InputMode::Mode => {
            mode_style = Style::default().fg(tui::style::Color::Green);
        }
        InputMode::Key => {
            key_style = Style::default().fg(tui::style::Color::Green);
        }
        InputMode::Recursion => {
            recursion_style = Style::default().fg(tui::style::Color::Green);
        }
        InputMode::Submit => {
            submit_style = Style::default().fg(tui::style::Color::Green);
        }
        InputMode::Save => {
            save_style = Style::default().fg(tui::style::Color::Green);
        }
        InputMode::RawText => {
            rawtext_style = Style::default().fg(tui::style::Color::Green);
        }
        InputMode::Load => {
            load_style = Style::default().fg(tui::style::Color::Green);
        }
    }

    let mut x_offset: i16 = (cursor_pos as i16 / (left_chunks[1].width as i16 - 2)) as i16 - (left_chunks[1].height as i16 - 4) as i16;

    if x_offset <= 0 {
        x_offset = 0;
    }

    let input_block = Block::default()
        .title("Input Text")
        .style(input_style)
        .borders(Borders::ALL);

    let input_para = Paragraph::new(inp_text)
        .block(input_block)
        .wrap(Wrap { trim: true })
        .scroll(( x_offset as u16,0 ));

    let decider_block = Block::default()
        .title("Decider (Encrypt|Decrypt)")
        .style(decider_style)
        .borders(Borders::ALL);

    let decider_para = Paragraph::new(decider_text.as_ref())
        .block(decider_block)
        .wrap(Wrap { trim: true });

    let mode_block = Block::default()
        .title("Mode (Default|Ascii)")
        .style(mode_style)
        .borders(Borders::ALL);

    let mode_para = Paragraph::new(mode_text.as_ref())
        .block(mode_block)
        .wrap(Wrap { trim: true });

    let key_block = Block::default()
        .title("Key")
        .style(key_style)
        .borders(Borders::ALL);

    let key_para = Paragraph::new(key_text.as_ref())
        .block(key_block)
        .wrap(Wrap { trim: true });

    let recursion_block = Block::default()
        .title("Recursion")
        .style(recursion_style)
        .borders(Borders::ALL);

    let recursion_para = Paragraph::new(recursion_text.as_ref())
        .block(recursion_block)
        .wrap(Wrap { trim: true });

    let submit_block = Paragraph::new("Submit")
        .alignment(tui::layout::Alignment::Center)
        .block(Block::default().style(submit_style).borders(Borders::ALL)); 

    let filler_block = Block::default();

    let output_para = Paragraph::new(output_text.as_ref())
        .block(Block::default().title("Output").borders(Borders::ALL))
        .wrap(Wrap { trim: true});

    let save_button = Paragraph::new("Save To File")
        .alignment(tui::layout::Alignment::Center)
        .wrap(Wrap { trim: true})
        .block(Block::default().style(save_style).borders(Borders::ALL)); 

    let rawtext_button = Paragraph::new("View Raw Text")
        .alignment(tui::layout::Alignment::Center)
        .wrap(Wrap { trim: true})
        .block(Block::default().style(rawtext_style).borders(Borders::ALL)); 

    let load_button = Paragraph::new("Load File")
        .alignment(tui::layout::Alignment::Center)
        .wrap(Wrap { trim: true})
        .block(Block::default().style(load_style).borders(Borders::ALL)); 
    
    f.render_widget(load_button, left_chunks[0]);
    f.render_widget(input_para, left_chunks[1]);
    f.render_widget(decider_para, left_chunks[2]);
    f.render_widget(mode_para, left_chunks[3]);
    f.render_widget(key_para, left_chunks[4]);
    f.render_widget(recursion_para, left_chunks[5]);
    f.render_widget(submit_block, left_chunks[6]);
    f.render_widget(filler_block, left_chunks[7]);
    
    f.render_widget(save_button, left_sub_chunks[0]);
    f.render_widget(rawtext_button, left_sub_chunks[1]);
    //f.render_widget(save_button, left_sub_chunks[0]);

    f.render_widget(output_para, right_chunks[0]);
}

fn handle_input(key: KeyCode, cursor_pos: usize, text: &mut String){
    match key {
        KeyCode::Char(c) => {
            text.insert(cursor_pos, c);
            
            //text.push(c)
        }
        KeyCode::Backspace => {
            if cursor_pos > 0 {
                text.remove(cursor_pos-1);
            }

            //text.pop();
        }
        _ => {}
    }
}

enum InputMode {
    Input,
    Decider,
    Mode,
    Key,
    Recursion,
    Submit,
    Save,
    RawText,
    Load
}