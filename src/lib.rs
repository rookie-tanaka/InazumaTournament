use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};
use rand::seq::SliceRandom;
use rand::thread_rng;
use indexmap::IndexMap;
use std::collections::HashMap;
use rand::Rng;
use gloo_net::http::Request;


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
    pub id: String, // "チーム名 (シリーズ略称) - モード" の形式
    pub team_name: String, // 純粋なチーム名
    pub series_short: String, // シリーズ略称 (e.g., "VIC")
    pub series_full: String, // シリーズフルネーム
    pub source: String, // "ストーリー", "クロニクル", "対戦"
    pub difficulties: Vec<Difficulty>,
    pub level: u8, // 試合で実際に使われるレベル
    pub difficulty_name: String, // 試合で実際に使われる難易度名
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TournamentSettings {
    pub player_team_level: u8,
    pub team_count: u8,
    pub level_tolerance_lower: u8,
    pub level_tolerance_upper: u8,
    pub level_win_rate_modifier: u8,
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
    pub participants: HashMap<String, Opponent>,
    pub level_win_rate_modifier: u8,
    pub rounds: Vec<Vec<Match>>,
    pub bye_teams: Vec<String>,
    pub status: String,
}

// JavaScriptに返す対戦可能な相手の情報
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PlayableOpponentInfo {
    pub count: usize,
    pub opponents: Vec<String>,
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


async fn load_opponents_from_csv() -> Result<Vec<Opponent>, String> {
    // /docs/Teams.csv/
    let csv_text = Request::get("Teams.csv")
        .send()
        .await
        .map_err(|e| format!("Failed to fetch CSV: {}", e))?
        .text()
        .await
        .map_err(|e| format!("Failed to read response text: {}", e))?;

    let mut reader = csv::Reader::from_reader(csv_text.as_bytes());
    
    let mut opponents_map: IndexMap<String, Opponent> = IndexMap::new();

    for result in reader.deserialize::<CsvRecord>() {
        let record = result.map_err(|e| e.to_string())?;
        
        let unique_id = format!("{} ({}) - {}", record.team_name, record.series_short, record.mode);

        let opponent = opponents_map.entry(unique_id.clone()).or_insert_with(|| {
            let series_full = match record.series_short.as_str() {
                "IE1" => "イナズマイレブン",
                "IE2" => "イナズマイレブン2 脅威の侵略者",
                "IE3" => "イナズマイレブン3 世界への挑戦!!",
                "GO1" => "イナズマイレブンGO",
                "GO2" => "イナズマイレブンGO2 クロノ・ストーン",
                "GO3" => "イナズマイレブンGO3 ギャラクシー",
                "ALS" => "イナズマイレブン アレスの天秤",
                "ORI" => "イナズマイレブン オリオンの刻印",
                "VIC" => "イナズマイレブン 英雄たちのヴィクトリーロード",
                _ => "不明なシリーズ",
            }.to_string();

            Opponent {
                id: unique_id,
                team_name: record.team_name.clone(),
                series_short: record.series_short.clone(),
                series_full,
                source: record.mode.clone(),
                difficulties: Vec::new(),
                level: 0, // 初期値
                difficulty_name: String::new(), // 初期値
            }
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
                // difficulties が空でなければ、最初の難易度をデフォルトとして設定
                if let Some(first_difficulty) = opponent.difficulties.first() {
                    opponent.level = first_difficulty.level;
                    opponent.difficulty_name = first_difficulty.name.clone();
                }
            }
        }
    }
    
    Ok(opponents_map.into_values().collect())
}

// 指定された設定に基づいて、対戦可能な相手チームのリストを返すヘルパー関数
async fn get_eligible_opponents(
    settings: &TournamentSettings,
) -> Result<Vec<Opponent>, String> {
    let all_opponents = load_opponents_from_csv().await?;
    let mut potential_opponents: Vec<Opponent> = Vec::new();
    let min_player_level = settings.player_team_level.saturating_sub(settings.level_tolerance_lower);
    let max_player_level = settings.player_team_level.saturating_add(settings.level_tolerance_upper);

    for opponent in all_opponents.iter() {
        if settings.unlocked_opponents.contains(&opponent.id) && settings.allowed_sources.contains(&opponent.source) {
            let mut best_difficulty: Option<&Difficulty> = None;
            let mut min_diff_level = 255;

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
                let mut new_opponent = opponent.clone();
                new_opponent.level = diff.level;
                new_opponent.difficulty_name = diff.name.clone();
                potential_opponents.push(new_opponent);
            }
            
        }
    }
    Ok(potential_opponents)
}


#[wasm_bindgen]
pub async fn get_playable_opponents_info(settings_val: JsValue) -> Result<JsValue, JsValue> {
    let settings: TournamentSettings = serde_wasm_bindgen::from_value(settings_val)
        .map_err(|e| JsValue::from_str(&format!("Failed to deserialize settings: {}", e)))?;
    
    let eligible_opponents = get_eligible_opponents(&settings).await
        .map_err(|e| JsValue::from_str(&e))?;
    
    let formatted_opponents: Vec<String> = eligible_opponents
        .iter()
        .map(|o| format!("{} (Lv.{})", o.id, o.level))
        .collect();

    let info = PlayableOpponentInfo {
        count: formatted_opponents.len(),
        opponents: formatted_opponents,
    };

    serde_wasm_bindgen::to_value(&info)
        .map_err(|e| JsValue::from_str(&e.to_string()))
}


