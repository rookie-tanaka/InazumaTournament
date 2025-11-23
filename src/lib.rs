use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};
use rand::seq::SliceRandom;
use rand::thread_rng;
use web_sys;
use std::collections::HashMap;

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
    pub name: String, // "チーム名 (シリーズ略称) - モード" の形式
    pub source: String,
    pub difficulties: Vec<Difficulty>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TournamentSettings {
    pub player_team_level: u8,
    pub team_count: u8,
    pub level_tolerance: u8,
    pub allowed_sources: Vec<String>,
    pub unlocked_opponents: Vec<String>, // "チーム名 (シリーズ略称) - モード" の形式
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
    pub bye_teams: Vec<String>,
    pub status: String,
}

// JavaScriptに返す対戦可能な相手の情報
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PlayableOpponentInfo {
    pub count: usize,
    pub opponents: Vec<String>, // "チーム名 (難易度名)" の形式
}


// --- CSVデータ処理 ---

#[derive(Debug, Deserialize)]
struct CsvRecord {
    #[serde(rename = "チーム名")]
    team_name: String,
    #[serde(rename = "シリーズ略称")]
    series_short: String,
    #[serde(rename = "モード")]
    mode: String,
    #[serde(rename = "難易度1")]
    d1_name: String,
    #[serde(rename = "難易度1レベル")]
    d1_level: String,
    #[serde(rename = "難易度2")]
    d2_name: String,
    #[serde(rename = "難易度2レベル")]
    d2_level: String,
    #[serde(rename = "難易度3")]
    d3_name: String,
    #[serde(rename = "難易度3レベル")]
    d3_level: String,
    #[serde(rename = "難易度4")]
    d4_name: String,
    #[serde(rename = "難易度4レベル")]
    d4_level: String,
}

fn load_opponents_from_csv() -> Result<Vec<Opponent>, String> {
    const CSV_DATA: &str = include_str!("../Teams.csv");
    let mut reader = csv::Reader::from_reader(CSV_DATA.as_bytes());
    
    // ユニークな名前をキーとする
    let mut opponents_map: HashMap<String, Opponent> = HashMap::new();

    for result in reader.deserialize::<CsvRecord>() {
        let record = result.map_err(|e| e.to_string())?;
        
        // チーム名 (シリーズ略称) - モード の形式でユニークな名前を生成
        let unique_name = format!("{} ({}) - {}", record.team_name, record.series_short, record.mode);

        let opponent = opponents_map.entry(unique_name.clone()).or_insert_with(|| Opponent {
            name: unique_name,
            source: record.mode.clone(),
            difficulties: Vec::new(),
        });
        
        let difficulties_data = [
            (&record.d1_name, &record.d1_level),
            (&record.d2_name, &record.d2_level),
            (&record.d3_name, &record.d3_level),
            (&record.d4_name, &record.d4_level),
        ];

        for (name, level_str) in difficulties_data.iter() {
            if !name.is_empty() && !level_str.is_empty() {
                if let Ok(level) = level_str.trim().parse::<u8>() {
                    opponent.difficulties.push(Difficulty {
                        name: name.trim().to_string(),
                        level,
                    });
                }
            }
        }
    }
    
    Ok(opponents_map.into_values().collect())
}

// 指定された設定に基づいて、対戦可能な相手チームと選択された難易度名のリストを返すヘルパー関数
fn get_eligible_opponents(
    settings: &TournamentSettings,
) -> Result<Vec<(String, String)>, String> { // (ユニークなチーム名, 難易度名)
    let all_opponents = load_opponents_from_csv()?;
    let mut potential_opponents: Vec<(String, String)> = Vec::new();
    let min_player_level = settings.player_team_level.saturating_sub(settings.level_tolerance);
    let max_player_level = settings.player_team_level.saturating_add(settings.level_tolerance);

    for opponent in all_opponents.iter() {
        if settings.unlocked_opponents.contains(&opponent.name) && settings.allowed_sources.contains(&opponent.source) {
            let mut best_difficulty: Option<&Difficulty> = None;
            let mut min_diff_level = 255; // レベル差の最小値

            for difficulty in opponent.difficulties.iter() {
                if difficulty.level >= min_player_level && difficulty.level <= max_player_level {
                    let current_diff_level = (settings.player_team_level as i16 - difficulty.level as i16).abs();
                    if current_diff_level < min_diff_level {
                        min_diff_level = current_diff_level;
                        best_difficulty = Some(difficulty);
                    }
                }
            }

            if let Some(diff) = best_difficulty {
                potential_opponents.push((opponent.name.clone(), diff.name.clone()));
            } else {
                 web_sys::console::log_1(&format!("Opponent '{}' has no suitable difficulty within tolerance.", opponent.name).into());
            }
        }
    }
    Ok(potential_opponents)
}


#[wasm_bindgen]
pub fn get_playable_opponents_info(settings_val: JsValue) -> Result<JsValue, JsValue> {
    let settings: TournamentSettings = serde_wasm_bindgen::from_value(settings_val)
        .map_err(|e| JsValue::from_str(&format!("Failed to deserialize settings: {}", e)))?;
    
    let eligible_opponents = get_eligible_opponents(&settings)
        .map_err(|e| JsValue::from_str(&e))?;
    
    let formatted_opponents: Vec<String> = eligible_opponents
        .iter()
        .map(|(name, difficulty_name)| format!("{} ({})", name, difficulty_name))
        .collect();

    let info = PlayableOpponentInfo {
        count: formatted_opponents.len(),
        opponents: formatted_opponents,
    };

    serde_wasm_bindgen::to_value(&info)
        .map_err(|e| JsValue::from_str(&e.to_string()))
}


