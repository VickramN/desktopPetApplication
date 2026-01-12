use rand::Rng;
use std::sync::Mutex;
use std::time::Instant;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::Manager;
use tauri::PhysicalSize;
use tauri::State;
use tauri::Emitter;

#[cfg(target_os = "macos")]
use objc::{msg_send, sel, sel_impl};

#[cfg(target_os = "windows")]
use windows::Win32::Foundation::RECT;
#[cfg(target_os = "windows")]
use windows::Win32::UI::WindowsAndMessaging::{SystemParametersInfoW, SPI_GETWORKAREA};

// Default window dimensions to ensure consistency
const DEFAULT_WINDOW_WIDTH: f32 = 400.0;
const DEFAULT_WINDOW_HEIGHT: f32 = 300.0;
const PET_WIDTH: f32 = 32.0; // Defined as constants to ensure consistency
const PET_HEIGHT: f32 = 30.0;

//the pet height has to be roughly 83-85px to sit on top of the menu bar for macOS. Should be noted this may vary depending on model
//Will have to test for Windows.

//Should write a conditional to change pet height dependant on OS or chang boundary detection

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
    window_width: f32,
    window_height: f32,
    animation_state: AnimationState,
    facing_direction: bool, // true for right, false for left
}

impl PetState {
    fn new(window_width: f32, window_height: f32) -> Self {
        // Use sensible defaults for initial window size from config (400x300)
        let effective_width = if window_width <= 0.0 {
            DEFAULT_WINDOW_WIDTH
        } else {
            window_width
        };
        let effective_height = if window_height <= 0.0 {
            DEFAULT_WINDOW_HEIGHT
        } else {
            window_height
        };

        println!(
            "Initializing pet with window size: {}x{}",
            effective_width, effective_height
        );

        PetState {
            x: effective_width / 2.0 - PET_WIDTH / 2.0,
            y: effective_height - PET_HEIGHT,
            velocity_x: 0.0,
            velocity_y: 0.0,
            last_update: Instant::now(),
            is_on_ground: true,
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

        const GRAVITY: f32 = 980.0;
        const JUMP_FORCE: f32 = -500.0;
        const MAX_SPEED_X: f32 = 200.0;
        const BOUNCE_FACTOR_X: f32 = 0.8; // Energy retention on x-bounce
        const RIGHT_BOUNCE_FACTOR: f32 = 0.5; // More energy loss on right boundary

        if !self.is_on_ground {
            self.velocity_y += GRAVITY * delta_time;
        }

        let mut rng = rand::thread_rng();

        if self.is_on_ground && rng.gen_bool(0.01) {
            self.velocity_y = JUMP_FORCE;

            if self.facing_direction {
                self.velocity_x = rng.gen_range(0.0..MAX_SPEED_X);
            } else {
                self.velocity_x = rng.gen_range(-MAX_SPEED_X..0.0);
            }

            if rng.gen_bool(0.1) {
                self.velocity_x *= -0.5;
            }

            self.is_on_ground = false;
        }

        // Update Position
        self.x += self.velocity_x * delta_time;
        self.y += self.velocity_y * delta_time;

        // Get effective window dimensions with non-zero check
        let effective_width = if window_width <= 10.0 {
            DEFAULT_WINDOW_WIDTH
        } else {
            window_width
        };
        let effective_height = if window_height <= 10.0 {
            DEFAULT_WINDOW_HEIGHT
        } else {
            window_height
        };

        // Floor Boundary (bottom of window)
        let floor = effective_height - PET_HEIGHT;
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
            self.velocity_x = -self.velocity_x * BOUNCE_FACTOR_X; // Bounce with loss of energy
            self.facing_direction = true;
        }

        // Right Boundary
        let right_boundary = effective_width - PET_WIDTH;
        if self.x > right_boundary {
            self.x = right_boundary;
            self.velocity_x = -self.velocity_x * RIGHT_BOUNCE_FACTOR; // Bounce with more loss of energy
            self.facing_direction = false;
        }

        const MOVEMENT_THRESHOLD: f32 = 5.0;

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
        } else if self.velocity_x.abs() > MOVEMENT_THRESHOLD {
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

#[tauri::command]
fn set_click_through(app: tauri::AppHandle, enabled: bool) {
    if let Some(window) = app.get_webview_window("main") {
        if let Err(e) = window.set_ignore_cursor_events(enabled) {
            println!("Failed to set click-through: {:?}", e);
        } else {
            println!("Click-through set to: {}", enabled);
        }
    }
}

// Platform-specific window setup
#[allow(unexpected_cfgs)]
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
            pet: Mutex::new(PetState::new(1920.0, 1032.0)),
        })
        .invoke_handler(tauri::generate_handler![
            get_pet_movement,
            reset_pet_position,
            set_click_through
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

                        println!("Windows: Configured to work area {}x{} at ({}, {})", 
                            width, height, work_area.left, work_area.top);
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

                window.show().expect("Failed to show window");
                println!("Window is now visible and ready")
            } else {
                println!("Warning: Could not find main window");
            }
            
            let settings = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&settings, &quit])?;

            let _tray = TrayIconBuilder::with_id("main-tray")
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .on_menu_event(|app, event| {
                    match event.id.as_ref(){
                        "settings"=> {
                            println!("Settings clicked from tray");
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.set_ignore_cursor_events(false);
                                let _ = window.emit("open-settings", ());
                                let _ = window.set_focus();
                            }
                        }
                        "quit" => {
                            
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.close();
                            }

                            if let Some(window) = app.get_webview_window("pet"){
                                let _ = window.close();
                            }

                            std::thread::sleep(std::time::Duration::from_millis(50));

                            app.exit(0);
                        }
                        _ => {}
                     }
                })
                .build(app)?;


            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Tauri application");
}
