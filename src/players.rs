use std::{sync::Mutex, time::{Instant, SystemTime}};
use egui::{emath::inverse_lerp, lerp, pos2};
use log::{info, warn};
use reqwest::Method;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerHeight {
    #[serde(alias = "display_name")]
    pub name: String,
    #[serde(alias = "user_id")]
    pub wsid: String,
    pub height: f32,
    pub color: Option<[f32; 3]>,
    pub rank: i64,
    pub pos: Option<[f32; 3]>,
    pub vel: Option<[f32; 3]>,
}

impl TryFrom<Value> for PlayerHeight {
    type Error = String;
    fn try_from(v: Value) -> Result<Self, Self::Error> {
        let name = v["display_name"].as_str().ok_or("Bad Name")?.to_string();
        let height = v["height"].as_f64().ok_or("Bad Height")? as f32;
        let color = v["color"].as_array().map(|a| {
            let mut out = [0.0; 3];
            for (i, c) in a.iter().enumerate() {
                out[i] = c.as_f64().unwrap_or(1.0) as f32;
            }
            out
        });
        let rank = v["rank"].as_i64().ok_or("bad rank")?;
        let pos = v["pos"].as_array().and_then(to_vec_3);
        let vel = v["vel"].as_array().and_then(to_vec_3);

        let wsid = v["user_id"].as_str().ok_or("Bad wsid")?.to_string();

        Ok(Self { name, height, color, rank, pos, vel, wsid })
    }
}

pub fn to_vec_3(v: &Vec<Value>) -> Option<[f32; 3]> {
    if v.len() != 3 {
        return None;
    }
    let mut out = [0.0; 3];
    for (i, c) in v.iter().enumerate() {
        out[i] = c.as_f64()? as f32;
    }
    Some(out)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UpdateType {
    Players,
    Render,
}

pub fn update_player_heights(out_heights: &Mutex<Vec<Vec<PlayerHeight>>>) {
    info!("Updating player heights");
    let resp = get_req("https://dips-plus-plus.xk.io/live_heights/global");
    info!("Got heights response: {:?}", resp);
    if let Ok(resp) = resp {
        let heights: Vec<Value> = resp.json().unwrap();
        let heights = heights.into_iter().filter_map(|h| h.try_into().ok()).collect();
        let mut out_heights = out_heights.lock().unwrap();
        out_heights.insert(0, heights);
        while out_heights.len() > 3 {
            out_heights.pop();
        }
    }
}

pub fn update_leaderboard(out_leaderboard: &Mutex<Vec<Vec<PlayerHeight>>>) {
    info!("Updating leaderboard");
    let resp = get_req("https://dips-plus-plus.xk.io/leaderboard/global");
    info!("Got lb response: {:?}", resp);
    if let Ok(resp) = resp {
        let leaderboard: Vec<PlayerHeight> = resp.json().unwrap();
        let mut out_leaderboard = out_leaderboard.lock().unwrap();
        out_leaderboard.insert(0, leaderboard); // .into_iter().take(20).collect()
        while out_leaderboard.len() > 3 {
            out_leaderboard.pop();
        }
    }
}

pub fn get_req(url: &str) -> Result<reqwest::blocking::Response, reqwest::Error> {
    reqwest::blocking::Client::new().request(Method::GET, url).header("User-Agent", "dd2-height-alarm").send()
}







#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerState {
    pub name: String,
    pub wsid: String,
    pub height: f32,
    pub floor: i32,
    pub last_alarm_time: Option<SystemTime>,
    pub last_alarm_floor: Option<i32>,
    pub last_floor: i32,
    pub last_height: f32,
    pub has_fired_alarm: bool
}

#[derive(Debug)]
pub enum AlarmState {
    NoAlarm,
    Alarm(i32),
}

impl PlayerState {
    pub fn new(p: &PlayerDeets) -> Self {
        Self {
            name: p.name.clone(),
            wsid: p.wsid.clone(),
            height: p.live_height,
            floor: height_to_floor(p.live_height),
            last_alarm_time: None,
            last_alarm_floor: None,
            last_floor: 0,
            last_height: p.live_height,
            has_fired_alarm: false,
        }
    }

