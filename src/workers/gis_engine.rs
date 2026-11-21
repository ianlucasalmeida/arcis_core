use crate::bus::messages::{AppCommand, AppResponse};
use crate::workers::analysis::{
    calcular_levantamento, construir_malha, exportar_relatorio, DatasetAtivo,
};
use reqwest::Client;
use serde::Deserialize;
use std::sync::mpsc::{Receiver, Sender};
use tokio::runtime::Runtime;

#[derive(Deserialize)]
struct NominatimResult {
    lat: String,
    lon: String,
    display_name: String,
}

#[derive(Deserialize)]
struct OpenMeteoElevation {
    elevation: Vec<f32>,
}

pub fn iniciar_motor_assincrono(
    rx_comandos: Receiver<AppCommand>,
    tx_respostas: Sender<AppResponse>,
) {
    std::thread::spawn(move || {
        let rt = Runtime::new().unwrap();

        rt.block_on(async {
            let client = Client::builder()
                .user_agent("Arcis-STEM-Engine/0.1.0")
                .build()
                .unwrap();

            let mut ultimo_dataset: Option<DatasetAtivo> = None;

            while let Ok(cmd) = rx_comandos.recv() {
                let AppCommand::ProcessarTerminal(texto) = cmd;
                let _ = tx_respostas.send(AppResponse::Log(format!("> {}", texto)));
                let partes: Vec<&str> = texto.split_whitespace().collect();
                if partes.is_empty() {
                    continue;
                }

                match partes[0].to_uppercase().as_str() {
                    "GEO" => {
                        let query = partes[1..].join(" ");
                        if query.is_empty() {
                            let _ = tx_respostas.send(AppResponse::Log(
                                "[USO] GEO <cidade ou local> (ex: GEO Garanhuns)".into(),
                            ));
                            continue;
                        }
                        if let Some(ds) =
                            executar_geocodificacao_e_dem(&client, &query, &tx_respostas).await
                        {
                            ultimo_dataset = Some(ds);
                        }
                    }
                    "DEM" => {
                        if partes.len() < 3 {
                            let _ = tx_respostas.send(AppResponse::Log(
                                "[USO] DEM <lat> <lon> (ex: DEM -8.1159 -35.0154)".into(),
                            ));
                            continue;
                        }
                        if let (Ok(lat), Ok(lon)) =
                            (partes[1].parse::<f64>(), partes[2].parse::<f64>())
                        {
                            if let Some(ds) = baixar_dem_e_analisar(
                                &client,
                                lat,
                                lon,
                                &format!("DEM_{:.4}_{:.4}", lat, lon),
                                &tx_respostas,
                            )
                            .await
                            {
                                ultimo_dataset = Some(ds);
                            }
                        } else {
                            let _ = tx_respostas.send(AppResponse::Log(
                                "[ERRO] Coordenadas inválidas. Use ponto decimal (ex: -8.1159 -35.0154).".into(),
                            ));
                        }
                    }
                    "LOAD" => {
                        if let Some(ds) = baixar_dem_e_analisar(
                            &client,
                            -8.1159,
                            -35.0154,
                            "Copernicus_DEM_Jaboatao.dem",
                            &tx_respostas,
                        )
                        .await
                        {
                            ultimo_dataset = Some(ds);
                        }
                    }
                    "SLOPE" | "ELEV" => {
                        let modo_slope = partes[0].eq_ignore_ascii_case("SLOPE");
                        if let Some(ds) = &ultimo_dataset {
                            let label = if modo_slope {
                                "Declividade (Slope)"
                            } else {
                                "Hipsometria (Elevação)"
                            };
                            let (metrics, slopes) = calcular_levantamento(ds, label);
                            let (vertices, indices) =
                                construir_malha(ds, &metrics, &slopes, modo_slope);

                            let _ = tx_respostas.send(AppResponse::Log(format!(
                                "[ANÁLISE] Modo alterado para: {}",
                                label
                            )));
                            let _ = tx_respostas.send(AppResponse::OperacaoConcluida {
                                nome_camada: ds.nome.clone(),
                                vertices,
                                indices,
                                metrics,
                            });
                        } else {
                            let _ = tx_respostas.send(AppResponse::Log(
                                "[AVISO] Carregue uma área primeiro com GEO, DEM ou LOAD.".into(),
                            ));
                        }
                    }
                    "EXPORT" => {
                        if let Some(ds) = &ultimo_dataset {
                            let (metrics, slopes) = calcular_levantamento(ds, "Exportação");
                            match exportar_relatorio(ds, &metrics, &slopes) {
                                Ok(msg) => {
                                    let _ = tx_respostas
                                        .send(AppResponse::Log(format!("[EXPORT] {}", msg)));
                                }
                                Err(e) => {
                                    let _ = tx_respostas
                                        .send(AppResponse::Log(format!("[ERRO] Exportação: {}", e)));
                                }
                            }
                        } else {
                            let _ = tx_respostas.send(AppResponse::Log(
                                "[AVISO] Nenhum dado carregado para exportar.".into(),
                            ));
                        }
                    }
                    _ => {
                        let _ = tx_respostas.send(AppResponse::Log(
                            "[INFO] Comandos: GEO <local> | DEM <lat> <lon> | LOAD | SLOPE | ELEV | EXPORT".into(),
                        ));
                    }
                }
            }
        });
    });
}
async fn executar_geocodificacao_e_dem(
    client: &Client,
    query: &str,
    tx: &Sender<AppResponse>,
) -> Option<DatasetAtivo> {
    let _ = tx.send(AppResponse::Log(format!(
        "[GEO] Consultando Nominatim para '{}'...",
        query
    )));

    let url = format!(
        "https://nominatim.openstreetmap.org/search?q={}&format=json&limit=1",
        query.replace(' ', "+")
    );

    match client.get(&url).send().await {
        Ok(resp) => match resp.json::<Vec<NominatimResult>>().await {
            Ok(resultados) if !resultados.is_empty() => {
                let r = &resultados[0];
                let lat: f64 = r.lat.parse().unwrap_or(0.0);
                let lon: f64 = r.lon.parse().unwrap_or(0.0);
                let _ = tx.send(AppResponse::Log(format!(
                    "[GEO] Encontrado: {} ({:.5}, {:.5})",
                    r.display_name, lat, lon
                )));

                let nome_camada = format!("DEM_{}.tif", query.replace(' ', "_"));
                baixar_dem_e_analisar(client, lat, lon, &nome_camada, tx).await
            }
            Ok(_) => {
                let _ = tx.send(AppResponse::Log(
                    "[GEO] Nenhum resultado encontrado.".into(),
                ));
                None
            }
            Err(e) => {
                let _ = tx.send(AppResponse::Log(format!("[ERRO] Parse JSON: {}", e)));
                None
            }
        },
        Err(e) => {
            let _ = tx.send(AppResponse::Log(format!("[ERRO] Rede Nominatim: {}", e)));
            None
        }
    }
}

