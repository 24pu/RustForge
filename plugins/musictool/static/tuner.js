import { PitchDetector } from 'pitchy';

// ---------- 平滑器 ----------
class EnhancedSmoother {
    constructor() {
        this.medianWindow = [];
        this.medianSize = 5;
        this.ewmaCents = null;
        this.lastDisplayCents = 0;
        this.deadzone = 2;
        this.centsHistory = [];
        this.historySize = 6;
        this.peakCents = null;
        this.peakDetected = false;
        this.peakTimer = null;
    }
    medianFilter(value) {
        this.medianWindow.push(value);
        if (this.medianWindow.length > this.medianSize) this.medianWindow.shift();
        const sorted = [...this.medianWindow].sort((a, b) => a - b);
        return sorted[Math.floor(sorted.length / 2)];
    }
    calculateStability(values) {
        if (values.length < 2) return 1;
        const mean = values.reduce((a, b) => a + b, 0) / values.length;
        const variance = values.reduce((sum, val) => sum + Math.pow(val - mean, 2), 0) / values.length;
        const stdDev = Math.sqrt(variance);
        return Math.max(0, Math.min(1, 1 - (stdDev / 10)));
    }
    getAdaptiveAlpha(stability) {
        if (stability > 0.8) return 0.12;
        if (stability > 0.5) return 0.20;
        return 0.35;
    }
    detectPeak(newCents, clarity) {
        if (this.peakDetected) return false;
        if (this.ewmaCents !== null && clarity > 0.7) {
            const delta = Math.abs(newCents - this.ewmaCents);
            if (delta > 15 && newCents !== 0) {
                this.peakDetected = true;
                this.peakCents = newCents;
                if (this.peakTimer) clearTimeout(this.peakTimer);
                this.peakTimer = setTimeout(() => {
                    this.peakDetected = false;
                    this.peakCents = null;
                }, 200);
                return true;
            }
        }
        return false;
    }
    smooth(rawCents, clarity) {
        if (rawCents === 0 || clarity < 0.4) return { cents: this.lastDisplayCents, stability: 0 };
        const medianCents = this.medianFilter(rawCents);
        this.centsHistory.push(medianCents);
        if (this.centsHistory.length > this.historySize) this.centsHistory.shift();
        const stability = this.calculateStability(this.centsHistory);
        const adaptiveAlpha = this.getAdaptiveAlpha(stability);
        const isPeak = this.detectPeak(medianCents, clarity);
        let smoothedCents;
        if (isPeak && this.peakCents !== null) {
            smoothedCents = this.peakCents;
            this.ewmaCents = smoothedCents;
        } else if (this.ewmaCents === null) {
            smoothedCents = medianCents;
            this.ewmaCents = smoothedCents;
        } else {
            smoothedCents = adaptiveAlpha * medianCents + (1 - adaptiveAlpha) * this.ewmaCents;
            this.ewmaCents = smoothedCents;
        }
        let displayCents = smoothedCents;
        if (Math.abs(smoothedCents - this.lastDisplayCents) < this.deadzone) displayCents = this.lastDisplayCents;
        else this.lastDisplayCents = smoothedCents;
        displayCents = Math.min(60, Math.max(-60, displayCents));
        return { cents: displayCents, stability: stability };
    }
    reset() {
        this.medianWindow = [];
        this.ewmaCents = null;
        this.lastDisplayCents = 0;
        this.centsHistory = [];
        this.peakDetected = false;
        if (this.peakTimer) clearTimeout(this.peakTimer);
    }
}

