use crate::app::ArcisState;
use crate::bus::messages::AppCommand;
use crate::theme::{self, BG_SIDEBAR, BG_SUBBAR, BG_VIEWPORT};
use crate::workers::terminal_parser;
use eframe::egui::{self, RichText, Vec2};
use std::sync::mpsc::Sender;

pub fn render_xcode_workspace(
    ui: &mut egui::Ui,
    state: &mut ArcisState,
    tx_comando: &Sender<AppCommand>,
) {
    let total_rect = ui.available_rect_before_wrap();
    ui.painter().rect_filled(total_rect, 0.0, BG_VIEWPORT);

    let toolbar_h = theme::TOOLBAR_H;
    let toolbar_rect =
        egui::Rect::from_min_size(total_rect.min, Vec2::new(total_rect.width(), toolbar_h));
    let body_rect = egui::Rect::from_min_max(
        egui::pos2(total_rect.min.x, total_rect.min.y + toolbar_h),
        total_rect.max,
    );

    // 1. TOP TOOLBAR (Estilo Xcode)
    render_top_toolbar(ui, toolbar_rect, state, tx_comando);

    // 2. CORPO EM 3 COLUNAS
    let left_w = if state.mostrar_esquerda {
        theme::NAVIGATOR_W
    } else {
        0.0
    };
    let right_w = if state.mostrar_direita {
        theme::INSPECTOR_W
    } else {
        0.0
    };
    let center_w = (body_rect.width() - left_w - right_w).max(200.0);

    if state.mostrar_esquerda {
        let left_rect =
            egui::Rect::from_min_size(body_rect.min, Vec2::new(left_w, body_rect.height()));
        render_left_navigator(ui, left_rect, state);
    }

    let center_rect = egui::Rect::from_min_size(
        egui::pos2(body_rect.min.x + left_w, body_rect.min.y),
        Vec2::new(center_w, body_rect.height()),
    );
    render_center_area(ui, center_rect, state, tx_comando);

    if state.mostrar_direita {
        let right_rect = egui::Rect::from_min_size(
            egui::pos2(center_rect.max.x, body_rect.min.y),
            Vec2::new(right_w, body_rect.height()),
        );
        render_right_inspector(ui, right_rect, state, tx_comando);
    }
}

fn render_top_toolbar(
    ui: &mut egui::Ui,
    rect: egui::Rect,
    state: &mut ArcisState,
    tx_comando: &Sender<AppCommand>,
) {
    ui.painter().rect_filled(rect, 0.0, theme::BG_TOOLBAR);
    theme::hairline_h(ui, rect, rect.max.y);

    ui.scope_builder(
        egui::UiBuilder::new().max_rect(rect.shrink2(Vec2::new(theme::PANEL_PAD, 4.0))),
        |ui| {
            ui.horizontal_centered(|ui| {
                // Ação rápida de Run/Load
                if ui
                    .button(RichText::new("▶").color(theme::TEXT_PRIMARY))
                    .on_hover_text("Carregar DEM padrão (LOAD)")
                    .clicked()
                {
                    let _ = tx_comando.send(AppCommand::ProcessarTerminal("LOAD".into()));
                }

                ui.add_space(6.0);
                for modulo in ["GIS", "CAD", "STEM"] {
                    let ativo = state.modulo_ativo == modulo;
                    if ui
                        .selectable_label(ativo, RichText::new(modulo).small())
                        .clicked()
                    {
                        state.modulo_ativo = modulo.into();
                    }
                }

                ui.add_space(12.0);

                // Campo de busca geoespacial real (Nominatim + Copernicus DEM)
                let busca_resp = ui.add(
                    egui::TextEdit::singleline(&mut state.busca_gis)
                        .hint_text("Cidade ou região (ex: Garanhuns, PE)")
                        .desired_width(210.0),
                );
                let disparar = ui.button("Buscar DEM").clicked()
                    || (busca_resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)));
                if disparar && !state.busca_gis.trim().is_empty() {
                    let _ = tx_comando.send(AppCommand::ProcessarTerminal(format!(
                        "GEO {}",
                        state.busca_gis.trim()
                    )));
                }

                ui.add_space(10.0);

                // Pílula de status (estilo Xcode)
                theme::pill_frame().show(ui, |ui| {
                    ui.label(
                        RichText::new(format!(
                            "Arcis_Core 〉{} 〉{} vértices",
                            state.modulo_ativo,
                            state.vertices_carregados.len()
                        ))
                        .small()
                        .color(theme::TEXT_MUTED),
                    );
                });

                // Toggles dos painéis à direita
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.toggle_value(&mut state.mostrar_direita, RichText::new("◨").small());
                    ui.toggle_value(&mut state.mostrar_console, RichText::new("⬓").small());
                    ui.toggle_value(&mut state.mostrar_esquerda, RichText::new("◧").small());
                });
            });
        },
    );
}

