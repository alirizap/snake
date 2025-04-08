use anyhow::Result;
use std::{
    io::{stdout, Stdout, Write},
    time::Duration,
};

use crossterm::{
    cursor,
    event::{poll, read, Event, KeyCode},
    execute, style,
    terminal::{
        disable_raw_mode, enable_raw_mode, size, Clear, ClearType, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
    ExecutableCommand, QueueableCommand,
};

struct Snake {
    body: Vec<(u16, u16)>,
}

impl Snake {
    fn new(head_x: u16, head_y: u16) -> Self {
        let (tail_x, tail_y) = (head_x + 1, head_y);
        Self {
            body: vec![(head_x, head_y), (tail_x, tail_y)],
        }
    }
}

struct World {
    min_x: u16,
    max_x: u16,
    min_y: u16,
    max_y: u16,
    snake: Snake,
    stdout: Stdout,
}

impl World {
    fn new(max_x: u16, max_y: u16) -> Self {
        Self {
            min_x: 2,
            max_x: max_x - 2,
            min_y: 2,
            max_y: max_y - 2,
            snake: Snake::new(max_x / 2, max_y / 2),
            stdout: stdout(),
        }
    }

    fn run(&mut self) -> Result<()> {
        loop {
            self.refresh_screen()?;
            self.process_keypress()?;
        }
    }

    fn refresh_screen(&mut self) -> Result<()> {
        self.stdout.queue(Clear(ClearType::All))?;
        self.draw_snake()?;
        self.stdout.flush()?;
        Ok(())
    }

    fn process_keypress(&mut self) -> Result<()> {
        if let Ok(true) = poll(Duration::from_millis(10)) {
            let event = read()?;
            if let Event::Key(key) = event {
                match key.code {
                    KeyCode::Char('q') => {
                        self.stdout.execute(cursor::Show).unwrap();
                        self.stdout.execute(LeaveAlternateScreen).unwrap();
                        disable_raw_mode().unwrap();
                        std::process::exit(0);
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }

    fn draw_snake(&mut self) -> Result<()> {
        let (head_x, head_y) = self.snake.body.first().unwrap();
        self.stdout.queue(cursor::MoveTo(*head_x, *head_y))?;
        self.stdout.queue(style::Print("◍"))?;

        for (x, y) in &self.snake.body[1..] {
            self.stdout.queue(cursor::MoveTo(*x, *y))?;
            self.stdout.queue(style::Print("●"))?;
        }
        Ok(())
    }
}

fn main() {
    enable_raw_mode().unwrap();
    let (max_x, max_y) = size().unwrap();
    execute!(stdout(), cursor::Hide, EnterAlternateScreen).unwrap();
    let mut world = World::new(max_x, max_y);
    if let Err(e) = world.run() {
        eprintln!("{e}");
    }
    execute!(stdout(), cursor::Show, LeaveAlternateScreen).unwrap();
    disable_raw_mode().unwrap();
}
