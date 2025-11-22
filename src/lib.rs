use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

// --- データ構造の定義 ---

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Difficulty {
    pub name: String,
    pub level: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Opponent {
    pub name: String,
    pub source: String,
    pub difficulties: Vec<Difficulty>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TournamentSettings {
    pub player_team_level: u8,
    pub team_count: u8,
    pub level_tolerance: u8,
    pub allowed_sources: Vec<String>,
    pub unlocked_opponents: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Match {
    pub team1: String,
    pub team2: String,
    pub winner: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Tournament {
    pub rounds: Vec<Vec<Match>>,
    pub status: String,
}

// --- TODO: これから実装する関数 ---
// #[wasm_bindgen]
// pub fn generate_tournament(settings: JsValue) -> Result<JsValue, JsValue> {
//     // 1. settings を TournamentSettings に変換
//     // 2. 対戦相手をフィルタリング＆抽選
//     // 3. トーナメントの組み合わせを作成
//     // 4. Tournament オブジェクトを JsValue に変換して返す
// }