// ---------- 乐器数据 (440Hz 基准) ----------
const INSTRUMENTS_RAW = {
    guitar: { name: '吉他', strings: [{ name: 'E2', freq: 82.41, order: 6, rangeLow: 65.93, rangeHigh: 98.89 }, { name: 'A2', freq: 110.00, order: 5, rangeLow: 88.00, rangeHigh: 132.00 }, { name: 'D3', freq: 146.83, order: 4, rangeLow: 117.46, rangeHigh: 176.20 }, { name: 'G3', freq: 196.00, order: 3, rangeLow: 156.80, rangeHigh: 235.20 }, { name: 'B3', freq: 246.94, order: 2, rangeLow: 197.55, rangeHigh: 296.33 }, { name: 'E4', freq: 329.63, order: 1, rangeLow: 263.70, rangeHigh: 395.56 }] },
    bass: { name: '贝斯', strings: [{ name: 'E1', freq: 41.20, order: 4, rangeLow: 34.00, rangeHigh: 49.44 }, { name: 'A1', freq: 55.00, order: 3, rangeLow: 46.75, rangeHigh: 66.00 }, { name: 'D2', freq: 73.42, order: 2, rangeLow: 62.41, rangeHigh: 88.10 }, { name: 'G2', freq: 98.00, order: 1, rangeLow: 83.30, rangeHigh: 117.60 }] },
    ukulele: { name: '尤克里里', strings: [{ name: 'G4', freq: 392.00, order: 4, rangeLow: 352.80, rangeHigh: 431.20 }, { name: 'C4', freq: 261.63, order: 3, rangeLow: 235.47, rangeHigh: 287.79 }, { name: 'E4', freq: 329.63, order: 2, rangeLow: 296.67, rangeHigh: 362.59 }, { name: 'A4', freq: 440.00, order: 1, rangeLow: 396.00, rangeHigh: 484.00 }] },
    ruan: { name: '阮', strings: [{ name: 'G2', freq: 98.00, order: 4, rangeLow: 83.30, rangeHigh: 117.60 }, { name: 'D3', freq: 146.83, order: 3, rangeLow: 124.81, rangeHigh: 176.20 }, { name: 'G3', freq: 196.00, order: 2, rangeLow: 166.60, rangeHigh: 235.20 }, { name: 'D4', freq: 293.66, order: 1, rangeLow: 249.61, rangeHigh: 352.39 }] },
    erhu: { name: '二胡', strings: [{ name: 'D4', freq: 293.66, order: 2, rangeLow: 249.61, rangeHigh: 352.39 }, { name: 'A4', freq: 440.00, order: 1, rangeLow: 374.00, rangeHigh: 506.00 }] }
};

// ---------- 状态 ----------
let currentBase = 440;
let currentInstrument = 'guitar';
let currentStrings = [];
let currentStringIndex = 0;
let currentString = null;

let audioContext = null, mediaStream = null, sourceNode = null, analyserNode = null;
let detector = null, inputBuffer = null, isRunning = false, detectionInterval = null;
const smoother = new EnhancedSmoother();
const BUFFER_SIZE = 2048;
const UPDATE_INTERVAL_MS = 50;

// DOM 引用
const targetStringEl = document.getElementById('targetString');
const targetFreqEl = document.getElementById('targetFreq');
const centsValueEl = document.getElementById('centsValue');
const meterFill = document.getElementById('meterFill');
const clarityFill = document.getElementById('clarityFill');
const clarityPercent = document.getElementById('clarityPercent');
const startBtn = document.getElementById('startBtn');
const stopBtn = document.getElementById('stopBtn');
const statusMsg = document.getElementById('statusMsg');
const detectedHint = document.getElementById('detectedHint');
const stringsContainer = document.getElementById('stringsContainer');
const instrumentSelector = document.getElementById('instrumentSelector');

// ---------- 辅助函数 ----------
function getAdjustedStrings(instrumentId, base) {
    const raw = INSTRUMENTS_RAW[instrumentId];
    if (!raw) return [];
    const factor = base / 440;
    return raw.strings.map(s => ({
        ...s,
        freq: s.freq * factor,
        rangeLow: s.rangeLow * factor,
        rangeHigh: s.rangeHigh * factor
    }));
}

// 切换基准 (暴露到全局)
window.switchBase = function(base) {
    if (base === currentBase) {
        statusMsg.innerHTML = `ℹ️ 已经是 ${base}Hz 基准`;
        statusMsg.className = 'text-center text-sm text-blue-600 bg-blue-50 rounded-full py-2 px-3';
        return;
    }
    currentBase = base;
    document.querySelectorAll('.base-btn').forEach(btn => {
        btn.classList.remove('bg-accent', 'text-white');
        btn.classList.add('bg-gray-100', 'text-gray-700');
        if (parseInt(btn.dataset.base) === base) {
            btn.classList.remove('bg-gray-100', 'text-gray-700');
            btn.classList.add('bg-accent', 'text-white');
        }
    });
    currentStrings = getAdjustedStrings(currentInstrument, currentBase);
    if (!currentStrings || currentStrings.length === 0) {
        statusMsg.innerHTML = `❌ 未找到乐器数据`;
        return;
    }
    currentStringIndex = 0;
    currentString = currentStrings[0];
    targetStringEl.textContent = currentString.name;
    targetFreqEl.textContent = currentString.freq.toFixed(2) + ' Hz';
    renderStrings();
    smoother.reset();
    detectedHint.innerHTML = `等待弹奏 ${currentString.name} 弦...`;
    detectedHint.style.color = '#999';
    centsValueEl.innerHTML = '--';
    meterFill.style.width = '50%';
    statusMsg.innerHTML = `✅ 已切换至 ${base}Hz 基准，请重新弹奏`;
    statusMsg.className = 'text-center text-sm text-green-600 bg-green-50 rounded-full py-2 px-3';
    if (isRunning) {
        statusMsg.innerHTML = `🎸 基准已切换至 ${base}Hz，请弹奏 ${currentString.name} 弦`;
    }
};

