//! Arcis design system — inspirado no Xcode / macOS Dark (Human Interface Guidelines).
//!
//! Princípios aplicados:
//!  * Clareza: hierarquia por camadas de superfície, não por bordas pesadas.
//!  * Deferência: o viewport é o protagonista; a UI é discreta e densa.
//!  * Profundidade: superfícies mais claras = mais "próximas" (toolbar > sidebar > canvas).
//!  * Cor com propósito: o azul do sistema só indica seleção/ação; verde/laranja/etc. só status.

use eframe::egui::{
    self, Color32, CornerRadius, FontFamily, FontId, RichText, Stroke, TextStyle, Vec2,
};

// ───────────────────────── Superfícies (do fundo para a frente) ─────────────────────────
pub const BG_VIEWPORT: Color32 = Color32::from_rgb(22, 22, 24);   // canvas 3D / editor
pub const BG_CONSOLE: Color32 = Color32::from_rgb(24, 24, 26);    // debug area
pub const BG_INPUT: Color32 = Color32::from_rgb(26, 26, 28);      // campos de texto / pílula
pub const BG_SIDEBAR: Color32 = Color32::from_rgb(30, 30, 32);    // navigator / inspector
pub const BG_PANEL: Color32 = BG_SIDEBAR;
pub const BG_SUBBAR: Color32 = Color32::from_rgb(36, 36, 38);     // breadcrumbs / barra de controles
pub const BG_TOOLBAR: Color32 = Color32::from_rgb(40, 40, 43);    // toolbar superior (mais "próxima")
pub const BG_CONTROL: Color32 = Color32::from_rgb(44, 44, 46);    // botões em repouso
pub const BG_CONTROL_HOVER: Color32 = Color32::from_rgb(58, 58, 60);
pub const BG_CARD: Color32 = Color32::from_rgb(28, 28, 30);       // splash / popovers

// ───────────────────────── Linhas ─────────────────────────
pub const HAIRLINE: Color32 = Color32::from_rgb(46, 46, 50);      // divisórias 1px
pub const BORDER_SUBTLE: Color32 = HAIRLINE;
pub const BORDER_STRONG: Color32 = Color32::from_rgb(70, 70, 74); // hover / card

// ───────────────────────── Texto (Apple "label" levels) ─────────────────────────
pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(235, 235, 240);
pub const TEXT_MUTED: Color32 = Color32::from_rgb(152, 152, 157);  // secondaryLabel
pub const TEXT_TERTIARY: Color32 = Color32::from_rgb(99, 99, 102); // tertiaryLabel

// ───────────────────────── Cores de sistema (macOS Dark) ─────────────────────────
pub const ACCENT_BLUE: Color32 = Color32::from_rgb(10, 132, 255);
pub const SYS_GREEN: Color32 = Color32::from_rgb(48, 209, 88);
pub const SYS_ORANGE: Color32 = Color32::from_rgb(255, 159, 10);
pub const SYS_PURPLE: Color32 = Color32::from_rgb(191, 90, 242);
pub const SYS_RED: Color32 = Color32::from_rgb(255, 69, 58);
pub const SYS_YELLOW: Color32 = Color32::from_rgb(255, 214, 10);

// ───────────────────────── Medidas (grade de 2/4/8 px) ─────────────────────────
pub const TOOLBAR_H: f32 = 34.0;
pub const BREADCRUMB_H: f32 = 24.0;
pub const CONTROLS_H: f32 = 28.0;
pub const FILTER_H: f32 = 26.0;
pub const CONSOLE_H: f32 = 185.0;
pub const NAVIGATOR_W: f32 = 245.0;
pub const INSPECTOR_W: f32 = 265.0;
pub const PANEL_PAD: f32 = 10.0;

pub const RADIUS_SM: u8 = 4; // botões, campos
pub const RADIUS_MD: u8 = 6; // pílula, popovers
pub const RADIUS_LG: u8 = 10; // cards / janelas

// ───────────────────────── Tipografia ─────────────────────────
pub fn mono(size: f32) -> FontId {
    FontId::new(size, FontFamily::Monospace)
}

/// Cabeçalho de seção em caixa-alta, estilo "Navigator" do Xcode.
pub fn section_label(text: &str) -> RichText {
    RichText::new(text).small().strong().color(TEXT_MUTED)
}

/// Linha de 1px horizontal no topo/rodapé de uma região.
pub fn hairline_h(ui: &egui::Ui, rect: egui::Rect, y: f32) {
    ui.painter().hline(rect.x_range(), y, Stroke::new(1.0, HAIRLINE));
}

/// Linha de 1px vertical na lateral de uma região.
pub fn hairline_v(ui: &egui::Ui, rect: egui::Rect, x: f32) {
    ui.painter().vline(x, rect.y_range(), Stroke::new(1.0, HAIRLINE));
}

/// Moldura da "pílula" de status central da toolbar.
pub fn pill_frame() -> egui::Frame {
    egui::Frame::NONE
        .fill(BG_INPUT)
        .stroke(Stroke::new(1.0, HAIRLINE))
        .corner_radius(CornerRadius::same(RADIUS_MD))
        .inner_margin(egui::Margin::symmetric(12, 3))
}

