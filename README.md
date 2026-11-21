# Arcis Core

Estação nativa de engenharia **GIS / CAD / STEM** de alta performance escrita em **Rust**, utilizando `eframe`/`egui` para interface desktop e `wgpu` para renderização 3D acelerada por GPU.

## Stack Tecnológica

- **Linguagem:** Rust
- **Interface & Design System:** `eframe` / `egui` (0.36.2) com layout inspirado no Xcode Dark (`src/theme.rs`, `src/ui/xcode_workspace.rs`)
- **Renderização 3D:** `wgpu` com shaders WGSL customizados e câmera orbital interativa (`src/render/wgpu_canvas.rs`)
- **Assincronismo & Rede:** `tokio`, `reqwest`, `serde`
- **APIs Geoespaciais:**
  - **Nominatim (OpenStreetMap):** geocodificação de endereços e municípios
  - **Open-Meteo Elevation API (Copernicus DEM 90m):** coleta de grades altimétricas reais

## Funcionalidades Implementadas

- **Workspace Estilo Xcode:** barra superior unificada com busca geoespacial, navegador lateral de camadas, console dividido (estado do kernel + terminal) e *Object Inspector* à direita.
- **Pipeline GIS Real:**
  - Busca por cidade/região (`GEO <local>`) ou coordenadas diretas (`DEM <lat> <lon>`).
  - Geração de malha 3D na GPU com controle de órbita (arraste), zoom (scroll) e opacidade.
  - Alternância em tempo real entre visualização **Hipsométrica (`ELEV`)** e **Mapa de Declividade (`SLOPE`)**.
- **Levantamento Topográfico (`src/workers/analysis.rs`):** cálculo de área ($km^2$), cota mínima/máxima/média, amplitude altimétrica, rugosidade (desvio padrão), declividade média/máxima (graus) e volume estimado de corte ($m^3$).
- **Exportação de Relatórios (`EXPORT`):** geração automática de relatório técnico em Markdown (`.md`) e matriz de dados em CSV (`.csv`) na pasta `exports/`.

## Em Desenvolvimento / Pendências

- **Mosaico de Satélite 2D (Esri World Imagery):** estrutura de tiles 3×3 e demarcação por clique implementadas, mas o carregamento da imagem no viewport está em depuração.
- **Módulos CAD e STEM:** estrutura de seleção pronta no shell; kernels dedicados pendentes.
- **Telemetria de Sistema:** substituição dos indicadores estáticos de CPU/RAM/VRAM no painel esquerdo por métricas reais.

## Como Executar

```bash
cargo run
Comandos do Console IntegradoComandoDescriçãoGEO <local>Geocodifica o local e baixa o DEM (ex: GEO Garanhuns, PE)DEM <lat> <lon>Baixa o DEM para coordenadas exatas (ex: DEM -8.1159 -35.0154)LOADCarrega a área padrão (Jaboatão dos Guararapes, PE)SLOPEAltera a malha 3D para o mapa de declividadeELEVRetorna a malha 3D para o modo hipsométricoEXPORTGera relatório .md e tabela .csv em exports/