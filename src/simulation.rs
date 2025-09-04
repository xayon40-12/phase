use std::ops::RangeInclusive;

use custom3d::Custom3d;
use egui::Frame;

pub mod custom3d;
pub mod ising;

pub enum Parameter {
    Slider {
        tag: &'static str,
        value: f32,
        logarithmic: bool,
        range: RangeInclusive<f32>,
    },
    Toggle {
        tag: &'static str,
        enable: bool,
    },
    Button {
        tag: &'static str,
    },
}

pub enum UpadeParameter {
    Slider { tag: &'static str, value: f32 },
    Toggle { tag: &'static str, enable: bool },
    Button { tag: &'static str },
}

pub trait Simulation: Send + 'static {
    fn reset(&mut self);

    fn egui_parameters(&self) -> Vec<Parameter>;
    fn update_parameter(&mut self, update: UpadeParameter);

    fn update(&mut self);
}

pub struct SimulationGUI {
    parameters: Vec<Parameter>,
    create_simulation: Box<dyn Fn() -> Box<dyn Simulation>>,
    simulation: Box<dyn Simulation>,
    custom3d: Custom3d,
}

impl SimulationGUI {
    pub fn new<'a>(
        cc: &'a eframe::CreationContext<'a>,
        create_simulation: Box<dyn Fn() -> Box<dyn Simulation>>,
    ) -> Self {
        let simulation = create_simulation();
        let parameters = simulation.egui_parameters();
        SimulationGUI {
            parameters,
            create_simulation,
            simulation,
            custom3d: Custom3d::new(cc).expect("Custom3d"),
        }
    }
}
impl eframe::App for SimulationGUI {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.simulation.update();
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("reset").clicked() {
                    self.simulation.reset();
                }
            });
            for p in self.parameters.iter_mut() {
                match p {
                    Parameter::Slider {
                        tag,
                        value,
                        logarithmic,
                        range,
                    } => {
                        if ui
                            .add(
                                egui::Slider::new(value, range.clone())
                                    .logarithmic(*logarithmic)
                                    .text(*tag),
                            )
                            .changed()
                        {
                            self.simulation
                                .update_parameter(UpadeParameter::Slider { tag, value: *value });
                        }
                    }
                    Parameter::Toggle { tag, enable } => {
                        if ui.toggle_value(enable, *tag).changed() {
                            self.simulation.update_parameter(UpadeParameter::Toggle {
                                tag,
                                enable: *enable,
                            });
                        }
                    }
                    Parameter::Button { tag } => {
                        if ui.button(*tag).clicked() {
                            self.simulation
                                .update_parameter(UpadeParameter::Button { tag });
                        }
                    }
                }
            }

            Frame::canvas(ui.style()).show(ui, |ui| {
                let desired_size = ui.available_size();
                let ratio = desired_size.x / desired_size.y;
                let rx = ratio.max(1.0);
                let ry = ratio.recip().max(1.0);
                let (_id, rect) = ui.allocate_space(desired_size);
                egui::Frame::canvas(ui.style()).show(ui, |ui| {
                    ui.painter().add(egui_wgpu::Callback::new_paint_callback(
                        rect,
                        self.custom3d.custom_callback(),
                    ));
                });
            });
        });
        ctx.request_repaint();
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn with_egui(create_simulation: Box<dyn Fn() -> Box<dyn Simulation>>) {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let native_options = eframe::NativeOptions::default();
    if let Err(err) = eframe::run_native(
        "Reinforcement",
        native_options,
        Box::new(|cc| Ok(Box::new(SimulationGUI::new(cc, create_simulation)))),
    ) {
        log::log!(log::Level::Error, "{err}");
    }
}

// When compiling to web using trunk:
#[cfg(target_arch = "wasm32")]
pub fn with_egui(create_simulation: Box<dyn Fn() -> Box<dyn Simulation>>) {
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
                Box::new(|cc| Ok(Box::new(SimulationGUI::new(cc, create_simulation)))),
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
