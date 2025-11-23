import init, { get_all_opponents, generate_tournament, update_match_result, get_playable_opponents_info } from "./pkg/InazumaTournament.js";

let currentTournament;
let allOpponents = [];

// DOM要素への参照
const playerLevelInput = document.getElementById('player-level');
const teamCountSelect = document.getElementById('team-count');
const levelToleranceInput = document.getElementById('level-tolerance');
const modeCheckboxes = document.querySelectorAll('input[name="modes"]');
const opponentListDiv = document.getElementById('opponent-list');
const generateBtn = document.getElementById('generate-btn');
const selectAllBtn = document.getElementById('select-all-opponents');
const deselectAllBtn = document.getElementById('deselect-all-opponents');
const tournamentStatusHeading = document.getElementById('tournament-status');
const tournamentBracketDiv = document.getElementById('tournament-bracket');
const playableOpponentsCountDisplay = document.querySelector('#playable-opponents-count span');


// --- 初期化処理 ---
async function initializeApp() {
    await init();
    
    try {
        allOpponents = get_all_opponents();
        populateOpponentList();
    } catch (e) {
        console.error("対戦相手リストの取得に失敗しました:", e);
        opponentListDiv.textContent = `エラー: ${e}`;
    }

    // イベントリスナーを設定
    generateBtn.addEventListener('click', handleGenerateTournament);
    selectAllBtn.addEventListener('click', () => {
        toggleAllOpponents(true);
        updatePlayableOpponentsCount(); // すべて選択後に更新
    });
    deselectAllBtn.addEventListener('click', () => {
        toggleAllOpponents(false);
        updatePlayableOpponentsCount(); // すべて解除後に更新
    });
    tournamentBracketDiv.addEventListener('click', handleMatchClick);

    // 設定変更時にチーム数を更新するリスナー
    playerLevelInput.addEventListener('change', updatePlayableOpponentsCount);
    teamCountSelect.addEventListener('change', updatePlayableOpponentsCount);
    levelToleranceInput.addEventListener('change', updatePlayableOpponentsCount);
    modeCheckboxes.forEach(cb => cb.addEventListener('change', updatePlayableOpponentsCount));
    opponentListDiv.addEventListener('change', updatePlayableOpponentsCount); // イベントデリゲーションで対応

    updatePlayableOpponentsCount(); // 初期表示
}

function populateOpponentList() {
    opponentListDiv.innerHTML = '';
    allOpponents
        .sort((a, b) => a.name.localeCompare(b.name, 'ja'))
        .forEach(opponent => {
            const div = document.createElement('div');
            const checkbox = document.createElement('input');
            checkbox.type = 'checkbox';
            checkbox.id = `opponent-${opponent.name.replace(/\s|\(|\)/g, '_')}`;
            checkbox.value = opponent.name;
            checkbox.checked = true;

            const label = document.createElement('label');
            label.htmlFor = `opponent-${opponent.name.replace(/\s|\(|\)/g, '_')}`;
            label.textContent = opponent.name;

            div.appendChild(checkbox);
            div.appendChild(label);
            opponentListDiv.appendChild(div);
        });
}

function toggleAllOpponents(checked) {
    document.querySelectorAll('#opponent-list input[type="checkbox"]').forEach(cb => cb.checked = checked);
}