async fn baixar_dem_e_analisar(
    client: &Client,
    lat_centro: f64,
    lon_centro: f64,
    nome_camada: &str,
    tx: &Sender<AppResponse>,
) -> Option<DatasetAtivo> {
    let _ = tx.send(AppResponse::Log(format!(
        "[DEM] Coletando grade 10x10 (Copernicus 90m) em {:.4}, {:.4}...",
        lat_centro, lon_centro
    )));

    let grid: usize = 10;
    let raio_graus = 0.02;

    let mut lats = Vec::with_capacity(grid * grid);
    let mut lons = Vec::with_capacity(grid * grid);

    for z in 0..grid {
        for x in 0..grid {
            let fx = (x as f64 / (grid - 1) as f64) * 2.0 - 1.0;
            let fz = (z as f64 / (grid - 1) as f64) * 2.0 - 1.0;
            lats.push(format!("{:.5}", lat_centro + fz * raio_graus));
            lons.push(format!("{:.5}", lon_centro + fx * raio_graus));
        }
    }

    let url = format!(
        "https://api.open-meteo.com/v1/elevation?latitude={}&longitude={}",
        lats.join(","),
        lons.join(",")
    );

    match client.get(&url).send().await {
        Ok(resp) => match resp.json::<OpenMeteoElevation>().await {
            Ok(data) if data.elevation.len() == grid * grid => {
                let ds = DatasetAtivo {
                    nome: nome_camada.to_string(),
                    lat_centro,
                    lon_centro,
                    raio_graus,
                    grid: grid as u32,
                    elevacoes: data.elevation,
                };

                let (metrics, slopes) = calcular_levantamento(&ds, "Hipsometria (Elevação)");
                let (vertices, indices) = construir_malha(&ds, &metrics, &slopes, false);

                let _ = tx.send(AppResponse::Log(format!(
                    "[ANÁLISE] Min: {:.1}m | Máx: {:.1}m | Amplitude: {:.1}m | Decliv. Máx: {:.1}°",
                    metrics.min_elev,
                    metrics.max_elev,
                    metrics.amplitude,
                    metrics.declividade_max_graus
                )));

                let _ = tx.send(AppResponse::OperacaoConcluida {
                    nome_camada: ds.nome.clone(),
                    vertices,
                    indices,
                    metrics,
                });

                Some(ds)
            }
            Ok(_) => {
                let _ = tx.send(AppResponse::Log(
                    "[ERRO] Quantidade de pontos retornada inesperada.".into(),
                ));
                None
            }
            Err(e) => {
                let _ = tx.send(AppResponse::Log(format!("[ERRO] Parse DEM: {}", e)));
                None
            }
        },
        Err(e) => {
            let _ = tx.send(AppResponse::Log(format!("[ERRO] Rede DEM: {}", e)));
            None
        }
    }
}

