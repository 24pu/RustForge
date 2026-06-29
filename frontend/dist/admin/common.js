// common.js

// 移除不再使用的 localStorage token 相关函数

/**
 * 清除本地存储中的用户权限及角色信息
 */
function clearAuth() {
    localStorage.removeItem('userRoles');
    localStorage.removeItem('userPermissions');
    localStorage.removeItem('userPermissionsTime');
}

/**
 * 通用 API 调用方法，依赖 Cookie 认证
 * @param {string} method - HTTP 方法 (GET, POST, PUT, DELETE 等)
 * @param {string} path   - 完整路径（例如 "/api/admin/plugins"）
 * @param {object|null} body - 请求体对象，会转为 JSON
 * @returns {Promise<any>} 解析后的 JSON 数据，若无响应体则返回 null
 * @throws 当响应状态非 2xx 时抛出错误，错误信息为响应文本
 */
async function apiCall(method, path, body) {
    const headers = { 'Content-Type': 'application/json' };
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


/**
 * 判断当前用户是否为管理员（拥有角色分配权限或 admin 权限）
 * @returns {Promise<boolean>}
 */
async function isAdmin() {
    const perms = await fetchUserPermissions();
    return perms.includes('role:assign') || perms.includes('admin');
}

/**
 * 退出登录，清理本地数据并跳转至登录页（根据当前页面自动选择前后台）
 */
async function requireAuth() {
    try {
        const userInfo = await apiCall('GET', '/api/me');
        if (userInfo && userInfo.roles) {
            localStorage.setItem('userRoles', JSON.stringify(userInfo.roles));
        }
        return true;
    } catch (e) {
        clearAuth();
        const isAdmin = window.location.pathname.startsWith('/admin');
        window.location.href = isAdmin ? '/admin/login.html' : '/login';
        return false;
    }
}

async function logout() {
    try {
        await fetch('/api/logout', { method: 'POST', credentials: 'include' });
    } catch (e) {}
    clearAuth();
    const isAdmin = window.location.pathname.startsWith('/admin');
    window.location.href = isAdmin ? '/admin/login.html' : '/login';
}

/**
 * 转义 HTML 特殊字符，防止 XSS
 * @param {string} str
 * @returns {string}
 */
function escapeHtml(str) {
    if (!str) return '';
    return str.replace(/[&<>]/g, m => {
        if (m === '&') return '&amp;';
        if (m === '<') return '&lt;';
        if (m === '>') return '&gt;';
        return m;
    });
}

// 模块级缓存：用户权限列表
let userPermissions = null;

/**
 * 获取当前用户的权限列表（带 5 分钟缓存）
 * @param {boolean} force - 是否强制刷新缓存
 * @returns {Promise<string[]>}
 */
async function fetchUserPermissions(force = false) {
    if (!force && userPermissions) return userPermissions;
    const cached = localStorage.getItem('userPermissions');
    const cachedTime = localStorage.getItem('userPermissionsTime');
    if (!force && cached && cachedTime && Date.now() - parseInt(cachedTime) < 5 * 60 * 1000) {
        userPermissions = JSON.parse(cached);
        return userPermissions;
    }
    const perms = await apiCall('GET', '/api/me/permissions');
    userPermissions = perms;
    localStorage.setItem('userPermissions', JSON.stringify(perms));
    localStorage.setItem('userPermissionsTime', Date.now().toString());
    return perms;
}

/**
 * 检查是否拥有指定权限
 * @param {string} perm
 * @returns {Promise<boolean>}
 */
async function hasPermission(perm) {
    const perms = await fetchUserPermissions();
    return perms.includes(perm);
}
/**
 * 加载顶部导航栏 (topbar) 并控制菜单显隐
 */

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
        let perms = [];
        try {
            perms = await fetchUserPermissions();
            console.log('[loadTopbar] 获取到的权限列表:', perms);
        } catch (err) {
            console.error('[loadTopbar] 获取权限失败，菜单将隐藏', err);
            perms = [];
        }

        // 辅助函数：安全地显示元素
        function setElementDisplay(id, shouldShow, displayValue = 'inline-block') {
            const elem = document.getElementById(id);
            if (!elem) {
                if (shouldShow) console.warn(`[loadTopbar] 未找到元素 #${id}`);
                return false;
            }
            if (shouldShow) elem.style.display = displayValue;
            return true;
        }

        // ---------- 桌面端菜单控制 ----------
        // 用户、角色、分类、设置
        setElementDisplay('users-nav', perms.includes('user:list'), 'inline-block');
        setElementDisplay('roles-nav', perms.includes('role:list'), 'inline-block');
        setElementDisplay('categories-nav', perms.includes('category:list'), 'inline-block');
        setElementDisplay('settings-nav', perms.includes('config:view'), 'inline-block');

        // 用户与权限组（父容器）
        const hasUserOrRolePerm = perms.includes('user:list') || perms.includes('role:list');
        if (hasUserOrRolePerm) {
            setElementDisplay('user-permission-group', true, 'inline-block');
        }

        // 主题管理
        if (perms.includes('theme:list')) {
            setElementDisplay('themes-nav', true, 'inline-block');
            setElementDisplay('themes-nav-mobile', true, 'block');
        }

        // 产品管理下拉组
        const hasProductPerm = perms.includes('product:list') || perms.includes('product:create') ||
                               perms.includes('product:edit') || perms.includes('product:delete');
        if (hasProductPerm) {
            setElementDisplay('product-group', true, 'inline-block');
            setElementDisplay('product-group-mobile', true, 'block');
        }

        // 产品分类菜单
        if (perms.includes('product_category:list')) {
            setElementDisplay('product-categories-nav', true, 'block');
            setElementDisplay('product-categories-nav-mobile', true, 'block');
        }
        // 订单管理菜单
        if (perms.includes('order:list')) {
            setElementDisplay('orders-nav', true, 'block');
            setElementDisplay('orders-nav-mobile', true, 'block');
        }

        // 媒体库菜单（已移到内容下拉内，但仍保留其显示控制）
        if (perms.includes('media:list')) {
            setElementDisplay('media-nav', true, 'inline-block');
            setElementDisplay('media-nav-mobile', true, 'block');
        }

        
        // ---------- 桌面端菜单控制 ----------
        // 插件管理（下拉组）
        if (perms.includes('plugin:list')) {
            setElementDisplay('plugins-group', true, 'inline-block');
            setElementDisplay('plugins-group-mobile', true, 'block');
            // 钩子管理作为子项，也使用相同权限
            setElementDisplay('hooks-nav', true, 'inline-block');
            setElementDisplay('hooks-nav-mobile', true, 'block');
        }


        // ---------- 模版管理（下拉菜单） ----------
        if (perms.includes('template:list')) {
            setElementDisplay('templates-group', true, 'inline-block');
            setElementDisplay('templates-group-mobile', true, 'block');
        }

        // ---------- 内容管理模块权限控制 ----------
        // 内容下拉组整体显示条件（有任一内容相关权限）
        const hasContentPermGroup = perms.includes('content:list') || perms.includes('content:create') ||
                                    perms.includes('category:list') || perms.includes('media:list');
        if (hasContentPermGroup) {
            setElementDisplay('content-group', true, 'inline-block');
            setElementDisplay('content-group-mobile', true, 'block');
        }

        // 精细控制添加内容、内容管理链接（可选）
        const addContentLink = document.querySelector('#content-group a[href="add-content.html"]');
        if (addContentLink) addContentLink.style.display = perms.includes('content:create') ? 'block' : 'none';
        const contentsLink = document.querySelector('#content-group a[href="contents.html"]');
        if (contentsLink) contentsLink.style.display = perms.includes('content:list') ? 'block' : 'none';

        // ---------- 移动端菜单控制 ----------
        setElementDisplay('users-nav-mobile', perms.includes('user:list'), 'block');
        setElementDisplay('roles-nav-mobile', perms.includes('role:list'), 'block');
        setElementDisplay('categories-nav-mobile', perms.includes('category:list'), 'block');
        setElementDisplay('settings-nav-mobile', perms.includes('config:view'), 'block');

        // 用户与权限组（移动端父容器）
        if (hasUserOrRolePerm) {
            setElementDisplay('user-permission-group-mobile', true, 'block');
        }

        // 移动端内容组控制（假设移动端也有对应的容器 id）
        if (hasContentPermGroup) {
            setElementDisplay('content-group-mobile', true, 'block');
        }

        // 获取站点配置（logo 与名称）
        let siteConfig = JSON.parse(localStorage.getItem('site_config') || '{}');
        if (!siteConfig.logo_url && !siteConfig.site_name) {
            try {
                const config = await apiCall('GET', '/api/admin/config');
                siteConfig = config;
                localStorage.setItem('site_config', JSON.stringify(config));
            } catch (err) {
                console.error('[loadTopbar] 加载站点配置失败', err);
            }
        }

        // 汉堡菜单切换
        const toggleBtn = document.getElementById('menu-toggle');
        const mobileMenu = document.getElementById('mobile-menu');
        if (toggleBtn && mobileMenu) {
            const newToggle = toggleBtn.cloneNode(true);
            toggleBtn.parentNode.replaceChild(newToggle, toggleBtn);
            newToggle.addEventListener('click', () => {
                mobileMenu.classList.toggle('hidden');
            });
        }
    } catch (err) {
        console.error('[loadTopbar] 加载导航栏失败', err);
    }
}

