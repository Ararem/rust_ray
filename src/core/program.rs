use glium::glutin;
use glium::glutin::event::{Event, WindowEvent};
use glium::glutin::event_loop::{ControlFlow, EventLoop};
use glium::glutin::window::WindowBuilder;
use glium::{Display, Surface};
use imgui::*;
use imgui::{Context, FontConfig, FontGlyphRanges, FontSource, Ui};
use imgui_glium_renderer::Renderer;
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use pretty_assertions::{self, assert_eq, assert_ne, assert_str_eq};
use std::path::Path;
use std::time::Instant;
use color_eyre::Report;
use tracing::*;
use crate::core::ui::{UiConfig, UiSystem};
use crate::core::ui;

#[instrument]
pub fn run_program() -> Result<(), Report> {
    info!("HELLO WORLD");
    let ui_system = ui::init("Test Title", UiConfig { multisampling: 2, vsync: true, hardware_acceleration: Some(false) })?;

    Ok(())
}
