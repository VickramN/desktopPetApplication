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
#[cfg(target_os = "windows")]
use windows::Win32::Foundation::POINT;
#[cfg(target_os = "windows")]
use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;
// Default window dimensions to ensure consistency
const DEFAULT_WINDOW_WIDTH: f32 = 400.0;
const DEFAULT_WINDOW_HEIGHT: f32 = 300.0;
const PET_WIDTH: f32 = 64.0; // Defined as constants to ensure consistency
const PET_HEIGHT: f32 = 64.0;



#[derive(Debug, Clone, Copy, PartialEq)]
enum AnimationState {
    IdleRight,
    IdleLeft,
    SleepingRight,
    SleepingLeft,
    IdleAlt1Right,
    IdleAlt1Left,
    IdleAlt2Right,
    IdleAlt2Left,
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
            AnimationState::SleepingRight => "sleep-right",
            AnimationState::SleepingLeft => "sleep-left",
            AnimationState::IdleAlt1Right => "idle-alt-1-right",
            AnimationState::IdleAlt1Left => "idle-alt-1-left",
            AnimationState::IdleAlt2Right => "idle-alt-2-right",
            AnimationState::IdleAlt2Left => "idle-alt-2-left",
            AnimationState::RunningRight => "run-right",
            AnimationState::RunningLeft => "run-left",
            AnimationState::JumpingRight => "jump-right",
            AnimationState::JumpingLeft => "jump-left",
            AnimationState::FallingRight => "fall-right",
            AnimationState::FallingLeft => "fall-left",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum EmotionState {
    Lonely,
    Neutral,
    Happy,
    Excited,
}

#[derive(Debug, Clone, Copy)]
struct PetNeeds {
    affection: f32,
    hunger: f32,
    energy: f32,
}

impl PetNeeds {
    fn new() -> Self {
        Self {
            affection: 50.0,
            hunger: 100.0,
            energy: 100.0,
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
    idle_timer: f32,       // how long we've been idle
    idle_duration: f32,    // how long to stay idle before moving
    action_timer: f32,     // how long current walk/run action lasts
    current_action: PetAction,
    love_timer: f32,
    needs: PetNeeds,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum PetAction {
    Idling,
    Walking,
    Running,
    Sleeping,
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
            idle_timer: 0.0,
            idle_duration: 2.0,   // start with a 2 second idle
            action_timer: 0.0,
            current_action: PetAction::Idling,
            love_timer: 0.0,
            needs: PetNeeds::new(),
        }
    }

    fn choose_idle_animation(&mut self) {
        let mut rng = rand::thread_rng();
        let roll: f32 = rng.gen();

        let affection = self.needs.affection;

        let alt_1_chance = if affection < 25.0 {
            0.10
        } else if affection < 50.0 {
            0.20
        } else if affection < 75.0 {
            0.30
        } else {
            0.40
        };

        let alt_2_chance = if affection < 25.0 {
            0.02
        } else if affection < 50.0 {
            0.10
        } else if affection < 75.0 {
            0.15
        } else {
            0.25
        };
    
        self.animation_state = if roll < 1.0 - alt_1_chance - alt_2_chance {
            if self.facing_direction { AnimationState::IdleRight } else { AnimationState::IdleLeft }
        } else if roll < 1.0 - alt_2_chance {
            if self.facing_direction { AnimationState::IdleAlt1Right } else { AnimationState::IdleAlt1Left }
        } else {
            if self.facing_direction { AnimationState::IdleAlt2Right } else { AnimationState::IdleAlt2Left }
        };
    }

    fn is_cursor_over_pet(&self, cursor_x: f32, cursor_y: f32) -> bool {
    const HITBOX_PADDING: f32 = 8.0;

    let left = self.x + HITBOX_PADDING;
    let right = self.x + PET_WIDTH - HITBOX_PADDING;
    let top = self.y + HITBOX_PADDING;
    let bottom = self.y + PET_HEIGHT - HITBOX_PADDING;

    cursor_x >= left
        && cursor_x <= right
        && cursor_y >= top
        && cursor_y <= bottom
    }

    fn emotion_state(&self) -> EmotionState {
        if self.needs.affection < 25.0 {
            EmotionState::Lonely
        } else if self.needs.affection < 50.0 {
            EmotionState::Neutral
        } else if self.needs.affection < 75.0 {
            EmotionState::Happy
        } else {
            EmotionState::Excited
        }
    }

    fn update(&mut self, window_width: f32, window_height: f32) {
    if (self.window_width - window_width).abs() > 1.0
        || (self.window_height - window_height).abs() > 1.0
    {
        self.window_width = window_width;
        self.window_height = window_height;
    }

    let now = Instant::now();
    let mut delta_time = now.duration_since(self.last_update).as_secs_f32();
    self.last_update = now;
    delta_time = delta_time.min(0.05);

    const AFFECTION_DECAY_PER_SECOND: f32 = 1.0;

    if self.love_timer <= 0.0 {
        self.needs.affection =
            (self.needs.affection - AFFECTION_DECAY_PER_SECOND * delta_time)
                .max(0.0);
    }


    if self.love_timer > 0.0 {
        self.love_timer -= delta_time;

        self.velocity_x = 0.0;

        self.current_action = PetAction::Idling;

        return;
}

    const GRAVITY: f32 = 980.0;
    const JUMP_FORCE: f32 = -480.0;
    const WALK_SPEED: f32 = 80.0;
    const RUN_SPEED: f32 = 200.0;
    const FRICTION: f32 = 6.0;        // ground deceleration multiplier
    const MOVEMENT_THRESHOLD: f32 = 8.0;

    let effective_width = if window_width <= 10.0 { DEFAULT_WINDOW_WIDTH } else { window_width };
    let effective_height = if window_height <= 10.0 { DEFAULT_WINDOW_HEIGHT } else { window_height };

    // --- Gravity ---
    if !self.is_on_ground {
        self.velocity_y += GRAVITY * delta_time;
    }

    let mut rng = rand::thread_rng();

    // --- Ground behaviour state machine ---
    if self.is_on_ground {
        match self.current_action {
            PetAction::Idling => {
                // Apply friction to bleed off any residual velocity
                self.velocity_x *= (1.0 - FRICTION * delta_time).max(0.0);

                self.idle_timer += delta_time;
                if self.idle_timer >= self.idle_duration {
                    // Decide next action
                    self.idle_timer = 0.0;

                    let sleep_chance = match self.emotion_state() {
                        EmotionState::Lonely => 0.15,
                        EmotionState::Neutral => 0.10,
                        EmotionState::Happy => 0.07,
                        EmotionState::Excited => 0.05,
                    };

                    let roll: f32 = rng.gen();

                    if roll < sleep_chance{
                        self.current_action = PetAction::Sleeping;
                        self.action_timer = rng.gen_range(20.0..30.0);

                        self.velocity_x = 0.0;

                        self.animation_state = if self.facing_direction {
                            AnimationState::SleepingRight
                        } else {
                            AnimationState:: SleepingLeft
                        };
                    } 
                    else if roll < 0.20 {
                        // Jump
                        self.velocity_y = JUMP_FORCE;
                        let speed = rng.gen_range(60.0..RUN_SPEED);
                        self.velocity_x = if self.facing_direction { speed } else { -speed };
                        self.is_on_ground = false;
                        self.current_action = PetAction::Idling; // reset after landing
                    } else if roll < 0.55 {
                        // Walk
                        self.current_action = PetAction::Walking;
                        self.action_timer = rng.gen_range(1.5..4.0);
                        // Randomly pick a direction
                        self.facing_direction = rng.gen_bool(0.5);
                    } else {
                        // Run
                        self.current_action = PetAction::Running;
                        self.action_timer = rng.gen_range(0.8..2.5);
                        self.facing_direction = rng.gen_bool(0.5);
                    }
                    // Next idle will last 1–4 seconds
                    self.idle_duration = rng.gen_range(1.0..4.0);
                }
            }

            PetAction::Walking => {
                let target_vx = if self.facing_direction { WALK_SPEED } else { -WALK_SPEED };
                // Smoothly accelerate toward walk speed
                self.velocity_x += (target_vx - self.velocity_x) * (FRICTION * delta_time).min(1.0);

                self.action_timer -= delta_time;
                if self.action_timer <= 0.0 {
                    self.current_action = PetAction::Idling;
                    self.idle_timer = 0.0;
                    self.choose_idle_animation();
                }
            }

            PetAction::Running => {
                let target_vx = if self.facing_direction { RUN_SPEED } else { -RUN_SPEED };
                // Faster acceleration for running
                self.velocity_x += (target_vx - self.velocity_x) * (FRICTION * 1.5 * delta_time).min(1.0);

                self.action_timer -= delta_time;
                if self.action_timer <= 0.0 {
                    self.current_action = PetAction::Idling;
                    self.idle_timer = 0.0;
                    self.choose_idle_animation();
                }
            }

            PetAction::Sleeping => {
                self.velocity_x = 0.0;

                self.action_timer -= delta_time;

                self.animation_state = if self.facing_direction {
                    AnimationState::SleepingRight
                } else {
                    AnimationState::SleepingLeft
                };

                if self.action_timer <= 0.0 {
                    self.current_action = PetAction::Idling;
                    self.idle_timer = 0.0;
                    self.idle_duration = rng.gen_range(1.0..4.0);
                }
            }
        }
    }

    // --- Position update ---
    self.x += self.velocity_x * delta_time;
    self.y += self.velocity_y * delta_time;

    // --- Boundaries ---
    let floor = effective_height - PET_HEIGHT;
    if self.y >= floor {
        self.y = floor;
        self.velocity_y = 0.0;
        if !self.is_on_ground {
            // Just landed — go idle briefly
            self.is_on_ground = true;
            self.current_action = PetAction::Idling;
            self.idle_timer = 0.0;
            self.idle_duration = rng.gen_range(0.5..2.0);
            self.choose_idle_animation();
        }
    }

    if self.y < 0.0 {
        self.y = 0.0;
        self.velocity_y = 0.0;
    }

    if self.x < 0.0 {
        self.x = 0.0;
        self.velocity_x = self.velocity_x.abs() * 0.5;
        self.facing_direction = true;
        if self.current_action != PetAction::Idling {
            // Reverse direction instead of stopping
            self.facing_direction = true;
        }
    }

    let right_boundary = effective_width - PET_WIDTH;
    if self.x > right_boundary {
        self.x = right_boundary;
        self.velocity_x = - self.velocity_x.abs() * 0.5;
        self.facing_direction = false;
    }

    // --- Animation state ---
    if self.current_action == PetAction::Sleeping {
        self.animation_state = if self.facing_direction {
            AnimationState::SleepingRight
        } else {
            AnimationState::SleepingLeft
        };
    
    }else if !self.is_on_ground {
        self.animation_state = if self.velocity_y < 0.0 {
            if self.facing_direction { AnimationState::JumpingRight } else { AnimationState::JumpingLeft }
        } else {
            if self.facing_direction { AnimationState::FallingRight } else { AnimationState::FallingLeft }
        };
    } else if self.velocity_x.abs() > RUN_SPEED * 0.6 {
        self.animation_state = if self.velocity_x > 0.0 { AnimationState::RunningRight } else { AnimationState::RunningLeft };
    } else if self.velocity_x.abs() > MOVEMENT_THRESHOLD {
        self.animation_state = if self.velocity_x > 0.0 { AnimationState::RunningRight } else { AnimationState::RunningLeft };
    } else {
        // While the pet is waiting, occasionally use one of the extra idle variants.
        // The frontend will fall back to normal idle if the current pet does not define it.
        let currently_idle = matches!(
            self.animation_state,
            AnimationState::IdleRight
                | AnimationState::IdleLeft
                | AnimationState::IdleAlt1Right
                | AnimationState::IdleAlt1Left
                | AnimationState::IdleAlt2Right
                | AnimationState::IdleAlt2Left
        );

        if !currently_idle {
            self.choose_idle_animation();
        }
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
fn get_pet_mood(state: State<AppState>) -> String {
    let pet = state.pet.lock().unwrap();

    match pet.emotion_state() {
        EmotionState::Lonely => "Lonely".to_string(),
        EmotionState::Neutral => "Neutral".to_string(),
        EmotionState::Happy => "Happy".to_string(),
        EmotionState::Excited => "Excited".to_string(),
    }
}
#[tauri::command]
fn pet_pet(state: State<AppState>) {
    let mut pet = state.pet.lock().unwrap();

    let was_already_loved = pet.love_timer > 0.0;

    pet.love_timer = 3.0;

    

    let affection_gain = match pet.emotion_state(){
        EmotionState::Lonely => 8.0,
        EmotionState::Neutral => 5.0,
        EmotionState::Happy => 3.0,
        EmotionState::Excited => 1.5,
    }

    pet.needs.affection = (pet.needs.affection + affection_gain).min(100.0);

    println!("Affection: {}", pet.needs.affection);
    pet.velocity_x = 0.0;
    pet.velocity_y = 0.0;
    pet.current_action = PetAction::Idling;

    if !was_already_loved {
        pet.choose_idle_animation();
    }
}

#[tauri::command]
fn get_pet_stats(state: State<AppState>) -> (f32, f32, f32, String) {
    let pet = state.pet.lock().unwrap();


    let mood = match pet.emotion_state() {
        EmotionState::Lonely => "Lonely",
        EmotionState::Neutral => "Neutral",
        EmotionState::Happy => "Happy",
        EmotionState::Excited => "Excited",
    };
    (
        pet.needs.affection,
        pet.needs.hunger,
        pet.needs.energy,
        mood.to_string(),
    )
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

#[cfg(target_os = "windows")]
fn get_cursor_position() -> Option<(f32, f32)> {
    unsafe {
        let mut point = POINT { x: 0, y: 0 };

        if GetCursorPos(&mut point).as_bool() {
            Some((point.x as f32, point.y as f32))
        } else {
            None
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
            set_click_through,
            pet_pet,
            get_pet_stats
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

                        const BORDER_FIX: i32 = 8;

                        // Set window size to match work area
                        window
                            .set_size(PhysicalSize::new(width as u32, height as u32))
                            .expect("Failed to resize window");

                        // Position at the top-left corner of the work area
                        window
                            .set_position(tauri::Position::Physical(tauri::PhysicalPosition::new(
                                work_area.left + BORDER_FIX,
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
                println!("Window is now visible and ready");

                #[cfg(target_os = "windows")]
                {
                    let app_handle = app.handle().clone();

                    std::thread::spawn(move || {
                        let mut is_currently_click_through = true;

                        loop {
                            std::thread::sleep(std::time::Duration::from_millis(150));

                            let Some(window) = app_handle.get_webview_window("main") else {
                                continue;
                            };

                            let Some((cursor_x, cursor_y)) = get_cursor_position() else {
                                continue;
                            };

                            let Some(state) = app_handle.try_state::<AppState>() else {
                                continue;
                            };

                            let pet = state.pet.lock().unwrap();

                            let cursor_over_pet =
                                pet.is_cursor_over_pet(cursor_x, cursor_y);

                            drop(pet);

                            let should_be_click_through = !cursor_over_pet;

                            if should_be_click_through != is_currently_click_through {
                                if let Err(error) =
                                    window.set_ignore_cursor_events(
                                should_be_click_through
                                    )
                                {
                                    println!(
                                        "Failed to toggle click-through: {:?}",
                                        error
                                    );
                                } else {
                                    is_currently_click_through =
                                        should_be_click_through;

                    
                                }
                            }
                        }
                    });
                }

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
