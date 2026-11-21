use crate::bus::messages::{AppCommand, AppResponse, ArcisVertex};
use egui_dock::DockState;
use std::sync::mpsc::{Receiver, Sender};

pub enum ArcisPanel {
    Viewport3D,
    WorkspaceLayers,
    Properties,
    TerminalLog,
}

pub struct ArcisState {
    pub camadas_ativas: Vec<String>,
    pub historico_terminal: Vec<String>,
    pub input_terminal: String,
    pub vertices_carregados: Vec<ArcisVertex>, // <--- Memória da Geometria 3D
}

pub struct ArcisApp {
    pub dock_state: DockState<ArcisPanel>,
    pub state: ArcisState,
    pub tx_comando: Sender<AppCommand>,
    pub rx_resposta: Receiver<AppResponse>,
}

impl ArcisApp {
    pub fn new(tx_comando: Sender<AppCommand>, rx_resposta: Receiver<AppResponse>) -> Self {
        let mut dock_state = DockState::new(vec![ArcisPanel::Viewport3D]);
        let surface = dock_state.main_surface_mut();
        let main_node = egui_dock::NodeIndex::root();

        // 1. Separa a Esquerda (Project Explorer / View Hierarchy) com 20% da tela
        let [_, rest] = surface.split_left(main_node, 0.2, vec![ArcisPanel::WorkspaceLayers]);
        
        // 2. Do restante, separa a Direita (Object Properties) com 25% do espaço restante
        let [center_area, _] = surface.split_right(rest, 0.75, vec![ArcisPanel::Properties]);
        
        // 3. O que sobrou no meio, divide entre Viewport (Topo 80%) e Terminal (Base 20%)
        let [_, _] = surface.split_below(center_area, 0.8, vec![ArcisPanel::TerminalLog]);

        Self {
            dock_state,
            state: ArcisState {
                camadas_ativas: vec!["NASA_Earthdata_Dataset.tif".into(), "Modelo_Base.step".into()],
                historico_terminal: vec!["[SYSTEM] Motor Arcis carregado. Módulos prontos.".into()],
                input_terminal: String::new(),
                vertices_carregados: std::sync::Arc::new(Vec::new()),
                indices_carregados: std::sync::Arc::new(Vec::new()),
            },
            tx_comando,
            rx_resposta,
        }
    }

    // Processa respostas do background a cada ciclo de renderização
    pub fn ler_mensagens_background(&mut self) {
        while let Ok(msg) = self.rx_resposta.try_recv() {
            match msg {
                AppResponse::Log(texto) => self.state.historico_terminal.push(texto),
                AppResponse::OperacaoConcluida { nome_camada, vertices, indices: _ } => {
                    // Guarda o nome no explorador de projeto
                    self.state.camadas_ativas.push(nome_camada);
                    // Armazena a geometria para enviar para a Placa de Vídeo
                    self.state.vertices_carregados = vertices;
                    self.state.historico_terminal.push("[GPU] Malha enviada para o buffer de renderização.".into());
                }
            }
        }
    }
}