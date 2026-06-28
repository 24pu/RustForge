// ========== 五运六气计算器（纯前端） ==========

const WUYUN = {
    // 常量
    LQ_DE: "巳亥厥阴风木 子午少阴君火 丑未太阴湿土 寅申少阳相火 卯酉阳明燥金 辰戌太阳寒水".split(' '),
    TG_DE: "甲 乙 丙 丁 戊 己 庚 辛 壬 癸".split(' '),
    DZ_DE: "寅 卯 辰 巳 午 未 申 酉 戌 亥 子 丑".split(' '),
    WY_DE: "甲己化土 乙庚化金 丙辛化水 丁壬化木 戊癸化火".split(' '),
    ZQS_DE: "厥阴风木 少阴君火 少阳相火 太阴湿土 阳明燥金 太阳寒水".split(' '),

    // 获取天干地支（修正版）
    getTgyear(year) {
        // 1900年天干为庚（索引6），地支为子（索引10）
        const tgIndex = ((year - 1900) % 10 + 6) % 10;
        const dzIndex = ((year - 1900) % 12 + 10) % 12;
        const tg = this.TG_DE[tgIndex] || '';
        const dz = this.DZ_DE[dzIndex] || '';
        return { tg, dz, full: `${tg} ${dz}` };
    },

    // 获取强弱
    getQr(tg) {
        const strong = ['甲', '丙', '戊', '庚', '壬'];
        const weak = ['乙', '丁', '己', '辛', '癸'];
        if (strong.includes(tg)) return { symbol: '↑', label: '强', class: 'text-green-600' };
        if (weak.includes(tg)) return { symbol: '↓', label: '弱', class: 'text-red-600' };
        return { symbol: '', label: '', class: '' };
    },

    // 计算五运六气
    getWulq(year, time) {
        const tgyear = this.getTgyear(year);
        const parts = tgyear.full.split(' ');
        const tg = parts[0];
        const dz = parts[1];

        // 司天
        let st = '';
        let stn = 0;
        this.LQ_DE.forEach((item, idx) => {
            if (item.includes(dz)) {
                st = item;
                stn = idx;
            }
        });

        // 在泉
        const zqn = (stn + 3) % 6;
        const zq = this.LQ_DE[zqn] || '';

        // 中运
        let zhy = '';
        this.WY_DE.forEach(item => {
            if (item.includes(tg)) {
                zhy = item;
            }
        });

        // 主气客气
        const { zhq, kq } = this.getZhKq(stn, year, time);

        return {
            wyst: st,
            wykq: kq,
            wyzhy: zhy,
            wyzhq: zhq,
            wyzq: zq,
            tgyear: tgyear.full,
            tg: tg,
            dz: dz,
            qr: this.getQr(tg)
        };
    },

    // 主气客气
    getZhKq(stnum, year, time) {
        const date = new Date(`${year}-${time}`);
        const boundaries = [
            { m: 1, d: 21 }, { m: 3, d: 21 }, { m: 5, d: 21 },
            { m: 7, d: 22 }, { m: 9, d: 22 }, { m: 11, d: 22 }
        ];
        let zqnum = 0;
        for (let i = 0; i < boundaries.length; i++) {
            const start = new Date(year, boundaries[i].m - 1, boundaries[i].d);
            const end = i === boundaries.length - 1
                ? new Date(year + 1, 0, 21)
                : new Date(year, boundaries[i + 1].m - 1, boundaries[i + 1].d);
            if (date >= start && date < end) {
                zqnum = i + 1;
                break;
            }
        }
        if (zqnum === 0) zqnum = 6;
        const zhq = this.ZQS_DE[zqnum - 1] || '';
        const kqnum = (stnum - 3 + zqnum) % 6;
        const kq = this.LQ_DE[kqnum] || '';
        return { zhq, kq };
    }
};