/**
 * 加载页脚（静态版权信息）
 */
async function loadFooter() {
    const container = document.getElementById('footer-container');
    if (!container) return;
    const year = new Date().getFullYear();
    container.innerHTML = `<div class="text-center text-gray-500 text-sm py-4 border-t mt-6">© ${year} 24pu.com. All rights reserved.</div>`;
}

/**
 * 打开插件设置模态框（iframe 方式）
 * @param {string} pluginName - 插件名称（已做 HTML 转义）
 */
async function openSettings(pluginName) {
    // 防止 XSS
    const safePluginName = escapeHtml(pluginName);
    const modal = document.createElement('div');
    modal.className = 'fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50';
    modal.innerHTML = `
        <div class="bg-white rounded-lg shadow-lg w-full max-w-2xl" style="height: 80vh;">
            <div class="flex justify-between items-center p-4 border-b">
                <h2 class="text-xl font-bold">插件设置 - ${safePluginName}</h2>
                <button id="modal-close" class="text-gray-500 hover:text-gray-700 text-2xl">&times;</button>
            </div>
            <iframe src="/plugins/${encodeURIComponent(pluginName)}/settings" class="w-full" style="height: calc(100% - 60px); border: none;"></iframe>
        </div>
    `;
    document.body.appendChild(modal);

    const closeBtn = document.getElementById('modal-close');
    if (closeBtn) {
        closeBtn.onclick = () => modal.remove();
    }
    // 点击遮罩关闭
    modal.addEventListener('click', (e) => {
        if (e.target === modal) modal.remove();
    });
}