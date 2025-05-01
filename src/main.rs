use std::{
    cell::RefCell,
    rc::Rc,
    sync::{Arc, Mutex},
};

use egui::{ScrollArea, Vec2};
use egui_plotter::EguiBackend;
use plot::PlotCommand;
use plotters::{chart::ChartBuilder, prelude::{IntoDrawingArea, PathElement}, series::LineSeries, style::{Color, IntoFont, BLACK, RED, WHITE}};
use rfd::AsyncFileDialog;
use rustpython_vm::{
    builtins::PyCode, scope::Scope, Interpreter, PyObjectRef, PyRef, VirtualMachine,
};

mod code_editor;
mod plot;

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
        "Stupyder",
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

type LoadFileEvent = Arc<Mutex<Option<(String, String)>>>;
type Logs = Rc<RefCell<Vec<String>>>;

pub struct TemplateApp {
    save_data: SaveData,

    plot_info: Vec<PlotCommand>,
    load_file_event: LoadFileEvent,
    kernel: Kernel,
}

pub struct Kernel {
    logs: Logs,
    interpreter: Interpreter,
    scope: Scope,
    code_obj: Option<PyRef<PyCode>>,
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct SaveData {
    file_name: String,
    source_code: String,
    run_schedule: RunSchedule,
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Copy, PartialEq, Eq, Default)]
enum RunSchedule {
    EachFrame,
    OnInteract,
    #[default]
    Manual,
}

impl Default for SaveData {
    fn default() -> Self {
        Self {
            file_name: "example_project.py".into(),
            source_code: r#"
print("Hello, world!")
"#
            .into(),
            run_schedule: RunSchedule::default(),
        }
    }
}

impl TemplateApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.tessellation_options_mut(|tess_options| {
            tess_options.feathering = false;
        });

        let save_data: SaveData = cc
            .storage
            .and_then(|storage| eframe::get_value(storage, eframe::APP_KEY))
            .unwrap_or_default();

        Self {
            kernel: Kernel::new_with_code(save_data.source_code.clone()),
            save_data,
            load_file_event: Default::default(),
            plot_info: vec![],
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
        self.per_frame_event_handlers();

        let mut do_run = match self.save_data.run_schedule {
            RunSchedule::Manual => false,
            RunSchedule::OnInteract => true,
            RunSchedule::EachFrame => {
                ctx.request_repaint();
                true
            }
        };

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
                        save_file(&self.save_data.source_code, &self.save_data.file_name);
                    }

                    if ui.button("Load default project").clicked() {
                        self.save_data = Default::default();
                    }
                });

                ui.menu_button("Theme", |ui| {
                    egui::widgets::global_theme_preference_buttons(ui);
                });

                ui.menu_button("Run", |ui| {
                    ui.selectable_value(
                        &mut self.save_data.run_schedule,
                        RunSchedule::Manual,
                        "Manual",
                    );
                    ui.selectable_value(
                        &mut self.save_data.run_schedule,
                        RunSchedule::OnInteract,
                        "On any interaction",
                    );
                    ui.selectable_value(
                        &mut self.save_data.run_schedule,
                        RunSchedule::EachFrame,
                        "Continuous",
                    );
                });

                if ui.button("Step").clicked() {
                    do_run = true;
                }
            });
        });

        egui::SidePanel::right("output").resizable(true).show(ctx, |ui| {
            let root = EguiBackend::new(&*ui).into_drawing_area();
            root.fill(&WHITE).unwrap();
            let mut chart = ChartBuilder::on(&root)
                .caption("y=x^2", ("sans-serif", 50).into_font())
                .margin(5)
                .x_label_area_size(30)
                .y_label_area_size(30)
                .build_cartesian_2d(-1f32..1f32, -0.1f32..1f32)
                .unwrap();

            chart.configure_mesh().draw().unwrap();

            chart
                .draw_series(LineSeries::new(
                    (-50..=50).map(|x| x as f32 / 50.0).map(|x| (x, x * x)),
                    &RED,
                ))
                .unwrap()
                .label("y = x^2")
                .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &RED));

            chart
                .configure_series_labels()
                .background_style(&WHITE.mix(0.8))
                .border_style(&BLACK)
                .draw()
                .unwrap();

            root.present().unwrap();
            drop(chart);
            drop(root);
            ui.allocate_space(ui.available_size());
        });

        egui::TopBottomPanel::bottom("cli and stuff")
            .resizable(true)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.heading("Console");
                    ui.with_layout(egui::Layout::right_to_left(Default::default()), |ui| {
                        if ui.button("Clear").clicked() {
                            self.kernel.logs.borrow_mut().clear();
                        }
                    });
                });
                let n = self.kernel.logs.borrow().len();
                egui::Frame::canvas(ui.style()).show(ui, |ui| {
                    egui::ScrollArea::vertical()
                        .id_salt("cli")
                        .stick_to_bottom(true)
                        .auto_shrink(false)
                        .show_rows(ui, 18.0, n, |ui, range| {
                            for row in &self.kernel.logs.borrow()[range] {
                                ui.label(row);
                            }
                        });
                });
            });

        let mut resp = None;
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("File name: ");
                ui.text_edit_singleline(&mut self.save_data.file_name);
            });
            ScrollArea::vertical()
                .auto_shrink(false)
                .id_salt("code")
                .show(ui, |ui| {
                    resp = Some(code_editor::code_editor_with_autoindent(
                        ui,
                        "the code editor".into(),
                        &mut self.save_data.source_code,
                        "py",
                    ));
                });
        });

        let resp = resp.unwrap();
        if resp.changed() {
            //self.kernel.load(self.save_data.source_code.clone());
        }

        if do_run {
            self.kernel.load(self.save_data.source_code.clone());
            self.kernel.run();
            self.plot_info = plot::pyplotter::dump_commands();
        }
    }
}