// --- 対戦可能なチーム数を更新し表示する ---
function updatePlayableOpponentsCount() {
    const settings = {
        player_team_level: parseInt(playerLevelInput.value, 10),
        team_count: parseInt(teamCountSelect.value, 10), // この値自体はRust側で使われないが、型定義に合わせる
        level_tolerance: parseInt(levelToleranceInput.value, 10),
        allowed_sources: Array.from(modeCheckboxes).filter(cb => cb.checked).map(cb => cb.value),
        unlocked_opponents: Array.from(document.querySelectorAll('#opponent-list input:checked')).map(cb => cb.value),
    };

    try {
        const info = get_playable_opponents_info(settings);
        playableOpponentsCountDisplay.textContent = info.count;
        if (info.count < parseInt(teamCountSelect.value, 10) -1) { // プレイヤーチームを除く
            playableOpponentsCountDisplay.style.color = 'red';
            // generateBtn.disabled = true; // 有効な対戦相手が足りない場合は生成ボタンを無効化しても良い
        } else {
            playableOpponentsCountDisplay.style.color = 'inherit';
            // generateBtn.disabled = false;
        }

    } catch (e) {
        console.error("対戦可能なチーム数の取得に失敗しました:", e);
        playableOpponentsCountDisplay.textContent = `エラー`;
        playableOpponentsCountDisplay.style.color = 'red';
    }
}


// --- トーナメント生成 ---
function handleGenerateTournament() {
    const settings = {
        player_team_level: parseInt(playerLevelInput.value, 10),
        team_count: parseInt(teamCountSelect.value, 10),
        level_tolerance: parseInt(levelToleranceInput.value, 10),
        allowed_sources: Array.from(modeCheckboxes).filter(cb => cb.checked).map(cb => cb.value),
        unlocked_opponents: Array.from(document.querySelectorAll('#opponent-list input:checked')).map(cb => cb.value),
    };

    // トーナメント生成前に、対戦可能なチーム数が足りているかチェック
    const requiredOpponents = settings.team_count -1; // プレイヤーチームを除く
    const playableInfo = get_playable_opponents_info(settings);
    if (playableInfo.count < requiredOpponents) {
        alert(`対戦可能なチームが足りません。${requiredOpponents}チーム必要ですが、${playableInfo.count}チームしかいません。設定を見直してください。`);
        return;
    }


    try {
        currentTournament = generate_tournament(settings);
        renderTournament();
    } catch (e) {
        console.error("トーナメントの生成に失敗しました:", e);
        tournamentBracketDiv.innerHTML = `<p style="color: red;">エラー: ${e}</p>`;
        tournamentStatusHeading.textContent = "エラー";
    }
}

// --- 試合結果の処理 ---
async function handleMatchClick(event) {
    const target = event.target;
    if (!target.classList.contains('winner-button')) return;

    // プレイヤーの試合結果をまず更新
    const roundIndex = parseInt(target.dataset.roundIndex, 10);
    const matchIndex = parseInt(target.dataset.matchIndex, 10);
    const winnerName = target.dataset.winnerName;

    try {
        currentTournament = update_match_result(currentTournament, roundIndex, matchIndex, winnerName);
        renderTournament(); // プレイヤーの試合結果を即座にUIに反映

        // もしゲームオーバーならここで終了
        if (currentTournament.status === "ゲームオーバー") return;

        // --- 残りのCPU戦を遅延付きで自動進行 ---
        await simulateRestOfRound(roundIndex);

    } catch (e) {
        console.error("試合結果の更新中にエラー:", e);
        alert(`試合結果の更新中にエラーが発生しました: ${e}`);
    }
}

const sleep = (ms) => new Promise(resolve => setTimeout(resolve, ms));

async function simulateRestOfRound(roundIndex) {
    const currentRound = currentTournament.rounds[roundIndex];
    if (!currentRound) return;

    // プレイヤーを含まない未対戦の試合をリストアップ
    const cpuMatches = [];
    currentRound.forEach((match, matchIndex) => {
        if (!match.winner && match.team1 !== 'プレイヤー' && match.team2 !== 'プレイヤー') {
            cpuMatches.push({ roundIndex, matchIndex, ...match });
        }
    });

    if (cpuMatches.length > 0) {
        console.log(`--- Round ${roundIndex + 1}のCPU戦を自動進行 ---`);
        for (const match of cpuMatches) {
            await sleep(1000); // 1秒待つ

            const winner = Math.random() < 0.5 ? match.team1 : match.team2;
            console.log(`CPU戦: ${match.team1} vs ${match.team2} => Winner: ${winner}`);
            
            try {
                currentTournament = update_match_result(currentTournament, match.roundIndex, match.matchIndex, winner);
                renderTournament(); // 1試合ずつUIを更新
            } catch (e) {
                console.error("CPU戦のシミュレーション中にエラー:", e);
                break; // エラーが出たらループを抜ける
            }
        }
    }
}


