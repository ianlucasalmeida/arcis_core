use crate::bus::messages::{ArcisVertex, DemMetrics};
use std::fs;
use std::path::Path;

pub struct DatasetAtivo {
    pub nome: String,
    pub lat_centro: f64,
    pub lon_centro: f64,
    pub raio_graus: f64,
    pub grid: u32,
    pub elevacoes: Vec<f32>,
}

/// Calcula métricas topográficas reais (declividade, rugosidade, área e volume)
pub fn calcular_levantamento(ds: &DatasetAtivo, modo: &str) -> (DemMetrics, Vec<f32>) {
    let n = ds.elevacoes.len() as f32;
    let min_elev = ds.elevacoes.iter().cloned().fold(f32::INFINITY, f32::min);
    let max_elev = ds.elevacoes.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    let media_elev = ds.elevacoes.iter().sum::<f32>() / n;
    let amplitude = max_elev - min_elev;

    // Rugosidade (desvio padrão das cotas)
    let variancia = ds
        .elevacoes
        .iter()
        .map(|e| (e - media_elev).powi(2))
        .sum::<f32>()
        / n;
    let rugosidade = variancia.sqrt();

    // Conversão aproximada de graus para metros na latitude do levantamento
    let metros_por_grau_lat = 111_320.0_f64;
    let metros_por_grau_lon = 111_320.0_f64 * ds.lat_centro.to_radians().cos();

    let largura_m = (ds.raio_graus * 2.0) * metros_por_grau_lon;
    let altura_m = (ds.raio_graus * 2.0) * metros_por_grau_lat;
    let dx = (largura_m / (ds.grid - 1) as f64) as f32;
    let dy = (altura_m / (ds.grid - 1) as f64) as f32;

    let area_km2 = (largura_m * altura_m) / 1_000_000.0;
    let area_celula_m2 = (dx as f64) * (dy as f64);

    // Volume estimado acima da cota mínima (referência de terraplenagem)
    let volume_corte_m3: f64 = ds
        .elevacoes
        .iter()
        .map(|&e| ((e - min_elev).max(0.0) as f64) * area_celula_m2)
        .sum();

    // Cálculo de declividade (Slope) por diferenças finitas em cada célula
    let g = ds.grid as usize;
    let mut slopes = vec![0.0_f32; g * g];
    let mut soma_slope = 0.0_f32;
    let mut max_slope = 0.0_f32;

    for z in 0..g {
        for x in 0..g {
            let x_prev = x.saturating_sub(1);
            let x_next = (x + 1).min(g - 1);
            let z_prev = z.saturating_sub(1);
            let z_next = (z + 1).min(g - 1);

            let dz_dx = (ds.elevacoes[z * g + x_next] - ds.elevacoes[z * g + x_prev])
                / (dx * (x_next - x_prev).max(1) as f32);
            let dz_dy = (ds.elevacoes[z_next * g + x] - ds.elevacoes[z_prev * g + x])
                / (dy * (z_next - z_prev).max(1) as f32);

            let grad = (dz_dx * dz_dx + dz_dy * dz_dy).sqrt();
            let graus = grad.atan().to_degrees();

            slopes[z * g + x] = graus;
            soma_slope += graus;
            if graus > max_slope {
                max_slope = graus;
            }
        }
    }

    let metrics = DemMetrics {
        lat_centro: ds.lat_centro,
        lon_centro: ds.lon_centro,
        grid_size: ds.grid,
        min_elev,
        max_elev,
        media_elev,
        amplitude,
        rugosidade,
        declividade_media_graus: soma_slope / n,
        declividade_max_graus: max_slope,
        area_km2,
        volume_corte_m3,
        modo_visualizacao: modo.to_string(),
    };

    (metrics, slopes)
}

