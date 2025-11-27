# イナズマトーナメント 仕様書

これは、イナズマイレブンの対戦ツール「イナズマトーナメント」の技術的な仕様をまとめたドキュメントです。

## 概要

このアプリケーションは、プレイヤーが設定した条件に基づいて、CPUチームとのトーナメントを自動生成し、進行をシミュレートするWebアプリケーションです。バックエンドロジックはRust (WASM)で、フロントエンドはバニラJavaScriptで構築されています。

---

## バックエンド (`src/lib.rs`)

バックエンドはRustで書かれ、WebAssemblyにコンパイルされてフロントエンドから利用されます。データの永続化は行わず、`Teams.csv` から対戦相手の情報を読み込みます。

### 主要なデータ構造 (struct)

Rust側で定義されている主要なデータ構造です。これらは `serde` を介して、JavaScriptのオブジェクトや値と相互に変換されます。

- **`Opponent`**: 対戦相手チーム一意の情報を表します。
  - `name`: `チーム名 (シリーズ略称) - モード` の形式の文字列。
  - `source`: "ストーリー", "クロニクル", "対戦" のいずれか。
  - `difficulties`: そのチームが持つ難易度とレベルのリスト (`Vec<Difficulty>`)。
  - `level`: トーナメントで実際に使用されるレベル。
  - `difficulty_name`: トーナメントで実際に使用される難易度名。

- **`TournamentSettings`**: フロントエンドからトーナメント生成時に受け取る設定。
  - `player_team_level`: プレイヤーのチームレベル。
  - `team_count`: 参加チーム数 (4, 8, 16)。
  - `level_tolerance_lower`: 許容レベル範囲(下限)。
  - `level_tolerance_upper`: 許容レベル範囲(上限)。
  - `level_win_rate_modifier`: CPU戦での1レベル差あたりの勝率影響度 (%)。
  - `allowed_sources`: 許可された対戦モードのリスト (例: `["ストーリー", "対戦"]`)。
  - `unlocked_opponents`: プレイヤーが解放済みの対戦相手の `name` のリスト。

- **`Tournament`**: 生成されたトーナメント全体の情報を保持します。このオブジェクトがフロントとバックエンドでやり取りされ、状態を管理します。
  - `participants`: トーナメント参加チームの全情報 (`HashMap<String, Opponent>`)。キーはチームの`name`。
  - `level_win_rate_modifier`: 設定された勝率影響度。
  - `rounds`: 各ラウンドの試合リスト (`Vec<Vec<Match>>`)。
  - `bye_teams`: 不戦勝のチームリスト。
  - `status`: "1回戦", "ゲームオーバー", "〇〇 優勝！" などの現在の状態。

- **`Match`**: 一つの試合を表します。
  - `team1`: チーム1の名前。
  - `team2`: チーム2の名前。
  - `winner`: 勝者の名前。決まっていない場合は `None`。

### 公開されている関数 (`#[wasm_bindgen]`)

JavaScriptから呼び出すことができる関数です。

1.  **`get_all_opponents() -> Vec<Opponent>`**
    - `Teams.csv` からすべての対戦相手の情報を読み込み、`Opponent`オブジェクトの配列として返します。フロントエンドの「解放済み対戦相手」リストの初期表示に使用されます。

2.  **`get_playable_opponents_info(settings: TournamentSettings) -> PlayableOpponentInfo`**
    - `TournamentSettings` を受け取り、現在の設定で対戦可能なチームの数と、そのチームの詳細リスト (`チーム名 (Lv.XX)`) を返します。設定画面の「対戦可能なチーム数」の表示更新に使用されます。

3.  **`generate_tournament(settings: TournamentSettings) -> Tournament`**
    - `TournamentSettings` を受け取り、条件に合う対戦相手をランダムに選出し、シャッフルしてトーナメントを生成します。生成された `Tournament` オブジェクトを返します。

4.  **`update_match_result(tournament: Tournament, round_index: usize, match_index: usize, winner_name: String) -> Tournament`**
    - `async`関数です。
    - プレイヤーが選択した試合結果 (`winner_name`) をトーナメントに反映します。
    - その後、同じラウンドの残りのCPU戦をすべて自動でシミュレートします。
        - CPU戦の勝率は、`Tournament` に保存されている各チームのレベルと `level_win_rate_modifier` を基に計算されます。
    - ラウンドの全試合が終了した場合、次のラウンドを生成します。
    - 更新された `Tournament` オブジェクトを返します。

---

## フロントエンド (`www/main.js`)

フロントエンドは、ユーザーからの入力を受け取り、それを整形してRust（WASM）関数に渡し、返ってきた結果を元にUIを更新する役割を担います。

### 主要なグローバル変数

- `currentTournament`: Rustから返された `Tournament` オブジェクトを保持します。アプリケーションの状態そのものです。
- `allOpponents`: `get_all_opponents()` によって取得された、すべての対戦相手の情報の配列です。

### 主要な関数

- **`initializeApp()`**
  - アプリケーションの初期化を行います。
  - WASMモジュールを初期化し、`get_all_opponents()` を呼び出して全対戦相手リストを取得します。
  - 各種UI要素にイベントリスナーを設定します。

- **`populateOpponentList()`**
  - 右側の「解放済み対戦相手」のチェックボックスリストを生成・更新します。
  - 「モード」と「シリーズ」のフィルター設定を読み取り、`allOpponents` 配列をフィルタリングして表示内容を決定します。

- **`updatePlayableOpponentsCount()`**
  - 現在の各種設定値から `TournamentSettings` オブジェクトを構築します。
  - Rustの `get_playable_opponents_info()` を呼び出し、返ってきた値で「対戦可能なチーム数」の表示を更新します。

- **`handleGenerateTournament()`**
  - 「トーナメントを生成！」ボタンが押されたときに実行されます。
  - 現在の各種設定値から `TournamentSettings` オブジェクトを構築します。
  - Rustの `generate_tournament()` を呼び出し、返ってきた `Tournament` オブジェクトを `currentTournament` に保存します。
  - `renderTournament()` を呼び出してトーナメント画面を描画します。

- **`handleMatchClick(event)`**
  - トーナメント画面で試合の勝者選択ボタンが押されたときに実行されます。
  - `async`関数です。
  - 選択された勝者情報を元に、`await update_match_result()` を呼び出してトーナメントの状態を更新します。
  - Rust側がCPU戦のシミュレーションまで含めてすべて処理してくれるので、この関数は結果を受け取って `renderTournament()` を呼び出すだけです。

- **`renderTournament()`**
  - `currentTournament` オブジェクトの内容に基づいて、トーナメント表のHTMLを動的に生成・描画します。
  - 内部にヘルパー関数 `getTeamDisplayHTML(teamName)` を持ち、チーム名から `currentTournament.participants` (JavaScriptの `Map` オブジェクト) を参照して、レベルや難易度名を含む詳細な表示を生成します。

