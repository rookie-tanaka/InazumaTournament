import init, { get_all_opponents, generate_tournament, update_match_result, get_playable_opponents_info } from "./pkg/InazumaTournament.js";

let currentTournament;
let allOpponents = [];

// DOM要素への参照
const settingsPanel = document.getElementById('settings-panel');
const tournamentPanel = document.getElementById('tournament-panel');
const backToSettingsBtn = document.getElementById('back-to-settings-btn');

const playerLevelInput = document.getElementById('player-level');
const teamCountSelect = document.getElementById('team-count');
const levelToleranceLowerInput = document.getElementById('level-tolerance-lower');
const levelToleranceUpperInput = document.getElementById('level-tolerance-upper');
const opponentListDiv = document.getElementById('opponent-list');
const generateBtn = document.getElementById('generate-btn');
const selectAllBtn = document.getElementById('select-all-opponents');
const deselectAllBtn = document.getElementById('deselect-all-opponents');
const tournamentStatusHeading = document.getElementById('tournament-status');
const tournamentBracketDiv = document.getElementById('tournament-bracket');
const playableOpponentsCountDisplay = document.querySelector('#playable-opponents-count span');
const seriesFilterRadios = document.getElementById('series-filter-radios');
const modesFieldset = document.getElementById('modes-fieldset');
const levelWinRateModifierInput = document.getElementById('level-win-rate-modifier');


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
        updatePlayableOpponentsCount();
    });
    deselectAllBtn.addEventListener('click', () => {
        toggleAllOpponents(false);
        updatePlayableOpponentsCount();
    });
    tournamentBracketDiv.addEventListener('click', handleMatchClick);
    backToSettingsBtn.addEventListener('click', showSettingsScreen);
    seriesFilterRadios.addEventListener('change', populateOpponentList);
    modesFieldset.addEventListener('change', () => {
        populateOpponentList();
        updatePlayableOpponentsCount();
    });


    // 設定変更時にチーム数を更新するリスナー
    playerLevelInput.addEventListener('change', updatePlayableOpponentsCount);
    teamCountSelect.addEventListener('change', updatePlayableOpponentsCount);
    levelToleranceLowerInput.addEventListener('change', updatePlayableOpponentsCount);
    levelToleranceUpperInput.addEventListener('change', updatePlayableOpponentsCount);
    opponentListDiv.addEventListener('change', updatePlayableOpponentsCount);
    levelWinRateModifierInput.addEventListener('change', updatePlayableOpponentsCount);

    updatePlayableOpponentsCount();
}

function populateOpponentList() {
    const allowedModes = Array.from(document.querySelectorAll('#modes-fieldset input:checked')).map(cb => cb.value);
    const selectedSeries = document.querySelector('input[name="series-filter"]:checked').value;

    // 既存のチェックボックスの状態を保存
    const checkedOpponents = new Set(
        Array.from(document.querySelectorAll('#opponent-list input:checked')).map(cb => cb.value)
    );
    const isFirstLoad = opponentListDiv.children.length === 0;

    opponentListDiv.innerHTML = '';
    allOpponents
        .filter(opponent => {
            // モードフィルター
            const modeMatch = allowedModes.includes(opponent.source);
            if (!modeMatch) return false;

            // シリーズフィルター
            if (selectedSeries === 'ALL') return true;
            const seriesMatch = opponent.name.match(/\((.*?)\)/);
            return seriesMatch && seriesMatch[1] === selectedSeries;
        })
        .forEach(opponent => {
            const div = document.createElement('div');
            const checkbox = document.createElement('input');
            checkbox.type = 'checkbox';
            checkbox.id = `opponent-${opponent.name.replace(/\s|\(|\)|\-/g, '_')}`;
            checkbox.value = opponent.name;
            
            // 初回ロード時はすべてチェック、それ以外は以前の状態を維持
            checkbox.checked = isFirstLoad || checkedOpponents.has(opponent.name);

            const label = document.createElement('label');
            label.htmlFor = `opponent-${opponent.name.replace(/\s|\(|\)|\-/g, '_')}`;
            label.textContent = opponent.name;

            div.appendChild(checkbox);
            div.appendChild(label);
            opponentListDiv.appendChild(div);
        });
}

function toggleAllOpponents(checked) {
    document.querySelectorAll('#opponent-list input[type="checkbox"]').forEach(cb => cb.checked = checked);
}

function showSettingsScreen() {
    tournamentPanel.hidden = true;
    settingsPanel.hidden = false;
}

function showTournamentScreen() {
    settingsPanel.hidden = true;
    tournamentPanel.hidden = false;
}

