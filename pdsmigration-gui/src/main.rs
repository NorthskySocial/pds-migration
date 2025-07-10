#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use crate::app::PdsMigrationApp;

mod agent;
mod app;
mod errors;
mod existing_pds_page;
mod home_page;
mod new_pds_page;
mod session_config;
mod styles;

fn main() -> eframe::Result {
    use std::time::Duration;
    use tokio::runtime::Runtime;

    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let rt = Runtime::new().expect("Unable to create Runtime");

    // Enter the runtime so that `tokio::spawn` is available immediately.
    let _enter = rt.enter();

    // Execute the runtime in its own thread.
    // The future doesn't have to do anything. In this example, it just sleeps forever.
    std::thread::spawn(move || {
        rt.block_on(async {
            loop {
                tokio::time::sleep(Duration::from_secs(3600)).await;
            }
        })
    });

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default(),
        ..Default::default()
    };

    eframe::run_native(
        "PDS Migration Tool",
        options,
        Box::new(|cc| {
            styles::setup_fonts(&cc.egui_ctx);
            Ok(Box::<PdsMigrationApp>::default())
        }),
    )
}