function switchInstrument(instrumentId) {
    currentInstrument = instrumentId;
    currentStrings = getAdjustedStrings(instrumentId, currentBase);
    if (!currentStrings || currentStrings.length === 0) return;
    currentStringIndex = 0;
    currentString = currentStrings[0];
    targetStringEl.textContent = currentString.name;
    targetFreqEl.textContent = currentString.freq.toFixed(2) + ' Hz';
    renderStrings();
    smoother.reset();
    document.querySelectorAll('.inst-btn').forEach(btn => {
        btn.classList.remove('bg-accent', 'text-white');
        btn.classList.add('bg-gray-100', 'text-gray-700');
    });
    const activeBtn = document.querySelector(`.inst-btn[data-instrument="${instrumentId}"]`);
    if (activeBtn) {
        activeBtn.classList.remove('bg-gray-100', 'text-gray-700');
        activeBtn.classList.add('bg-accent', 'text-white');
    }
    if (isRunning) {
        const name = INSTRUMENTS_RAW[instrumentId].name;
        statusMsg.innerHTML = `🎸 已切换至 ${name}，请弹奏 ${currentString.name} 弦`;
        statusMsg.className = 'text-center text-sm text-green-600 bg-green-50 rounded-full py-2 px-3';
    }
    detectedHint.innerHTML = `等待弹奏 ${currentString.name} 弦...`;
    centsValueEl.innerHTML = '--';
    meterFill.style.width = '50%';
}

function renderStrings() {
    if (!currentStrings || currentStrings.length === 0) {
        stringsContainer.innerHTML = '<div class="text-center text-gray-500">无弦数据</div>';
        return;
    }
    stringsContainer.innerHTML = `<div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 gap-2">` +
        currentStrings.map((str, idx) => `<div class="string-btn bg-gray-100 hover:bg-gray-200 rounded-xl p-2 text-center cursor-pointer transition" data-index="${idx}" data-freq="${str.freq}" data-name="${str.name}"><div class="font-bold text-dark">${str.name}</div><div class="text-xs text-gray-500">${str.freq.toFixed(2)} Hz</div>${str.order ? `<div class="text-xs text-gray-400">${str.order}弦</div>` : ''}</div>`).join('') +
        `</div>`;
    document.querySelectorAll('.string-btn').forEach(btn => btn.addEventListener('click', () => selectString(parseInt(btn.dataset.index))));
    highlightSelectedString();
}

function selectString(index) {
    if (!currentStrings || index >= currentStrings.length) return;
    currentStringIndex = index;
    currentString = currentStrings[index];
    targetStringEl.textContent = currentString.name;
    targetFreqEl.textContent = currentString.freq.toFixed(2) + ' Hz';
    highlightSelectedString();
    smoother.reset();
    if (isRunning) statusMsg.innerHTML = `🎸 已选择 ${currentString.name} 弦，请弹奏`;
}

function highlightSelectedString() {
    document.querySelectorAll('.string-btn').forEach((btn, idx) => {
        btn.classList.remove('ring-2', 'ring-accent', 'bg-accent/10');
        if (idx === currentStringIndex) btn.classList.add('ring-2', 'ring-accent', 'bg-accent/10');
    });
}

function updateStringTuneStatus(isInTune) {
    const btns = document.querySelectorAll('.string-btn');
    btns.forEach((btn, idx) => {
        if (idx === currentStringIndex) {
            if (isInTune) btn.classList.add('bg-green-100', 'border-green-300');
            else btn.classList.remove('bg-green-100', 'border-green-300');
        }
    });
}

function isInRange(freq) {
    return freq >= currentString.rangeLow && freq <= currentString.rangeHigh;
}

