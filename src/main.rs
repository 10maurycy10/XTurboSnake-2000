// XTurboSnake 2000
// -----------------
// A state of the art snake game.
// 
// Tested to work on Debain 11/10
// Should work on BSDs, OSX, Windows 11-7, and andrioid 
//
// Audio requires alsa/pulseaudio/pipewire/windows-api

use sdl2::audio::AudioCallback;
use sdl2::audio::AudioSpecDesired;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::AudioSubsystem;

const DEFAULT_PIXEL_SCALING: usize = 20;
const WORLD_W: usize = 8;
const WORLD_H: usize = 8;
const WORLD_A: usize = WORLD_W * WORLD_H;
const SNAKE_INIT_SPEED: u32 = 30;

// snake direction states
const DIR_DOWN: u32 = 0;
const DIR_UP: u32 = 1;
const DIR_LEFT: u32 = 2;
const DIR_RIGHT: u32 = 3;

// code for making sounds with sdl2
struct SquareWave {
    phase_inc: f32,
    phase: f32,
    volume: f32,
}

impl AudioCallback for SquareWave {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        // Generate a square wave
        for x in out.iter_mut() {
            *x = if self.phase <= 0.5 {
                self.volume
            } else {
                -self.volume
            };
            self.phase = (self.phase + self.phase_inc) % 1.0;
        }
    }
}

// a function to play a beep this takes a long time (20ms)
// TODO: Fix this.
fn beep(audio: &mut AudioSubsystem, spec: &AudioSpecDesired) {
    use std::time::Duration;
    let device = audio
        .open_playback(None, spec, |spec| {
            // initialize the audio callback
            SquareWave {
                phase_inc: 500.0 / spec.freq as f32,
                phase: 0.0,
                volume: 0.25,
            }
        })
        .unwrap();
    device.resume();
    std::thread::sleep(Duration::from_millis(20));
}


// container for snake state
struct Snake {
    len: u32,
    screen: [u8; WORLD_A],
    bits: [u8; WORLD_A],
    x: u32,
    y: u32,
    ctr: usize,
    dir: u32,
    new_dir: u32,
    snake_sp: u32,
    fruit_pos: usize,
    fruit_rng: usize,
    game_over_timer: u32,
    snake_move_timer: u32,
    game_over: bool,
}

// initalize new snake state
fn init_snake() -> Snake {
    Snake {
        len: 1,
        screen: [0; WORLD_A],
        bits: [0; WORLD_A],
        x: 3,
        y: 4,
        snake_sp: SNAKE_INIT_SPEED,
        dir: DIR_DOWN,
        new_dir: DIR_DOWN,
        fruit_pos: 20,
        fruit_rng: 0,
        ctr: 0,
        game_over_timer: 0,
        snake_move_timer: 0,
        game_over: false,
    }
}

// ran once per frame
fn snake_tick(snake: &mut Snake, audio: &mut AudioSubsystem, spec: &AudioSpecDesired) {
    snake.dir = snake.new_dir;

    if !snake.game_over {
        if snake.snake_move_timer == 0 {
            // reset the movement timer
            snake.snake_move_timer = snake.snake_sp;
            // move the snake
            match snake.dir {
                DIR_DOWN => {
                    if (WORLD_H as u32 -1) == snake.y {
                        snake.y = 0
                    } else {
                        snake.y += 1
                    }
                }
                DIR_UP => {
                    if snake.y == 0 {
                        snake.y = 7
                    } else {
                        snake.y -= 1
                    }
                }
                DIR_LEFT => {
                    if snake.x == 0 {
                        snake.x = 7
                    } else {
                        snake.x -= 1
                    }
                }
                DIR_RIGHT => {
                    if snake.x == (WORLD_W as u32-1) {
                        snake.x = 0
                    } else {
                        snake.x += 1
                    }
                }
                _ => unreachable!(),
            }
            // check for fruit colisone
            if (snake.x + snake.y * WORLD_H as u32) as usize == snake.fruit_pos {
                beep(audio, spec);
                // set the new pos based on the rng
                snake.fruit_pos = snake.fruit_rng % 64;
                // scamble the rng so that if a fruit is eaten w/out input it will be moved
                snake.fruit_rng += 43;
                snake.len += 1
            }

            let mut bit_i = snake.len;
            // move all snake bits to a higher index
            while (bit_i > 0) {
                snake.bits[bit_i as usize] = snake.bits[bit_i as usize - 1];
                bit_i -= 1;
            }

            // check for snake-snake colisions
            // this is done after shifting snake bits to prevent colisons with a part of the tail that wont be rendered.
            // however this allow the creation of a seemless snake loop.
            for i in 0..snake.len {
                if (snake.x + snake.y * WORLD_H as u32) as u8 == snake.bits[i as usize] {
                    // end game
		    println!("lost game at len of {}",snake.len);
		    snake.game_over = true
                }
            }

            // add the snake's location to the bits array
            snake.bits[0] = (snake.x + snake.y * WORLD_H as u32) as u8
        } else {
            snake.snake_move_timer -= 1
        }
    // if game is over..
    } else {
        // wait until timer is over
        if snake.game_over_timer == 0 {
            // reset timer
            snake.game_over_timer = 20;
	    if snake.len != 0 {
                // if the len is not zero reduce len and play a beep
                beep(audio, spec);
                snake.len -= 1;
            } else {
		snake.len = 1;
		snake.game_over = false;
	    }
        } else {
            snake.game_over_timer -= 1;
        }
    }

    // redering code ...
    // clear screen
    for i in 0..(WORLD_W * WORLD_H) {
        snake.screen[i] = 0
    }
    // draw snake bits
    for i in 0..snake.len {
        snake.screen[snake.bits[i as usize] as usize] = 1
    }
    // draw fruit
    if snake.ctr % 32 > 16 {
        snake.screen[snake.fruit_pos] = 1
    }
    // increment frame counter
    snake.ctr += 1;
}