// ───────────────────────── Tema egui ─────────────────────────
pub fn apply_apple_dark_theme(ctx: &egui::Context) {
    let mut style = (*ctx.style_of(egui::Theme::Dark)).clone();

    // 1. Escala tipográfica compacta (densidade de IDE)
    style.text_styles = [
        (TextStyle::Heading, FontId::new(13.0, FontFamily::Proportional)),
        (TextStyle::Body, FontId::new(12.0, FontFamily::Proportional)),
        (TextStyle::Monospace, FontId::new(11.5, FontFamily::Monospace)),
        (TextStyle::Button, FontId::new(12.0, FontFamily::Proportional)),
        (TextStyle::Small, FontId::new(10.5, FontFamily::Proportional)),
    ]
    .into();

    // 2. Espaçamento justo
    style.spacing.item_spacing = Vec2::new(6.0, 4.0);
    style.spacing.button_padding = Vec2::new(8.0, 3.0);
    style.spacing.indent = 14.0;
    style.spacing.interact_size.y = 20.0;
    style.spacing.slider_width = 120.0;
    style.spacing.scroll.bar_width = 6.0;
    style.spacing.scroll.floating = true; // scrollbars overlay, como no macOS

    // 3. Cores
    let mut v = egui::Visuals::dark();
    v.panel_fill = BG_PANEL;
    v.window_fill = BG_CARD;
    v.window_stroke = Stroke::new(1.0, BORDER_STRONG);
    v.window_corner_radius = CornerRadius::same(RADIUS_LG);
    v.menu_corner_radius = CornerRadius::same(RADIUS_MD);
    v.extreme_bg_color = BG_INPUT;
    v.faint_bg_color = Color32::from_rgb(35, 35, 37);
    v.hyperlink_color = ACCENT_BLUE;
    v.slider_trailing_fill = true;

    v.selection.bg_fill = ACCENT_BLUE.linear_multiply(0.35);
    v.selection.stroke = Stroke::new(1.0, ACCENT_BLUE);

    let r = CornerRadius::same(RADIUS_SM);

    v.widgets.noninteractive.bg_fill = BG_PANEL;
    v.widgets.noninteractive.weak_bg_fill = BG_PANEL;
    v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, HAIRLINE);
    v.widgets.noninteractive.fg_stroke = Stroke::new(1.0, TEXT_PRIMARY);
    v.widgets.noninteractive.corner_radius = r;

    // Repouso: botões "chapados", sem borda visível (estilo Xcode)
    v.widgets.inactive.bg_fill = BG_CONTROL;
    v.widgets.inactive.weak_bg_fill = Color32::TRANSPARENT;
    v.widgets.inactive.bg_stroke = Stroke::NONE;
    v.widgets.inactive.fg_stroke = Stroke::new(1.0, TEXT_PRIMARY);
    v.widgets.inactive.corner_radius = r;

    v.widgets.hovered.bg_fill = BG_CONTROL_HOVER;
    v.widgets.hovered.weak_bg_fill = BG_CONTROL_HOVER;
    v.widgets.hovered.bg_stroke = Stroke::new(1.0, BORDER_STRONG);
    v.widgets.hovered.fg_stroke = Stroke::new(1.0, Color32::WHITE);
    v.widgets.hovered.corner_radius = r;
    v.widgets.hovered.expansion = 0.0;

    v.widgets.active.bg_fill = ACCENT_BLUE;
    v.widgets.active.weak_bg_fill = ACCENT_BLUE;
    v.widgets.active.bg_stroke = Stroke::NONE;
    v.widgets.active.fg_stroke = Stroke::new(1.0, Color32::WHITE);
    v.widgets.active.corner_radius = r;
    v.widgets.active.expansion = 0.0;

    v.widgets.open.bg_fill = BG_CONTROL_HOVER;
    v.widgets.open.weak_bg_fill = BG_CONTROL_HOVER;
    v.widgets.open.corner_radius = r;

    style.visuals = v;

    // Aplica o tema escuro independente da preferência do sistema
    ctx.set_style_of(egui::Theme::Dark, style.clone());
    ctx.set_style_of(egui::Theme::Light, style);
}

// ───────────────────────── egui_dock ─────────────────────────
pub fn dock_style(egui_style: &egui::Style) -> egui_dock::Style {
    let mut style = egui_dock::Style::from_egui(egui_style);

    // Divisórias ultrafinas de 1px
    style.separator.width = 1.0;
    style.separator.color_idle = HAIRLINE;
    style.separator.color_hovered = ACCENT_BLUE;
    style.separator.color_dragged = ACCENT_BLUE;

    // Barra de abas discreta (jump bar do Xcode)
    style.tab_bar.bg_fill = BG_SUBBAR;
    style.tab_bar.height = 24.0;
    style.tab_bar.hline_color = HAIRLINE;

    // Corpo dos painéis sem bordas grossas
    style.tab.tab_body.bg_fill = BG_PANEL;
    style.tab.tab_body.stroke = Stroke::NONE;
    style.tab.active.bg_fill = BG_PANEL;
    style.tab.active.text_color = TEXT_PRIMARY;
    style.tab.focused.bg_fill = BG_PANEL;
    style.tab.focused.text_color = TEXT_PRIMARY;
    style.tab.inactive.bg_fill = BG_SUBBAR;
    style.tab.inactive.text_color = TEXT_MUTED;

    style
}