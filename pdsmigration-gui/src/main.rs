#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use crate::app::PdsMigrationApp;
use tracing::Level;
use tracing_subscriber::filter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

mod agent;
mod app;
mod error_window;
mod errors;
mod session_config;
mod styles;
mod success_window;

fn main() -> eframe::Result {
    use std::time::Duration;
    use tokio::runtime::Runtime;

    let filter = filter::Targets::new().with_target("pdsmigration", Level::INFO);

    let collector = egui_tracing::EventCollector::default();
    tracing_subscriber::registry()
        .with(collector.clone())
        .with(filter)
        .init();

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
            Ok(Box::new(PdsMigrationApp::new(cc, collector)))
        }),
    )
}
