use crate::bus::messages::{AppCommand, AppResponse, ArcisVertex, DemMetrics};
use crate::splash::SplashState;
use eframe::egui;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Arc;

pub enum AppScreen {
    Splash(SplashState),
    Workspace,
}

#[derive(Clone, Copy)]
pub struct SatMapMeta {
    pub zoom: u8,
    pub tile_x0: i32,
    pub tile_y0: i32,
}

pub struct ArcisState {
    pub modulo_ativo: String,
    pub busca_gis: String,
    pub camadas_ativas: Vec<String>,
    pub camada_selecionada: usize,
    pub historico_terminal: Vec<String>,
    pub input_terminal: String,
    pub filtro_hierarquia: String,
    pub filtro_console: String,
    pub opacidade_viewport: f32,
    pub mostrar_esquerda: bool,
    pub mostrar_direita: bool,
    pub mostrar_console: bool,
    // Câmera Orbital 3D
    pub cam_yaw: f32,
    pub cam_pitch: f32,
    pub cam_zoom: f32,
    // Mapa de Satélite 2D e Demarcações
    pub modo_2d_satelite: bool,
    pub sat_texture: Option<egui::TextureHandle>,
    pub sat_pending_rgba: Option<(u32, u32, Vec<u8>)>,
    pub sat_meta: Option<SatMapMeta>,
    pub demarcacoes: Vec<(f64, f64)>,
    // Dados reais da GPU e Levantamento
    pub vertices_carregados: Arc<Vec<ArcisVertex>>,
    pub indices_carregados: Arc<Vec<u32>>,
    pub metricas_ativas: Option<DemMetrics>,
}

pub struct ArcisApp {
    pub screen: AppScreen,
    pub state: ArcisState,
    pub tx_comando: Sender<AppCommand>,
    pub rx_resposta: Receiver<AppResponse>,
}

impl ArcisApp {
    pub fn new(tx_comando: Sender<AppCommand>, rx_resposta: Receiver<AppResponse>) -> Self {
        Self {
            screen: AppScreen::Splash(SplashState::new()),
            state: ArcisState {
                modulo_ativo: "GIS".into(),
                busca_gis: "Garanhuns, PE".into(),
                camadas_ativas: Vec::new(),
                camada_selecionada: 0,
                historico_terminal: vec![
                    "[SYSTEM] Módulo GIS ativo. Busque um local no topo ou use GEO/DEM/SLOPE/EXPORT no console.".into(),
                ],
                input_terminal: String::new(),
                filtro_hierarquia: String::new(),
                filtro_console: String::new(),
                opacidade_viewport: 1.0,
                mostrar_esquerda: true,
                mostrar_direita: true,
                mostrar_console: true,
                cam_yaw: 0.6,
                cam_pitch: 0.7,
                cam_zoom: 1.8,
                modo_2d_satelite: true,
                sat_texture: None,
                sat_pending_rgba: None,
                sat_meta: None,
                demarcacoes: Vec::new(),
                vertices_carregados: Arc::new(Vec::new()),
                indices_carregados: Arc::new(Vec::new()),
                metricas_ativas: None,
            },
            tx_comando,
            rx_resposta,
        }
    }

    pub fn ler_mensagens_background(&mut self) {
        while let Ok(msg) = self.rx_resposta.try_recv() {
            match msg {
                AppResponse::Log(texto) => self.state.historico_terminal.push(texto),
                AppResponse::OperacaoConcluida {
                    nome_camada,
                    vertices,
                    indices,
                    metrics,
                } => {
                    if !self.state.camadas_ativas.contains(&nome_camada) {
                        self.state.camadas_ativas.push(nome_camada);
                    }
                    self.state.camada_selecionada =
                        self.state.camadas_ativas.len().saturating_sub(1);
                    self.state.vertices_carregados = Arc::new(vertices);
                    self.state.indices_carregados = Arc::new(indices);
                    self.state.metricas_ativas = Some(metrics);
                    self.state
                        .historico_terminal
                        .push("[GPU] Malha 3D atualizada na placa de vídeo.".into());
                }
                AppResponse::MapaSatelitePronto {
                    zoom,
                    tile_x0,
                    tile_y0,
                    width,
                    height,
                    rgba,
                } => {
                    self.state.sat_meta = Some(SatMapMeta {
                        zoom,
                        tile_x0,
                        tile_y0,
                    });
                    self.state.sat_pending_rgba = Some((width, height, rgba));
                }
            }
        }
    }
}