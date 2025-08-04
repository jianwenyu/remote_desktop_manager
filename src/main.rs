#![windows_subsystem = "windows"]

mod app;
mod client;
mod encryption;

use app::AppState;

use eframe::NativeOptions;

fn main() {
    println!("Remote Desktop Manager is running.");
    let native_options = NativeOptions {
        window_builder: Some(Box::new(|builder| {
            builder
                .with_inner_size(eframe::epaint::Vec2::new(600.0, 400.0))
                .with_title("Remote Desktop Manager")
        })),
        ..Default::default()
    };
    let _ = eframe::run_native(
        "Remote Desktop Manager",
        native_options,
        Box::new(|_cc| Box::new(AppState::new())),
    );
}
