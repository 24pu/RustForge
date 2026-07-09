// wuyun-settings.js
const pluginName = 'wuyun';
let currentHookId = null;
let currentLang = '';

const langSelect = document.getElementById('lang-select');
const langStatus = document.getElementById('lang-status');
const navContent = document.getElementById('nav-content');
const messageDiv = document.getElementById('message');
const addBtn = document.getElementById('add-nav-btn');
const saveBtn = document.getElementById('save-btn');
const deleteBtn = document.getElementById('delete-btn');

// 显示消息
function showMessage(text, type) {
    messageDiv.textContent = text;
    messageDiv.className = 'mt-3 text-sm font-medium text-center ' + (type === 'green' ? 'text-green-600' : 'text-red-600');
    clearTimeout(window._msgTimer);
    window._msgTimer = setTimeout(() => {
        messageDiv.textContent = '';
        messageDiv.className = 'mt-3 text-sm font-medium text-center';
    }, 4000);
}

// 加载语言选项
async function loadLanguageOptions() {
    try {
        const resp = await fetch('/api/admin/lang-settings', { credentials: 'include' });
        if (!resp.ok) throw new Error('获取语言配置失败');
        const data = await resp.json();
        const availableLangs = data.available_langs || [];

        langSelect.innerHTML = '';
        const generalOption = document.createElement('option');
        generalOption.value = '';
        generalOption.textContent = '🌐 通用（所有语言）';
        langSelect.appendChild(generalOption);

        availableLangs.forEach(lang => {
            const opt = document.createElement('option');
            opt.value = lang.code;
            opt.textContent = `${lang.name} (${lang.code})`;
            langSelect.appendChild(opt);
        });

        langSelect.value = '';
        currentLang = '';
    } catch (e) {
        console.warn('加载语言配置失败，使用默认选项', e);
        langSelect.innerHTML = `
            <option value="">🌐 通用（所有语言）</option>
            <option value="zh">🇨🇳 中文</option>
            <option value="en">🇬🇧 英文</option>
            <option value="ja">🇯🇵 日文</option>
            <option value="ko">🇰🇷 韩文</option>
        `;
        langSelect.value = '';
    }
}

// 加载钩子内容（根据语言）
async function loadNavHook(lang) {
    const url = `/api/admin/plugins/${pluginName}/hooks?hook_name=nav${lang ? '&lang=' + encodeURIComponent(lang) : ''}`;
    try {
        const resp = await fetch(url, { credentials: 'include' });
        if (!resp.ok) throw new Error('加载失败');
        const data = await resp.json();
        // 按插件名和语言精确匹配
        let target = data.find(h => h.plugin_name === pluginName && h.lang === lang) ||
                     data.find(h => h.plugin_name === pluginName) ||
                     null;
        if (target) {
            currentHookId = target.id;
            navContent.value = target.content || '';
            langStatus.textContent = `当前编辑：${target.lang || '通用'}`;
        } else {
            currentHookId = null;
            navContent.value = '';
            langStatus.textContent = '暂无内容，可新建';
        }
    } catch (e) {
        showMessage('加载钩子数据失败', 'red');
        langStatus.textContent = '加载失败';
    }
}

// 保存钩子（创建或更新）
async function saveNavHook(content, lang) {
    if (currentHookId) {
        const resp = await fetch(`/api/admin/plugins/${pluginName}/hooks/${currentHookId}`, {
            method: 'PUT',
            headers: { 'Content-Type': 'application/json' },
            credentials: 'include',
            body: JSON.stringify({ content, lang })
        });
        if (!resp.ok) throw new Error('更新失败');
    } else {
        const resp = await fetch(`/api/admin/plugins/${pluginName}/hooks`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            credentials: 'include',
            body: JSON.stringify({
                hook_name: 'nav',
                content,
                sort_order: 10,
                lang: lang || ''
            })
        });
        if (!resp.ok) throw new Error('创建失败');
        const newData = await resp.json();
        currentHookId = newData.id;
    }
}

// 语言切换
langSelect.addEventListener('change', function() {
    currentLang = this.value;
    loadNavHook(currentLang);
});

// 一键添加
addBtn.addEventListener('click', async function() {
    const lang = langSelect.value;
    const navHtml = `<li><a href="/plugins/wuyun/">{{ wuyun_title }}</a></li>`;
    if (navContent.value.includes('/plugins/wuyun/')) {
        showMessage('✅ 已存在于当前版本，无需重复添加', 'green');
        return;
    }
    const newContent = navContent.value ? navContent.value + '\n' + navHtml : navHtml;
    try {
        await saveNavHook(newContent, lang);
        await loadNavHook(lang);
        showMessage('✅ 已添加到导航栏（' + (lang || '通用') + '）', 'green');
    } catch (e) {
        showMessage('❌ 添加失败', 'red');
        console.error(e);
    }
});

// 保存
saveBtn.addEventListener('click', async function() {
    const content = navContent.value.trim();
    if (!content) {
        showMessage('请输入内容', 'red');
        return;
    }
    const lang = langSelect.value;
    try {
        await saveNavHook(content, lang);
        await loadNavHook(lang);
        showMessage('✅ 保存成功', 'green');
    } catch (e) {
        showMessage('❌ 保存失败', 'red');
        console.error(e);
    }
});

// 删除
deleteBtn.addEventListener('click', async function() {
    if (!currentHookId) {
        showMessage('当前没有可删除的钩子', 'red');
        return;
    }
    if (!confirm('确定要删除当前语言版本的导航钩子吗？')) return;
    try {
        const resp = await fetch(`/api/admin/plugins/${pluginName}/hooks/${currentHookId}`, {
            method: 'DELETE',
            credentials: 'include'
        });
        if (!resp.ok) throw new Error('删除失败');
        currentHookId = null;
        navContent.value = '';
        loadNavHook(langSelect.value);
        showMessage('✅ 已删除', 'green');
    } catch (e) {
        showMessage('❌ 删除失败', 'red');
        console.error(e);
    }
});

// 初始化
(async function init() {
    await loadLanguageOptions();
    langSelect.value = '';
    currentLang = '';
    await loadNavHook('');
})();