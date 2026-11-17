//! Original ProjectCraft egui application running directly inside a web canvas.
//! First public slice: synthetic sample and browser-local save only. No server-side
//! project storage, identity, customer data, or collaboration is exposed.
#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

#[cfg(target_arch = "wasm32")]
mod browser {
    use std::time::{Duration, Instant};

    use eframe::egui;
    use projectcraft_engine::{Session, sample};
    use projectcraft_ui_egui::App;
    use wasm_bindgen::{JsCast, prelude::*};

    const STORAGE_KEY: &str = "projectcraft-local-snapshot-v1";

    struct WebApp {
        app: App,
        last_saved: String,
        last_check: Instant,
    }

    fn storage() -> Option<web_sys::Storage> {
        web_sys::window()?.local_storage().ok().flatten()
    }

    impl WebApp {
        fn new() -> Self {
            let project = storage()
                .and_then(|s| s.get_item(STORAGE_KEY).ok().flatten())
                .and_then(|text| Session::load_json(&text).ok())
                .unwrap_or_else(sample);
            let initial = project.save_json().unwrap_or_default();
            Self {
                app: App::new(project, None),
                last_saved: initial,
                last_check: Instant::now(),
            }
        }

        fn save_local(&mut self) {
            if self.last_check.elapsed() < Duration::from_millis(750) {
                return;
            }
            self.last_check = Instant::now();
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
            ui.horizontal_wrapped(|ui| {
                ui.strong("ProjectCraft");
                ui.separator();
                ui.label("Browser-local preview: changes stay on this device, not on a server.");
            });
            ui.separator();
            self.app.ui(ui);
            // Native file dialogs cannot operate on browser sandbox paths.
            // Do not silently attempt to save or open a server file.
            if let Some(request) = self.app.take_file_request() {
                self.app.file_picked(request, None);
            }
            self.save_local();
            ui.ctx().request_repaint_after(Duration::from_secs(1));
        }
    }

    #[wasm_bindgen(start)]
    pub fn boot() {
        wasm_bindgen_futures::spawn_local(async {
            let Some(canvas) = web_sys::window()
                .and_then(|w| w.document())
                .and_then(|d| d.get_element_by_id("projectcraft_canvas"))
                .and_then(|e| e.dyn_into::<web_sys::HtmlCanvasElement>().ok())
            else {
                web_sys::console::error_1(&"ProjectCraft: canvas missing".into());
                return;
            };
            let runner = eframe::WebRunner::new();
            if let Err(err) = runner
                .start(
                    canvas,
                    eframe::WebOptions::default(),
                    Box::new(|_| Ok(Box::new(WebApp::new()))),
                )
                .await
            {
                web_sys::console::error_1(&err);
            }
        });
    }
}
