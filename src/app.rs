use std::{error::Error, f64::consts::E, sync::{Arc, Mutex}, time::Duration};

use cpal::{traits::{DeviceTrait, HostTrait}, SupportedStreamConfig};
use egui::{ahash::{HashMap, HashMapExt}, emath::inverse_lerp, lerp, pos2, vec2, Align2, Color32, FontId, Rect, Response, Rgba, Sense, TextStyle, Vec2, Visuals};
use serde::{Deserialize, Serialize};

use crate::{play_sound::{play_for_player_floor, test_play_sounds, SoundTheme}, players::*};

#[derive(Debug)]
pub struct HAppState {
    pub sound_devices: Result<Arc<Vec<(usize, String, SupportedStreamConfig)>>, Box<dyn Error>>,
    pub last_update_request: std::time::Instant,
    pub last_lb_request: std::time::Instant,
    pub rx_refresh: std::sync::mpsc::Receiver<UpdateType>,
    pub tx_refresh: std::sync::mpsc::Sender<UpdateType>,
}

impl HAppState {
    pub fn new() -> Self {
        let sound_devices = list_sound_devices().into();
        let last_update_request = std::time::Instant::now().checked_sub(Duration::from_secs(5)).unwrap_or(std::time::Instant::now());
        let last_lb_request = last_update_request;
        let (tx, rx) = std::sync::mpsc::channel();
        Self {
            sound_devices,
            last_update_request,
            last_lb_request,
            rx_refresh: rx,
            tx_refresh: tx,
        }
    }
}

impl Default for HAppState {
    fn default() -> Self {
        Self::new()
    }
}


#[derive(Debug, Serialize, Deserialize)]
pub struct HeightsApp {
    pub sound_dev_ix: usize,
    pub live_heights: Arc<Mutex<Vec<Vec<PlayerHeight>>>>,
    pub leaderboard: Arc<Mutex<Vec<Vec<PlayerHeight>>>>,
    #[serde(skip)]
    pub s: HAppState,
    pub players: Option<CurrPlayers>,
    pub show_settings: bool,
    pub player_states: HashMap<String, PlayerState>,
    pub floor_alarm_start: i32,
    pub persistent_alarm: bool,
}

impl Default for HeightsApp {
    fn default() -> Self {
        Self::new()
    }
}

lazy_static::lazy_static! {
    static ref BigMono: TextStyle = TextStyle::Name("bigmono".into());
}

impl HeightsApp {
    pub fn new() -> Self {
        let live_heights = Arc::new(Mutex::new(Vec::new()));
        let leaderboard = Arc::new(Mutex::new(Vec::new()));
        Self {
            sound_dev_ix: usize::MAX,
            live_heights,
            leaderboard,
            s: HAppState::default(),
            players: None,
            show_settings: false,
            player_states: HashMap::new(),
            floor_alarm_start: 11,
            persistent_alarm: true,
        }
    }

    fn check_update_heights(&mut self, ctx: &egui::Context) {
        if self.s.last_update_request.elapsed() > Duration::from_secs(5) {
            let ctx1 = ctx.clone();
            let lh = self.live_heights.clone();
            let refresh = self.s.tx_refresh.clone();
            std::thread::spawn(move || {
                update_player_heights(&lh);
                refresh.send(UpdateType::Players).unwrap();
                ctx1.request_repaint();
            });
            self.s.last_update_request = std::time::Instant::now();
        }
        if self.s.last_lb_request.elapsed() > Duration::from_millis(61_234) {
            let ctx1 = ctx.clone();
            let lb = self.leaderboard.clone();
            let refresh = self.s.tx_refresh.clone();
            std::thread::spawn(move || {
                update_leaderboard(&lb);
                // refresh.send(UpdateType::Players).unwrap();
                // ctx1.request_repaint();
            });
            self.s.last_lb_request = std::time::Instant::now();
        }
    }

    fn draw_sound_device_selector(&mut self, ui: &mut egui::Ui) {
        let selected = match &self.s.sound_devices {
            Ok(devices) => {
                if let Some((_, name, _)) = devices.get(self.sound_dev_ix) {
                    name.clone()
                } else if let Some((_, name, _)) = devices.get(0) {
                    name.clone()
                } else {
                    "None (No Devices?)".to_string()
                }
            },
            Err(e) => format!("Error: {}", e),
        };
        egui::ComboBox::from_label("Sound Device")
            .selected_text(selected).show_ui(ui, |ui| {
            if let Ok(devices) = &self.s.sound_devices {
                for (i, name, _) in devices.iter() {
                    let r = ui.selectable_value(&mut self.sound_dev_ix, *i, name);
                }
            }
        });
    }