// --- 対戦可能なチーム数を更新し表示する ---
function updatePlayableOpponentsCount() {
    const settings = {
        player_team_level: parseInt(playerLevelInput.value, 10),
        team_count: parseInt(teamCountSelect.value, 10),
        level_tolerance_lower: parseInt(levelToleranceLowerInput.value, 10),
        level_tolerance_upper: parseInt(levelToleranceUpperInput.value, 10),
        level_win_rate_modifier: parseInt(levelWinRateModifierInput.value, 10),
        allowed_sources: Array.from(document.querySelectorAll('#modes-fieldset input:checked')).map(cb => cb.value),
        unlocked_opponents: Array.from(document.querySelectorAll('#opponent-list input:checked')).map(cb => cb.value),
    };

    try {
        const info = get_playable_opponents_info(settings);
        playableOpponentsCountDisplay.textContent = info.count;
        if (info.count < parseInt(teamCountSelect.value, 10) -1) {
            playableOpponentsCountDisplay.parentElement.style.color = 'red';
        } else {
            playableOpponentsCountDisplay.parentElement.style.color = 'inherit';
        }
    } catch (e) {
        console.error("対戦可能なチーム数の取得に失敗しました:", e);
        playableOpponentsCountDisplay.textContent = `エラー`;
        playableOpponentsCountDisplay.parentElement.style.color = 'red';
    }
}


// --- トーナメント生成 ---
function handleGenerateTournament() {
    const settings = {
        player_team_level: parseInt(playerLevelInput.value, 10),
        team_count: parseInt(teamCountSelect.value, 10),
        level_tolerance_lower: parseInt(levelToleranceLowerInput.value, 10),
        level_tolerance_upper: parseInt(levelToleranceUpperInput.value, 10),
        level_win_rate_modifier: parseInt(levelWinRateModifierInput.value, 10),
        allowed_sources: Array.from(document.querySelectorAll('#modes-fieldset input:checked')).map(cb => cb.value),
        unlocked_opponents: Array.from(document.querySelectorAll('#opponent-list input:checked')).map(cb => cb.value),
    };

    const requiredOpponents = settings.team_count -1;
    const playableInfo = get_playable_opponents_info(settings);
    if (playableInfo.count < requiredOpponents) {
        alert(`対戦可能なチームが足りません。${requiredOpponents}チーム必要ですが、${playableInfo.count}チームしかいません。設定を見直してください。`);
        return;
    }

    try {
        currentTournament = generate_tournament(settings);
        renderTournament();
        showTournamentScreen();
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

    // UIを即時更新してユーザーにフィードバック
    const matchItem = target.closest('.match-item');
    const winnerControl = matchItem.querySelector('.winner-control');
    winnerControl.innerHTML = `<span class="winner-display">処理中...</span>`;


    const roundIndex = parseInt(target.dataset.roundIndex, 10);
    const matchIndex = parseInt(target.dataset.matchIndex, 10);
    const winnerName = target.dataset.winnerName;

    // Rustの処理を少し遅延させて、UIの更新を確実に見せる
    setTimeout(async () => {
        try {
            currentTournament = await update_match_result(currentTournament, roundIndex, matchIndex, winnerName);
            renderTournament();
        } catch (e) {
            console.error("試合結果の更新中にエラー:", e);
            alert(`試合結果の更新中にエラーが発生しました: ${e}`);
            // エラーが発生した場合、UIを元に戻すか、エラーメッセージを表示する
            winnerControl.innerHTML = `<span style="color: red;">エラー</span>`;
        }
    }, 50); // 50ミリ秒の遅延
}

// --- UI描画 ---
function renderTournament() {
    tournamentStatusHeading.textContent = currentTournament.status;
    tournamentBracketDiv.innerHTML = '';

    const getTeamDisplayHTML = (teamName) => {
        if (teamName === 'プレイヤー') {
            return `<span class="team-name player">プレイヤー</span>`;
        }
        const opponent = currentTournament.participants.get(teamName);
        if (opponent) {
            return `<span class="team-name opponent">${opponent.name} <span class="team-details">(Lv.${opponent.level}, ${opponent.difficulty_name})</span></span>`;
        }
        return `<span class="team-name">${teamName}</span>`; // フォールバック
    };

    const isTournamentOver = currentTournament.status === "ゲームオーバー" || currentTournament.status.includes("優勝");

    if (isTournamentOver) {
        const winnerName = currentTournament.status.includes("優勝") 
            ? currentTournament.status.replace(" 優勝！", "") 
            : null;
        const message = winnerName 
            ? `${getTeamDisplayHTML(winnerName)} 優勝！ おめでとう！`
            : "ゲームオーバー！";
        const messageColor = winnerName ? "green" : "red";

        tournamentBracketDiv.innerHTML = `<div style="color: ${messageColor}; font-size: 1.5em; text-align: center;">${message}</div>`;
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
            matchTeamsDiv.innerHTML = `${getTeamDisplayHTML(match.team1)} vs ${getTeamDisplayHTML(match.team2)}`;

            const winnerControlDiv = document.createElement('div');
            winnerControlDiv.className = 'winner-control';

            if (match.winner) {
                winnerControlDiv.innerHTML = `<span class="winner-display">勝者: ${getTeamDisplayHTML(match.winner)}</span>`;
            } else {
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
        byeDiv.innerHTML = `<h4>不戦勝チーム:</h4><ul>${currentTournament.bye_teams.map(team => `<li>${getTeamDisplayHTML(team)}</li>`).join('')}</ul>`;
        tournamentBracketDiv.appendChild(byeDiv);
    }
}

// --- アプリケーション開始 ---
initializeApp();
