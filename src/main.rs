use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyModifiers},
    execute, queue,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{self, ClearType},
};
use image::{imageops::FilterType, GenericImageView};
use std::io::{self, Write};
use std::path::Path;
use std::process::Command;
use std::thread;
use std::time::Duration;
use std::fs::OpenOptions;

const MIN_DIMENSION: u32 = 480;
const MAX_DIMENSION: u32 = 720;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> Result<()> {
    std::panic::set_hook(Box::new(|panic_info| {
        log(&format!("PANIC: {:?}", panic_info));
    }));

    match run_app() {
        Ok(_) => {
            log("App exited normally");
            Ok(())
        }
        Err(e) => {
            log(&format!("App error: {}", e));
            Err(e)
        }
    }
}

fn log(msg: &str) {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("/tmp/imgopt.log")
        .unwrap();
    writeln!(file, "{}", msg).ok();
}

fn run_app() -> Result<()> {
    log("App starting");
    terminal::enable_raw_mode()?;
    log("Raw mode enabled");
    let mut stdout = io::stdout();

    execute!(
        stdout,
        terminal::Clear(ClearType::All),
        cursor::MoveTo(0, 0)
    )?;

    show_drop_zone(&mut stdout)?;
    log("Drop zone shown, entering loop");

    let mut input_buffer = String::new();

    loop {
        if !event::poll(Duration::from_millis(100))? {
            continue;
        }

        match event::read()? {
            Event::Key(key_event) => {
                log(&format!("Key event: {:?}", key_event));
                match key_event.code {
                    KeyCode::Esc => {
                        log("Breaking on ESC");
                        break;
                    }
                    KeyCode::Char('c') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                        log("Breaking on Ctrl+C");
                        break;
                    }
                    KeyCode::Char('d') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                        log("Breaking on Ctrl+D");
                        break;
                    }
                    KeyCode::Char(c) => {
                        input_buffer.push(c);

                        // Auto-process when closing quote is detected (drag complete)
                        if (c == '\'' || c == '"') && input_buffer.len() > 2 {
                            let path = input_buffer.trim().trim_matches('\'').trim_matches('"');
                            if path.ends_with(".png") || path.ends_with(".jpg") || path.ends_with(".jpeg") ||
                               path.ends_with(".PNG") || path.ends_with(".JPG") || path.ends_with(".JPEG") ||
                               path.ends_with(".gif") || path.ends_with(".GIF") || path.ends_with(".webp") {
                                log(&format!("Auto-processing: {}", path));
                                process_image(&mut stdout, path)?;
                                input_buffer.clear();
                                thread::sleep(Duration::from_secs(2));
                                execute!(stdout, terminal::Clear(ClearType::All), cursor::MoveTo(0, 0))?;
                                show_drop_zone(&mut stdout)?;
                            }
                        }
                    }
                    KeyCode::Enter => {
                        if !input_buffer.is_empty() {
                            let path = input_buffer.trim().trim_matches('\'').trim_matches('"');
                            process_image(&mut stdout, path)?;
                            input_buffer.clear();
                            thread::sleep(Duration::from_secs(2));
                            execute!(stdout, terminal::Clear(ClearType::All), cursor::MoveTo(0, 0))?;
                            show_drop_zone(&mut stdout)?;
                        }
                    }
                    KeyCode::Backspace => {
                        input_buffer.pop();
                    }
                    _ => {}
                }
            }
            Event::Paste(data) => {
                log(&format!("Paste event: {}", data));
                // Drag and drop triggers paste event!
                let path = data.trim().trim_matches('\'').trim_matches('"');
                if !path.is_empty() {
                    log(&format!("Processing: {}", path));
                    process_image(&mut stdout, path)?;
                    thread::sleep(Duration::from_secs(2));
                    execute!(stdout, terminal::Clear(ClearType::All), cursor::MoveTo(0, 0))?;
                    show_drop_zone(&mut stdout)?;
                    log("Back to drop zone");
                }
            }
            evt => {
                log(&format!("Other event: {:?}", evt));
            }
        }
    }

    log("Exited main loop");
    execute!(stdout, terminal::Clear(ClearType::All), cursor::MoveTo(0, 0))?;
    terminal::disable_raw_mode()?;
    log("Disabled raw mode");
    println!("👋 Goodbye!");
    Ok(())
}