pub fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    // open a window with X/Wayland/Windows-api
    let window = video_subsystem
        .window(
            "XTurboSnake 2000",
            (WORLD_H * DEFAULT_PIXEL_SCALING) as u32,
            (WORLD_W * DEFAULT_PIXEL_SCALING) as u32,
        )
        .position_centered()
        //        .resizable()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window
        .into_canvas()
        .build()
        .expect("cant initalize renderer!");

    // allow playing audio w/ pulse/pipewire/alsa/windowsapi
    let mut audio_subsystem = sdl_context.audio().unwrap();

    let audio_spec = AudioSpecDesired {
        freq: Some(44100),
        channels: Some(1), // mono
        samples: None,     // default sample size
    };

    canvas.clear();
    canvas.present();

    // init event loop
    let mut event_pump = sdl_context.event_pump()?;

    // init game data
    let mut snake = init_snake();

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                Event::KeyDown { keycode, .. } => {
                    // add frame counter to rng to prevent predictable berhavior
                    snake.fruit_rng += snake.ctr;
                    // change the new_dir of the snake
                    match keycode {
                        Some(Keycode::W) => {
                            if snake.dir != DIR_DOWN {
                                snake.new_dir = DIR_UP
                            }
                        }
                        Some(Keycode::A) => {
                            if snake.dir != DIR_RIGHT {
                                snake.new_dir = DIR_LEFT
                            }
                        }
                        Some(Keycode::S) => {
                            if snake.dir != DIR_UP {
                                snake.new_dir = DIR_DOWN
                            }
                        }
                        Some(Keycode::D) => {
                            if snake.dir != DIR_LEFT {
                                snake.new_dir = DIR_RIGHT
                            }
                        }
                        _ => (),
                    }
                }
                _ => {}
            }
        }

        // run snake, and update snake.screen
        snake_tick(&mut snake, &mut audio_subsystem, &audio_spec);

        // DRAW
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();

        canvas.set_draw_color(Color::RGB(200, 0, 0));
        for y in 0..WORLD_H {
            for x in 0..WORLD_W {
                if snake.screen[x + y * WORLD_W] != 0 {
                    canvas.fill_rect(Rect::new(
                        (x * DEFAULT_PIXEL_SCALING) as i32,
                        (y * DEFAULT_PIXEL_SCALING) as i32,
                        DEFAULT_PIXEL_SCALING as u32,
                        DEFAULT_PIXEL_SCALING as u32,
                    )).unwrap();
                }
            }
        }
        // draw the snake head a lighter color
        canvas.set_draw_color(Color::RGB(255, 0, 0));
        canvas.fill_rect(Rect::new(
            snake.x as i32 * DEFAULT_PIXEL_SCALING as i32,
            snake.y as i32 * DEFAULT_PIXEL_SCALING as i32,
            DEFAULT_PIXEL_SCALING as u32,
            DEFAULT_PIXEL_SCALING as u32,
        )).unwrap();

        //
        canvas.present();

        use std::time::Duration;
        // arbitrary delay
        ::std::thread::sleep(Duration::new(0, 1_000_000u32 * 8));
    }

    println!("Quit at len: {}",snake.len);

    Ok(())
}
