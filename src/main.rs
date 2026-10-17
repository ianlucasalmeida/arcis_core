mod app;
mod bus;
mod render;
mod ui;
mod workers;

use app::ArcisApp;
use eframe::egui;
use egui_dock::DockArea;
use std::sync::mpsc;
use ui::ArcisTabViewer;

impl eframe::App for ArcisApp {
    fn ui(&mut self, ui: &mut eframe::egui::Ui, _frame: &mut eframe::Frame) {
        self.ler_mensagens_background();
        ui.ctx().set_visuals(egui::Visuals::dark());

        // Agora o viewer empresta apenas as partes necessárias do App
        let mut viewer = ArcisTabViewer {
            state: &mut self.state,
            tx_comando: &self.tx_comando,
        };

        DockArea::new(&mut self.dock_state)
            .style(egui_dock::Style::from_egui(ui.style().as_ref()))
            .show_inside(ui, &mut viewer);
            
        ui.ctx().request_repaint();
    }
}

fn main() -> eframe::Result<()> {
    let (tx_comando, rx_comando_worker) = mpsc::channel();
    let (tx_resposta_worker, rx_resposta) = mpsc::channel();

    workers::gis_engine::iniciar_motor_assincrono(rx_comando_worker, tx_resposta_worker);

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1280.0, 720.0]),
        // Habilitamos suporte explícito à aceleração WGPU nativa
        wgpu_options: eframe::egui_wgpu::WgpuConfiguration {
            ..Default::default()
        },
        ..Default::default()
    };

    eframe::run_native(
        "Arcis - STEM Engine",
        options,
        Box::new(|cc| {
            // Inicializa a Pipeline Gráfica na GPU no momento em que a janela abre
            if let Some(wgpu_state) = cc.wgpu_render_state.as_ref() {
                let device = &wgpu_state.device;
                let target_format = wgpu_state.target_format;
                
                let render_pipeline = render::wgpu_canvas::ArcisRenderPipeline::new(device, target_format);
                wgpu_state.renderer.write().callback_resources.insert(render_pipeline);
            }

            Ok(Box::new(ArcisApp::new(tx_comando, rx_resposta)))
        }),
    )
}