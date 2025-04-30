use std::sync::{Arc, Mutex};

use rfd::AsyncFileDialog;

mod code_editor;

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 300.0])
            .with_min_inner_size([300.0, 220.0])
            .with_icon(
                // NOTE: Adding an icon is optional
                eframe::icon_data::from_png_bytes(&include_bytes!("../assets/icon-256.png")[..])
                    .expect("Failed to load icon"),
            ),
        ..Default::default()
    };
    eframe::run_native(
        "eframe template",
        native_options,
        Box::new(|cc| Ok(Box::new(TemplateApp::new(cc)))),
    )
}

// When compiling to web using trunk:
#[cfg(target_arch = "wasm32")]
fn main() {
    use eframe::wasm_bindgen::JsCast as _;

    // Redirect `log` message to `console.log` and friends:
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        let document = web_sys::window()
            .expect("No window")
            .document()
            .expect("No document");

        let canvas = document
            .get_element_by_id("the_canvas_id")
            .expect("Failed to find the_canvas_id")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("the_canvas_id was not a HtmlCanvasElement");

        let start_result = eframe::WebRunner::new()
            .start(
                canvas,
                web_options,
                Box::new(|cc| Ok(Box::new(TemplateApp::new(cc)))),
            )
            .await;

        // Remove the loading text and spinner:
        if let Some(loading_text) = document.get_element_by_id("loading_text") {
            match start_result {
                Ok(_) => {
                    loading_text.remove();
                }
                Err(e) => {
                    loading_text.set_inner_html(
                        "<p> The app has crashed. See the developer console for details. </p>",
                    );
                    panic!("Failed to start eframe: {e:?}");
                }
            }
        }
    });
}

type LoadFileEvent = Arc<Mutex<Option<String>>>;

pub struct TemplateApp {
    pub save_data: SaveData,

    pub load_file_event: LoadFileEvent,
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct SaveData {
    file_name: String,
    source_code: String,
}

impl Default for SaveData {
    fn default() -> Self {
        Self {
            file_name: "example_project.py".into(),
            source_code: r#"
print("Hello, world!")
"#
            .into(),
        }
    }
}

impl TemplateApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let save_data = cc
            .storage
            .and_then(|storage| eframe::get_value(storage, eframe::APP_KEY))
            .unwrap_or_default();

        Self {
            save_data,
            load_file_event: Default::default(),
        }
    }
}

impl eframe::App for TemplateApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, &self.save_data);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.per_frame_handlers();

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    #[cfg(not(target_arch = "wasm32"))]
                    if ui.button("Quit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }

                    if ui.button("Open").clicked() {
                        pick_file(&self.load_file_event);
                    }

                    if ui.button("Save").clicked() {
                        save_file(&self.save_data.source_code);
                    }

                    if ui.button("Load default project").clicked() {
                        self.save_data = Default::default();
                    }
                });

                ui.menu_button("Theme", |ui| {
                    egui::widgets::global_theme_preference_buttons(ui);
                })
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            code_editor::code_editor_with_autoindent(
                ui,
                "the code editor".into(),
                &mut self.save_data.source_code,
                "py",
            )
        });
    }
}

impl TemplateApp {
    fn per_frame_handlers(&mut self) {
        if let Some(file) = self.load_file_event.lock().unwrap().take() {
            self.save_data.source_code = file;
        }
    }
}

fn pick_file(event: &LoadFileEvent) {
    let picker = AsyncFileDialog::new().add_filter("py", &["py"]).pick_file();

    let event = event.clone();

    #[cfg(target_arch = "wasm32")]
    {
        wasm_bindgen_futures::spawn_local(async move {
            if let Some(file) = picker.await {
                if let Ok(code) = String::from_utf8(file.read().await) {
                    *event.lock().unwrap() = Some(code);
                }
            }
        });
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        smol::spawn(async move {
            if let Some(file) = picker.await {
                match String::from_utf8(file.read().await) {
                    Err(e) => eprintln!("{e}"),
                    Ok(code) => *event.lock().unwrap() = Some(code),
                }
            }
        }).detach();
    }
}

fn save_file(code: &str) {
    let code = code.to_string().into_bytes();

    let picker = AsyncFileDialog::new().add_filter("py", &["py"]).save_file();

    #[cfg(target_arch = "wasm32")]
    {
        wasm_bindgen_futures::spawn_local(async move {
            if let Some(file) = picker.await {
                let _ = file.write(&code).await;
            }
        });
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        smol::spawn(async move {
            if let Some(file) = picker.await {
                if let Err(e) = file.write(&code).await {
                    eprintln!("{e}")
                }
            }
        }).detach();
    }
}
