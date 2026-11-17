//! Original ProjectCraft egui application running directly inside a web canvas.
//! First public slice: synthetic sample and browser-local save only. No server-side
//! project storage, identity, customer data, or collaboration is exposed.
#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

#[cfg(target_arch = "wasm32")]
mod browser {
    // Not `std::time::Instant`: on wasm32-unknown-unknown it panics ("time not
    // implemented on this platform"), which left the live page black. egui's own
    // frame clock (`InputState::time`, seconds) works on every target.
    use std::time::Duration;

    use eframe::egui;
    use projectcraft_engine::{Session, sample};
    use projectcraft_ui_egui::App;
    use wasm_bindgen::{JsCast, prelude::*};

    const STORAGE_KEY: &str = "projectcraft-local-snapshot-v1";

    // Keep browser startup failures visible even if the canvas cannot be painted.
    fn startup_message(message: &str) {
        if let Some(status) = web_sys::window().and_then(|w| w.document()).and_then(|d| d.get_element_by_id("boot-status")) {
            status.set_text_content(Some(message));
        }
    }

    fn startup_ready() {
        if let Some(status) = web_sys::window().and_then(|w| w.document()).and_then(|d| d.get_element_by_id("boot-status")) {
            let _ = status.set_attribute("hidden", "true");
        }
    }

    struct WebApp {
        app: App,
        last_saved: String,
        last_check: f64,
    }

    fn storage() -> Option<web_sys::Storage> {
        web_sys::window()?.local_storage().ok().flatten()
    }

    impl WebApp {
        fn new() -> Self {
            let project =
                storage().and_then(|s| s.get_item(STORAGE_KEY).ok().flatten()).and_then(|text| Session::load_json(&text).ok()).unwrap_or_else(sample);
            let initial = project.save_json().unwrap_or_default();
            Self { app: App::new(project, None), last_saved: initial, last_check: f64::NEG_INFINITY }
        }

        fn save_local(&mut self, now: f64) {
            if now - self.last_check < 0.75 {
                return;
            }
            self.last_check = now;
            let Ok(current) = self.app.session.save_json() else {
                return;
            };
            if current != self.last_saved {
                if storage().is_some_and(|s| s.set_item(STORAGE_KEY, &current).is_ok()) {
                    self.last_saved = current;
                }
            }
        }
    }

    impl eframe::App for WebApp {
        fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
            // This strip sits outside the app's own panels, so paint the theme's panel fill
            // under everything first: unfilled, the strip and the gap below it showed the
            // canvas's dark clear colour, with dark text on it.
            ui.painter().rect_filled(ui.max_rect(), 0.0, ui.visuals().panel_fill);
            egui::Frame::new().fill(ui.visuals().panel_fill).inner_margin(egui::Margin::symmetric(8, 4)).show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.horizontal_wrapped(|ui| {
                    ui.strong("ProjectCraft");
                    ui.separator();
                    ui.label("Browser-local preview: changes stay on this device, not on a server.");
                });
            });
            self.app.ui(ui);
            startup_ready();
            // Native file dialogs cannot operate on browser sandbox paths.
            // Do not silently attempt to save or open a server file.
            if let Some(request) = self.app.take_file_request() {
                self.app.file_picked(request, None);
            }
            let now = ui.ctx().input(|i| i.time);
            self.save_local(now);
            ui.ctx().request_repaint_after(Duration::from_secs(1));
        }
    }

    #[wasm_bindgen(start)]
    pub fn boot() {
        startup_message("Initializing ProjectCraft (WebGL2)…");
        wasm_bindgen_futures::spawn_local(async {
            let Some(canvas) = web_sys::window()
                .and_then(|w| w.document())
                .and_then(|d| d.get_element_by_id("projectcraft_canvas"))
                .and_then(|e| e.dyn_into::<web_sys::HtmlCanvasElement>().ok())
            else {
                startup_message("ProjectCraft failed to start: browser canvas missing.");
                web_sys::console::error_1(&"ProjectCraft: canvas missing".into());
                return;
            };
            let runner = eframe::WebRunner::new();
            if let Err(err) = runner.start(canvas, eframe::WebOptions::default(), Box::new(|_| Ok(Box::new(WebApp::new())))).await {
                startup_message(&format!("ProjectCraft could not initialize WebGL: {err:?}. Check that WebGL2 is enabled in this browser."));
                web_sys::console::error_1(&err);
            }
        });
    }
}