    // draw test, settings, minimize, close button
    fn draw_buttons_top_right(&mut self, ui: &mut egui::Ui, cur: egui::Rect, area: egui::Vec2) {
        let size = area.x * 0.09;
        let pad = ui.spacing().item_spacing.y * 2.;
        let close_rect = egui::Rect::from_min_size(pos2(cur.max.x - (pad + size), cur.min.y + pad), vec2(size, size));
        let min_rect = egui::Rect::from_min_size(pos2(cur.max.x - (pad + size) * 2.0, cur.min.y + pad), vec2(size, size));
        let set_rect = egui::Rect::from_min_size(pos2(cur.max.x - (pad + size) * 3.0, cur.min.y + pad), vec2(size, size));
        let test_rect = egui::Rect::from_min_size(pos2(cur.max.x - (pad + size) * 4.0, cur.min.y + pad), vec2(size, size));
        let close_color = if close_rect.contains(ui.input(|i| i.pointer.hover_pos()).unwrap_or(pos2(-1000.0, -1000.0))) {
            Color32::from_rgb(255, 0, 128)
        } else {
            Color32::TRANSPARENT
        };
        let min_color = if min_rect.contains(ui.input(|i| i.pointer.hover_pos()).unwrap_or(pos2(-1000.0, -1000.0))) {
            Color32::from_rgb(255, 255, 128)
        } else {
            Color32::TRANSPARENT
        };
        let set_color = if set_rect.contains(ui.input(|i| i.pointer.hover_pos()).unwrap_or(pos2(-1000.0, -1000.0))) {
            Color32::from_rgb(0, 255, 128)
        } else {
            Color32::TRANSPARENT
        };
        let mut test_color = Color32::TRANSPARENT;
        if test_rect.contains(ui.input(|i| i.pointer.hover_pos()).unwrap_or(pos2(-1000.0, -1000.0))) {
            test_color = Color32::from_rgb(128, 128, 255);
        }

        let close_clicked = close_rect.contains(click_to_point_or_neg(ui));
        let min_clicked = min_rect.contains(click_to_point_or_neg(ui));
        let settings_clicked = set_rect.contains(click_to_point_or_neg(ui));
        let test_clicked = test_rect.contains(click_to_point_or_neg(ui));

        ui.painter().rect_stroke(close_rect, 0.0, (1.5, close_color));
        ui.painter().rect_stroke(min_rect, 0.0, (1.5, min_color));
        ui.painter().rect_stroke(set_rect, 0.0, (1.5, set_color));
        ui.painter().rect_stroke(test_rect, 0.0, (1.5, test_color));

        ui.painter().text(close_rect.center(), egui::Align2::CENTER_CENTER, "X", TextStyle::Heading.resolve(ui.style()), Color32::WHITE);
        ui.painter().text(min_rect.center(), egui::Align2::CENTER_CENTER, "-", TextStyle::Heading.resolve(ui.style()), Color32::WHITE);
        ui.painter().text(set_rect.center(), egui::Align2::CENTER_CENTER, "⚙", TextStyle::Heading.resolve(ui.style()), Color32::WHITE);
        ui.painter().text(test_rect.center(), egui::Align2::CENTER_CENTER, "📃", TextStyle::Heading.resolve(ui.style()), Color32::WHITE);

        if close_clicked {
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
        }
        if min_clicked {
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Minimized(true));
        }
        if settings_clicked {
            self.show_settings = !self.show_settings;
        }
        if test_clicked {
            std::thread::spawn(|| {
                test_play_sounds();
            });
        }
    }

    fn check_insert_fonts(&mut self, ctx: &egui::Context) {
        ctx.style_mut(|s| {
            if s.text_styles.contains_key(&BigMono) {
                return;
            }
            s.text_styles.insert(TextStyle::Name("bigmono".into()), FontId::new(16.0, eframe::epaint::FontFamily::Monospace));
        });
    }

    fn draw_settings(&mut self, ui: &mut egui::Ui) {
        ui.add(egui::Slider::new(&mut self.floor_alarm_start, 0..=16).text("Floor Alarm Start"));
        ui.checkbox(&mut self.persistent_alarm, "(TODO) Keep alarm going till you acknowledge");
    }
}