// --- UI描画 ---
function renderTournament() {
    tournamentStatusHeading.textContent = currentTournament.status;
    tournamentBracketDiv.innerHTML = '';

    const isTournamentOver = currentTournament.status === "ゲームオーバー" || currentTournament.status.includes("優勝");

    if (isTournamentOver) {
        const messageColor = currentTournament.status === "ゲームオーバー" ? "red" : "green";
        const message = currentTournament.status.includes("優勝") ? `${currentTournament.status} おめでとう！` : "ゲームオーバー！";
        tournamentBracketDiv.innerHTML = `<p style="color: ${messageColor}; font-size: 2em; text-align: center;">${message}</p>`;
        return;
    }

    currentTournament.rounds.forEach((round, roundIndex) => {
        const roundDiv = document.createElement('div');
        roundDiv.className = 'tournament-round';
        roundDiv.innerHTML = `<h3>${roundIndex + 1}回戦</h3>`;
        
        const matchesList = document.createElement('ul');
        matchesList.className = 'match-list';

        round.forEach((match, matchIndex) => {
            const matchLi = document.createElement('li');
            matchLi.className = 'match-item';
            
            const matchTeamsDiv = document.createElement('div');
            matchTeamsDiv.className = 'match-teams';
            matchTeamsDiv.innerHTML = `<span class="team-name">${match.team1}</span> vs <span class="team-name">${match.team2}</span>`;

            const winnerControlDiv = document.createElement('div');
            winnerControlDiv.className = 'winner-control';

            if (match.winner) {
                winnerControlDiv.innerHTML = `<span class="winner-display">勝者: ${match.winner}</span>`;
            } else {
                // プレイヤーの試合の場合のみボタンを表示
                if (match.team1 === 'プレイヤー' || match.team2 === 'プレイヤー') {
                    const btn1 = document.createElement('button');
                    btn1.textContent = `▲ ${match.team1} の勝ち`;
                    btn1.dataset.roundIndex = roundIndex;
                    btn1.dataset.matchIndex = matchIndex;
                    btn1.dataset.winnerName = match.team1;
                    btn1.className = 'winner-button';

                    const btn2 = document.createElement('button');
                    btn2.textContent = `▲ ${match.team2} の勝ち`;
                    btn2.dataset.roundIndex = roundIndex;
                    btn2.dataset.matchIndex = matchIndex;
                    btn2.dataset.winnerName = match.team2;
                    btn2.className = 'winner-button';

                    winnerControlDiv.appendChild(btn1);
                    winnerControlDiv.appendChild(btn2);
                } else {
                    winnerControlDiv.innerHTML = `<span class="cpu-match-pending">結果待機中...</span>`;
                }
            }
            matchLi.appendChild(matchTeamsDiv);
            matchLi.appendChild(winnerControlDiv);
            matchesList.appendChild(matchLi);
        });
        roundDiv.appendChild(matchesList);
        tournamentBracketDiv.appendChild(roundDiv);
    });

    if (currentTournament.bye_teams && currentTournament.bye_teams.length > 0) {
        const byeDiv = document.createElement('div');
        byeDiv.className = 'bye-teams';
        byeDiv.innerHTML = `<h4>不戦勝チーム:</h4><ul>${currentTournament.bye_teams.map(team => `<li>${team}</li>`).join('')}</ul>`;
        tournamentBracketDiv.appendChild(byeDiv);
    }
}

// --- アプリケーション開始 ---
initializeApp();