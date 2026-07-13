// ============================================================
//  Musictool 插件 – 节拍器
//  支持 BPM 30-240，拍号 2/4, 3/4, 4/4, 6/8
// ============================================================

let audioContext = null;
let isRunning = false;
let timerId = null;
let currentBeat = 0;
let beatsPerMeasure = 4;
let bpm = 120;
let nextNoteTime = 0;
let scheduleInterval = null;

// DOM 引用
const bpmSlider = document.getElementById('bpmSlider');
const bpmDisplay = document.getElementById('bpmDisplay');
const beatIndicator = document.getElementById('beatIndicator');
const startBtn = document.getElementById('metronomeStartBtn');
const stopBtn = document.getElementById('metronomeStopBtn');
const statusMsg = document.getElementById('metronomeStatus');

// 拍号按钮
const timeSignatureBtns = document.querySelectorAll('.time-signature-btn');

// 节拍器状态
let isMetronomeRunning = false;

// ---- 初始化节拍指示器 ----
function initBeatIndicator(beats) {
    beatIndicator.innerHTML = '';
    for (let i = 0; i < beats; i++) {
        const dot = document.createElement('div');
        dot.className = 'beat-dot' + (i === 0 ? ' accent' : '');
        dot.dataset.index = i;
        beatIndicator.appendChild(dot);
    }
}

// ---- 更新拍号 ----
function setTimeSignature(beats) {
    beatsPerMeasure = beats;
    initBeatIndicator(beats);
    currentBeat = 0;
    updateBeatIndicator(0);
}

// ---- 更新节拍指示器 ----
function updateBeatIndicator(beatIndex) {
    const dots = beatIndicator.querySelectorAll('.beat-dot');
    dots.forEach((dot, i) => {
        dot.classList.toggle('active', i === beatIndex);
    });
}

// ---- 播放点击音 ----
function playClick(accent) {
    if (!audioContext) return;
    
    const oscillator = audioContext.createOscillator();
    const gainNode = audioContext.createGain();
    
    oscillator.connect(gainNode);
    gainNode.connect(audioContext.destination);
    
    // 强拍（重音）使用不同频率和音量
    if (accent) {
        oscillator.frequency.value = 1000;
        gainNode.gain.value = 0.8;
    } else {
        oscillator.frequency.value = 800;
        gainNode.gain.value = 0.5;
    }
    
    oscillator.type = 'sine';
    oscillator.start();
    oscillator.stop(audioContext.currentTime + 0.05);
    gainNode.gain.exponentialRampToValueAtTime(0.001, audioContext.currentTime + 0.05);
}

// ---- 调度节拍 ----
function scheduleBeat() {
    if (!isMetronomeRunning) return;
    
    const now = audioContext.currentTime;
    const interval = 60 / bpm;
    
    // 计算下一个节拍时间
    if (nextNoteTime < now) {
        nextNoteTime = now + 0.01;
    }
    
    // 播放当前节拍
    const accent = currentBeat === 0;
    playClick(accent);
    updateBeatIndicator(currentBeat);
    
    // 更新状态显示
    const beatNum = currentBeat + 1;
    statusMsg.textContent = `🎵 节拍 ${beatNum}/${beatsPerMeasure} · ${bpm} BPM`;
    
    // 移动到下一个节拍
    currentBeat = (currentBeat + 1) % beatsPerMeasure;
    nextNoteTime += interval;
    
    // 继续调度
    if (isMetronomeRunning) {
        scheduleInterval = setTimeout(scheduleBeat, (nextNoteTime - audioContext.currentTime) * 1000);
    }
}

// ---- 启动节拍器 ----
async function startMetronome() {
    if (isMetronomeRunning) return;
    
    try {
        // 创建音频上下文
        if (!audioContext) {
            audioContext = new (window.AudioContext || window.webkitAudioContext)();
        }
        if (audioContext.state === 'suspended') {
            await audioContext.resume();
        }
        
        isMetronomeRunning = true;
        currentBeat = 0;
        nextNoteTime = audioContext.currentTime;
        
        startBtn.disabled = true;
        stopBtn.disabled = false;
        statusMsg.textContent = `🎵 节拍器运行中 · ${bpm} BPM`;
        statusMsg.className = 'text-center text-sm text-green-600 bg-green-50 rounded-full py-2 px-3';
        
        // 禁用 BPM 滑块和拍号按钮
        bpmSlider.disabled = true;
        timeSignatureBtns.forEach(btn => btn.disabled = true);
        
        // 开始调度
        scheduleBeat();
    } catch (error) {
        console.error('启动节拍器失败:', error);
        statusMsg.textContent = '❌ 启动失败，请检查音频权限';
        statusMsg.className = 'text-center text-sm text-red-600 bg-red-50 rounded-full py-2 px-3';
    }
}