fn render_left_navigator(ui: &mut egui::Ui, rect: egui::Rect, state: &mut ArcisState) {
    ui.painter().rect_filled(rect, 0.0, BG_SIDEBAR);
    theme::hairline_v(ui, rect, rect.max.x);

    let filter_h = theme::FILTER_H;
    let content_rect =
        egui::Rect::from_min_max(rect.min, egui::pos2(rect.max.x, rect.max.y - filter_h));
    let filter_rect =
        egui::Rect::from_min_max(egui::pos2(rect.min.x, rect.max.y - filter_h), rect.max);

    ui.scope_builder(
        egui::UiBuilder::new().max_rect(content_rect.shrink(8.0)),
        |ui| {
            ui.vertical(|ui| {
                // Seção de Gauges (CPU, Memória, GPU)
                ui.label(theme::section_label("ARCIS ENGINE • PID 4869"));
                ui.add_space(4.0);
                render_gauge_row(ui, "CPU", "2%", theme::SYS_GREEN);
                render_gauge_row(ui, "Memory", "38.4 MB", theme::ACCENT_BLUE);
                render_gauge_row(ui, "GPU VRAM", "124 MB", theme::SYS_PURPLE);
                render_gauge_row(ui, "I/O Disk", "0 KB/s", theme::SYS_ORANGE);

                ui.add_space(8.0);
                ui.separator();
                ui.add_space(4.0);

                ui.label(theme::section_label("HIERARQUIA DE CAMADAS"));
                ui.add_space(4.0);

                egui::ScrollArea::vertical().show(ui, |ui| {
                    let filtro = state.filtro_hierarquia.to_lowercase();
                    for (i, camada) in state.camadas_ativas.iter().enumerate() {
                        if !filtro.is_empty() && !camada.to_lowercase().contains(&filtro) {
                            continue;
                        }
                        let selecionada = state.camada_selecionada == i;
                        let resp = ui.selectable_label(
                            selecionada,
                            RichText::new(format!("◆ {}", camada)).small(),
                        );
                        if resp.clicked() {
                            state.camada_selecionada = i;
                        }
                    }
                });
            });
        },
    );

    // Barra de filtro no rodapé esquerdo
    theme::hairline_h(ui, filter_rect, filter_rect.min.y);
    ui.scope_builder(
        egui::UiBuilder::new().max_rect(filter_rect.shrink2(Vec2::new(6.0, 3.0))),
        |ui| {
            ui.add(
                egui::TextEdit::singleline(&mut state.filtro_hierarquia)
                    .hint_text("Filter")
                    .desired_width(ui.available_width()),
            );
        },
    );
}

fn render_gauge_row(ui: &mut egui::Ui, label: &str, value: &str, dot_color: egui::Color32) {
    ui.horizontal(|ui| {
        ui.label(RichText::new("■").small().color(dot_color));
        ui.label(RichText::new(label).small().color(theme::TEXT_PRIMARY));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(RichText::new(value).small().color(theme::TEXT_MUTED));
        });
    });
}

