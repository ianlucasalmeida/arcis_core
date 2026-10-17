// src/bus/messages.rs
use bytemuck::{Pod, Zeroable};

// A estrutura matemática base de qualquer geometria no Arcis
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct ArcisVertex {
    pub position: [f32; 3], // Coordenadas X, Y, Z
    pub color: [f32; 3],    // Cor R, G, B (Baseado na elevação ou material)
}

pub enum AppCommand {
    ProcessarTerminal(String),
    CarregarModeloGeoespacial(String), // Comando real de operação
}

pub enum AppResponse {
    Log(String),
    // A resposta agora carrega a geometria pesada gerada pelo Worker
    OperacaoConcluida {
        nome_camada: String,
        vertices: Vec<ArcisVertex>,
        indices: Vec<u32>,
    },
}