fn click_to_point_or_neg(ui: &egui::Ui) -> egui::Pos2 {
    if ui.input(|i| i.pointer.button_clicked(egui::PointerButton::Primary)) {
        ui.input(|i| i.pointer.interact_pos()).unwrap_or(pos2(-1000.0, -1000.0))
    } else {
        pos2(-1000.0, -1000.0)
    }
}

impl eframe::App for HeightsApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if !ctx.style().visuals.dark_mode {
            ctx.set_visuals(Visuals::dark());
        }
        self.check_update_heights(ctx);
        self.check_insert_fonts(ctx);
        // egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
        //     ui.horizontal(|ui| {
        //         self.draw_sound_device_selector(ui);
        //     });
        // });
        let r = egui::CentralPanel::default().show(ctx, |ui: &mut egui::Ui| {
            let cur = ui.cursor();
            let size = ui.available_size();
            if !self.show_settings {
                let lh = self.live_heights.lock().unwrap();
                let lb = self.leaderboard.lock().unwrap();
                let update_alarm = self.players.is_none();
                let players = self.players.get_or_insert_with(|| CurrPlayers::new(&lh, &lb));
                let mut alarm_player = None;
                if update_alarm {
                    players.live.iter().for_each(|pd| {
                        if !self.player_states.contains_key(&pd.wsid) {
                            self.player_states.insert(pd.wsid.clone(), PlayerState::new(pd));
                        }
                        let alarm = self.player_states.get_mut(&pd.wsid).unwrap().update(pd);
                        if matches!(alarm, AlarmState::Alarm(i) if i >= self.floor_alarm_start) {
                            eprintln!("⚠️⚠️⚠️⚠️ Alarm: {:?} ⚠️⚠️⚠️⚠️", alarm);
                            alarm_player = Some(pd.clone());
                        }
                    });
                }
                if let Some(p) = alarm_player {
                    std::thread::spawn(move || {
                        play_for_player_floor(SoundTheme::Announcer, &p.wsid, height_to_floor(p.live_height) as i64);
                    });
                }
                players.draw_live_heights(ui);
            } else {
                ui.allocate_rect(Rect::from_min_max(pos2(0.0, 0.0), pos2(300.0, 50.0)), Sense::hover());
                self.draw_settings(ui);
            }

            self.draw_buttons_top_right(ui, cur, size);

        });

        // ctx.request_repaint();

        if let Ok(ty) = self.s.rx_refresh.try_recv() {
            // eprintln!("Got refresh: {:?}", ty);
            ctx.request_repaint();
            match ty {
                UpdateType::Players => {
                    self.players = None;
                },
                UpdateType::Render => {},
            }
        } else {
            ctx.request_repaint_after(Duration::from_millis(100));
        }

        let r = r.response.interact(Sense::drag());
        if ctx.is_being_dragged(r.id) {
            ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
        }
    }
}


#[derive(Debug, Serialize, Deserialize)]
pub struct CurrPlayers {
    pub live: Vec<PlayerDeets>,
    pub fade_player_hover: f32,
}

pub struct PlayerAux {
    pub last_live: Option<PlayerHeight>,
    pub pb: Option<PlayerHeight>,
    pub last_pb: Option<PlayerHeight>,
}