fn render_center_area(
    ui: &mut egui::Ui,
    rect: egui::Rect,
    state: &mut ArcisState,
    tx_comando: &Sender<AppCommand>,
) {
    let console_h = if state.mostrar_console {
        theme::CONSOLE_H
    } else {
        0.0
    };
    let breadcrumb_h = theme::BREADCRUMB_H;
    let controls_h = theme::CONTROLS_H;

    let breadcrumb_rect =
        egui::Rect::from_min_size(rect.min, Vec2::new(rect.width(), breadcrumb_h));
    let viewport_rect = egui::Rect::from_min_max(
        egui::pos2(rect.min.x, breadcrumb_rect.max.y),
        egui::pos2(
            rect.max.x,
            (rect.max.y - console_h - controls_h).max(breadcrumb_rect.max.y + 80.0),
        ),
    );
    let controls_rect = egui::Rect::from_min_size(
        egui::pos2(rect.min.x, viewport_rect.max.y),
        Vec2::new(rect.width(), controls_h),
    );

    // Breadcrumbs superiores
    ui.painter().rect_filled(breadcrumb_rect, 0.0, BG_SUBBAR);
    theme::hairline_h(ui, breadcrumb_rect, breadcrumb_rect.max.y);
    ui.scope_builder(
        egui::UiBuilder::new()
            .max_rect(breadcrumb_rect.shrink2(Vec2::new(theme::PANEL_PAD, 2.0))),
        |ui| {
            ui.horizontal_centered(|ui| {
                let camada = state
                    .camadas_ativas
                    .get(state.camada_selecionada)
                    .map(|s| s.as_str())
                    .unwrap_or("Nenhuma");
                ui.label(
                    RichText::new(format!("Arcis 〉{} 〉{}", state.modulo_ativo, camada))
                        .small()
                        .color(theme::TEXT_MUTED),
                );
            });
        },
    );

    // Sobe a textura de satélite para a GPU assim que chegar do worker
    if let Some((w, h, rgba)) = state.sat_pending_rgba.take() {
        let color_img = egui::ColorImage::from_rgba_unmultiplied([w as usize, h as usize], &rgba);
        state.sat_texture = Some(ui.ctx().load_texture(
            "arcis_sat_mosaic",
            color_img,
            egui::TextureOptions::LINEAR,
        ));
    }

    // Viewport Principal (2D Satélite ou 3D Terreno WGPU)
    ui.painter().rect_filled(viewport_rect, 0.0, BG_VIEWPORT);

    if state.modo_2d_satelite {
        if let (Some(tex), Some(meta)) = (&state.sat_texture, state.sat_meta) {
            // Centraliza o mosaico quadrado dentro do viewport
            let side = viewport_rect.width().min(viewport_rect.height());
            let img_rect = egui::Rect::from_center_size(viewport_rect.center(), Vec2::splat(side));

            ui.painter().image(
                tex.id(),
                img_rect,
                egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                egui::Color32::WHITE,
            );

            // Clique no mapa -> converte pixel para (lat, lon) e dispara análise altimétrica
            let map_resp =
                ui.interact(img_rect, ui.id().with("sat_map_click"), egui::Sense::click());
            if map_resp.clicked() {
                if let Some(pos) = map_resp.interact_pointer_pos() {
                    let u = ((pos.x - img_rect.min.x) / img_rect.width()) as f64;
                    let v = ((pos.y - img_rect.min.y) / img_rect.height()) as f64;
                    let n = 2.0_f64.powi(meta.zoom as i32);
                    let tx = meta.tile_x0 as f64 + u * 3.0;
                    let ty = meta.tile_y0 as f64 + v * 3.0;

                    let lon = tx / n * 360.0 - 180.0;
                    let lat = (std::f64::consts::PI * (1.0 - 2.0 * ty / n))
                        .sinh()
                        .atan()
                        .to_degrees();

                    state.demarcacoes.push((lat, lon));
                    let _ = tx_comando.send(AppCommand::ProcessarTerminal(format!(
                        "DEM {:.5} {:.5}",
                        lat, lon
                    )));
                }
            }

            // Desenha os pinos demarcados sobre o satélite
            let n = 2.0_f64.powi(meta.zoom as i32);
            for (idx, &(lat, lon)) in state.demarcacoes.iter().enumerate() {
                let px_f = (lon + 180.0) / 360.0 * n;
                let lat_rad = lat.to_radians();
                let py_f = (1.0
                    - (lat_rad.tan() + 1.0 / lat_rad.cos()).ln() / std::f64::consts::PI)
                    / 2.0
                    * n;

                let u = ((px_f - meta.tile_x0 as f64) / 3.0) as f32;
                let v = ((py_f - meta.tile_y0 as f64) / 3.0) as f32;
                if (0.0..=1.0).contains(&u) && (0.0..=1.0).contains(&v) {
                    let p = egui::pos2(
                        img_rect.min.x + u * img_rect.width(),
                        img_rect.min.y + v * img_rect.height(),
                    );
                    ui.painter().circle_filled(p, 5.0, theme::SYS_ORANGE);
                    ui.painter().circle_stroke(
                        p,
                        8.0,
                        egui::Stroke::new(1.5, egui::Color32::WHITE),
                    );
                    ui.painter().text(
                        p + Vec2::new(10.0, -8.0),
                        egui::Align2::LEFT_CENTER,
                        format!("P{} ({:.4}, {:.4})", idx + 1, lat, lon),
                        theme::mono(10.5),
                        egui::Color32::WHITE,
                    );
                }
            }
        } else {
            ui.painter().text(
                viewport_rect.center(),
                egui::Align2::CENTER_CENTER,
                "Busque um local na barra superior para carregar o satélite",
                theme::mono(12.0),
                theme::TEXT_MUTED,
            );
        }
    } else {
        // Interação de câmera orbital 3D no Viewport (arraste = órbita, scroll = zoom)
        let vp_resp = ui.interact(
            viewport_rect,
            ui.id().with("arcis_viewport_3d"),
            egui::Sense::click_and_drag(),
        );
        if vp_resp.dragged() {
            let delta = vp_resp.drag_delta();
            state.cam_yaw += delta.x * 0.01;
            state.cam_pitch = (state.cam_pitch + delta.y * 0.01).clamp(0.1, 1.5);
        }
        if vp_resp.hovered() {
            let scroll = ui.input(|i| i.smooth_scroll_delta.y);
            if scroll.abs() > 0.0 {
                state.cam_zoom = (state.cam_zoom - scroll * 0.003).clamp(0.6, 5.0);
            }
        }

        // Viewport 3D WGPU
        ui.painter()
            .add(eframe::egui_wgpu::Callback::new_paint_callback(
                viewport_rect,
                crate::render::wgpu_canvas::WgpuCallback {
                    vertices: state.vertices_carregados.clone(),
                    indices: state.indices_carregados.clone(),
                    yaw: state.cam_yaw,
                    pitch: state.cam_pitch,
                    zoom: state.cam_zoom,
                    opacity: state.opacidade_viewport,
                },
            ));
    }

    // Barra de controles do Canvas (Modo 2D/3D + Sliders)
    ui.painter().rect_filled(controls_rect, 0.0, BG_SUBBAR);
    theme::hairline_h(ui, controls_rect, controls_rect.min.y);
    theme::hairline_h(ui, controls_rect, controls_rect.max.y);
    ui.scope_builder(
        egui::UiBuilder::new().max_rect(controls_rect.shrink2(Vec2::new(theme::PANEL_PAD, 3.0))),
        |ui| {
            ui.horizontal_centered(|ui| {
                ui.selectable_value(
                    &mut state.modo_2d_satelite,
                    true,
                    RichText::new("2D Satélite").small(),
                );
                ui.selectable_value(
                    &mut state.modo_2d_satelite,
                    false,
                    RichText::new("3D Terreno").small(),
                );
                ui.separator();

                if state.modo_2d_satelite {
                    if ui.button(RichText::new("Limpar Pinos").small()).clicked() {
                        state.demarcacoes.clear();
                    }
                } else {
                    ui.label(RichText::new("Opacidade").small().color(theme::TEXT_MUTED));
                    ui.add(
                        egui::Slider::new(&mut state.opacidade_viewport, 0.0..=1.0)
                            .show_value(false),
                    );
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let dica = if state.modo_2d_satelite {
                        "Clique no mapa p/ demarcar e analisar • SIRGAS 2000"
                    } else {
                        "Arraste p/ orbitar • Scroll p/ zoom • SIRGAS 2000"
                    };
                    ui.label(RichText::new(dica).small().color(theme::TEXT_MUTED));
                });
            });
        },
    );

    // Console Dividido (Estilo Xcode Debug Area)
    if state.mostrar_console {
        let console_rect =
            egui::Rect::from_min_max(egui::pos2(rect.min.x, controls_rect.max.y), rect.max);
        render_split_console(ui, console_rect, state, tx_comando);
    }
}

