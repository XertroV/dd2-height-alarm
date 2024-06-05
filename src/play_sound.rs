use cpal::traits::{HostTrait, StreamTrait, DeviceTrait};
use rodio::OutputStream;
use rand::prelude::SliceRandom;

use crate::sounds;

pub enum SoundTheme {
    Announcer,
    CustomSound
}

pub fn play_for_player_floor(theme: SoundTheme, user: &str, floor: i64) {
    let mut sounds = match theme {
        SoundTheme::Announcer => match floor {
            11 => vec![sounds::get_guy_has_reached_floor(), sounds::get_guy_eleven()],
            12 => vec![sounds::get_guy_has_reached_floor(), sounds::get_guy_twelve()],
            13 => vec![sounds::get_guy_has_reached_floor(), sounds::get_guy_thirteen()],
            14 => vec![sounds::get_guy_has_reached_floor(), sounds::get_guy_fourteen()],
            15 => vec![sounds::get_guy_has_reached_floor(), sounds::get_guy_fifteen()],
            16 => vec![sounds::get_guy_has_reached_floor(), sounds::get_guy_sixteen()],
            17 => vec![sounds::get_guy_is_approaching_the_end()],
            _ if floor < 11 => vec![sounds::get_guy_has_reached_floor()],
            _ => vec![sounds::get_guy_alarm_alarm()],
        },
        _ => vec![sounds::get_guy_alarm_alarm()],
    };
    sounds.insert(0, get_player_name(user));
    sounds.insert(0, random_alarm());
    play_sounds(sounds);
}

pub fn get_player_name(wsid: &str) -> &'static [u8] {
    match wsid {
        "5d6b14db-4d41-47a4-93e2-36a3bf229f9b" => sounds::get_guy_bren(),
        "d46fb45d-d422-47c9-9785-67270a311e25" => sounds::get_guy_elconn(),
        "e5a9863b-1844-4436-a8a8-cea583888f8b" => sounds::get_guy_hazard(),
        "e3ff2309-bc24-414a-b9f1-81954236c34b" => sounds::get_guy_lars(),
        "d320a237-1b0a-4069-af83-f2c09fbf042e" => sounds::get_guy_mudda(),
        "0fd26a9f-8f70-4f51-85e1-fe99a4ed6ffb" => sounds::get_guy_schmaniol(),
        "da4642f9-6acf-43fe-88b6-b120ff1308ba" => sounds::get_guy_scrapie(),
        "803695f6-8319-4b8e-8c28-44856834fe3b" => sounds::get_guy_simo(),
        _ => sounds::get_guy_someone(),
    }
}

pub fn play_sounds(sounds: Vec<&'static [u8]>) {
    let (_stream, stream_h) = OutputStream::try_default().unwrap();
    let sink = rodio::Sink::try_new(&stream_h).unwrap();
    for sound in sounds {
        sink.append(rodio::Decoder::new(std::io::Cursor::new(sound)).unwrap());
    }
    sink.set_volume(1.2);
    sink.play();
    sink.sleep_until_end();
}

pub fn test_play_sounds() {
    play_sounds(vec![random_alarm(), sounds::get_guy_bren(), sounds::get_guy_has_reached_floor(), sounds::get_guy_sixteen()]);
}

pub fn random_alarm() -> &'static [u8] {
    let alarm = *([
        1,
        2,
        3,
        0
    ].choose_mut(&mut rand::thread_rng()).unwrap());
    match alarm {
        1 => sounds::get_guy_alarm_12(),
        2 => sounds::get_guy_alarm_14(),
        3 => sounds::get_guy_alarm_16(),
        _ => sounds::get_guy_alarm_alarm(),
    }
}
