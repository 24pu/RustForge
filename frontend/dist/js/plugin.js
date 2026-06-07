// 通用插件调用函数
async function callPlugin(pluginName, input) {
    const res = await fetch(`/api/plugin/${pluginName}`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(input)
    });
    if (!res.ok) throw new Error(await res.text());
    return res.json();
}