fn render_split_console(
    ui: &mut egui::Ui,
    rect: egui::Rect,
    state: &mut ArcisState,
    tx_comando: &Sender<AppCommand>,
) {
    ui.painter().rect_filled(rect, 0.0, theme::BG_CONSOLE);
    let split_x = rect.min.x + rect.width() * 0.36;
    theme::hairline_v(ui, rect, split_x);

    // Esquerda: Variáveis / Estado do Kernel
    let vars_rect = egui::Rect::from_min_max(rect.min, egui::pos2(split_x, rect.max.y));
    ui.scope_builder(
        egui::UiBuilder::new().max_rect(vars_rect.shrink(8.0)),
        |ui| {
            ui.vertical(|ui| {
                ui.label(theme::section_label("KERNEL STATE"));
                ui.add_space(4.0);
                ui.label(
                    RichText::new(format!("modulo = \"{}\"", state.modulo_ativo))
                        .font(theme::mono(11.0)),
                );
                ui.label(
                    RichText::new(format!("vertices = {}", state.vertices_carregados.len()))
                        .font(theme::mono(11.0)),
                );
                ui.label(
                    RichText::new(format!("indices = {}", state.indices_carregados.len()))
                        .font(theme::mono(11.0)),
                );
                ui.label(
                    RichText::new(format!("camadas = {}", state.camadas_ativas.len()))
                        .font(theme::mono(11.0)),
                );
                ui.label(
                    RichText::new(format!("pinos = {}", state.demarcacoes.len()))
                        .font(theme::mono(11.0)),
                );
            });
        },
    );

    // Direita: Terminal LLDB/Arcis
    let term_rect = egui::Rect::from_min_max(egui::pos2(split_x + 1.0, rect.min.y), rect.max);
    ui.scope_builder(
        egui::UiBuilder::new().max_rect(term_rect.shrink(8.0)),
        |ui| {
            ui.vertical(|ui| {
                let log_h = (ui.available_height() - 26.0).max(40.0);
                egui::ScrollArea::vertical()
                    .max_height(log_h)
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        for log in &state.historico_terminal {
                            let cor = if log.starts_with('>') {
                                theme::SYS_GREEN
                            } else {
                                theme::TEXT_PRIMARY
                            };
                            ui.label(RichText::new(log).font(theme::mono(11.0)).color(cor));
                        }
                    });

                ui.separator();
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new("(arcis)")
                            .font(theme::mono(11.0))
                            .color(theme::ACCENT_BLUE),
                    );
                    let resp = ui.add(
                        egui::TextEdit::singleline(&mut state.input_terminal)
                            .font(theme::mono(11.0))
                            .desired_width(ui.available_width()),
                    );
                    if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        let cmd = terminal_parser::limpar_input(&state.input_terminal);
                        if !cmd.is_empty() {
                            let _ = tx_comando.send(AppCommand::ProcessarTerminal(cmd));
                            state.input_terminal.clear();
                        }
                        resp.request_focus();
                    }
                });
            });
        },
    );
}

