use crate::Direction::*;
use anyhow::Result;
use std::{
    io::{stdout, Stdout, Write},
    thread::sleep,
    time::Duration,
};

use crossterm::{
    cursor,
    event::{poll, read, Event, KeyCode},
    execute,
    style::{self, Stylize},
    terminal::{
        disable_raw_mode, enable_raw_mode, size, Clear, ClearType, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
    ExecutableCommand, QueueableCommand,
};

#[derive(PartialEq, Clone, Copy)]
enum Direction {
    Up,
    Down,
    Right,
    Left,
}

struct Snake {
    body: Vec<(u16, u16)>,
    head_dir: Direction,
}

impl Snake {
    fn new(head_x: u16, head_y: u16) -> Self {
        let (tail_x, tail_y) = (head_x + 1, head_y);
        Self {
            body: vec![(head_x, head_y), (tail_x, tail_y)],
            head_dir: Left,
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
            self.snake_move();
            sleep(Duration::from_millis(100));
        }
    }

    fn refresh_screen(&mut self) -> Result<()> {
        self.stdout.queue(Clear(ClearType::All))?;
        self.draw_statusbar()?;
        self.draw_snake()?;
        self.stdout.flush()?;
        Ok(())
    }

    fn process_keypress(&mut self) -> Result<()> {
        if let Ok(true) = poll(Duration::from_millis(10)) {
            let event = read()?;
            if let Event::Key(key) = event {
                let cur_dir = self.snake.head_dir;
                match key.code {
                    KeyCode::Char('q') => {
                        self.stdout.execute(cursor::Show).unwrap();
                        self.stdout.execute(LeaveAlternateScreen).unwrap();
                        disable_raw_mode().unwrap();
                        std::process::exit(0);
                    }
                    KeyCode::Char('w') if cur_dir != Down => self.snake.head_dir = Up,
                    KeyCode::Char('a') if cur_dir != Right => self.snake.head_dir = Left,
                    KeyCode::Char('s') if cur_dir != Up => self.snake.head_dir = Down,
                    KeyCode::Char('d') if cur_dir != Left => self.snake.head_dir = Right,
                    _ => {}
                }
            }
        }
        Ok(())
    }

    fn draw_statusbar(&mut self) -> Result<()> {
        let msg = format!(" press q to exit | moving <w,a,s,d> ");
        self.stdout.queue(cursor::MoveTo(0, self.max_y + 1))?;
        self.stdout.queue(style::Print(msg.black().on_grey()))?;
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

    fn snake_move(&mut self) {
        self.snake_new_head();
        self.snake.body.pop();
    }

    fn snake_new_head(&mut self) {
        let (head_x, head_y) = self.snake.body.first().unwrap();
        let new_head = match self.snake.head_dir {
            Up => {
                if *head_y == self.min_y {
                    (*head_x, self.max_y - 1)
                } else {
                    (*head_x, *head_y - 1)
                }
            }
            Down => {
                if *head_y == self.max_y - 1 {
                    (*head_x, self.min_y + 1)
                } else {
                    (*head_x, *head_y + 1)
                }
            }
            Left => {
                if *head_x == self.min_x {
                    (self.max_x - 1, *head_y)
                } else {
                    (*head_x - 1, *head_y)
                }
            }
            Right => {
                if *head_x == self.max_x - 1 {
                    (self.min_x + 1, *head_y)
                } else {
                    (*head_x + 1, *head_y)
                }
            }
        };
        self.snake.body.insert(0, new_head);
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