function computeCents(detectedFreq) {
    return detectedFreq <= 0 ? 0 : 1200 * Math.log2(detectedFreq / currentString.freq);
}

function updateUI(cents, detectedFreq, clarity, stability) {
    const clarityPercentValue = Math.round(clarity * 100);
    clarityFill.style.width = `${clarityPercentValue}%`;
    clarityPercent.innerText = `${clarityPercentValue}%`;

    const inRange = detectedFreq > 0 && isInRange(detectedFreq) && clarity > 0.45;
    if (!inRange || detectedFreq <= 0) {
        if (detectedFreq > 0 && !isInRange(detectedFreq) && clarity > 0.4) {
            detectedHint.innerHTML = `⚠️ ${detectedFreq.toFixed(1)} Hz 不在 ${currentString.name} 范围内`;
            detectedHint.style.color = '#e65100';
            statusMsg.innerHTML = `⚠️ 请弹奏 ${currentString.name} 弦`;
            statusMsg.className = 'text-center text-sm text-orange-600 bg-orange-50 rounded-full py-2 px-3';
        } else {
            detectedHint.innerHTML = `等待弹奏 ${currentString.name} 弦...`;
            detectedHint.style.color = '#999';
            if (isRunning) {
                statusMsg.innerHTML = `🎸 请弹奏 ${currentString.name} 弦`;
                statusMsg.className = 'text-center text-sm text-green-600 bg-green-50 rounded-full py-2 px-3';
            }
        }
        meterFill.style.width = '50%';
        centsValueEl.innerHTML = '--';
        centsValueEl.style.color = '#999';
        return;
    }

    detectedHint.innerHTML = `🎵 ${detectedFreq.toFixed(1)} Hz · 稳定度 ${Math.round(stability * 100)}%`;
    detectedHint.style.color = '#4caf50';

    const direction = cents > 3 ? '偏高 ↑' : (cents < -3 ? '偏低 ↓' : '准！');
    statusMsg.innerHTML = `🎸 ${currentString.name} 弦 · ${direction}`;
    statusMsg.className = 'text-center text-sm text-green-600 bg-green-50 rounded-full py-2 px-3';

    const centsDisplay = cents > 0 ? `+${Math.round(cents)}` : `${Math.round(cents)}`;
    centsValueEl.innerHTML = centsDisplay;
    let percent = (cents + 50) / 100;
    percent = Math.min(1, Math.max(0, percent));
    meterFill.style.width = `${percent * 100}%`;

    if (Math.abs(cents) < 3) {
        centsValueEl.style.color = '#4caf50';
        meterFill.style.background = "#4caf50";
        updateStringTuneStatus(true);
    } else if (Math.abs(cents) < 10) {
        centsValueEl.style.color = '#ff9800';
        meterFill.style.background = "#ff9800";
        updateStringTuneStatus(false);
    } else {
        centsValueEl.style.color = '#f44336';
        meterFill.style.background = "#f44336";
        updateStringTuneStatus(false);
    }
}

// ---------- 音频处理 ----------
function detectionLoop() {
    if (!isRunning || !analyserNode || !detector) return;
    try {
        analyserNode.getFloatTimeDomainData(inputBuffer);
        const [rawFreq, clarity] = detector.findPitch(inputBuffer, audioContext.sampleRate);
        let validFreq = 0, rawCents = 0;
        if (rawFreq > 30 && rawFreq < 500 && clarity > 0.35) {
            if (isInRange(rawFreq)) {
                validFreq = rawFreq;
                rawCents = computeCents(validFreq);
            }
        }
        const smoothResult = smoother.smooth(rawCents, clarity);
        updateUI(smoothResult.cents, validFreq, clarity, smoothResult.stability);
    } catch (e) { console.warn(e); }
}

function startDetectionLoop() {
    if (detectionInterval) clearInterval(detectionInterval);
    detectionInterval = setInterval(() => detectionLoop(), UPDATE_INTERVAL_MS);
}

function stopDetectionLoop() {
    if (detectionInterval) clearInterval(detectionInterval);
}

