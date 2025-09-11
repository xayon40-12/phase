use std::ops::RangeInclusive;

use egui::Frame;
use egui_wgpu::RenderState;
use instant::SystemTime;
use render_square::RenderSquare;
use wgpu::ShaderModule;

pub mod atomic_f32;
pub mod ising;
pub mod render_square;

/// Enumeration of the possible parameters that a simulation needs to display inside the egui UI.
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

/// Enumeration for updating the value of the parameters from [Parameter] once they have been changed in the egui UI. This enum is provided to the [Simulation] through its [Simulation::update_parameter] method.
pub enum UpadeParameter {
    Slider { tag: &'static str, value: f32 },
    Toggle { tag: &'static str, enable: bool },
    Button { tag: &'static str },
}

/// Trait to define the behavior of a simulation with respect to the egui event loop.
pub trait Simulation: Send + 'static {
    /// Provides a list of parameter to be desplayed by egui.
    fn egui_parameters(&self) -> Vec<Parameter>;
    /// Update a parameter which was changed in the egui UI.
    fn update_parameter(&mut self, update: UpadeParameter);
    /// Contrust the physics pipeline in the GPU and return a [Physics](crate::gpu::physics::Physics) needed to update the physics (run the compute pipeline) and setup the rendering inside egui with [RenderSquare].
    fn physics(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        shader_module: &wgpu::ShaderModule,
        seed: u128,
        width: u32,
        height: u32,
    ) -> Box<dyn crate::gpu::physics::Physics>;
}
/// Strut that handles the setup of egui and wgpu, and then starts the [Simulation] and handles the update of the different parameters (see [Parameter]). The rendering of the simulation is performed with the [CallbackTrait](egui_wgpu::CallbackTrait) from [egui_wgpu] used by the [RenderSquare] helper.
pub struct SimulationGUI {
    parameters: Vec<Parameter>,
    simulation: Box<dyn Simulation>,
    render_square: RenderSquare,
    width: u32,
    height: u32,
    shader_module: ShaderModule,
}

impl SimulationGUI {
    pub fn new<'a>(cc: &'a eframe::CreationContext<'a>, simulation: Box<dyn Simulation>) -> Self {
        let parameters = simulation.egui_parameters();
        let width = 1024;
        let height = 1024;

        let wgpu_render_state = cc
            .wgpu_render_state
            .as_ref()
            .expect("No wgpu render state available.");

        let shader_module = unsafe {
            wgpu_render_state.device.create_shader_module_trusted(
                wgpu::ShaderModuleDescriptor {
                    label: Some("Shader module"),
                    source: wgpu::util::make_spirv(crate::SPIRV),
                },
                wgpu::ShaderRuntimeChecks::unchecked(),
            )
        };
        let render_square = Self::new_render_square(
            wgpu_render_state,
            &shader_module,
            &*simulation,
            width,
            height,
        );
        SimulationGUI {
            parameters,
            simulation,
            render_square,
            width,
            height,
            shader_module,
        }
    }
    fn new_render_square(
        wgpu_render_state: &RenderState,
        shader_module: &ShaderModule,
        simulation: &dyn Simulation,
        width: u32,
        height: u32,
    ) -> RenderSquare {
        let seed =
            unsafe { std::mem::transmute(SystemTime::UNIX_EPOCH.elapsed().unwrap().as_millis()) };
        let physics = simulation.physics(
            &wgpu_render_state.device,
            &wgpu_render_state.queue,
            &shader_module,
            seed,
            width,
            height,
        );
        RenderSquare::new(wgpu_render_state, &shader_module, physics)
    }
}
impl eframe::App for SimulationGUI {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
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
                let (_id, rect) = ui.allocate_space(desired_size);
                // If the rendering size changed, create a new [RenderSquare] with the new size.
                if self.width != rect.width() as u32 || self.height != rect.height() as u32 {
                    self.width = rect.width() as u32;
                    self.height = rect.height() as u32;
                    let wgpu_render_state = frame
                        .wgpu_render_state()
                        .expect("No wgpu render state available.");
                    self.render_square = Self::new_render_square(
                        wgpu_render_state,
                        &self.shader_module,
                        &*self.simulation,
                        self.width,
                        self.height,
                    );
                }
                ui.painter().add(egui_wgpu::Callback::new_paint_callback(
                    rect,
                    self.render_square,
                ));
            });
        });
        ctx.request_repaint();
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn with_egui(simulation: Box<dyn Simulation>) {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let native_options = eframe::NativeOptions::default();
    if let Err(err) = eframe::run_native(
        "Phase",
        native_options,
        Box::new(|cc| Ok(Box::new(SimulationGUI::new(cc, simulation)))),
    ) {
        log::log!(log::Level::Error, "{err}");
    }
}

// When compiling to web using trunk:
#[cfg(target_arch = "wasm32")]
pub fn with_egui(simulation: Box<dyn Simulation>) {
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
                Box::new(|cc| Ok(Box::new(SimulationGUI::new(cc, simulation)))),
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
