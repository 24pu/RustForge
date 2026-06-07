// common.js

// 删除不再使用的 localStorage token 相关函数
// function getAuthToken() { ... }
// function setAuth(token) { ... }

function clearAuth() {
    localStorage.removeItem('userRoles');
    localStorage.removeItem('userPermissions');
    localStorage.removeItem('userPermissionsTime');
}

async function apiCall(method, path, body) {
    const headers = { 'Content-Type': 'application/json' };
    // 注意：path 应为完整路径，例如 "/api/admin/plugins"
    const response = await fetch(path, {
        method,
        credentials: 'include',        // 依赖 Cookie 认证
        headers,
        body: body ? JSON.stringify(body) : undefined,
    });
    const text = await response.text();
    if (!response.ok) throw new Error(text);
    return text ? JSON.parse(text) : null;
}

async function requireAuth() {
    // 直接调用完整路径，依赖 Cookie 中的 auth_token
    try {
        const userInfo = await apiCall('GET', '/api/me');   // 完整路径
        if (userInfo.roles) {
            localStorage.setItem('userRoles', JSON.stringify(userInfo.roles));
        }
        return true;
    } catch (e) {
        clearAuth();
        window.location.href = '/admin/login.html';
        return false;
    }
}

async function isAdmin() {
    const perms = await fetchUserPermissions();
    return perms.includes('role:assign') || perms.includes('admin');
}


async function logout() {
    try {
        await fetch('/api/logout', { method: 'POST', credentials: 'include' });
    } catch(e) {}
    clearAuth();
    window.location.href = '/admin/login.html';
}

function escapeHtml(str) {
    if (!str) return '';
    return str.replace(/[&<>]/g, m => {
        if (m === '&') return '&amp;';
        if (m === '<') return '&lt;';
        if (m === '>') return '&gt;';
        return m;
    });
}

// 加载导航
async function loadTopbar() {
    const container = document.getElementById('topbar-container');
    if (!container) return;
    try {
        const response = await fetch('/admin/components/topbar.html');
        const html = await response.text();
        container.innerHTML = html;
        // 高亮当前页面
        const currentPage = window.location.pathname.split('/').pop();
        document.querySelectorAll('.nav-link').forEach(link => {
            const href = link.getAttribute('href');
            if (href === currentPage) {
                link.classList.add('text-yellow-400', 'font-semibold');
            }
        });

        // 获取用户权限
        const perms = await fetchUserPermissions();

        // 桌面端菜单显示
        const desktopUsersNav = document.getElementById('users-nav');
        const desktopRolesNav = document.getElementById('roles-nav');
        const desktopCategoriesNav = document.getElementById('categories-nav');
        const desktopSettingsNav = document.getElementById('settings-nav');
        if (perms.includes('user:list') && desktopUsersNav) desktopUsersNav.style.display = 'inline-block';
        if (perms.includes('role:list') && desktopRolesNav) desktopRolesNav.style.display = 'inline-block';
        if (perms.includes('category:list') && desktopCategoriesNav) desktopCategoriesNav.style.display = 'inline-block';
        if (perms.includes('config:view') && desktopSettingsNav) desktopSettingsNav.style.display = 'inline-block';
        if (perms.includes('theme:list')) {
            const themesNav = document.getElementById('themes-nav');
            const themesNavMobile = document.getElementById('themes-nav-mobile');
            if (themesNav) themesNav.style.display = 'inline-block';
            if (themesNavMobile) themesNavMobile.style.display = 'block';
        }
        // 移动端菜单显示
        const mobileUsersNav = document.getElementById('users-nav-mobile');
        const mobileRolesNav = document.getElementById('roles-nav-mobile');
        const mobileCategoriesNav = document.getElementById('categories-nav-mobile');
        const mobileSettingsNav = document.getElementById('settings-nav-mobile');
        if (perms.includes('user:list') && mobileUsersNav) mobileUsersNav.style.display = 'block';
        if (perms.includes('role:list') && mobileRolesNav) mobileRolesNav.style.display = 'block';
        if (perms.includes('category:list') && mobileCategoriesNav) mobileCategoriesNav.style.display = 'block';
        if (perms.includes('config:view') && mobileSettingsNav) mobileSettingsNav.style.display = 'block';
        if (perms.includes('media:list')) {
            const mediaNav = document.getElementById('media-nav');
            const mediaNavMobile = document.getElementById('media-nav-mobile');
            if (mediaNav) mediaNav.style.display = 'inline-block';
            if (mediaNavMobile) mediaNavMobile.style.display = 'block';
        }
        if (perms.includes('plugin:list')) {
            const pluginsNav = document.getElementById('plugins-nav');
            const pluginsNavMobile = document.getElementById('plugins-nav-mobile');
            if (pluginsNav) pluginsNav.style.display = 'inline-block';
            if (pluginsNavMobile) pluginsNavMobile.style.display = 'block';
        }

        // 获取系统配置（如果尚未获取，可以调用 API 或使用缓存）
        let siteConfig = JSON.parse(localStorage.getItem('site_config') || '{}');
        if (!siteConfig.logo_url && !siteConfig.site_name) {
            // 如果 localStorage 中没有，则从后端获取
            try {
                const config = await apiCall('GET', '/api/admin/config'); // 完整路径
                siteConfig = config;
                localStorage.setItem('site_config', JSON.stringify(config));
            } catch (err) {
                console.error('Failed to load site config', err);
            }
        }
       

        // 汉堡菜单切换
        const toggleBtn = document.getElementById('menu-toggle');
        const mobileMenu = document.getElementById('mobile-menu');
        if (toggleBtn && mobileMenu) {
            toggleBtn.addEventListener('click', () => {
                mobileMenu.classList.toggle('hidden');
            });
        }
    } catch (err) {
        console.error('Failed to load topbar:', err);
    }
}