#[wasm_bindgen]
pub async fn generate_tournament(settings_val: JsValue) -> Result<JsValue, JsValue> {
    let settings: TournamentSettings = serde_wasm_bindgen::from_value(settings_val)
        .map_err(|e| JsValue::from_str(&format!("Failed to deserialize settings: {}", e)))?;

    let mut rng = thread_rng();

    let potential_opponents = get_eligible_opponents(&settings).await
        .map_err(|e| JsValue::from_str(&e))?;
    
    let num_opponents_to_select = (settings.team_count as i32 - 1).max(0) as usize;
    if potential_opponents.len() < num_opponents_to_select {
        return Err(JsValue::from_str(&format!("Not enough eligible opponents. Required: {}, Available: {}", num_opponents_to_select, potential_opponents.len())));
    }
    
    let selected_opponents: Vec<Opponent> = potential_opponents
        .choose_multiple(&mut rng, num_opponents_to_select)
        .cloned()
        .collect();
    
    let participants_map: HashMap<String, Opponent> = selected_opponents
        .into_iter()
        .map(|o| (o.id.clone(), o))
        .collect();

    let mut participant_names: Vec<String> = participants_map.keys().cloned().collect();
    participant_names.push("プレイヤー".to_string());
    participant_names.shuffle(&mut rng);

    let mut first_round_matches: Vec<Match> = Vec::new();
    let mut bye_teams: Vec<String> = Vec::new();
    let mut participants_iter = participant_names.into_iter();

    if participants_iter.len() % 2 != 0 {
        if let Some(team_name) = participants_iter.next() {
            bye_teams.push(team_name);
        }
    }

    while let (Some(team1), Some(team2)) = (participants_iter.next(), participants_iter.next()) {
        first_round_matches.push(Match { team1, team2, winner: None });
    }

    let tournament = Tournament {
        participants: participants_map,
        level_win_rate_modifier: settings.level_win_rate_modifier,
        rounds: vec![first_round_matches],
        bye_teams,
        status: "1回戦".to_string(),
    };

    serde_wasm_bindgen::to_value(&tournament).map_err(|e| JsValue::from_str(&format!("Failed to serialize tournament: {}", e)))
}

#[wasm_bindgen]
pub async fn update_match_result(
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
            if player_is_in_match && winner_name != "プレイヤー" {
                tournament.status = "ゲームオーバー".to_string();
                return serde_wasm_bindgen::to_value(&tournament).map_err(|e| JsValue::from_str(&e.to_string()));
            }
        } else {
            return Err(JsValue::from_str("Match index out of bounds."));
        }
    } else {
        return Err(JsValue::from_str("Round index out of bounds."));
    }

    let mut rng = thread_rng();
    let round_matches_clone = tournament.rounds[round_index].clone();
    for (i, m) in round_matches_clone.iter().enumerate() {
        if m.winner.is_none() && m.team1 != "プレイヤー" && m.team2 != "プレイヤー" {
            let team1 = tournament.participants.get(&m.team1).ok_or_else(|| JsValue::from_str("Team 1 not found"))?;
            let team2 = tournament.participants.get(&m.team2).ok_or_else(|| JsValue::from_str("Team 2 not found"))?;

            let level_diff = (team1.level as i16 - team2.level as i16).abs() as u8;
            let modifier = tournament.level_win_rate_modifier;
            let win_rate_bonus = (level_diff * modifier).min(50); 

            let team1_is_stronger = team1.level > team2.level;
            let stronger_team_win_rate = 0.5 + (win_rate_bonus as f64 / 100.0);

            let winner = if rng.gen_bool(stronger_team_win_rate) {
                if team1_is_stronger { &team1.id } else { &team2.id }
            } else {
                if team1_is_stronger { &team2.id } else { &team1.id }
            };
            
            if let Some(match_to_update) = tournament.rounds[round_index].get_mut(i) {
                match_to_update.winner = Some(winner.clone());
            }
        }
    }
    
    let current_round_finished = tournament.rounds[round_index].iter().all(|m| m.winner.is_some());

    if current_round_finished {
        let mut winners: Vec<String> = tournament.rounds[round_index]
            .iter()
            .filter_map(|m| m.winner.clone())
            .collect();
        
        winners.append(&mut tournament.bye_teams);
        tournament.bye_teams.clear();

        if winners.len() == 1 {
            tournament.status = format!("{} 優勝！", winners[0]);
        } else {
            winners.shuffle(&mut rng);
            let mut next_round_matches: Vec<Match> = Vec::new();
            let mut participants_iter = winners.into_iter();

            if participants_iter.len() % 2 != 0 {
                if let Some(team_name) = participants_iter.next() {
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
            } else if tournament.bye_teams.len() == 1 { // 決勝で片方が不戦勝の場合
                 tournament.status = format!("{} 優勝！", tournament.bye_teams[0]);
                 tournament.bye_teams.clear();
            }
        }
    }

    serde_wasm_bindgen::to_value(&tournament)
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

#[wasm_bindgen]
pub async fn get_all_opponents() -> Result<JsValue, JsValue> {
    let opponents = load_opponents_from_csv().await
        .map_err(|e| JsValue::from_str(&e))?;
    serde_wasm_bindgen::to_value(&opponents)
        .map_err(|e| JsValue::from_str(&e.to_string()))
}