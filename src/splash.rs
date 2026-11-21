use crate::theme;
use eframe::egui::{self, Align2, Color32, FontId, RichText, Stroke, Vec2};
use std::time::Instant;

pub struct SplashState {
    inicio: Instant,
    etapas: Vec<&'static str>,
}

impl SplashState {
    pub fn new() -> Self {
        Self {
            inicio: Instant::now(),
            etapas: vec![
                "Verificando ambiente Linux / Wayland...",
                "Inicializando adaptador WGPU (Vulkan)...",
                "Compilando shaders WGSL e pipeline 3D...",
                "Iniciando runtime assíncrono (Tokio Workers)...",
                "Carregando projeções SIRGAS 2000 / UTM 25S...",
                "Sistema pronto.",
            ],
        }
    }

    /// Retorna `true` quando o carregamento terminar
    pub fn show(&mut self, ui: &mut egui::Ui) -> bool {
        let elapsed = self.inicio.elapsed().as_secs_f32();
        let duracao_total = 2.6;
        let progresso = (elapsed / duracao_total).clamp(0.0, 1.0);
        let idx = ((progresso * (self.etapas.len() - 1) as f32) as usize)
            .min(self.etapas.len() - 1);

        let rect = ui.available_rect_before_wrap();
        ui.painter().rect_filled(rect, 0.0, theme::BG_VIEWPORT);

        // Card central estilo macOS
        let card_size = Vec2::new(460.0, 260.0);
        let card_rect = egui::Rect::from_center_size(rect.center(), card_size);
        let radius = theme::RADIUS_LG as f32;

        ui.painter().rect_filled(card_rect, radius, theme::BG_CARD);
        ui.painter().rect_stroke(
            card_rect,
            radius,
            Stroke::new(1.0, theme::BORDER_STRONG),
            egui::StrokeKind::Inside,
        );

        ui.scope_builder(egui::UiBuilder::new().max_rect(card_rect.shrink(28.0)), |ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("ARCIS")
                            .font(FontId::proportional(26.0))
                            .strong()
                            .color(Color32::WHITE),
                    );
                    ui.add_space(6.0);
                    ui.label(
                        RichText::new("v0.1.0 • STEM Engine")
                            .font(FontId::proportional(12.0))
                            .color(theme::TEXT_MUTED),
                    );
                });

                ui.label(
                    RichText::new("Geospatial, CAD & Real-Time Hardware Pipeline")
                        .small()
                        .color(theme::TEXT_MUTED),
                );

                ui.add_space(22.0);

                // Checklist de verificações do sistema
                for (i, etapa) in self.etapas.iter().enumerate() {
                    if i <= idx {
                        let concluida = i < idx || progresso >= 1.0;
                        let (icone, cor) = if concluida {
                            ("●", theme::SYS_GREEN)
                        } else {
                            ("◌", theme::ACCENT_BLUE)
                        };
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(icone).small().color(cor));
                            ui.label(
                                RichText::new(*etapa)
                                    .font(theme::mono(11.0))
                                    .color(if concluida {
                                        theme::TEXT_MUTED
                                    } else {
                                        theme::TEXT_PRIMARY
                                    }),
                            );
                        });
                    }
                }

                ui.add_space(16.0);

                // Barra de progresso fina
                let (bar_rect, _) = ui.allocate_exact_size(
                    Vec2::new(ui.available_width(), 4.0),
                    egui::Sense::hover(),
                );
                ui.painter().rect_filled(bar_rect, 2.0, theme::BG_CONTROL);
                let fill_rect = egui::Rect::from_min_size(
                    bar_rect.min,
                    Vec2::new(bar_rect.width() * progresso, bar_rect.height()),
                );
                ui.painter().rect_filled(fill_rect, 2.0, theme::ACCENT_BLUE);

                ui.painter().text(
                    egui::pos2(card_rect.right() - 28.0, card_rect.bottom() - 20.0),
                    Align2::RIGHT_BOTTOM,
                    format!("{:.0}%", progresso * 100.0),
                    theme::mono(10.5),
                    theme::TEXT_MUTED,
                );
            });
        });

        progresso >= 1.0
    }
}