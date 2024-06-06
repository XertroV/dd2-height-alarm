
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use eframe::icon_data::{from_png_bytes, IconDataExt};
use egui::vec2;
use log::{info, warn};
use std::{
    fs::{self, File},
    io::{self, Write},
    path::Path,
    sync::{Arc, Mutex},
};

mod app;
mod players;
mod sounds;
mod play_sound;

fn main() {
    log::set_max_level(log::LevelFilter::Debug);
    env_logger::init();
    let icon_data = from_png_bytes(include_bytes!("../bell-icon.png")).unwrap();

    let native_opts = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(vec2(300.0, 800.0))
            .with_min_inner_size(vec2(300.0, 800.0))
            .with_title("DD2 Height Alarm")
            // .with_resizable(false)
            .with_close_button(true)
            // .with_max_inner_size(vec2(300.0, 800.0))
            .with_taskbar(true)
            .with_decorations(true)
            // .with_maximize_button(false)
            // .with_active(true)
            .with_icon(icon_data),
        ..Default::default()
    };
    let r = eframe::run_native(
        "DD2 Height Alarm",
        native_opts,
        Box::new(|cc| Box::new(app::HeightsApp::new())),
    );
    if let Err(err) = r {
        eprintln!("Error: {}", err);
    }
}
