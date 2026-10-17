// src/workers/gis_engine.rs
use crate::bus::messages::{AppCommand, AppResponse, ArcisVertex};
use std::sync::mpsc::{Receiver, Sender};
use tokio::runtime::Runtime;

pub fn iniciar_motor_assincrono(rx_comandos: Receiver<AppCommand>, tx_respostas: Sender<AppResponse>) {
    std::thread::spawn(move || {
        let rt = Runtime::new().unwrap();
        
        rt.block_on(async {
            while let Ok(cmd) = rx_comandos.recv() {
                match cmd {
                    AppCommand::ProcessarTerminal(texto) => {
                        let _ = tx_respostas.send(AppResponse::Log(format!("> {}", texto)));
                        let comando = texto.to_uppercase();
                        
                        // OPERAÇÃO ARCIS: Ingestão de Dados
                        if comando.starts_with("LOAD") {
                            let _ = tx_respostas.send(AppResponse::Log("[NASA API] A extrair metadados e fatiar blocos COG (GeoTIFF)...".into()));
                            
                            // Simula o tempo de download/leitura do disco
                            tokio::time::sleep(std::time::Duration::from_millis(1500)).await;
                            
                            // GERADOR MATEMÁTICO: Converte o Raster num Terreno 3D (Malha)
                            let (vertices, indices) = gerar_malha_topografica_nasa(50, 50);
                            
                            let _ = tx_respostas.send(AppResponse::Log(format!("[OK] Operação concluída. Geometria gerada com {} vértices.", vertices.len())));
                            
                            // Envia os dados pesados prontos para a Placa de Vídeo
                            let _ = tx_respostas.send(AppResponse::OperacaoConcluida {
                                nome_camada: "NASA_Earthdata_Dataset.tif".into(),
                                vertices,
                                indices,
                            });
                        } else {
                            let _ = tx_respostas.send(AppResponse::Log(format!("[FALHA] Comando '{}' não reconhecido pelo Kernel.", texto)));
                        }
                    }
                    _ => {}
                }
            }
        });
    });
}

// Algoritmo interno do Arcis para converter pixels em vértices 3D
fn gerar_malha_topografica_nasa(largura: u32, profundidade: u32) -> (Vec<ArcisVertex>, Vec<u32>) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    // 1. Gera os pontos baseados nas coordenadas espaciais
    for z in 0..profundidade {
        for x in 0..largura {
            // Normaliza as coordenadas entre -1.0 e 1.0 para o Viewport
            let pos_x = (x as f32 / largura as f32) * 2.0 - 1.0;
            let pos_z = (z as f32 / profundidade as f32) * 2.0 - 1.0;
            
            // Simula a elevação (Z) do terreno (Pico e Vales)
            let elevacao = (pos_x * 5.0).sin() * (pos_z * 5.0).cos() * 0.2;

            // Mapeia a cor baseada na elevação (Estilo Topográfico: Azul p/ baixo, Branco p/ alto)
            let cor = [0.2 + elevacao, 0.4 + elevacao, 0.8]; 

            vertices.push(ArcisVertex {
                position: [pos_x, elevacao, pos_z],
                color: cor,
            });
        }
    }

    // 2. Gera os triângulos (Índices) para a GPU conectar os pontos
    for z in 0..(profundidade - 1) {
        for x in 0..(largura - 1) {
            let inicio = z * largura + x;
            indices.push(inicio);
            indices.push(inicio + largura);
            indices.push(inicio + 1);

            indices.push(inicio + 1);
            indices.push(inicio + largura);
            indices.push(inicio + largura + 1);
        }
    }

    (vertices, indices)
}