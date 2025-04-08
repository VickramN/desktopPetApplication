use rand::Rng;
use std::sync::Mutex;
use std::time::Instant;
use tauri::State;

#[derive(Debug, Clone, Copy, PartialEq)]
enum AnimationState {
    IdleRight,
    IdleLeft,
    RunningRight,
    RunningLeft,
    JumpingRight,
    JumpingLeft,
    FallingRight,
    FallingLeft,
}

impl AnimationState {
    fn to_string(&self) -> &'static str {
        match self {
            AnimationState::IdleRight => "idle-right",
            AnimationState::IdleLeft => "idle-left",
            AnimationState::RunningRight => "run-right",
            AnimationState::RunningLeft => "run-left",
            AnimationState::JumpingRight => "jump-right",
            AnimationState::JumpingLeft => "jump-left",
            AnimationState::FallingRight => "fall-right",
            AnimationState::FallingLeft => "fall-left",
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct PetState {
    x: f32,
    y: f32,
    velocity_x: f32,
    velocity_y: f32,
    last_update: Instant,
    is_on_ground: bool,
    pet_width: f32,
    pet_height: f32,
    window_width: f32,
    window_height: f32,
    animation_state: AnimationState,
    facing_direction: bool, // true for right, false for left
}

impl PetState {
    fn new(window_width: f32, window_height: f32) -> Self {
        let pet_width = 100.0;
        let pet_height = 100.0;

        // Use sensible defaults for initial window size from config (400x300)
        let effective_width = if window_width <= 0.0 {
            400.0
        } else {
            window_width
        };
        let effective_height = if window_height <= 0.0 {
            300.0
        } else {
            window_height
        };

        println!(
            "Initializing pet with window size: {}x{}",
            effective_width, effective_height
        );

        PetState {
            x: effective_width / 2.0 - pet_width / 2.0,
            y: effective_height - pet_height,
            velocity_x: 0.0,
            velocity_y: 0.0,
            last_update: Instant::now(),
            is_on_ground: true,
            pet_width,
            pet_height,
            window_width: effective_width,
            window_height: effective_height,
            animation_state: AnimationState::IdleRight,
            facing_direction: true,
        }
    }

    fn update(&mut self, window_width: f32, window_height: f32) {
        // Only log when window size actually changes to reduce spam
        if (self.window_width - window_width).abs() > 1.0
            || (self.window_height - window_height).abs() > 1.0
        {
            println!(
                "Window size changed: {}x{} -> {}x{}",
                self.window_width, self.window_height, window_width, window_height
            );
            self.window_width = window_width;
            self.window_height = window_height;
        }

        let now = Instant::now();
        let mut delta_time = now.duration_since(self.last_update).as_secs_f32();
        self.last_update = now;

        // Cap delta time to prevent jumps after application freeze
        delta_time = delta_time.min(0.05);

        let gravity = 980.0;
        let jump_force = -500.0;
        let max_speed_x = 200.0;

        if !self.is_on_ground {
            self.velocity_y += gravity * delta_time;
        }

        let mut rng = rand::thread_rng();
        if self.is_on_ground && rng.gen_bool(0.01) {
            self.velocity_y = jump_force;
            self.velocity_x = rng.gen_range(-max_speed_x..max_speed_x);
            self.is_on_ground = false;
        }

        // Update Position
        self.x += self.velocity_x * delta_time;
        self.y += self.velocity_y * delta_time;

        // Get effective window dimensions with non-zero check
        let effective_width = if window_width <= 10.0 {
            400.0
        } else {
            window_width
        };
        let effective_height = if window_height <= 10.0 {
            300.0
        } else {
            window_height
        };

        // Floor Boundary (bottom of window)
        let floor = effective_height - self.pet_height - 15.0;
        if self.y > floor {
            self.y = floor;
            self.velocity_y = 0.0;
            self.is_on_ground = true;
        }

        // Ceiling Boundary (top of window)
        if self.y < 0.0 {
            self.y = 0.0;
            self.velocity_y = 0.0;
        }

        // Left Boundary
        if self.x < 0.0 {
            self.x = 0.0;
            self.velocity_x = -self.velocity_x * 0.8; // Bounce with loss of energy
            self.facing_direction = true;
        }

        // Right Boundary
        let right_boundary = effective_width - self.pet_width;
        if self.x > right_boundary {
            self.x = right_boundary;
            self.velocity_x = -self.velocity_x * 0.5; // Bounce with more loss of energy
            self.facing_direction = false;
        }

        if !self.is_on_ground {
            if self.velocity_y < 0.0 {
                // Going up (jumping)
                self.animation_state = if self.facing_direction {
                    AnimationState::JumpingRight
                } else {
                    AnimationState::JumpingLeft
                };
            } else {
                // Coming down (falling)
                self.animation_state = if self.facing_direction {
                    AnimationState::FallingRight
                } else {
                    AnimationState::FallingLeft
                };
            }
        } else if self.velocity_x.abs() > 5.0 {
            // Running
            self.animation_state = if self.velocity_x >= 0.0 {
                AnimationState::RunningRight
            } else {
                AnimationState::RunningLeft
            };
        } else {
            // Idle
            self.animation_state = if self.facing_direction {
                AnimationState::IdleRight
            } else {
                AnimationState::IdleLeft
            };
        }
    }
}

struct AppState {
    pet: Mutex<PetState>,
}

#[tauri::command]
fn get_pet_movement(
    state: State<AppState>,
    window_width: f32,
    window_height: f32,
) -> (f32, f32, String) {
    let mut pet = state.pet.lock().unwrap();

    // Update pet with the current window dimensions
    pet.update(window_width, window_height);

    (pet.x, pet.y, pet.animation_state.to_string().to_string())
}

#[tauri::command]
fn reset_pet_position(
    state: State<AppState>,
    window_width: f32,
    window_height: f32,
) -> (f32, f32, String) {
    let mut pet = state.pet.lock().unwrap();
    *pet = PetState::new(window_width, window_height);
    (pet.x, pet.y, pet.animation_state.to_string().to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    println!("Starting desktop pet application");

    tauri::Builder::default()
        .manage(AppState {
            pet: Mutex::new(PetState::new(400.0, 300.0)), // Match tauri.conf.json initial size
        })
        .invoke_handler(tauri::generate_handler![
            get_pet_movement,
            reset_pet_position
        ])
        .run(tauri::generate_context!())
        .expect("error while running Tauri application");
}
