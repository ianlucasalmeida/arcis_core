use crate::app::{ArcisState, ArcisPanel};
use crate::bus::messages::AppCommand;
use crate::workers::terminal_parser;
use eframe::egui;
use std::sync::mpsc::Sender;

// Desacoplamento da App Inteira
pub struct ArcisTabViewer<'a> {
    pub state: &'a mut ArcisState,
    pub tx_comando: &'a Sender<AppCommand>,
}

impl<'a> egui_dock::TabViewer for ArcisTabViewer<'a> {
    type Tab = ArcisPanel;

    fn id(&mut self, tab: &mut Self::Tab) -> egui_dock::egui::Id {
        match tab {
            ArcisPanel::Viewport3D => egui_dock::egui::Id::new("Viewport3D"),
            ArcisPanel::WorkspaceLayers => egui_dock::egui::Id::new("WorkspaceLayers"),
            ArcisPanel::Properties => egui_dock::egui::Id::new("Properties"),
            ArcisPanel::TerminalLog => egui_dock::egui::Id::new("TerminalLog"),
        }
    }

    fn title(&mut self, tab: &mut Self::Tab) -> egui_dock::egui::WidgetText {
        match tab {
            ArcisPanel::Viewport3D => "Arcis Core Viewport".into(),
            ArcisPanel::WorkspaceLayers => "Project Explorer".into(),
            ArcisPanel::Properties => "Properties".into(),
            ArcisPanel::TerminalLog => "Console".into(),
        }
    }

    fn ui(&mut self, ui: &mut egui_dock::egui::Ui, tab: &mut Self::Tab) {
        match tab {
            ArcisPanel::Viewport3D => {
                let (rect, _response) = ui.allocate_exact_size(
                    ui.available_size(),
                    eframe::egui::Sense::drag(),
                );
                ui.painter().rect_filled(rect, 0.0, eframe::egui::Color32::from_rgb(15, 15, 15));
                ui.painter().add(eframe::egui_wgpu::Callback::new_paint_callback(
                    rect,
                    crate::render::wgpu_canvas::WgpuCallback {},
                ));
            }
            ArcisPanel::WorkspaceLayers => {
                ui.heading("Camadas Ativas");
                ui.separator();
                for camada in &self.state.camadas_ativas {
                    ui.label(format!("▶ {}", camada));
                }
            }
            ArcisPanel::Properties => {
                ui.label("Ferramenta Ativa: Seleção");
                ui.label("Projeção: UTM Zona 25S");
            }
            ArcisPanel::TerminalLog => {
                egui::ScrollArea::vertical().stick_to_bottom(true).show(ui, |ui| {
                    for log in &self.state.historico_terminal {
                        ui.label(log);
                    }
                });
                ui.separator();
                ui.horizontal(|ui| {
                    ui.label("CMD >");
                    let response = ui.text_edit_singleline(&mut self.state.input_terminal);

                    if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        let cmd = terminal_parser::limpar_input(&self.state.input_terminal);
                        if !cmd.is_empty() {
                            let _ = self.tx_comando.send(AppCommand::ProcessarTerminal(cmd));
                            self.state.input_terminal.clear();
                        }
                        response.request_focus();
                    }
                });
            }
        }
    }
}