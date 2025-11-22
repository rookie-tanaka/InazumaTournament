# InazumaTournament 設計図

これは、RustとWebAssemblyで開発を進める上での設計図です。

## データ構造 (Rustのstruct)

アプリケーションで扱う主要なデータ構造を定義します。

- **`Opponent` (対戦相手チーム):**
  - `name: String`: チームの名前 (例: "木戸川清修")
  - `source: String`: どのモードのチームか ("ストーリー", "クロニクル", "対戦")
  - `difficulties: Vec<Difficulty>`: そのチームが持ってる難易度のリスト

- **`Difficulty` (難易度):**
  - `name: String`: 難易度名 (例: "チームレベル10", "チームレベル30")
  - `level: u8`: この難易度におけるチームレベル

- **`TournamentSettings` (トーナメントの設定):**
  - `player_team_level: u8`: プレイヤーのチームレベル
  - `team_count: u8`: トーナメントに参加するチームの数 (4, 8, 16など)
  - `level_tolerance: u8`: 抽選する相手のチームレベルの許容範囲 (例: ±5)
  - `allowed_sources: Vec<String>`: 抽選対象にするモードのリスト
  - `unlocked_opponents: Vec<String>`: 解放済みの対戦相手名のリスト

- **`Tournament` (トーナメント全体):**
  - `rounds: Vec<Vec<Match>>`: 各ラウンドの対戦の組み合わせ
  - `status: String`: トーナメントの状態 ("進行中", "優勝", "ゲームオーバー")

- **`Match` (試合):**
  - `team1: String`: チーム1の名前
  - `team2: String`: チーム2の名前
  - `winner: Option<String>`: 勝者の名前 (まだ決まっていなければNone)


## 関数 (JavaScriptから呼び出すRust関数)

Webフロントエンド (JavaScript) から呼び出すための、WebAssemblyとして公開する関数を定義します。

- **`setup(opponent_data_json: String)`:**
  - **役割:** アプリケーションの初期化時に一度だけ呼び出します。
  - **引数:** 全対戦相手のデータを含むJSON文字列。
  - **処理:** 対戦相手データをRust側で保持し、抽選可能な状態にします。

- **`generate_tournament(settings_json: String) -> String`:**
  - **役割:** ユーザーが設定を入力し、「トーナメント生成」ボタンを押したときに呼び出します。
  - **引数:** `TournamentSettings`に対応するJSON文字列。
  - **戻り値:** 生成された`Tournament`の状態を表すJSON文字列。

- **`update_match_result(tournament_json: String, match_id: u32, winner_name: String) -> String`:**
  - **役割:** 試合結果が入力されたときに呼び出します。
  - **引数:**
    - `tournament_json`: 現在の`Tournament`の状態を表すJSON文字列。
    - `match_id`: 結果を更新する試合のID。
    - `winner_name`: 勝ったチームの名前。
  - **戻り値:** 結果を反映した、更新後の`Tournament`の状態を表すJSON文字列。
