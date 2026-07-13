// ============================================================
//  Musictool 插件 – 音阶练习器
//  吉他指板 + 钢琴联动
//  唱名显示：固定 C 大调数字简谱（C=1, D=2, E=3, F=4, G=5, A=6, B=7）
// ============================================================

(function() {
    // ---- 吉他音高矩阵 ----
    const GUITAR_PITCH_MATRIX = [
        ["E4", "F4", "F#4", "G4", "G#4", "A4", "A#4", "B4", "C5", "C#5", "D5", "D#5", "E5"],
        ["B3", "C4", "C#4", "D4", "D#4", "E4", "F4", "F#4", "G4", "G#4", "A4", "A#4", "B4"],
        ["G3", "G#3", "A3", "A#3", "B3", "C4", "C#4", "D4", "D#4", "E4", "F4", "F#4", "G4"],
        ["D3", "D#3", "E3", "F3", "F#3", "G3", "G#3", "A3", "A#3", "B3", "C4", "C#4", "D4"],
        ["A2", "A#2", "B2", "C3", "C#3", "D3", "D#3", "E3", "F3", "F#3", "G3", "G#3", "A3"],
        ["E2", "F2", "F#2", "G2", "G#2", "A2", "A#2", "B2", "C3", "C#3", "D3", "D#3", "E3"]
    ];
    const STRING_NAMES = ['1弦', '2弦', '3弦', '4弦', '5弦', '6弦'];
    const FRETS = 12;

    // ---- 音符映射 ----
    const NOTE_SEMI = { 'C':0, 'C#':1, 'D':2, 'D#':3, 'E':4, 'F':5, 'F#':6, 'G':7, 'G#':8, 'A':9, 'A#':10, 'B':11 };
    const SEMI_NOTES = ['C', 'C#', 'D', 'D#', 'E', 'F', 'F#', 'G', 'G#', 'A', 'A#', 'B'];

    // ---- 音阶模式 ----
    const SCALE_PATTERNS = {
        major: [0,2,4,5,7,9,11],
        natural_minor: [0,2,3,5,7,8,10],
        harmonic_minor: [0,2,3,5,7,8,11],
        major_penta: [0,2,4,7,9],
        minor_penta: [0,3,5,7,10],
        dorian: [0,2,3,5,7,9,10],
        mixolydian: [0,2,4,5,7,9,10],
        lydian: [0,2,4,6,7,9,11],
        phrygian: [0,1,3,5,7,8,10],
        locrian: [0,1,3,5,6,8,10]
    };
    const SCALE_NAMES = {
        major:'大调音阶', natural_minor:'自然小调', harmonic_minor:'和声小调',
        major_penta:'大调五声', minor_penta:'小调五声', dorian:'多利亚', mixolydian:'混合利底亚',
        lydian:'利底亚', phrygian:'弗里吉亚', locrian:'洛克利亚'
    };

    // ---- 状态 ----
    let currentRoot = 'C';
    let currentScaleType = 'major';
    let activePianoHighlightSemi = null;
    let pianoHighlightTimeout = null;
    let audioCtx = null;

    // ---- DOM ----
    const canvas = document.getElementById('guitarCanvas');
    const ctx = canvas.getContext('2d');
    const scaleSelect = document.getElementById('scaleTypeSelect');
    const rootDiv = document.getElementById('rootNoteButtons');
    const scaleNameSpan = document.getElementById('currentScaleName');
    const solfegeContainer = document.getElementById('solfegeContainer');
    const scaleDetailSpan = document.getElementById('scaleNotesDetail');

    // ---- 音频 ----
    function initAudio() {
        if (!audioCtx) audioCtx = new (window.AudioContext || window.webkitAudioContext)();
        return audioCtx;
    }

    function pitchToFreq(pitchWithOctave) {
        const match = pitchWithOctave.match(/^([A-G][#b]?)(-?\d+)$/);
        if (!match) return 440;
        let note = match[1];
        let oct = parseInt(match[2], 10);
        if (note === 'Db') note = 'C#';
        if (note === 'Eb') note = 'D#';
        if (note === 'Gb') note = 'F#';
        if (note === 'Ab') note = 'G#';
        if (note === 'Bb') note = 'A#';
        const semi = NOTE_SEMI[note];
        if (semi === undefined) return 440;
        const absoluteSemi = (oct + 1) * 12 + semi;
        const A4_SEMI = 69;
        return 440 * Math.pow(2, (absoluteSemi - A4_SEMI) / 12);
    }

    function getBaseNote(pitch) {
        let m = pitch.match(/^([A-G][#b]?)/);
        if (!m) return 'C';
        let note = m[1];
        if (note === 'Db') note = 'C#';
        if (note === 'Eb') note = 'D#';
        if (note === 'Gb') note = 'F#';
        if (note === 'Ab') note = 'G#';
        if (note === 'Bb') note = 'A#';
        return note;
    }

    // ---- 播放音色 ----
    function playGuitarSound(freq) {
        if (!freq) return;
        try {
            const ctxA = initAudio();
            if (ctxA.state === 'suspended') ctxA.resume();
            const now = ctxA.currentTime;
            const osc = ctxA.createOscillator();
            osc.type = 'triangle';
            osc.frequency.value = freq;
            const gain = ctxA.createGain();
            gain.gain.setTargetAtTime(0.25, now, 0.008);
            gain.gain.setTargetAtTime(0.001, now + 0.15, 0.28);
            const filter = ctxA.createBiquadFilter();
            filter.type = 'lowpass';
            filter.frequency.value = 3500;
            osc.connect(filter);
            filter.connect(gain);
            gain.connect(ctxA.destination);
            osc.start();
            osc.stop(now + 0.8);
            setTimeout(() => { try { osc.disconnect(); filter.disconnect(); gain.disconnect(); } catch(e) {} }, 900);
        } catch(e) {}
    }

    function playPianoSound(freq) {
        if (!freq) return;
        try {
            const ctxA = initAudio();
            if (ctxA.state === 'suspended') ctxA.resume();
            const now = ctxA.currentTime;
            const osc = ctxA.createOscillator();
            osc.type = 'sine';
            osc.frequency.value = freq;
            const gain = ctxA.createGain();
            gain.gain.setValueAtTime(0.35, now);
            gain.gain.exponentialRampToValueAtTime(0.0001, now + 1.2);
            const filter = ctxA.createBiquadFilter();
            filter.type = 'lowpass';
            filter.frequency.value = 2800;
            osc.connect(filter);
            filter.connect(gain);
            gain.connect(ctxA.destination);
            osc.start();
            osc.stop(now + 1.2);
            setTimeout(() => { try { osc.disconnect(); filter.disconnect(); gain.disconnect(); } catch(e) {} }, 1300);
        } catch(e) {}
    }

    // ---- 获取音阶音符集合 ----
    function getScaleNoteSet(root, type) {
        let set = new Set();
        let rootIdx = NOTE_SEMI[root] || 0;
        let pat = SCALE_PATTERNS[type];
        if (!pat) return set;
        for (let i of pat) set.add((rootIdx + i) % 12);
        return set;
    }

    // ---- 绘制指板 ----
    function drawFretboard() {
        const w = canvas.width, h = canvas.height;
        ctx.clearRect(0, 0, w, h);
        ctx.fillStyle = '#fefefe';
        ctx.fillRect(0, 0, w, h);

        const startY = 50, endY = h - 40;
        const strSpacing = (endY - startY) / 5;
        const fretStartX = 70, fretEndX = w - 40;
        const fretStep = (fretEndX - fretStartX) / FRETS;

        // 品丝
        ctx.beginPath();
        ctx.strokeStyle = '#cdd9ed';
        ctx.lineWidth = 1.2;
        for (let f = 0; f <= FRETS; f++) {
            let x = fretStartX + f * fretStep;
            ctx.moveTo(x, startY - 5);
            ctx.lineTo(x, endY + 5);
            ctx.stroke();
            if (f > 0 && f <= 12 && (f === 3 || f === 5 || f === 7 || f === 9)) {
                let dx = x - fretStep / 2;
                ctx.beginPath();
                ctx.arc(dx, startY + (endY - startY) / 2, 3.5, 0, 2 * Math.PI);
                ctx.fillStyle = '#cbdae9';
                ctx.fill();
            }
            if (f === 12) {
                let dx = x - fretStep / 2;
                ctx.beginPath();
                ctx.arc(dx, startY + (endY - startY) / 2 - 10, 3, 0, 2 * Math.PI);
                ctx.arc(dx, startY + (endY - startY) / 2 + 10, 3, 0, 2 * Math.PI);
                ctx.fill();
            }
        }

        // 琴弦
        ctx.lineWidth = 1.5;
        for (let s = 0; s < 6; s++) {
            let y = startY + s * strSpacing;
            ctx.beginPath();
            ctx.moveTo(fretStartX - 5, y);
            ctx.lineTo(fretEndX + 5, y);
            ctx.strokeStyle = '#bdc7db';
            ctx.stroke();
            ctx.fillStyle = '#6c7f9e';
            ctx.font = "bold 11px 'Segoe UI'";
            ctx.fillText(STRING_NAMES[s], fretStartX - 30, y + 4);
        }

        const scaleSet = getScaleNoteSet(currentRoot, currentScaleType);
        const rootSemi = NOTE_SEMI[currentRoot];
        const isPenta = (currentScaleType === 'major_penta' || currentScaleType === 'minor_penta');

        // 绘制音符
        for (let fret = 0; fret <= FRETS; fret++) {
            for (let str = 0; str < 6; str++) {
                const pitch = GUITAR_PITCH_MATRIX[str][fret];
                const baseNote = getBaseNote(pitch);
                const noteSemi = NOTE_SEMI[baseNote];
                const inScale = scaleSet.has(noteSemi);
                const isRoot = (noteSemi === rootSemi);
                const isHighlight = (activePianoHighlightSemi !== null && noteSemi === activePianoHighlightSemi);

                let fillColor = "#eef2f9", radius = 11;
                if (isHighlight) {
                    fillColor = "#8b5cf6";
                    radius = 14;
                } else if (inScale) {
                    if (isRoot) { fillColor = "#e67e22"; radius = 14; }
                    else { fillColor = isPenta ? "#5f9b89" : "#4a7c9c"; radius = 12; }
                } else {
                    fillColor = "#eef2f9";
                    radius = 8;
                }

                let x = fretStartX + fret * fretStep;
                if (x > fretEndX + 10) continue;
                let y = startY + str * strSpacing;

                ctx.beginPath();
                ctx.arc(x, y, radius, 0, 2 * Math.PI);
                ctx.fillStyle = fillColor;
                ctx.fill();

                if (isRoot && inScale && !isHighlight) {
                    ctx.beginPath();
                    ctx.arc(x, y, radius + 2, 0, 2 * Math.PI);
                    ctx.strokeStyle = "#f7bc7a";
                    ctx.lineWidth = 2;
                    ctx.stroke();
                }

                ctx.fillStyle = (inScale || isHighlight) ? "#ffffff" : "#7b8fae";
                ctx.font = `bold ${radius > 11 ? 12 : 10}px monospace`;
                ctx.fillText(pitch, x - 8, y + 4);
            }
        }

        ctx.font = "9px monospace";
        ctx.fillStyle = "#8f9fbb";
        for (let f = 1; f <= FRETS; f++) {
            let x = fretStartX + f * fretStep - fretStep / 2;
            ctx.fillText(f, x - 3, startY - 8);
        }
        ctx.fillStyle = "#7a8dab";
        ctx.font = "italic 10px";
        ctx.fillText("0 空弦", fretStartX + 5, startY - 12);
    }

    // ---- 获取数字简谱（固定 C 大调基准） ----
    function getScaleDegree(noteSemi) {
        // 固定以 C 大调为基准：C=1, D=2, E=3, F=4, G=5, A=6, B=7
        const degreeMap = {
            0: '1',    // C
            1: '#1',   // C#
            2: '2',    // D
            3: '#2',   // D#
            4: '3',    // E
            5: '4',    // F
            6: '#4',   // F#
            7: '5',    // G
            8: '#5',   // G#
            9: '6',    // A
            10: '#6',  // A#
            11: '7'    // B
        };
        return degreeMap[noteSemi] || '?';
    }

    // ---- 更新 UI ----
    function updateUI() {
        drawFretboard();

        let intervals = SCALE_PATTERNS[currentScaleType] || [];
        let rootSemi = NOTE_SEMI[currentRoot];
        let noteNames = intervals.map(i => { let s = (rootSemi + i) % 12; return SEMI_NOTES[s]; });

        // 生成固定 C 大调数字简谱（不随根音变化）
        let solf = noteNames.map(n => {
            let semi = NOTE_SEMI[n];
            return getScaleDegree(semi);
        });

        // 构建 HTML
        let html = '';
        for (let i = 0; i < noteNames.length; i++) {
            let isRoot = (NOTE_SEMI[noteNames[i]] === rootSemi);
            let displayName = solf[i];
            html += `<div class="solfege-item ${isRoot ? 'root-highlight' : ''}">${displayName} <span style="font-size:0.65rem;">(${noteNames[i]})</span></div>`;
        }
        solfegeContainer.innerHTML = html;
        scaleDetailSpan.innerHTML = `📍 ${currentRoot} ${SCALE_NAMES[currentScaleType]} 音序: ${noteNames.join(' · ')}`;
        scaleNameSpan.innerText = `${currentRoot} ${SCALE_NAMES[currentScaleType]} (${intervals.length}音)`;
    }

    // ---- 钢琴键盘 ----
    function buildPianoKeyboard() {
        const container = document.getElementById('pianoKeyboard');
        container.innerHTML = '';
        const octaveNotes = [
            { type: 'white', note: 'C' }, { type: 'black', note: 'C#' }, { type: 'white', note: 'D' },
            { type: 'black', note: 'D#' }, { type: 'white', note: 'E' }, { type: 'white', note: 'F' },
            { type: 'black', note: 'F#' }, { type: 'white', note: 'G' }, { type: 'black', note: 'G#' },
            { type: 'white', note: 'A' }, { type: 'black', note: 'A#' }, { type: 'white', note: 'B' }
        ];
        const startOctave = 3, endOctave = 5;

        for (let oct = startOctave; oct <= endOctave; oct++) {
            for (let item of octaveNotes) {
                if (oct === endOctave && item.note === 'B' && oct === 5) {
                    const pitch = `${item.note}${oct}`;
                    const keyDiv = document.createElement('div');
                    keyDiv.className = `piano-key ${item.type}`;
                    keyDiv.setAttribute('data-pitch', pitch);
                    const label = document.createElement('div');
                    label.className = 'key-label';
                    label.innerText = pitch;
                    keyDiv.appendChild(label);
                    keyDiv.onclick = (e) => { e.stopPropagation(); onPianoKeyClick(pitch); };
                    container.appendChild(keyDiv);
                    break;
                }
                const pitch = `${item.note}${oct}`;
                const keyDiv = document.createElement('div');
                keyDiv.className = `piano-key ${item.type}`;
                keyDiv.setAttribute('data-pitch', pitch);
                const label = document.createElement('div');
                label.className = 'key-label';
                label.innerText = pitch;
                keyDiv.appendChild(label);
                keyDiv.onclick = (e) => { e.stopPropagation(); onPianoKeyClick(pitch); };
                container.appendChild(keyDiv);
            }
        }
    }

    // ---- 钢琴点击 ----
    function onPianoKeyClick(pitchWithOctave) {
        let match = pitchWithOctave.match(/^([A-G][#b]?)(\d+)$/);
        if (!match) return;
        let baseNote = match[1];
        if (baseNote === 'Db') baseNote = 'C#';
        if (baseNote === 'Eb') baseNote = 'D#';
        if (baseNote === 'Gb') baseNote = 'F#';
        if (baseNote === 'Ab') baseNote = 'G#';
        if (baseNote === 'Bb') baseNote = 'A#';
        const semi = NOTE_SEMI[baseNote];
        if (semi === undefined) return;

        if (pianoHighlightTimeout) clearTimeout(pianoHighlightTimeout);
        activePianoHighlightSemi = semi;
        drawFretboard();
        pianoHighlightTimeout = setTimeout(() => {
            activePianoHighlightSemi = null;
            drawFretboard();
        }, 700);

        const freq = pitchToFreq(pitchWithOctave);
        playPianoSound(freq);

        const degree = getScaleDegree(semi);
        const toast = document.createElement('div');
        toast.innerText = `🎹 钢琴 ${pitchWithOctave} → 指板高亮 (${baseNote}，数字 ${degree})`;
        toast.style.cssText = 'position:fixed;bottom:120px;left:50%;transform:translateX(-50%);background:#2c3e66cc;backdrop-filter:blur(6px);padding:4px 14px;border-radius:40px;color:white;font-size:12px;z-index:999;';
        document.body.appendChild(toast);
        setTimeout(() => toast.remove(), 900);
    }

    // ---- 指板点击 ----
    function onGuitarClick(e) {
        const rect = canvas.getBoundingClientRect();
        const sx = canvas.width / rect.width, sy = canvas.height / rect.height;
        let mx = (e.clientX - rect.left) * sx, my = (e.clientY - rect.top) * sy;

        const startY = 50, endY = canvas.height - 40;
        const strSpacing = (endY - startY) / 5;
        const fretStartX = 70, fretEndX = canvas.width - 40;
        const fretStep = (fretEndX - fretStartX) / FRETS;

        let strIdx = -1, minDist = 12;
        for (let s = 0; s < 6; s++) {
            let yL = startY + s * strSpacing, d = Math.abs(my - yL);
            if (d < minDist) { minDist = d; strIdx = s; }
        }
        if (strIdx === -1 || minDist > 20) return;

        let fretIdx = -1, minFret = 25;
        for (let f = 0; f <= FRETS; f++) {
            let xL = fretStartX + f * fretStep, d = Math.abs(mx - xL);
            if (d < minFret) { minFret = d; fretIdx = f; }
        }
        if (fretIdx === -1 || minFret > 28) return;

        const pitch = GUITAR_PITCH_MATRIX[strIdx][fretIdx];
        const freq = pitchToFreq(pitch);
        playGuitarSound(freq);

        const baseNote = getBaseNote(pitch);
        const semi = NOTE_SEMI[baseNote];
        const scaleSet = getScaleNoteSet(currentRoot, currentScaleType);
        const isIn = scaleSet.has(semi);
        const isRt = (semi === NOTE_SEMI[currentRoot]);
        const degree = getScaleDegree(semi);

        let msg = `${STRING_NAMES[strIdx]} ${fretIdx}品 → ${pitch}`;
        if (isRt) msg += " 🌟根音";
        if (isIn) msg += ` ✓音阶音 (数字 ${degree})`;
        else msg += " ✗非音阶";

        const t = document.createElement('div');
        t.innerText = msg;
        t.style.cssText = 'position:fixed;bottom:80px;left:50%;transform:translateX(-50%);background:#1f2a44e0;backdrop-filter:blur(6px);padding:6px 16px;border-radius:40px;color:#f3f6fc;z-index:999;font-size:13px;';
        document.body.appendChild(t);
        setTimeout(() => t.remove(), 1000);
    }

    // ---- 根音按钮 ----
    function createRootBtns() {
        rootDiv.innerHTML = '';
        ['C', 'C#', 'D', 'D#', 'E', 'F', 'F#', 'G', 'G#', 'A', 'A#', 'B'].forEach(n => {
            let btn = document.createElement('button');
            btn.className = 'note-btn';
            btn.textContent = n;
            btn.dataset.note = n;
            btn.onclick = () => {
                currentRoot = n;
                updateRootActive();
                updateUI();
            };
            rootDiv.appendChild(btn);
        });
        updateRootActive();
    }

    function updateRootActive() {
        document.querySelectorAll('.note-btn').forEach(btn => {
            if (btn.dataset.note === currentRoot) btn.classList.add('active');
            else btn.classList.remove('active');
        });
    }

    // ---- 初始化 ----
    function init() {
        createRootBtns();
        buildPianoKeyboard();

        scaleSelect.onchange = (e) => {
            currentScaleType = e.target.value;
            updateUI();
        };
        canvas.onclick = onGuitarClick;
        canvas.addEventListener('touchstart', (e) => {
            e.preventDefault();
            const t = e.touches[0];
            canvas.dispatchEvent(new MouseEvent('click', { clientX: t.clientX, clientY: t.clientY }));
        });

        document.addEventListener('click', () => {
            if (audioCtx && audioCtx.state === 'suspended') audioCtx.resume();
        }, { once: true });

        updateUI();
    }

    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', init);
    } else {
        init();
    }
})();