    pub fn update(&mut self, p: &PlayerDeets) -> AlarmState {
        self.last_floor = self.floor;
        self.last_height = self.height;
        let next_floor = height_to_floor(p.live_height);
        // alarm condition: new floor, higher than last
        // let new_floor = next_floor != self.floor;
        let better_floor = next_floor > self.floor;
        let better_floor = next_floor > self.last_alarm_floor.unwrap_or_default() && better_floor;
        let worse_floor = next_floor < self.floor;
        self.floor = next_floor;
        self.height = p.live_height;
        if worse_floor && next_floor < self.last_alarm_floor.unwrap_or_default() - 2 {
            eprintln!("{} reset: {} -> {}", &p.name, self.last_alarm_floor.unwrap_or_default(), next_floor);
            self.reset_alarm();
            return AlarmState::NoAlarm;
        } else if better_floor {
            eprintln!("{} better: {} -> {}", &p.name, self.last_alarm_floor.unwrap_or_default(), next_floor);
            self.last_alarm_time = Some(SystemTime::now());
            self.last_alarm_floor = Some(next_floor);
            self.has_fired_alarm = false;
            return AlarmState::Alarm(next_floor);
        }
        AlarmState::NoAlarm
    }

    fn reset_alarm(&mut self) {
        self.last_alarm_time = None;
        self.last_alarm_floor = None;
        self.has_fired_alarm = false;
    }
}




#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlayerDeets {
    pub name: String,
    pub wsid: String,
    pub rank: i64,
    pub last_rank: Option<i64>,
    pub live_height: f32,
    pub last_height: Option<f32>,
    pub pb_height: f32,
    pub pb_rank: i64,
    pub color: egui::Rgba,
    pub pos: Option<[f32; 3]>,
    pub vel: Option<[f32; 3]>,
    pub last_pos: Option<[f32; 3]>,
    pub last_vel: Option<[f32; 3]>,
    pub t: f32,
}

impl PlayerDeets {
    pub fn get_x_t(&self) -> f32 {
        inverse_lerp(500.0..=1030.0, self.get_pos()[0]).unwrap_or(0.5)
    }

    pub fn get_pos(&self) -> [f32; 3] {
        let p1 = self.pos.unwrap_or([768.0, self.live_height, 768.0]);
        let p2 = self.last_pos.unwrap_or([768.0, self.last_height.unwrap_or(self.live_height), 768.0]);
        let t = simple_easing::quad_in_out(self.t);
        [
            lerp(p2[0]..=p1[0], t),
            lerp(p2[1]..=p1[1], t),
            lerp(p2[2]..=p1[2], t),
        ]
    }
}





pub fn height_to_pos2(height: f32, rect: egui::Rect) -> egui::Pos2 {
    let height = height / DD2_FLOOR_HEIGHTS[18];
    let y = rect.top() + (1.0 - height) * rect.height();
    pos2(rect.center().x, y)
}

pub fn height_to_floor(height: f32) -> i32 {
    let mut floor = 0;
    for (i, h) in DD2_FLOOR_HEIGHTS.iter().enumerate() {
        if height < *h {
            floor = (i as i32) - 1;
            break;
        }
    }
    floor
}


pub const DD2_FLOOR_HEIGHTS : [f32; 19] = [
    8.0, // Floor gang
    104.0, // 01
    208.0, // 02
    312.0, // 03
    416.0, // 04
    520.0, // 05
    624.0, // 06
    728.0, // 07
    832.0, // 08
    936.0, // 09
    1040.0, // 10
    1144.0, // 11
    1264.0, // 12 -- 48 -> 64
    1376.0, // 13 -- 52 -> 76
    1480.0, // 14 -- 56 -> 80
    1584.0, // 15 -- 60 -> 84
    1688.0, // 16 -- 64 -> 88
    1793.0, // 17 - end
    1910.0  // finish
];