let userPermissions = null;

async function fetchUserPermissions(force = false) {
    if (!force && userPermissions) return userPermissions;
    const cached = localStorage.getItem('userPermissions');
    const cachedTime = localStorage.getItem('userPermissionsTime');
    if (!force && cached && cachedTime && Date.now() - parseInt(cachedTime) < 5 * 60 * 1000) {
        userPermissions = JSON.parse(cached);
        return userPermissions;
    }
    const perms = await apiCall('GET', '/api/me/permissions'); // 完整路径
    userPermissions = perms;
    localStorage.setItem('userPermissions', JSON.stringify(perms));
    localStorage.setItem('userPermissionsTime', Date.now().toString());
    return perms;
}

async function hasPermission(perm) {
    const perms = await fetchUserPermissions();
    return perms.includes(perm);
}

async function loadFooter() {
    const container = document.getElementById('footer-container');
    if (!container) return;
    const year = new Date().getFullYear();
    container.innerHTML = `<div class="text-center text-gray-500 text-sm py-4 border-t mt-6">© ${year} 24pu.com. All rights reserved.</div>`;
}

async function openSettings(pluginName) {
    // 创建模态框
    const modal = document.createElement('div');
    modal.className = 'fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50';
    modal.innerHTML = `
        <div class="bg-white rounded-lg shadow-lg w-full max-w-2xl" style="height: 80vh;">
            <div class="flex justify-between items-center p-4 border-b">
                <h2 class="text-xl font-bold">插件设置 - ${pluginName}</h2>
                <button id="modal-close" class="text-gray-500 hover:text-gray-700 text-2xl">&times;</button>
            </div>
            <iframe src="/plugins/${pluginName}/settings" class="w-full" style="height: calc(100% - 60px); border: none;"></iframe>
        </div>
    `;
    document.body.appendChild(modal);

    document.getElementById('modal-close').onclick = () => modal.remove();
    // 点击遮罩关闭
    modal.addEventListener('click', function(e) {
        if (e.target === modal) modal.remove();
    });
}