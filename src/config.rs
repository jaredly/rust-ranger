use ron::de::from_reader;
use serde::Deserialize;
use std::fs::File;
use std::sync::Mutex;

pub static CONFIG_FILE: &'static str = "assets/config.ron";

#[derive(Debug, Deserialize, Clone, Copy)]
pub struct Config {
    pub screen_size: usize,
    pub zoom: f32,
    pub arrowhead_density: f32,
    pub throw_mul: f32,
    pub throw_max: f32,
    pub arrowhead_size: f32,
    pub fletching_torque: f32,
    pub fletching_max_torque: f32,
    pub fletching_min: f32,
    pub fletching_min_vel: f32,
    pub pickup_cooldown: f32,
    pub pickup_switch: f32,
    pub pickup_empty_angle: f32,
    pub show_colliders: bool,
    pub prevent_stuck: bool,
}

impl Default for Config {
    fn default() -> Self {
        read(CONFIG_FILE).unwrap()
    }
}

pub fn read(path: &str) -> ron::de::Result<Config> {
    let f = File::open(path).expect("Failed opening file");
    from_reader(f)
}

// fn get() -> &mut Config {
//     CONFIG.lock().unwrap().as_mut()
// }

pub fn with<R, F: FnOnce(&mut Config) -> R>(f: F) -> R {
    f(&mut CONFIG.lock().unwrap())
}

lazy_static! {
    static ref CONFIG: Mutex<Config> = Mutex::new(read(CONFIG_FILE).unwrap());
}

macro_rules! config {
    ($name: ident) => {
        crate::config::with(|config|config.$name)
    };
}