fn show_drop_zone(stdout: &mut io::Stdout) -> Result<()> {
    let (width, height) = terminal::size()?;
    let center_y = height / 2;

    // Simple mode for narrow terminals
    if width < 50 {
        let lines = vec![
            "📸 IMAGE OPTIMIZER",
            "Drop images here",
            "480-720px",
            "(ESC to quit)",
        ];

        for (i, line) in lines.iter().enumerate() {
            let x = (width.saturating_sub(line.len() as u16)) / 2;
            queue!(
                stdout,
                cursor::MoveTo(x, center_y.saturating_sub(2) + i as u16),
                SetForegroundColor(if i == 0 { Color::Blue } else { Color::White }),
                Print(line),
                ResetColor,
            )?;
        }
    } else {
        // Full box mode for wider terminals
        let box_width = 44;
        let box_height = 10;
        let start_x = (width.saturating_sub(box_width)) / 2;
        let start_y = (height.saturating_sub(box_height)) / 2;

        queue!(
            stdout,
            cursor::MoveTo(start_x, start_y),
            SetForegroundColor(Color::Blue),
            Print("╔════════════════════════════════════════╗"),
            cursor::MoveTo(start_x, start_y + 1),
            Print("║                                        ║"),
            cursor::MoveTo(start_x, start_y + 2),
            Print("║      📸  IMAGE OPTIMIZER  📸          ║"),
            cursor::MoveTo(start_x, start_y + 3),
            Print("║                                        ║"),
            cursor::MoveTo(start_x, start_y + 4),
            Print("║      Drop images here                  ║"),
            cursor::MoveTo(start_x, start_y + 5),
            Print("║      480-720px clamping                ║"),
            cursor::MoveTo(start_x, start_y + 6),
            Print("║                                        ║"),
            cursor::MoveTo(start_x, start_y + 7),
            SetForegroundColor(Color::DarkGrey),
            Print("║      (ESC or Ctrl+C to quit)           ║"),
            SetForegroundColor(Color::Blue),
            cursor::MoveTo(start_x, start_y + 8),
            Print("║                                        ║"),
            cursor::MoveTo(start_x, start_y + 9),
            Print("╚════════════════════════════════════════╝"),
            ResetColor,
        )?;
    }

    queue!(stdout, cursor::Hide)?;
    stdout.flush()?;
    Ok(())
}

fn process_image(stdout: &mut io::Stdout, path: &str) -> Result<()> {
    execute!(stdout, terminal::Clear(ClearType::All), cursor::MoveTo(0, 0))?;

    let (term_width, term_height) = terminal::size()?;
    let center_x = term_width / 2;
    let center_y = term_height / 2;

    queue!(
        stdout,
        cursor::MoveTo(center_x.saturating_sub(10), center_y.saturating_sub(3)),
        SetForegroundColor(Color::Yellow),
        Print("⚡ Processing image..."),
        ResetColor,
    )?;
    stdout.flush()?;

    let path_obj = Path::new(path);

    if !path_obj.exists() {
        queue!(
            stdout,
            cursor::MoveTo(center_x.saturating_sub(7), center_y.saturating_sub(1)),
            SetForegroundColor(Color::Red),
            Print("❌ File not found"),
            ResetColor,
        )?;
        stdout.flush()?;
        return Ok(());
    }

    // Load image
    let img = match image::open(path_obj) {
        Ok(img) => img,
        Err(_) => {
            queue!(
                stdout,
                cursor::MoveTo(center_x.saturating_sub(10), center_y.saturating_sub(1)),
                SetForegroundColor(Color::Red),
                Print("❌ Could not open image"),
                ResetColor,
            )?;
            stdout.flush()?;
            return Ok(());
        }
    };

    let (width, height) = img.dimensions();
    let orig_text = format!("Original: {}x{}px", width, height);

    queue!(
        stdout,
        cursor::MoveTo(center_x.saturating_sub((orig_text.len() / 2) as u16), center_y.saturating_sub(1)),
        SetForegroundColor(Color::DarkYellow),
        Print(&orig_text),
        ResetColor,
    )?;
    stdout.flush()?;

    // Calculate new dimensions
    let max_dim = width.max(height);
    let target_dim = if max_dim > MAX_DIMENSION {
        MAX_DIMENSION
    } else if max_dim < MIN_DIMENSION {
        MIN_DIMENSION
    } else {
        max_dim
    };

    let (new_width, new_height) = if target_dim != max_dim {
        if width > height {
            (target_dim, (height * target_dim) / width)
        } else {
            ((width * target_dim) / height, target_dim)
        }
    } else {
        (width, height)
    };

    let opt_text = format!("Optimized: {}x{}px", new_width, new_height);
    queue!(
        stdout,
        cursor::MoveTo(center_x.saturating_sub((opt_text.len() / 2) as u16), center_y),
        SetForegroundColor(Color::Green),
        Print(&opt_text),
        ResetColor,
    )?;
    stdout.flush()?;

    // Resize image
    let resized = img.resize_exact(new_width, new_height, FilterType::Lanczos3);

    // Save to temp file
    let temp_path = "/tmp/imgopt_temp.png";
    if resized.save(temp_path).is_err() {
        queue!(
            stdout,
            cursor::MoveTo(center_x.saturating_sub(13), center_y + 2),
            SetForegroundColor(Color::Red),
            Print("❌ Failed to save temp file"),
            ResetColor,
        )?;
        stdout.flush()?;
        return Ok(());
    }

    // Copy to clipboard using osascript
    let _ = Command::new("osascript")
        .arg("-e")
        .arg(format!(
            "set the clipboard to (read (POSIX file \"{}\") as «class PNGf»)",
            temp_path
        ))
        .output();

    // Cleanup
    let _ = std::fs::remove_file(temp_path);

    queue!(
        stdout,
        cursor::MoveTo(center_x.saturating_sub(12), center_y + 2),
        SetForegroundColor(Color::Green),
        Print("✅ Copied to clipboard!"),
        cursor::MoveTo(center_x.saturating_sub(14), center_y + 4),
        SetForegroundColor(Color::DarkGreen),
        Print("Ready to paste into Claude..."),
        ResetColor,
    )?;
    stdout.flush()?;

    Ok(())
}