async function initAudio() {
    try {
        const stream = await navigator.mediaDevices.getUserMedia({ audio: { echoCancellation: false, noiseSuppression: false, autoGainControl: false } });
        mediaStream = stream;
        audioContext = new(window.AudioContext || window.webkitAudioContext)();
        sourceNode = audioContext.createMediaStreamSource(stream);
        analyserNode = audioContext.createAnalyser();
        analyserNode.fftSize = BUFFER_SIZE;
        sourceNode.connect(analyserNode);
        detector = PitchDetector.forFloat32Array(analyserNode.fftSize);
        inputBuffer = new Float32Array(detector.inputLength);
        if (audioContext.state === 'suspended') await audioContext.resume();
        return { success: true };
    } catch (error) {
        let errorMsg = '无法获取麦克风权限';
        if (error.name === 'NotAllowedError') errorMsg = '请允许麦克风权限';
        else if (error.name === 'NotFoundError') errorMsg = '未检测到麦克风';
        statusMsg.innerHTML = `❌ ${errorMsg}`;
        statusMsg.className = 'text-center text-sm text-red-600 bg-red-50 rounded-full py-2 px-3';
        return { success: false };
    }
}

async function startTuner() {
    if (isRunning) return;
    statusMsg.innerHTML = '🎤 正在请求麦克风...';
    const initResult = await initAudio();
    if (!initResult.success) return;
    isRunning = true;
    smoother.reset();
    startDetectionLoop();
    startBtn.disabled = true;
    stopBtn.disabled = false;
    statusMsg.innerHTML = `🎸 请弹奏 ${currentString.name} 弦`;
    statusMsg.className = 'text-center text-sm text-green-600 bg-green-50 rounded-full py-2 px-3';
    detectedHint.innerHTML = `已选择 ${currentString.name} 弦，弹奏即可`;
}

async function stopTuner() {
    if (!isRunning) return;
    isRunning = false;
    stopDetectionLoop();
    if (audioContext) await audioContext.close();
    if (sourceNode) sourceNode.disconnect();
    if (analyserNode) analyserNode.disconnect();
    if (mediaStream) mediaStream.getTracks().forEach(track => track.stop());
    detector = null;
    smoother.reset();
    meterFill.style.width = '50%';
    centsValueEl.innerHTML = '--';
    clarityFill.style.width = '0%';
    clarityPercent.innerText = '0%';
    startBtn.disabled = false;
    stopBtn.disabled = true;
    statusMsg.innerHTML = '⏸️ 已停止';
    statusMsg.className = 'text-center text-sm text-gray-500 bg-gray-50 rounded-full py-2 px-3';
    document.querySelectorAll('.string-btn').forEach(btn => btn.classList.remove('bg-green-100'));
}

// ---------- 初始化乐器选择按钮 ----------
function initInstrumentButtons() {
    const locales = window.__plugin_locales__ || {};
    const instruments = [
        { id: 'guitar', label: locales.guitar || '吉他' },
        { id: 'bass', label: locales.bass || '贝斯' },
        { id: 'ukulele', label: locales.ukulele || '尤克里里' },
        { id: 'ruan', label: locales.ruan || '阮' },
        { id: 'erhu', label: locales.erhu || '二胡' }
    ];
    instrumentSelector.innerHTML = instruments.map(inst =>
        `<button class="inst-btn px-4 py-2 rounded-full text-sm font-medium transition bg-gray-100 text-gray-700 hover:bg-accent hover:text-white" data-instrument="${inst.id}">${inst.label}</button>`
    ).join('');
    // 默认高亮吉他
    const defaultBtn = document.querySelector('.inst-btn[data-instrument="guitar"]');
    if (defaultBtn) {
        defaultBtn.classList.remove('bg-gray-100', 'text-gray-700');
        defaultBtn.classList.add('bg-accent', 'text-white');
    }
    // 事件监听
    instrumentSelector.addEventListener('click', (e) => {
        const btn = e.target.closest('.inst-btn');
        if (!btn) return;
        switchInstrument(btn.dataset.instrument);
    });
}

// ---------- 初始化 ----------
currentStrings = getAdjustedStrings('guitar', currentBase);
if (currentStrings && currentStrings.length > 0) {
    currentString = currentStrings[0];
    targetStringEl.textContent = currentString.name;
    targetFreqEl.textContent = currentString.freq.toFixed(2) + ' Hz';
}
initInstrumentButtons();
renderStrings();

// ---------- 事件绑定 ----------
startBtn.addEventListener('click', startTuner);
stopBtn.addEventListener('click', stopTuner);

window.addEventListener('beforeunload', () => {
    if (mediaStream) mediaStream.getTracks().forEach(track => track.stop());
    if (audioContext) audioContext.close();
    if (detectionInterval) clearInterval(detectionInterval);
});

console.log('🎵 调音器已初始化 (Musictool 插件)');