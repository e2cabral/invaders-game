use std::error::Error;
use rusty_audio::Audio;
use std::{io, thread};
use crossterm::{terminal, ExecutableCommand, event};
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::cursor::{Hide, Show};
use std::time::{Duration, Instant};
use crossterm::event::{KeyCode, Event};
use std::sync::mpsc;
use invaders::frame;
use invaders::render;
use invaders::frame::{new_frame, Drawable};
use invaders::player::Player;
use invaders::invaders::Invaders;

fn main() -> Result<(), Box<dyn Error>> {
    // <editor-fold desc="Setup the start of the game">
    let mut audio = Audio::new();
    audio.add("explode", "explode.wav");
    audio.add("lose", "lose.wav");
    audio.add("move", "move.wav");
    audio.add("pew", "pew.wav");
    audio.add("startup", "startup.wav");
    audio.add("win", "win.wav");

    audio.play("startup");
    // </editor-fold>

    // <editor-fold desc="Terminal Section">
    let mut stdout = io::stdout();
    terminal::enable_raw_mode()?;
    stdout.execute(EnterAlternateScreen)?;
    stdout.execute(Hide)?;
    // </editor-fold>

    // <editor-fold desc="Separate thread to render the loop">
    let (render_tx, render_rx) = mpsc::channel();
    let render_handle = thread::spawn(move || {
        let mut last_frame = frame::new_frame();
        let mut stdout = io::stdout();
        render::render(&mut stdout, &last_frame, &last_frame, true);
        loop {
            let curr_frame = match render_rx.recv() {
                Ok(x) => x,
                Err(_) => break,
            };
            render::render(&mut stdout, &last_frame, &curr_frame, false);
            last_frame = curr_frame;
        }
    });
    // </editor-fold>

    // <editor-fold desc="Game Loop Section">
    // <editor-fold desc="Player Implementation">
    let mut player = Player::new();
    // </editor-fold>
    let mut instant = Instant::now();
    let mut invaders = Invaders::new();
    'gameloop: loop {
        let delta = instant.elapsed();
        instant = Instant::now();
        // <editor-fold desc="Per Frame Init Section">
        let mut curr_frame = new_frame();
        // </editor-fold>
        // <editor-fold desc="Inputs Section">
        while event::poll(Duration::default())? {
            if let Event::Key(key_event) = event::read()? {
                match key_event.code {
                    KeyCode::Left => player.move_left(),
                    KeyCode::Right => player.move_right(),
                    KeyCode::Char(' ') | KeyCode::Enter => {
                        if player.shoot() {
                            audio.play("pew");
                        }
                    }
                    KeyCode::Esc | KeyCode::Char('q') => {
                        audio.play("lose");
                        break 'gameloop;
                    },
                    _ => {}
                }
            }
        }
        // </editor-fold>
        player.update(delta);
        if invaders.update(delta) {
            audio.play("move");
        }
        if player.detect_hits(&mut invaders) {
            audio.play("explode");
        }
        // <editor-fold desc="Draw & Render Section">
        let drawables: Vec<&dyn Drawable> = vec![&player, &invaders];
        for drawable in drawables {
            drawable.draw(&mut curr_frame);
        }
        let _ = render_tx.send(curr_frame);
        thread::sleep(Duration::from_millis(1));
        // </editor-fold>

        if invaders.all_killed() {
            audio.play("win");
            break 'gameloop;
        }
        if invaders.reached_bottom() {
            audio.play("lose");
            break 'gameloop;
        }
    }
    // </editor-fold>

    // <editor-fold desc="Cleanup Section">
    drop(render_tx);
    render_handle.join().unwrap();
    audio.wait();
    stdout.execute(Show)?;
    stdout.execute(LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;
    Ok(())
    // </editor-fold>
}