/// Constrói a malha 3D no modo "ELEVATION" (hipsometria) ou "SLOPE" (declividade)
pub fn construir_malha(
    ds: &DatasetAtivo,
    metrics: &DemMetrics,
    slopes: &[f32],
    modo_slope: bool,
) -> (Vec<ArcisVertex>, Vec<u32>) {
    let mut vertices = Vec::with_capacity((ds.grid * ds.grid) as usize);
    let mut indices = Vec::new();
    let range = (metrics.max_elev - metrics.min_elev).max(1.0);
    let max_slope = metrics.declividade_max_graus.max(1.0);

    for z in 0..ds.grid {
        for x in 0..ds.grid {
            let idx = (z * ds.grid + x) as usize;
            let norm_e = (ds.elevacoes[idx] - metrics.min_elev) / range;

            let pos_x = (x as f32 / (ds.grid - 1) as f32) * 1.6 - 0.8;
            let pos_z = (z as f32 / (ds.grid - 1) as f32) * 1.6 - 0.8;
            let pos_y = norm_e * 0.45 - 0.15;

            let cor = if modo_slope {
                // Mapa de risco/declividade: Verde (plano) -> Amarelo -> Vermelho (íngreme)
                let s = (slopes[idx] / max_slope).clamp(0.0, 1.0);
                [s, 1.0 - s * 0.8, 0.15]
            } else if norm_e < 0.33 {
                [0.1, 0.4 + norm_e, 0.5]
            } else if norm_e < 0.7 {
                [0.2 + norm_e * 0.8, 0.65, 0.2]
            } else {
                [0.85, 0.5 + norm_e * 0.4, 0.45 + norm_e * 0.4]
            };

            vertices.push(ArcisVertex {
                position: [pos_x, pos_y, pos_z],
                color: cor,
            });
        }
    }

    for z in 0..(ds.grid - 1) {
        for x in 0..(ds.grid - 1) {
            let i = z * ds.grid + x;
            indices.push(i);
            indices.push(i + ds.grid);
            indices.push(i + 1);

            indices.push(i + 1);
            indices.push(i + ds.grid);
            indices.push(i + ds.grid + 1);
        }
    }

    (vertices, indices)
}

/// Exporta Relatório Técnico (.md) e Matriz de Elevação (.csv) para a pasta exports/
pub fn exportar_relatorio(
    ds: &DatasetAtivo,
    metrics: &DemMetrics,
    slopes: &[f32],
) -> Result<String, std::io::Error> {
    let dir = Path::new("exports");
    fs::create_dir_all(dir)?;

    let base_name = ds.nome.replace(['/', ' ', '.'], "_");
    let md_path = dir.join(format!("relatorio_{}.md", base_name));
    let csv_path = dir.join(format!("dados_{}.csv", base_name));

    let md = format!(
        "# Relatório Técnico de Levantamento — Arcis STEM Engine\n\n\
        **Camada:** `{}`  \n\
        **Coordenadas Centrais:** `{:.5}, {:.5}` (SIRGAS 2000 / WGS84)  \n\
        **Grade Amostral:** `{} x {}` pontos  \n\n\
        ## Métricas Topográficas\n\
        | Indicador | Valor |\n\
        | :--- | ---: |\n\
        | Área Levantada | {:.3} km² |\n\
        | Cota Mínima | {:.2} m |\n\
        | Cota Máxima | {:.2} m |\n\
        | Cota Média | {:.2} m |\n\
        | Amplitude Altimétrica | {:.2} m |\n\
        | Rugosidade (Desvio Padrão) | {:.2} m |\n\
        | Declividade Média | {:.2}° |\n\
        | Declividade Máxima | {:.2}° |\n\
        | Volume Acima da Cota Base | {:.0} m³ |\n",
        ds.nome,
        metrics.lat_centro,
        metrics.lon_centro,
        metrics.grid_size,
        metrics.grid_size,
        metrics.area_km2,
        metrics.min_elev,
        metrics.max_elev,
        metrics.media_elev,
        metrics.amplitude,
        metrics.rugosidade,
        metrics.declividade_media_graus,
        metrics.declividade_max_graus,
        metrics.volume_corte_m3
    );
    fs::write(&md_path, md)?;

    let mut csv = String::from("grid_x,grid_z,elevacao_m,declividade_graus\n");
    for z in 0..ds.grid {
        for x in 0..ds.grid {
            let idx = (z * ds.grid + x) as usize;
            csv.push_str(&format!(
                "{},{},{:.2},{:.2}\n",
                x, z, ds.elevacoes[idx], slopes[idx]
            ));
        }
    }
    fs::write(&csv_path, csv)?;

    Ok(format!(
        "Relatório salvo em '{}' e '{}'",
        md_path.display(),
        csv_path.display()
    ))
}