// ---- 停止节拍器 ----
function stopMetronome() {
    if (!isMetronomeRunning) return;
    
    isMetronomeRunning = false;
    
    if (scheduleInterval) {
        clearTimeout(scheduleInterval);
        scheduleInterval = null;
    }
    
    startBtn.disabled = false;
    stopBtn.disabled = true;
    statusMsg.textContent = '⏸️ 节拍器已停止';
    statusMsg.className = 'text-center text-sm text-gray-500 bg-gray-50 rounded-full py-2 px-3';
    
    // 启用控件
    bpmSlider.disabled = false;
    timeSignatureBtns.forEach(btn => btn.disabled = false);
    
    // 重置指示器
    currentBeat = 0;
    updateBeatIndicator(0);
}

// ---- BPM 变化 ----
bpmSlider.addEventListener('input', function() {
    bpm = parseInt(this.value);
    bpmDisplay.textContent = bpm;
    
    // 如果节拍器正在运行，动态调整速度
    if (isMetronomeRunning) {
        // 简单实现：停止并重新开始
        // 更优雅的方式是调整 interval，但为了简化，我们重启
        const wasRunning = isMetronomeRunning;
        if (wasRunning) {
            // 停止当前调度
            isMetronomeRunning = false;
            if (scheduleInterval) {
                clearTimeout(scheduleInterval);
                scheduleInterval = null;
            }
            // 重新启动
            isMetronomeRunning = true;
            currentBeat = 0;
            nextNoteTime = audioContext.currentTime;
            scheduleBeat();
        }
    }
});

// ---- 拍号切换 ----
timeSignatureBtns.forEach(btn => {
    btn.addEventListener('click', function() {
        if (this.disabled) return;
        
        const beats = parseInt(this.dataset.beats);
        setTimeSignature(beats);
        
        // 更新按钮样式
        timeSignatureBtns.forEach(b => {
            b.classList.remove('bg-accent', 'text-white');
            b.classList.add('bg-gray-100', 'text-gray-700');
        });
        this.classList.remove('bg-gray-100', 'text-gray-700');
        this.classList.add('bg-accent', 'text-white');
        
        statusMsg.textContent = `📐 已切换至 ${beats}/4 拍号`;
        statusMsg.className = 'text-center text-sm text-blue-600 bg-blue-50 rounded-full py-2 px-3';
    });
});

// ---- 启动按钮 ----
startBtn.addEventListener('click', startMetronome);

// ---- 停止按钮 ----
stopBtn.addEventListener('click', stopMetronome);

// ---- 页面卸载清理 ----
window.addEventListener('beforeunload', function() {
    if (isMetronomeRunning) {
        stopMetronome();
    }
    if (audioContext) {
        audioContext.close();
    }
});

// ---- 标签切换（与调音器联动） ----
// 当切换到节拍器标签时，如果调音器正在运行，自动停止
document.addEventListener('DOMContentLoaded', function() {
    // 初始化节拍指示器
    setTimeSignature(4);
    
    // 监听标签切换
    const tabs = document.querySelectorAll('.tab-btn');
    tabs.forEach(tab => {
        tab.addEventListener('click', function() {
            const target = this.dataset.tab;
            
            // 更新标签样式
            tabs.forEach(t => t.classList.remove('active'));
            this.classList.add('active');
            
            // 切换内容
            document.querySelectorAll('.tab-content').forEach(content => {
                content.classList.remove('active');
            });
            document.getElementById(`tab-${target}`).classList.add('active');
            
            // 如果切换到调音器，停止节拍器
            if (target === 'tuner' && isMetronomeRunning) {
                stopMetronome();
            }
            
            // 如果切换到节拍器，停止调音器
            if (target === 'metronome') {
                // 调音器的停止逻辑在 tuner.js 中处理
                const stopBtn = document.getElementById('stopBtn');
                if (stopBtn && !stopBtn.disabled) {
                    stopBtn.click();
                }
            }
        });
    });
});

console.log('🎵 节拍器已初始化 (Musictool 插件)');