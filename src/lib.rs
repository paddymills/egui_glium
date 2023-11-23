//! [`egui`] bindings for [`glium`](https://github.com/glium/glium).
//!
//! The main type you want to use is [`EguiGlium`].
//!
//! If you are writing an app, you may want to look at [`eframe`](https://docs.rs/eframe) instead.
//!
//! ## Feature flags
#![cfg_attr(feature = "document-features", doc = document_features::document_features!())]
//!

#![allow(clippy::float_cmp)]
#![allow(clippy::manual_range_contains)]
#![forbid(unsafe_code)]

mod painter;
use egui::ViewportIdPair;
use glium::glutin::surface::WindowSurface;
pub use painter::Painter;

pub use egui_winit;

use egui_winit::winit::event_loop::EventLoopWindowTarget;
pub use egui_winit::EventResponse;

// ----------------------------------------------------------------------------

/// Convenience wrapper for using [`egui`] from a [`glium`] app.
pub struct EguiGlium {
    pub egui_ctx: egui::Context,
    pub egui_winit: egui_winit::State,
    pub painter: crate::Painter,

    shapes: Vec<egui::epaint::ClippedShape>,
    textures_delta: egui::TexturesDelta,
}

impl EguiGlium {
    pub fn new<E>(
        display: &glium::Display<WindowSurface>,
        window: &winit::window::Window,
        event_loop: &EventLoopWindowTarget<E>,
    ) -> Self {
        let painter = crate::Painter::new(display);

        let pixels_per_point = window.scale_factor() as f32;
        let mut egui_winit = egui_winit::State::new(event_loop, Some(pixels_per_point), Some(painter.max_texture_side()));
        // egui_winit.set_max_texture_side(painter.max_texture_side());
        // egui_winit.set_pixels_per_point(pixels_per_point);

        Self {
            egui_ctx: Default::default(),
            egui_winit,
            painter,
            shapes: Default::default(),
            textures_delta: Default::default(),
        }
    }

    pub fn on_event(&mut self, event: &winit::event::WindowEvent) -> EventResponse {
        self.egui_winit.on_window_event(&self.egui_ctx, event)
    }

    /// Returns `true` if egui requests a repaint.
    ///
    /// Call [`Self::paint`] later to paint.
    pub fn run(
        &mut self,
        window: &winit::window::Window,
        run_ui: impl FnMut(&egui::Context),
    ) -> std::time::Duration {
        let raw_input = self.egui_winit.take_egui_input(window, ViewportIdPair::from_self_and_parent(self.egui_ctx.viewport_id(), self.egui_ctx.parent_viewport_id()));
        let egui::FullOutput {
            platform_output,
            // repaint_after,
            textures_delta,
            shapes,
            pixels_per_point,
            viewport_output,
        } = self.egui_ctx.run(raw_input, run_ui);

        self.egui_winit
            .handle_platform_output(window, self.egui_ctx.viewport_id(), &self.egui_ctx, platform_output);

        self.shapes = shapes;
        self.textures_delta.append(textures_delta);

        std::time::Duration::from_secs(0)
        // TODO: re-impl this
        // repaint_after
    }

    /// Paint the results of the last call to [`Self::run`].
    pub fn paint<T: glium::Surface>(
        &mut self,
        display: &glium::Display<WindowSurface>,
        target: &mut T,
    ) {
        let shapes = std::mem::take(&mut self.shapes);
        let textures_delta = std::mem::take(&mut self.textures_delta);
        let clipped_primitives = self.egui_ctx.tessellate(shapes, self.egui_ctx.pixels_per_point());
        self.painter.paint_and_update_textures(
            display,
            target,
            self.egui_ctx.pixels_per_point(),
            &clipped_primitives,
            &textures_delta,
        );
    }
}
