mod app;
mod bus;
mod render;
mod splash;
mod theme;
mod ui;
mod workers;

use app::{AppScreen, ArcisApp};
use eframe::egui;
use std::sync::mpsc;

impl eframe::App for ArcisApp {
    fn ui(&mut self, ui: &mut eframe::egui::Ui, _frame: &mut eframe::Frame) {
        self.ler_mensagens_background();
        theme::apply_apple_dark_theme(ui.ctx());

        match &mut self.screen {
            AppScreen::Splash(splash) => {
                if splash.show(ui) {
                    self.screen = AppScreen::Workspace;
                }
            }
            AppScreen::Workspace => {
                ui::render_xcode_workspace(ui, &mut self.state, &self.tx_comando);
            }
        }

        ui.ctx().request_repaint();
    }
}

fn main() -> eframe::Result<()> {
    let (tx_comando, rx_comando_worker) = mpsc::channel();
    let (tx_resposta_worker, rx_resposta) = mpsc::channel();

    workers::gis_engine::iniciar_motor_assincrono(rx_comando_worker, tx_resposta_worker);

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1366.0, 820.0]),
        wgpu_options: eframe::egui_wgpu::WgpuConfiguration {
            ..Default::default()
        },
        ..Default::default()
    };

    eframe::run_native(
        "Arcis - STEM Engine",
        options,
        Box::new(|cc| {
            if let Some(wgpu_state) = cc.wgpu_render_state.as_ref() {
                let device = &wgpu_state.device;
                let target_format = wgpu_state.target_format;

                let render_pipeline =
                    render::wgpu_canvas::ArcisRenderPipeline::new(device, target_format);
                wgpu_state
                    .renderer
                    .write()
                    .callback_resources
                    .insert(render_pipeline);
            }

            Ok(Box::new(ArcisApp::new(tx_comando, rx_resposta)))
        }),
    )
}