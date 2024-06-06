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
            1 => vec![sounds::get_guy_has_reached_floor(), sounds::get_guy_one()],
            2 => vec![sounds::get_guy_has_reached_floor(), sounds::get_guy_two()],
            3 => vec![sounds::get_guy_has_reached_floor(), sounds::get_guy_three()],
            4 => vec![sounds::get_guy_has_reached_floor(), sounds::get_guy_four()],
            5 => vec![sounds::get_guy_has_reached_floor(), sounds::get_guy_five()],
            6 => vec![sounds::get_guy_has_reached_floor(), sounds::get_guy_six()],
            7 => vec![sounds::get_guy_has_reached_floor(), sounds::get_guy_seven()],
            8 => vec![sounds::get_guy_has_reached_floor(), sounds::get_guy_eight()],
            9 => vec![sounds::get_guy_has_reached_floor(), sounds::get_guy_nine()],
            10 => vec![sounds::get_guy_has_reached_floor(), sounds::get_guy_ten()],
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
        "f37147a8-36f3-4c58-9577-bf0faff3aafa" => sounds::get_guy_samifying(),
        "af30b7a1-fc37-485f-94bf-f00e39805d8c" => sounds::get_guy_ixxonn(),
        "076d23a5-51a6-48aa-8d99-9d618cd13c93" => sounds::get_guy_clover(),
        "c1e8bbec-8bb3-40b3-9b0e-52e3cb36015e" => sounds::get_guy_skandear(),
        "05477e79-25fd-48c2-84c7-e1621aa46517" => sounds::get_guy_granadyy(),
        "ed14ac85-1252-4cc7-8efd-49cd72938f9d" => sounds::get_guy_jxliano(),
        "fc54a67c-7bd3-4b33-aa7d-a77f13a7b621" => sounds::get_guy_mtat(),
        "a65ad790-180b-44cb-8643-7e1e87d15a83" => sounds::get_guy_gunafunkurmonky(),
        "f2220e91-6de5-41c4-b4c9-986cefee0c3f" => sounds::get_guy_bijanz(),
        "a4699c4c-e6c1-4005-86f6-55888f854e6f" => sounds::get_guy_talliebird(),
        "3ac715f3-8838-45de-a165-78db581ee084" => sounds::get_guy_warl(),
        "80b2989a-5a17-4d23-b134-46cce29571fa" => sounds::get_guy_dam(),
        "ce9e4eb6-be30-429c-9487-20ce620c2de8" => sounds::get_guy_jave(),
        "3433b0f2-5acd-47a8-b32c-8c811c984a9f" => sounds::get_guy_hugo(),
        "e07e9ea9-daa5-4496-9908-9680e35da02b" => sounds::get_guy_birdie(),
        "2f5735f4-04e5-4c9c-a60a-780648c02611" => sounds::get_guy_tres(),
        "908bfb12-b5e4-4342-ac68-4e61217375e2" => sounds::get_guy_tylerz(),
        "c1299945-2692-42dd-b28d-03c32dea8768" => sounds::get_guy_youronlyhope7(),
        "06496fad-70f7-49bc-80c6-d62caa7a9de4" => sounds::get_guy_hefest(),
        "b05db0f8-d845-47d2-b0e5-795717038ac6" => sounds::get_guy_massa(),
        "0c857beb-fd95-4449-a669-21fb310cacae" => sounds::get_guy_carljr(),
        "794a286c-44d9-4276-83ce-431cba7bab74" => sounds::get_guy_marius(),
        "011b6ce7-e49d-4fba-beeb-7650bbb26557" => sounds::get_guy_wally(),
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
