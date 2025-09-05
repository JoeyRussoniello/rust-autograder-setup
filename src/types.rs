use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct AutoTest {
    pub name: String,
    pub timeout: u64,
    pub points: u32,
}
