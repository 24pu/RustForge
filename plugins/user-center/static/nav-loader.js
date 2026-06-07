function loadNav(currentPage) {
    return fetch('/plugins/user-center/static/nav.html')
        .then(res => res.text())
        .then(html => {
            // 替换模板变量
            const locales = window.__plugin_locales__ || {};
            for (const [key, value] of Object.entries(locales)) {
                html = html.replace(new RegExp(`\\{\\{\\s*${key}\\s*\\}\\}`, 'g'), value);
            }
            
            document.getElementById('user-center-nav').innerHTML = html;
            
            // 高亮当前页面
            document.querySelectorAll('[data-nav]').forEach(link => {
                if (link.getAttribute('data-nav') === currentPage) {
                    link.classList.add('bg-gray-100', 'font-semibold');
                }
            });
            
            // 绑定退出事件
            const logoutBtn = document.getElementById('logout-btn');
            if (logoutBtn) {
                logoutBtn.addEventListener('click', async function() {
                    try {
                        await fetch('/api/logout', { method: 'POST', credentials: 'include' });
                    } catch(e) {}
                    window.location.href = '/';
                });
            }
        });
}