fn render_right_inspector(
    ui: &mut egui::Ui,
    rect: egui::Rect,
    state: &ArcisState,
    tx_comando: &Sender<AppCommand>,
) {
    ui.painter().rect_filled(rect, 0.0, BG_SIDEBAR);
    theme::hairline_v(ui, rect, rect.min.x);

    ui.scope_builder(
        egui::UiBuilder::new().max_rect(rect.shrink(theme::PANEL_PAD)),
        |ui| {
            ui.vertical(|ui| {
                ui.label(theme::section_label("OBJECT INSPECTOR"));
                ui.add_space(6.0);

                let camada = state
                    .camadas_ativas
                    .get(state.camada_selecionada)
                    .map(|s| s.as_str())
                    .unwrap_or("Nenhuma camada");

                egui::Grid::new("xcode_inspector_grid")
                    .num_columns(2)
                    .spacing(Vec2::new(10.0, 5.0))
                    .show(ui, |ui| {
                        inspector_row(ui, "Object", camada);
                        inspector_row(ui, "Vertices", &state.vertices_carregados.len().to_string());
                        inspector_row(ui, "Indices", &state.indices_carregados.len().to_string());
                        inspector_row(
                            ui,
                            "Opacity",
                            &format!("{:.0}%", state.opacidade_viewport * 100.0),
                        );
                    });

                ui.add_space(12.0);
                ui.separator();
                ui.add_space(6.0);

                if let Some(m) = &state.metricas_ativas {
                    ui.label(theme::section_label("LEVANTAMENTO TOPOGRÁFICO"));
                    ui.add_space(4.0);
                    egui::Grid::new("xcode_survey_grid")
                        .num_columns(2)
                        .spacing(Vec2::new(10.0, 5.0))
                        .show(ui, |ui| {
                            inspector_row(
                                ui,
                                "Lat / Lon",
                                &format!("{:.4}, {:.4}", m.lat_centro, m.lon_centro),
                            );
                            inspector_row(ui, "Modo", &m.modo_visualizacao);
                            inspector_row(ui, "Área", &format!("{:.2} km²", m.area_km2));
                            inspector_row(
                                ui,
                                "Min / Máx",
                                &format!("{:.1}m / {:.1}m", m.min_elev, m.max_elev),
                            );
                            inspector_row(ui, "Média", &format!("{:.1} m", m.media_elev));
                            inspector_row(ui, "Amplitude", &format!("{:.1} m", m.amplitude));
                            inspector_row(ui, "Rugosidade", &format!("{:.2} m", m.rugosidade));
                            inspector_row(
                                ui,
                                "Decliv. Méd.",
                                &format!("{:.2}°", m.declividade_media_graus),
                            );
                            inspector_row(
                                ui,
                                "Decliv. Máx.",
                                &format!("{:.2}°", m.declividade_max_graus),
                            );
                            inspector_row(
                                ui,
                                "Vol. Corte",
                                &format!("{:.0} m³", m.volume_corte_m3),
                            );
                        });

                    ui.add_space(12.0);
                    ui.separator();
                    ui.add_space(6.0);
                    ui.label(theme::section_label("OPERAÇÕES GIS"));
                    ui.add_space(4.0);

                    ui.horizontal(|ui| {
                        if ui.button("Elevação").clicked() {
                            let _ = tx_comando.send(AppCommand::ProcessarTerminal("ELEV".into()));
                        }
                        if ui.button("Declividade").clicked() {
                            let _ = tx_comando.send(AppCommand::ProcessarTerminal("SLOPE".into()));
                        }
                    });
                    ui.add_space(4.0);
                    if ui.button("Exportar Relatório (.md / .csv)").clicked() {
                        let _ = tx_comando.send(AppCommand::ProcessarTerminal("EXPORT".into()));
                    }
                } else {
                    ui.label(
                        RichText::new(
                            "Busque uma cidade na barra superior para iniciar o levantamento.",
                        )
                        .small()
                        .color(theme::TEXT_MUTED),
                    );
                }
            });
        },
    );
}

fn inspector_row(ui: &mut egui::Ui, label: &str, value: &str) {
    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
        ui.label(RichText::new(label).small().color(theme::TEXT_MUTED));
    });
    ui.label(RichText::new(value).small().color(theme::TEXT_PRIMARY));
    ui.end_row();
}