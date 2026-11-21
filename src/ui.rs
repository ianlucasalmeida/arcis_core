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
                // Barra Superior do Viewport (Breadcrumbs)
                ui.horizontal(|ui| {
                    ui.label("Arcis 〉Módulo GIS 〉NASA_Earthdata_Dataset.tif");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("⛶").clicked() { /* Ação de Tela Cheia futura */ }
                    });
                });
                ui.separator();

                // Área do Renderizador WGPU
                let (rect, _response) = ui.allocate_exact_size(
                    egui::vec2(ui.available_width(), ui.available_height() - 35.0), // Reserva espaço p/ Slider base
                    egui::Sense::drag(),
                );
                ui.painter().rect_filled(rect, 0.0, eframe::egui::Color32::from_rgb(18, 18, 18));
                
                ui.painter().add(eframe::egui_wgpu::Callback::new_paint_callback(
                    rect,
                    crate::render::wgpu_canvas::WgpuCallback {
                        vertices: self.state.vertices_carregados.clone(),
                        indices: self.state.indices_carregados.clone(),
                    },
                ));

                ui.separator();
                // Barra Inferior do Viewport (Slider de opacidade / Timeline temporal)
                ui.horizontal(|ui| {
                    ui.label("👁 Opacidade");
                    let mut opacidade = 1.0;
                    ui.add(egui::Slider::new(&mut opacidade, 0.0..=1.0).show_value(false));
                    ui.separator();
                    ui.label("⏱ Timeline (Satélite)");
                });
            }
            ArcisPanel::WorkspaceLayers => {
                // Indicadores de Sistema (Topo Esquerdo)
                ui.label(egui::RichText::new("Monitoramento de Hardware").strong());
                ui.horizontal(|ui| { ui.label("CPU"); ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| { ui.label("2%") }); });
                ui.horizontal(|ui| { ui.label("Memória GPU"); ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| { ui.label("124 MB") }); });
                ui.separator();
                
                // Árvore de Camadas (View Hierarchy)
                ui.label(egui::RichText::new("Hierarquia do Projeto").strong());
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for camada in &self.state.camadas_ativas {
                        ui.collapsing(format!("🌐 {}", camada), |ui| {
                            ui.label("  ↳ Matriz de Vértices");
                            ui.label("  ↳ Textura Base (RGB)");
                        });
                    }
                });
            }
            ArcisPanel::Properties => {
                // Inspetor de Objeto detalhado (Painel Direito)
                ui.heading("Object Inspector");
                ui.separator();
                
                egui::Grid::new("prop_grid").striped(true).show(ui, |ui| {
                    ui.label("Classe"); ui.label("Arcis::GeoTIFF"); ui.end_row();
                    ui.label("Projeção"); ui.label("EPSG:31985"); ui.end_row();
                    ui.label("Vértices"); ui.label(format!("{}", self.state.vertices_carregados.len())); ui.end_row();
                    ui.label("Visibilidade"); ui.label("Opaco"); ui.end_row();
                });
                
                ui.add_space(15.0);
                ui.label(egui::RichText::new("Atributos Espaciais").strong());
                ui.separator();
                ui.label("Latitude: -8.1159 (Jaboatão)");
                ui.label("Longitude: -35.0154");
            }
            ArcisPanel::TerminalLog => {
                // Terminal (Estilo LLDB / Console)
                egui::ScrollArea::vertical().stick_to_bottom(true).max_height(ui.available_height() - 30.0).show(ui, |ui| {
                    for log in &self.state.historico_terminal {
                        // Colore mensagens do usuário e respostas do sistema de forma diferente
                        if log.starts_with(">") {
                            ui.label(egui::RichText::new(log).color(egui::Color32::from_rgb(0, 251, 143))); // Verde neon
                        } else {
                            ui.label(egui::RichText::new(log).color(egui::Color32::LIGHT_GRAY));
                        }
                    }
                });
                
                ui.separator();
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("(cmd)").strong());
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