async fn baixar_mapa_satelite(
    client: &Client,
    lat: f64,
    lon: f64,
    zoom: u8,
    tx: &Sender<AppResponse>,
) {
    let _ = tx.send(AppResponse::Log(format!(
        "[SAT] Baixando mosaico 3x3 (Esri World Imagery, zoom {})...",
        zoom
    )));

    let n = 2.0_f64.powi(zoom as i32);
    let cx = ((lon + 180.0) / 360.0 * n).floor() as i32;
    let lat_rad = lat.to_radians();
    let cy = ((1.0 - (lat_rad.tan() + 1.0 / lat_rad.cos()).ln() / std::f64::consts::PI) / 2.0 * n)
        .floor() as i32;

    let tile_x0 = cx - 1;
    let tile_y0 = cy - 1;
    let width = 256 * 3;
    let height = 256 * 3;
    let mut rgba = vec![24u8; (width * height * 4) as usize];

    for dy in 0..3 {
        for dx in 0..3 {
            let tx_i = tile_x0 + dx;
            let ty_i = tile_y0 + dy;
            let url = format!(
                "https://server.arcgisonline.com/ArcGIS/rest/services/World_Imagery/MapServer/tile/{}/{}/{}",
                zoom, ty_i, tx_i
            );

            if let Ok(resp) = client.get(&url).send().await {
                if let Ok(bytes) = resp.bytes().await {
                    if let Ok(img) = image::load_from_memory(&bytes) {
                        let tile_rgba = img.to_rgba8();
                        for py in 0..256u32 {
                            for px in 0..256u32 {
                                let p = tile_rgba.get_pixel(px, py);
                                let dst_x = (dx as u32) * 256 + px;
                                let dst_y = (dy as u32) * 256 + py;
                                let idx = ((dst_y * width + dst_x) * 4) as usize;
                                rgba[idx..idx + 4].copy_from_slice(&p.0);
                            }
                        }
                    }
                }
            }
        }
    }

    let _ = tx.send(AppResponse::MapaSatelitePronto {
        zoom,
        tile_x0,
        tile_y0,
        width,
        height,
        rgba,
    });
    let _ = tx.send(AppResponse::Log(
        "[SAT] Mosaico de satélite pronto. Clique no mapa para demarcar e analisar.".into(),
    ));
}