impl CurrPlayers {
    pub fn new(lhs: &Vec<Vec<PlayerHeight>>, lbs: &Vec<Vec<PlayerHeight>>) -> Self {
        let mut live: Vec<PlayerDeets> = Vec::new();
        let empty = vec![];
        let lh1 = lhs.get(0).unwrap_or(&empty);
        let last_lh = lhs.get(1).unwrap_or(&empty);
        let lb1 = lbs.get(0).unwrap_or(&empty);
        let last_lb = lbs.get(1).unwrap_or(&empty);

        for ph in lh1.iter() {
            let aux = find_player_aux(ph, last_lh, lb1, last_lb);
            live.push(PlayerDeets {
                name: ph.name.clone(),
                wsid: ph.wsid.clone(),
                rank: ph.rank,
                last_rank: aux.last_live.as_ref().map(|lh| lh.rank),
                live_height: ph.height,
                last_height: aux.last_live.as_ref().map(|lh| lh.height),
                pb_height: aux.pb.as_ref().map(|pb| pb.height).unwrap_or(0.0),
                pb_rank: aux.pb.as_ref().map(|pb| pb.rank).unwrap_or(0),
                color: ph.color.map(|c| Rgba::from_rgb(c[0], c[1], c[2])).unwrap_or_else(|| Rgba::from_rgb(1.0, 1.0, 1.0)),
                pos: ph.pos,
                vel: ph.vel,
                last_pos: aux.last_live.as_ref().and_then(|lh| lh.pos),
                last_vel: aux.last_live.as_ref().and_then(|lh| lh.vel),
                t: 0.0,
            });
        }
        Self { live, fade_player_hover: 0.0 }
    }

    pub fn draw_live_heights(&mut self, ui: &mut egui::Ui) {
        let size = ui.available_size();
        let pad = size.y * 0.04;
        let cursor = ui.cursor().min;
        let tl = cursor + vec2(pad, pad * 2.0);
        let br = cursor + vec2(size.x - pad, size.y - pad);
        let rect = egui::Rect::from_two_pos(tl, br);

        self.draw_bottom_left(ui, rect);
        // finish at top, floor gang at bottom
        self.draw_floor_labels(ui, rect);
        let hovered = self.draw_players(ui, rect);
        self.draw_players_hovered(ui, hovered, rect);
        self.update_players(ui);
    }

    fn draw_players_hovered(&mut self, ui: &mut egui::Ui, hovered: Vec<PlayerDeets>, mut rect: egui::Rect) {
        if hovered.is_empty() {
            return;
        }
        rect = rect.with_max_x(self.get_x_min_max().y);
        let font_id = BigMono.resolve(ui.style());
        // let _ = ui.style_mut().override_font_id.insert(font_id);
        // let dt = ui.input(|i| i.stable_dt);
        // match hovered.len() {
        //     0 => self.fade_player_hover = (self.fade_player_hover - dt * 2.0).max(0.0),
        //     _ => self.fade_player_hover = (self.fade_player_hover + dt * 2.0).min(1.0),
        // };
        let avg_height = hovered.iter().map(|pd| pd.live_height).sum::<f32>() / hovered.len() as f32;
        let draw_lower_half = avg_height > DD2_FLOOR_HEIGHTS[9];
        let rects = rect.split_top_bottom_at_fraction(0.5);
        let rect = match draw_lower_half {
            true => rects.1,
            false => rects.0,
        };
        let fill_color = Color32::BLACK.gamma_multiply(0.75);
        let stroke_col = ui.style().visuals.text_color();
        ui.painter().rect(rect, 3.0, fill_color, (1.0, stroke_col));
        let mut pos = rect.left_top() + vec2(10.0, 10.0);

        for pd in hovered.iter().rev().take(5) {
            let next_bounds = self.draw_player_info(ui, pos, pd, font_id.clone());
            pos.y += next_bounds.height() + 10.0;
        }

        ui.style_mut().override_font_id = None;
    }

    fn draw_player_info(&self, ui: &egui::Ui, ui_pos: egui::Pos2, pd: &PlayerDeets, font_id: FontId) -> Rect {
        let pos = pd.pos.unwrap_or_default();
        let label = format!("{} @ {:.1} m\n<{:.1}, {:.1}, {:.1}>\nPB: {:.2} m (#{})", pd.name, pd.live_height, pos[0], pos[1], pos[2], pd.pb_height, pd.pb_rank);
        let col = ui.style().visuals.text_color();
        ui.painter().text(ui_pos, Align2::LEFT_TOP, label, font_id, col)
    }

    fn draw_bottom_left(&self, ui: &mut egui::Ui, rect: egui::Rect) {
        let label = "← Start".to_string();
        let pos = rect.left_bottom() + vec2(-20.0, rect.height() * 0.03);
        let font_id = egui::TextStyle::Monospace.resolve(ui.style());
        ui.painter().text(pos, Align2::LEFT_TOP, label, font_id, ui.style().visuals.text_color());
    }