impl TemplateApp {
    fn per_frame_event_handlers(&mut self) {
        if let Some(file) = self.load_file_event.lock().unwrap().take() {
            (self.save_data.source_code, self.save_data.file_name) = file;
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
                    *event.lock().unwrap() = Some((code, file.file_name()));
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
                    Ok(code) => *event.lock().unwrap() = Some((code, file.file_name())),
                }
            }
        })
        .detach();
    }
}

fn save_file(code: &str, file_name: &str) {
    let code = code.to_string().into_bytes();

    let picker = AsyncFileDialog::new()
        .set_file_name(file_name)
        .add_filter("py", &["py"])
        .save_file();

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
        })
        .detach();
    }
}

fn anon_object(vm: &VirtualMachine, name: &str) -> PyObjectRef {
    let py_type = vm.builtins.get_attr("type", vm).unwrap();
    let args = (name, vm.ctx.new_tuple(vec![]), vm.ctx.new_dict());
    py_type.call(args, vm).unwrap()
}

fn install_stdout(vm: &VirtualMachine, logs: Logs) {
    let sys = vm.import("sys", 0).unwrap();

    let stdout = anon_object(vm, "InternalStdout");

    let writer = vm.new_function("write", move |s: String| logs.borrow_mut().push(s));

    stdout.set_attr("write", writer, vm).unwrap();

    sys.set_attr("stdout", stdout.clone(), vm).unwrap();
}

impl Kernel {
    pub fn new() -> Self {
        let interpreter = Interpreter::with_init(Default::default(), |vm| {
            vm.add_native_modules(rustpython_stdlib::get_module_inits());
            /*
            vm.add_native_module(
                "rust_py_module".to_owned(),
                Box::new(rust_py_module::make_module),
            );
            */
            vm.add_native_module(
                "ndarray".to_owned(),
                Box::new(rustpython_ndarray::make_module),
            )
        });

        let logs = Logs::default();

        let scope = interpreter.enter(|vm| {
            // Create scope
            let scope = vm.new_scope_with_builtins();
            install_stdout(vm, logs.clone());

            scope
        });

        Self {
            scope,
            interpreter,
            logs,
            code_obj: None,
        }
    }

    pub fn new_with_code(code: String) -> Self {
        let mut inst = Self::new();
        inst.load(code);
        inst
    }

    pub fn load(&mut self, code: String) {
        self.interpreter.enter(|vm| {
            let code_obj = vm.compile(
                &code,
                rustpython_vm::compiler::Mode::Exec,
                "the code you just wrote in the thingy".to_owned(),
            );
            match code_obj {
                Ok(obj) => {
                    self.code_obj = Some(obj);
                }
                Err(compile_err) => {
                    self.logs
                        .borrow_mut()
                        .push(format!("Compile error: {:#?}", compile_err));
                }
            }
        });
    }

    pub fn run(&mut self) {
        let Some(code) = self.code_obj.clone() else {
            return;
        };

        let scope = self.scope.clone();
        let error = self.interpreter.enter(move |vm| {
            if let Err(exec_err) = vm.run_code_obj(code, scope) {
                let mut s = String::new();
                vm.write_exception(&mut s, &exec_err).unwrap();
                Some(s)
            } else {
                None
            }
        });

        if let Some(e) = error {
            self.logs.borrow_mut().push(format!("Error: {e}"));
        }
    }
}
