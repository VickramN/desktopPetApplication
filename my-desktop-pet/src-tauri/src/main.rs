// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    my_desktop_pet_lib::run() // This will call the run() function in lib.rs
}
