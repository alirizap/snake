use crate::Direction::*;
use anyhow::Result;
use rand::Rng;
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

const MAX_MOVE_DELAY: u64 = 150;
const MIN_MOVE_DELAY: u64 = 50;

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
    score: u64,
}

impl Snake {
    fn new(head_x: u16, head_y: u16) -> Self {
        let (tail_x, tail_y) = (head_x + 1, head_y);
        Self {
            body: vec![(head_x, head_y), (tail_x, tail_y)],
            head_dir: Left,
            score: 0,
        }
    }
}

struct Target {
    x: u16,
    y: u16,
}

impl Target {
    fn new(min_x: u16, max_x: u16, min_y: u16, max_y: u16) -> Self {
        Self {
            x: rand::rng().random_range(min_x..=max_x),
            y: rand::rng().random_range(min_y..=max_y),
        }
    }
}

struct World {
    min_x: u16,
    max_x: u16,
    min_y: u16,
    max_y: u16,
    snake: Snake,
    target: Target,
    update_target_position: bool,
    game_over: bool,
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
            target: Target::new(3, max_x - 4, 3, max_y - 4),
            update_target_position: true,
            game_over: false,
            stdout: stdout(),
        }
    }

    fn run(&mut self) -> Result<()> {
        loop {
            self.refresh_screen()?;
            self.process_keypress()?;
            self.snake_move();
            if self.check_failure() {
                self.draw_failure_banner()?;
            }
            self.check_collision();
            sleep(Duration::from_millis(self.snake_move_delay()));
        }
    }

    fn refresh_screen(&mut self) -> Result<()> {
        if !self.game_over {
            self.stdout.queue(Clear(ClearType::All))?;
            self.draw_statusbar()?;
            self.draw_snake()?;
            self.draw_target()?;
            self.stdout.flush()?;
        }
        Ok(())
    }

    fn restart(&mut self) {
        self.snake = Snake::new(self.max_x / 2, self.max_y / 2);
        self.target = Target::new(3, self.max_x - 2, 3, self.max_y - 2);
        self.update_target_position = true;
        self.game_over = false;
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
                    KeyCode::Char('w') if cur_dir != Down && !self.game_over => {
                        self.snake.head_dir = Up
                    }
                    KeyCode::Char('a') if cur_dir != Right && !self.game_over => {
                        self.snake.head_dir = Left
                    }
                    KeyCode::Char('s') if cur_dir != Up && !self.game_over => {
                        self.snake.head_dir = Down
                    }
                    KeyCode::Char('d') if cur_dir != Left && !self.game_over => {
                        self.snake.head_dir = Right
                    }
                    KeyCode::Enter => self.restart(),
                    _ => {}
                }
            }
        }
        Ok(())
    }

    fn draw_statusbar(&mut self) -> Result<()> {
        let msg = format!(
            " press q to exit | moving <w,a,s,d>, restart <enter> | score {} ",
            self.snake.score
        );
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

    fn draw_target(&mut self) -> Result<()> {
        if self.update_target_position {
            let has_conflict = |tx, ty| {
                for (sx, sy) in &self.snake.body {
                    if *sx == tx && *sy == ty {
                        return true;
                    }
                }
                false
            };

            while has_conflict(self.target.x, self.target.y) {
                self.target = Target::new(3, self.max_x - 1, 3, self.max_y - 1);
            }
            self.update_target_position = false;
        }
        self.stdout
            .queue(cursor::MoveTo(self.target.x, self.target.y))?;
        self.stdout.queue(style::Print("●"))?;
        Ok(())
    }

    fn draw_failure_banner(&mut self) -> Result<()> {
        self.game_over = true;
        let x = (self.max_x / 2) - 12;
        let y = (self.max_y / 2) - 2;
        let s = format!("║       Score: {:05}           ║", self.snake.score);
        execute!(
            self.stdout,
            cursor::MoveTo(x, y),
            style::SetBackgroundColor(style::Color::White),
            style::SetForegroundColor(style::Color::Black),
            style::Print("╔══════════════════════════════╗"),
            cursor::MoveTo(x, y + 1),
            style::Print("║       Game Over              ║"),
            cursor::MoveTo(x, y + 2),
            style::Print(s),
            cursor::MoveTo(x, y + 3),
            style::Print("╚══════════════════════════════╝"),
            style::ResetColor
        )?;
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

    fn check_collision(&mut self) {
        let (head_x, head_y) = self.snake.body.first().unwrap();
        if *head_x == self.target.x && *head_y == self.target.y {
            self.snake_new_head();
            self.snake.score += 1;
            self.update_target_position = true;
        }
    }

    fn check_failure(&self) -> bool {
        let (head_x, head_y) = self.snake.body.first().unwrap();
        for (x, y) in &self.snake.body[1..] {
            if head_x == x && head_y == y {
                return true;
            }
        }
        false
    }

    fn snake_move_delay(&self) -> u64 {
        if self.snake.score > MAX_MOVE_DELAY {
            MAX_MOVE_DELAY
        } else {
            let n = MAX_MOVE_DELAY - self.snake.score;
            std::cmp::max(n, MIN_MOVE_DELAY)
        }
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
