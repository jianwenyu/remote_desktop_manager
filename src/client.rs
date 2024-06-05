use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Client {
    pub name: String,
    pub ip: String,
    pub password: String,
}

pub enum AppMode {
    Normal,
    Adding,
    Editing,
    Removing,
    About,
}
