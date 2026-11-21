use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct ArcisVertex {
    pub position: [f32; 3],
    pub color: [f32; 3],
}

#[derive(Clone, Debug)]
pub struct DemMetrics {
    pub lat_centro: f64,
    pub lon_centro: f64,
    pub grid_size: u32,
    pub min_elev: f32,
    pub max_elev: f32,
    pub media_elev: f32,
    pub amplitude: f32,
    pub rugosidade: f32,
    pub declividade_media_graus: f32,
    pub declividade_max_graus: f32,
    pub area_km2: f64,
    pub volume_corte_m3: f64,
    pub modo_visualizacao: String,
}

pub enum AppCommand {
    ProcessarTerminal(String),
}

pub enum AppResponse {
    Log(String),
    OperacaoConcluida {
        nome_camada: String,
        vertices: Vec<ArcisVertex>,
        indices: Vec<u32>,
        metrics: DemMetrics,
    },
    MapaSatelitePronto {
        zoom: u8,
        tile_x0: i32,
        tile_y0: i32,
        width: u32,
        height: u32,
        rgba: Vec<u8>,
    },
}