#[wasm_bindgen]
pub fn generate_tournament(settings_val: JsValue) -> Result<JsValue, JsValue> {
    web_sys::console::log_1(&"generate_tournament called".into());

    let settings: TournamentSettings = serde_wasm_bindgen::from_value(settings_val)
        .map_err(|e| JsValue::from_str(&format!("Failed to deserialize settings: {}", e)))?;

    web_sys::console::log_1(&format!("Received settings: {:?}", settings).into());

    let mut rng = thread_rng();

    let potential_opponents = get_eligible_opponents(&settings)
        .map_err(|e| JsValue::from_str(&e))?;
    
    web_sys::console::log_1(&format!("Potential opponents with selected difficulties: {:?}", potential_opponents).into());

    // 2. チーム数-1だけ、ランダムに抽選
    let num_opponents_to_select = (settings.team_count as i32 - 1).max(0) as usize;
    if potential_opponents.is_empty() {
        return Err(JsValue::from_str("No opponents found that match the criteria."));
    }
    // `potential_opponents`のタプルの最初の要素はすでにユニークな名前
    let selected_opponent_teams: Vec<String> = potential_opponents
        .choose_multiple(&mut rng, num_opponents_to_select)
        .map(|(unique_name, difficulty_name)| format!("{} ({})", unique_name, difficulty_name))
        .collect();
    
    let mut participants = selected_opponent_teams;
    participants.push("プレイヤー".to_string());
    participants.shuffle(&mut rng);

    web_sys::console::log_1(&format!("Selected participants for the tournament: {:?}", participants).into());

    // 3. 対戦表を作成 (1回戦のみ)
    let mut first_round_matches: Vec<Match> = Vec::new();
    let mut bye_teams: Vec<String> = Vec::new();
    let mut participants_iter = participants.into_iter();

    if participants_iter.len() % 2 != 0 {
        if let Some(team_name) = participants_iter.next() {
            web_sys::console::log_1(&format!("{} has a bye in the first round.", &team_name).into());
            bye_teams.push(team_name);
        }
    }

    while let (Some(team1), Some(team2)) = (participants_iter.next(), participants_iter.next()) {
        first_round_matches.push(Match { team1, team2, winner: None });
    }

    let tournament = Tournament {
        rounds: vec![first_round_matches],
        bye_teams,
        status: "1回戦".to_string(),
    };
    web_sys::console::log_1(&format!("Generated tournament structure: {:?}", tournament).into());

    serde_wasm_bindgen::to_value(&tournament).map_err(|e| JsValue::from_str(&format!("Failed to serialize tournament: {}", e)))
}

#[wasm_bindgen]
pub fn update_match_result(
    tournament_val: JsValue,
    round_index: usize,
    match_index: usize,
    winner_name: String,
) -> Result<JsValue, JsValue> {
    let mut tournament: Tournament = serde_wasm_bindgen::from_value(tournament_val)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    if let Some(round) = tournament.rounds.get_mut(round_index) {
        if let Some(match_to_update) = round.get_mut(match_index) {
            if match_to_update.winner.is_some() {
                return serde_wasm_bindgen::to_value(&tournament).map_err(|e| JsValue::from_str(&e.to_string()));
            }
            match_to_update.winner = Some(winner_name.clone());

            let player_is_in_match = match_to_update.team1 == "プレイヤー" || match_to_update.team2 == "プレイヤー";
            if player_is_in_match && match_to_update.winner.as_deref() != Some("プレイヤー") {
                tournament.status = "ゲームオーバー".to_string();
                web_sys::console::log_1(&"Game Over!".into());
                return serde_wasm_bindgen::to_value(&tournament).map_err(|e| JsValue::from_str(&e.to_string()));
            }
        } else {
            return Err(JsValue::from_str("Match index out of bounds."));
        }
    } else {
        return Err(JsValue::from_str("Round index out of bounds."));
    }

    let current_round_finished = tournament.rounds[round_index].iter().all(|m| m.winner.is_some());

    if current_round_finished {
        web_sys::console::log_1(&format!("Round {} finished.", round_index + 1).into());
        let mut rng = thread_rng();

        let mut winners: Vec<String> = tournament.rounds[round_index]
            .iter()
            .filter_map(|m| m.winner.clone())
            .collect();
        
        winners.append(&mut tournament.bye_teams);
        tournament.bye_teams.clear();

        // 優勝者決定
        if winners.len() == 1 {
            tournament.status = format!("{} 優勝！", winners[0]);
            web_sys::console::log_1(&"Tournament finished!".into());
            return serde_wasm_bindgen::to_value(&tournament).map_err(|e| JsValue::from_str(&e.to_string()));
        }

        // 次のラウンドの組み合わせを作成
        let mut next_round_matches: Vec<Match> = Vec::new();
        let mut participants_iter = winners.into_iter();

        if participants_iter.len() % 2 != 0 {
            if let Some(team_name) = participants_iter.next() {
                web_sys::console::log_1(&format!("{} has a bye in the next round.", &team_name).into());
                tournament.bye_teams.push(team_name);
            }
        }

        while let (Some(team1), Some(team2)) = (participants_iter.next(), participants_iter.next()) {
            next_round_matches.push(Match { team1, team2, winner: None });
        }

        if !next_round_matches.is_empty() {
            tournament.rounds.push(next_round_matches);
            let next_round_num = tournament.rounds.len();
            tournament.status = format!("{}回戦", next_round_num);
        }
    }

    serde_wasm_bindgen::to_value(&tournament)
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

#[wasm_bindgen]
pub fn get_all_opponents() -> Result<JsValue, JsValue> {
    let opponents = load_opponents_from_csv()
        .map_err(|e| JsValue::from_str(&e))?;
    serde_wasm_bindgen::to_value(&opponents)
        .map_err(|e| JsValue::from_str(&e.to_string()))
}