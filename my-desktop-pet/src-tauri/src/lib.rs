use rand::Rng;
use std::sync::Mutex;
use std::time::Instant;
use tauri::Manager;
use tauri::PhysicalSize;
use tauri::State;

#[cfg(target_os = "macos")]
use objc::{msg_send, sel, sel_impl};

#[cfg(target_os = "windows")]
use windows::Win32::Foundation::RECT;
#[cfg(target_os = "windows")]
use windows::Win32::UI::WindowsAndMessaging::{SystemParametersInfoW, SPI_GETWORKAREA};

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
        let floor = effective_height - self.pet_height;
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

// Platform-specific window setup
fn setup_window_properties(window: &tauri::WebviewWindow) {
    // Set up click-through functionality based on platform

    #[cfg(target_os = "macos")]
    unsafe {
        if let Ok(ns_window) = window.ns_window() {
            let ns_window = ns_window as cocoa::base::id;
            let _: () = msg_send![ns_window, setIgnoresMouseEvents: true];
            println!("macOS: Set window to ignore mouse events");
        }
    }

    #[cfg(target_os = "windows")]
    {
        // Use Tauri's built-in function instead of direct Win32 API calls
        if let Err(e) = window.set_ignore_cursor_events(true) {
            println!("Failed to set ignore cursor events: {:?}", e);
        } else {
            println!("Windows: Set window to ignore mouse events");
        }
    }

    // For Linux and other platforms, we rely on the standard Tauri API
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        if let Err(e) = window.set_ignore_cursor_events(true) {
            println!("Failed to set ignore cursor events: {:?}", e);
        } else {
            println!("Set window to ignore cursor events");
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    println!("Starting desktop pet application");

    tauri::Builder::default()
        .manage(AppState {
            pet: Mutex::new(PetState::new(400.0, 300.0)),
        })
        .invoke_handler(tauri::generate_handler![
            get_pet_movement,
            reset_pet_position
        ])
        .setup(|app| {
            // Get the main window
            if let Some(window) = app.get_webview_window("main") {
                // Resize window based on platform
                #[cfg(target_os = "windows")]
                {
                    unsafe {
                        // Get the work area (screen size excluding taskbar)
                        let mut work_area = RECT::default();
                        SystemParametersInfoW(
                            SPI_GETWORKAREA,
                            0,
                            Some(&mut work_area as *mut _ as *mut std::ffi::c_void),
                            windows::Win32::UI::WindowsAndMessaging::SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
                        );

                        // Calculate work area dimensions
                        let width = work_area.right - work_area.left;
                        let height = work_area.bottom - work_area.top;

                        // Set window size to match work area
                        window
                            .set_size(PhysicalSize::new(width as u32, height as u32))
                            .expect("Failed to resize window");

                        // Position at the top-left corner of the work area
                        window
                            .set_position(tauri::Position::Physical(tauri::PhysicalPosition::new(
                                work_area.left,
                                work_area.top,
                            )))
                            .expect("Failed to position window");

                        println!("Resized window to match work area: {}x{}", width, height);
                    }
                }

                // For non-Windows platforms, use the full screen
                #[cfg(not(target_os = "windows"))]
                {
                    if let Some(monitor) = window.primary_monitor().expect("Failed to get monitors")
                    {
                        let size = monitor.size();

                        window
                            .set_size(PhysicalSize::new(size.width, size.height))
                            .expect("Failed to resize window");

                        window
                            .set_position(tauri::Position::Physical(tauri::PhysicalPosition::new(
                                0, 0,
                            )))
                            .expect("Failed to position window");

                        println!(
                            "Resized window to match monitor: {}x{}",
                            size.width, size.height
                        );
                    }
                }

                setup_window_properties(&window);
            } else {
                println!("Warning: Could not find main window");
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Tauri application");
}