    fn update_players(&mut self, ui: &mut egui::Ui) {
        let dt = ui.input(|i| i.stable_dt);
        let is = self.get_inv_speed();
        for pd in self.live.iter_mut() {
            if pd.t < 1.0 {
                ui.ctx().request_repaint();
            }
            pd.t = (pd.t + dt * is).min(1.0);
        }
    }

    fn get_inv_speed(&self) -> f32 {
        0.5
    }

    fn draw_floor_labels(&self, ui: &mut egui::Ui, rect: egui::Rect) {
        // let mut y = rect.top();
        for (i, height) in DD2_FLOOR_HEIGHTS.iter().enumerate() {
            let pos = height_to_pos2(*height, rect);
            let label = match i {
                0 => "FG".to_string(),
                17 => "End".to_string(),
                18 => "Fin".to_string(),
                _ => format!("{:02}", i),
            };
            self.draw_floor_tick(ui, pos);
            self.draw_floor_label(ui, pos2(30., pos.y), label, *height);
        }
    }

    fn get_x_min_max(&self) -> Vec2 {
        vec2(40., 290.)
    }

    fn draw_floor_tick(&self, ui: &mut egui::Ui, pos: egui::Pos2) {
        let xmm = self.get_x_min_max();
        let tl = pos2(xmm.x, pos.y - 1.0);
        let br = pos2(xmm.y, pos.y + 1.0);
        let rect = egui::Rect::from_two_pos(tl, br);
        ui.painter().rect(rect, 1.0, Color32::WHITE.gamma_multiply(0.25), (0.0, Color32::BLACK));
    }

    fn draw_floor_label(&self, ui: &mut egui::Ui, pos: egui::Pos2, label: String, height: f32) {
        let font_size = ui.spacing().item_spacing.y;
        let pos = pos2(pos.x, pos.y + font_size);
        let font_id = egui::TextStyle::Monospace.resolve(ui.style());
        let text_bounds = ui.fonts(|f| f.layout_no_wrap(label.clone(), font_id.clone(), Color32::WHITE).size());
        let mut tr = egui::Rect::from_min_size(pos - vec2(text_bounds.x + 5.0, text_bounds.y * 0.75 + 2.0), text_bounds + vec2(10.0, 4.0));
        let hovered = tr.contains(ui.input(|i| i.pointer.hover_pos()).unwrap_or(pos2(-1000.0, -1000.0)));
        if hovered {
            let h_label = format!("- {:.0} m", height);
            let h_text_bounds = ui.fonts(|f| f.layout_no_wrap(h_label.clone(), font_id.clone(), Color32::WHITE).size());
            tr.max.x += h_text_bounds.x + 5.0;
            ui.painter().rect(tr, 3.0, Color32::BLACK.gamma_multiply(0.45), (1.0, ui.style().visuals.text_color()));
            let tr = ui.painter().text(pos + vec2(5.0, font_size * -1.0), egui::Align2::LEFT_CENTER, h_label, font_id.clone(), ui.style().visuals.text_color());
        }
        let tr = ui.painter().text(pos - vec2(0.0, font_size * 1.0), egui::Align2::RIGHT_CENTER, label, font_id.clone(), ui.style().visuals.text_color());
    }

    fn draw_players(&mut self, ui: &mut egui::Ui, rect: egui::Rect) -> Vec<PlayerDeets> {
        let xmm: Vec2 = self.get_x_min_max();
        let mut hovered = vec![];
        for pd in self.live.iter().rev() {
            let x_t = pd.get_x_t();
            let mut pos = height_to_pos2(pd.get_pos()[1], rect);
            pos.x = lerp(xmm.x..=xmm.y, x_t);
            let color = pd.color;
            let radius = 5.0;
            ui.painter().circle(pos, radius, color.multiply(0.5), (1.0, Rgba::from_rgb(0.0, 0.0, 0.0)));
            let r = self.draw_player_label(ui, pos, x_t, pd);
            if r {
                hovered.push((*pd).clone());
            }
        }
        hovered
    }

    fn draw_player_label(&self, ui: &mut egui::Ui, pos: egui::Pos2, x_t: f32, pd: &PlayerDeets) -> bool {
        let left_else_right = x_t < 0.5;
        let lr_sign = if left_else_right { 1.0 } else { -1.0 };
        let font_size = ui.spacing().item_spacing.y;
        let font_id = egui::TextStyle::Monospace.resolve(ui.style());
        let mut label = format!("{} @ {:.1}", pd.name, pd.live_height);
        if let Some(last_height) = pd.last_height {
            let delta = pd.live_height - last_height;
            let delta_str = if delta > 0.0 {
                format!("+{:.1}", delta)
            } else {
                format!("{:.1}", delta)
            };
            label = format!("{}\n({})", label, delta_str);
        }
        let pos = pos2(pos.x + (7.0 * lr_sign), pos.y + font_size * 0.35);
        let align = if left_else_right { egui::Align2::LEFT_CENTER } else { egui::Align2::RIGHT_CENTER };
        // get size of text and draw bg
        let text_size = ui.fonts(|f| f.layout_no_wrap(label.clone(), font_id.clone(), Color32::WHITE).size());
        let bg_size = text_size + vec2(4.0, 0.0);
        let mut bg_pos = pos - vec2(2.0 * lr_sign, text_size.y * 0.6);
        if !left_else_right { bg_pos.x -= bg_size.x; }
        // let bg_rect = egui::Rect::from_min_size(bg_pos,  bg_size);
        let bg_color = Rgba::BLACK.multiply(0.5).into();
        // ui.painter().rect_filled(bg_rect, 0.0, bg_color);
        // ui.painter().text(pos, align, label.clone(), font_id.clone(), ui.style().visuals.text_color());

        draw_text_with_bg(ui, pos, label.clone(), font_id.clone(), bg_color, align, vec2(6.0, 4.0))
    }
}


fn draw_text_with_bg(ui: &mut egui::Ui, pos: egui::Pos2, text: String, font_id: FontId, bg_color: Color32, align: egui::Align2, pad: Vec2) -> bool {
    let text_size = ui.fonts(|f| f.layout_no_wrap(text.clone(), font_id.clone(), Color32::WHITE).size());
    // let bg_size = text_size + vec2(4.0, 0.0);
    let bg_size = text_size + pad;
    // let mut bg_pos = pos - vec2(2.0, text_size.y * 0.6);
    let mut bg_pos = pos - vec2(pad.x / 2.0, text_size.y * 0.56 + pad.y / 2.0);
    if align == egui::Align2::RIGHT_CENTER {
        // bg_pos.x -= bg_size.x - 2.0;
        bg_pos.x -= bg_size.x - pad.x;
    }
    let bg_rect = egui::Rect::from_min_size(bg_pos,  bg_size);
    let text_col = ui.style().visuals.text_color();
    ui.painter().rect(bg_rect, 3.0, bg_color, (1.0, text_col));
    ui.painter().text(pos, align, text.clone(), font_id, ui.style().visuals.text_color());
    let r = ui.input(|i| i.pointer.hover_pos()).map(|p| bg_rect.contains(p)).unwrap_or(false);
    r
}


fn avg_rgba(a: Rgba, b: Rgba) -> Rgba {
    let c1 = a.to_array();
    let c2 = b.to_array();
    let r = (c1[0] + c2[0]) / 2.0;
    let g = (c1[1] + c2[1]) / 2.0;
    let b = (c1[2] + c2[2]) / 2.0;
    let a = (c1[3] + c2[3]) / 2.0;
    Rgba::from_rgb(r, g, b).multiply(a)
}




fn find_player_aux(ph: &PlayerHeight, last_lh: &Vec<PlayerHeight>, lb1: &Vec<PlayerHeight>, last_lb: &Vec<PlayerHeight>) -> PlayerAux {
    let last_live = last_lh.iter().find(|pl| pl.name == ph.name).cloned();
    let pb = lb1.iter().find(|pl| pl.name == ph.name).cloned();
    let last_pb = last_lb.iter().find(|pl| pl.name == ph.name).cloned();
    PlayerAux { last_live, pb, last_pb }
}









fn list_sound_devices() -> Result<Arc<Vec<(usize, String, SupportedStreamConfig)>>, Box<dyn Error>>
{
    let mut ret = Vec::new();
    let host = cpal::default_host();
    let devices = host.output_devices()?;
    for (i, device) in devices.enumerate() {
        ret.push((i, device.name()?, device.default_output_config()?));
    }
    Ok(ret.into())
}
