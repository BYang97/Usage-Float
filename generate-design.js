#!/usr/bin/env bun
// OpenCode Usage Float — OpenPencil design generator
// Writes directly to .pen file

const fs = require('fs');

const $ = (id, name, type, o = {}) => ({ id, name, type, ...o });

const c = {
  surface:       '#1a1b1e',
  surfaceAlt:    '#222327',
  surfaceHover:  '#2c2e33',
  surfaceBorder: '#2f3036',
  textPrimary:   '#e4e5e7',
  textSecondary: '#8b8d97',
  textTertiary:  '#5c5e66',
  accentBlue:    '#4a9eff',
  accentGreen:   '#34d399',
  accentCyan:    '#22d3ee',
  statusOk:      '#34d399',
  statusWarning: '#fbbf24',
  statusDanger:  '#ef4444',
};

const GB = 'rgba(26,27,30,0.72)';
const X = String.fromCharCode(10005);

// ============================================================
// SHARED COMPONENTS
// ============================================================
const comps = [
  // ProviderBadge
  { id:'comp:badge', name:'ProviderBadge', type:'frame', reusable:true, width:150, height:24, layout:'horizontal', gap:8, alignItems:'center',
    children:[
      { id:'badge:dot', name:'StatusDot', type:'ellipse', width:8, height:8, fill:c.accentGreen },
      { id:'badge:label', name:'Label', type:'text', content:'OpenCode GO', fontFamily:'Inter', fontSize:13, fontWeight:600, fill:c.textPrimary, textGrowth:'fixed-width' },
    ]},
  // QuotaRing
  { id:'comp:ring', name:'QuotaRing', type:'frame', reusable:true, width:80, height:80,
    children:[
      { id:'ring:bg', name:'RingBG', type:'ellipse', width:72, height:72, x:4, y:4, fill:'none', stroke:{ align:'inside', thickness:6, fill:c.surfaceBorder } },
      { id:'ring:fg', name:'RingFG', type:'ellipse', width:72, height:72, x:4, y:4, fill:'none', stroke:{ align:'inside', thickness:6, fill:c.accentBlue } },
      { id:'ring:pct', name:'Percentage', type:'text', content:'82%', fontFamily:'Inter', fontSize:18, fontWeight:700, fill:c.textPrimary, x:40, y:38, textAlign:'center', textAlignVertical:'center' },
    ]},
  // SidebarItem
  { id:'comp:si', name:'SidebarItem', type:'frame', reusable:true, width:220, height:36, layout:'horizontal', gap:10, padding:[0,12,0,12], alignItems:'center', cornerRadius:6,
    children:[
      { id:'si:icon', name:'Icon', type:'rectangle', width:16, height:16, fill:c.textTertiary, cornerRadius:3 },
      { id:'si:label', name:'Label', type:'text', content:'Menu Item', fontFamily:'Inter', fontSize:14, fontWeight:500, fill:c.textSecondary, textGrowth:'fixed-width' },
    ]},
];

// ============================================================
// PAGE 1 — Float Widget (320×180)
// ============================================================
const floatWidget = {
  id:'page:float', name:'Float Widget', type:'frame',
  width:320, height:180, fill:GB, cornerRadius:12, clip:true,
  effect:[
    { type:'drop-shadow', color:'rgba(0,0,0,0.4)', offset:{x:0,y:8}, blur:32 },
    { type:'background-blur', blur:20 },
  ],
  children:[
    { id:'float:drag', name:'DragBar', type:'rectangle', width:320, height:36, fill:'transparent' },
    { id:'float:badge', name:'Provider', type:'ref', ref:'comp:badge', x:16, y:10 },
    // close
    { id:'float:close', name:'CloseBtn', type:'frame', width:24, height:24, x:284, y:8,
      children:[
        { id:'float:close-x', name:'CloseX', type:'text', content:X, fontFamily:'Inter', fontSize:12, fontWeight:400, fill:c.textTertiary, x:12, y:12, textAlign:'center', textAlignVertical:'center' },
      ]},
    // min
    { id:'float:min', name:'MinBtn', type:'frame', width:24, height:24, x:256, y:8,
      children:[
        { id:'float:min-line', name:'MinLine', type:'rectangle', width:10, height:2, fill:c.textTertiary, x:7, y:11, cornerRadius:1 },
      ]},
    // ring
    { id:'float:ring', name:'Ring', type:'ref', ref:'comp:ring', x:120, y:46 },
    // info row
    { id:'float:info', name:'InfoRow', type:'frame', width:288, height:20, x:16, y:128, layout:'horizontal', gap:8, justifyContent:'space-between', alignItems:'center',
      children:[
        { id:'float:5h', name:'FiveHourLabel', type:'text', content:'5h Window', fontFamily:'Inter', fontSize:12, fontWeight:500, fill:c.textSecondary, textGrowth:'fixed-width' },
        { id:'float:dot', name:'Sep', type:'ellipse', width:3, height:3, fill:c.textTertiary },
        { id:'float:cnt', name:'Countdown', type:'text', content:'Reset: 01:42:30', fontFamily:'Inter', fontSize:12, fontWeight:400, fill:c.textTertiary, textGrowth:'fixed-width' },
      ]},
    // footer
    { id:'float:footer', name:'Footer', type:'frame', width:288, height:24, x:16, y:150,
      children:[
        { id:'float:ft', name:'FooterText', type:'text', content:'Open Dashboard >', fontFamily:'Inter', fontSize:11, fontWeight:500, fill:c.accentBlue, x:144, y:12, textAlign:'center', textAlignVertical:'center' },
      ]},
  ],
};

// ============================================================
// PAGE 2 — Dashboard (1200×800)
// ============================================================
const SBW = 240;
const HH = 52;
const CX = SBW;
const CY = HH;
const CW = 960;
const CH = 748;

const planCard = {
  id:'dash:plan', name:'PlanCard', type:'frame', width:912, height:80, fill:c.surfaceAlt, cornerRadius:8, layout:'horizontal', gap:40, padding:[16,20,16,20], alignItems:'center',
  children:[
    { id:'dash:pi', name:'PlanIcon', type:'frame', width:48, height:48, fill:c.accentBlue, cornerRadius:8,
      children:[
        { id:'dash:pii', name:'PlanIconInner', type:'rectangle', width:24, height:24, x:12, y:12, fill:c.accentCyan, cornerRadius:4 },
      ]},
    { id:'dash:pinfo', name:'PlanInfo', type:'frame', width:300, height:48, layout:'vertical', gap:4,
      children:[
        { id:'dash:pn', name:'PlanName', type:'text', content:'GO Monthly', fontFamily:'Inter', fontSize:16, fontWeight:600, fill:c.textPrimary },
        { id:'dash:pd', name:'PlanDesc', type:'text', content:'OpenCode GO - Active', fontFamily:'Inter', fontSize:13, fontWeight:400, fill:c.textSecondary },
      ]},
    { id:'dash:pe', name:'PlanExpire', type:'frame', width:160, height:48, layout:'vertical', gap:4,
      children:[
        { id:'dash:pel', name:'ExpireLabel', type:'text', content:'Expire', fontFamily:'Inter', fontSize:11, fontWeight:500, fill:c.textTertiary },
        { id:'dash:ped', name:'ExpireDate', type:'text', content:'2026-08-20', fontFamily:'Inter', fontSize:14, fontWeight:500, fill:c.textPrimary },
      ]},
    { id:'dash:ps', name:'PlanStatus', type:'frame', width:80, height:24, cornerRadius:4, fill:'rgba(52,211,153,0.1)', layout:'horizontal', gap:6, padding:[0,8,0,8], alignItems:'center',
      children:[
        { id:'dash:psd', name:'StatusDot', type:'ellipse', width:6, height:6, fill:c.statusOk },
        { id:'dash:pst', name:'StatusText', type:'text', content:'Active', fontFamily:'Inter', fontSize:11, fontWeight:500, fill:c.statusOk },
      ]},
  ],
};

const quotaGrid = {
  id:'dash:qgrid', name:'QuotaGrid', type:'frame', width:912, height:160, layout:'horizontal', gap:16,
  children:[
    // 5H
    { id:'dash:q5', name:'FiveHourCard', type:'frame', width:293, height:160, fill:c.surfaceAlt, cornerRadius:8, layout:'vertical', gap:12, padding:[16,16,16,16],
      children:[
        { id:'dash:q5t', name:'Q5Title', type:'text', content:'5 Hour Window', fontFamily:'Inter', fontSize:13, fontWeight:600, fill:c.textSecondary },
        { id:'dash:q5p', name:'Q5Pct', type:'text', content:'82%', fontFamily:'Inter', fontSize:28, fontWeight:700, fill:c.textPrimary },
        { id:'dash:q5b', name:'Q5Bar', type:'frame', width:261, height:8,
          children:[
            { id:'dash:q5bb', name:'Q5BarBG', type:'rectangle', width:261, height:8, fill:c.surfaceBorder, cornerRadius:4 },
            { id:'dash:q5bf', name:'Q5BarFG', type:'rectangle', width:214, height:8, fill:c.accentGreen, cornerRadius:4 },
          ]},
        { id:'dash:q5r', name:'Q5Reset', type:'text', content:'Reset: 01:42:30', fontFamily:'Inter', fontSize:12, fontWeight:400, fill:c.textTertiary },
      ]},
    // Weekly
    { id:'dash:qw', name:'WeeklyCard', type:'frame', width:293, height:160, fill:c.surfaceAlt, cornerRadius:8, layout:'vertical', gap:12, padding:[16,16,16,16],
      children:[
        { id:'dash:qwt', name:'QWTitle', type:'text', content:'Weekly Window', fontFamily:'Inter', fontSize:13, fontWeight:600, fill:c.textSecondary },
        { id:'dash:qwp', name:'QWPct', type:'text', content:'63%', fontFamily:'Inter', fontSize:28, fontWeight:700, fill:c.textPrimary },
        { id:'dash:qwb', name:'QWBar', type:'frame', width:261, height:8,
          children:[
            { id:'dash:qwbb', name:'QWBarBG', type:'rectangle', width:261, height:8, fill:c.surfaceBorder, cornerRadius:4 },
            { id:'dash:qwbf', name:'QWBarFG', type:'rectangle', width:164, height:8, fill:c.statusWarning, cornerRadius:4 },
          ]},
        { id:'dash:qwr', name:'QWReset', type:'text', content:'Reset: Friday 09:00', fontFamily:'Inter', fontSize:12, fontWeight:400, fill:c.textTertiary },
      ]},
    // Monthly
    { id:'dash:qm', name:'MonthlyCard', type:'frame', width:293, height:160, fill:c.surfaceAlt, cornerRadius:8, layout:'vertical', gap:12, padding:[16,16,16,16],
      children:[
        { id:'dash:qmt', name:'QMTitle', type:'text', content:'Monthly', fontFamily:'Inter', fontSize:13, fontWeight:600, fill:c.textSecondary },
        { id:'dash:qmp', name:'QMPct', type:'text', content:'45%', fontFamily:'Inter', fontSize:28, fontWeight:700, fill:c.textPrimary },
        { id:'dash:qmb', name:'QMBar', type:'frame', width:261, height:8,
          children:[
            { id:'dash:qmbb', name:'QMBarBG', type:'rectangle', width:261, height:8, fill:c.surfaceBorder, cornerRadius:4 },
            { id:'dash:qmbf', name:'QMBarFG', type:'rectangle', width:117, height:8, fill:c.accentBlue, cornerRadius:4 },
          ]},
      ]},
  ],
};

const tokenUsage = {
  id:'dash:tk', name:'TokenUsage', type:'frame', width:600, height:280, fill:c.surfaceAlt, cornerRadius:8, layout:'vertical', gap:16, padding:[16,20,16,20],
  children:[
    { id:'dash:tkh', name:'TKHeader', type:'text', content:'Token Usage', fontFamily:'Inter', fontSize:14, fontWeight:600, fill:c.textPrimary },
    // stats row
    { id:'dash:tks', name:'TKStats', type:'frame', width:560, height:48, layout:'horizontal', gap:16,
      children:[
        { id:'dash:tkt', name:'TKToday', type:'frame', width:170, height:48, layout:'vertical', gap:4,
          children:[
            { id:'dash:tktl', name:'TKTodayL', type:'text', content:'Today', fontFamily:'Inter', fontSize:11, fontWeight:500, fill:c.textTertiary },
            { id:'dash:tktv', name:'TKTodayV', type:'text', content:'8.5M', fontFamily:'Inter', fontSize:22, fontWeight:700, fill:c.textPrimary },
          ]},
        { id:'dash:tk7', name:'TK7d', type:'frame', width:170, height:48, layout:'vertical', gap:4,
          children:[
            { id:'dash:tk7l', name:'TK7dL', type:'text', content:'7 Days', fontFamily:'Inter', fontSize:11, fontWeight:500, fill:c.textTertiary },
            { id:'dash:tk7v', name:'TK7dV', type:'text', content:'42M', fontFamily:'Inter', fontSize:22, fontWeight:700, fill:c.textPrimary },
          ]},
        { id:'dash:tk3', name:'TK30d', type:'frame', width:170, height:48, layout:'vertical', gap:4,
          children:[
            { id:'dash:tk3l', name:'TK30dL', type:'text', content:'30 Days', fontFamily:'Inter', fontSize:11, fontWeight:500, fill:c.textTertiary },
            { id:'dash:tk3v', name:'TK30dV', type:'text', content:'180M', fontFamily:'Inter', fontSize:22, fontWeight:700, fill:c.textPrimary },
          ]},
      ]},
    // chart
    { id:'dash:tc', name:'TKChart', type:'frame', width:560, height:140, fill:c.surface, cornerRadius:6,
      children:[
        { id:'dash:tcg1', name:'Grid1', type:'rectangle', width:520, height:1, x:20, y:30, fill:c.surfaceBorder },
        { id:'dash:tcg2', name:'Grid2', type:'rectangle', width:520, height:1, x:20, y:60, fill:c.surfaceBorder },
        { id:'dash:tcg3', name:'Grid3', type:'rectangle', width:520, height:1, x:20, y:90, fill:c.surfaceBorder },
        { id:'dash:tcg4', name:'Grid4', type:'rectangle', width:520, height:1, x:20, y:120, fill:c.surfaceBorder },
        // area fill
        { id:'dash:tca', name:'Area', type:'path',
          geometry:'M 20 120 L 40 100 L 60 80 L 80 85 L 100 65 L 120 70 L 140 50 L 160 55 L 180 40 L 200 45 L 220 35 L 240 38 L 260 28 L 280 32 L 300 22 L 320 25 L 340 30 L 360 20 L 380 25 L 400 30 L 420 35 L 440 28 L 460 18 L 480 22 L 500 15 L 520 18 L 540 12 L 540 120 Z',
          fill:'rgba(74,158,255,0.12)' },
        // line
        { id:'dash:tcl', name:'Line', type:'path',
          geometry:'M 20 120 L 40 100 L 60 80 L 80 85 L 100 65 L 120 70 L 140 50 L 160 55 L 180 40 L 200 45 L 220 35 L 240 38 L 260 28 L 280 32 L 300 22 L 320 25 L 340 30 L 360 20 L 380 25 L 400 30 L 420 35 L 440 28 L 460 18 L 480 22 L 500 15 L 520 18 L 540 12',
          stroke:{ align:'center', thickness:2, fill:c.accentBlue }, fill:'none' },
        { id:'dash:tcx', name:'XLabel', type:'text', content:'Mon   Wed   Fri   Sun', fontFamily:'Inter', fontSize:9, fontWeight:400, fill:c.textTertiary, x:20, y:130 },
      ]},
  ],
};

const modelUsage = {
  id:'dash:md', name:'ModelUsage', type:'frame', width:296, height:280, fill:c.surfaceAlt, cornerRadius:8, layout:'vertical', gap:16, padding:[16,20,16,20],
  children:[
    { id:'dash:mdh', name:'MDHeader', type:'text', content:'Model Usage', fontFamily:'Inter', fontSize:14, fontWeight:600, fill:c.textPrimary },
    // GPT
    { id:'dash:mdg', name:'MD_GPT', type:'frame', width:256, height:60, layout:'vertical', gap:8,
      children:[
        { id:'dash:mdgh', name:'MD_GPT_H', type:'frame', width:256, height:16, layout:'horizontal', justifyContent:'space-between',
          children:[
            { id:'dash:mdgn', name:'MD_GPT_N', type:'text', content:'GPT', fontFamily:'Inter', fontSize:13, fontWeight:500, fill:c.textPrimary },
            { id:'dash:mdgp', name:'MD_GPT_P', type:'text', content:'60%', fontFamily:'Inter', fontSize:13, fontWeight:500, fill:c.accentBlue },
          ]},
        { id:'dash:mdgb', name:'MD_GPT_B', type:'frame', width:256, height:8,
          children:[
            { id:'dash:mdgbb', name:'MD_GPT_BB', type:'rectangle', width:256, height:8, fill:c.surfaceBorder, cornerRadius:4 },
            { id:'dash:mdgbf', name:'MD_GPT_BF', type:'rectangle', width:154, height:8, fill:c.accentBlue, cornerRadius:4 },
          ]},
      ]},
    // Claude
    { id:'dash:mdc', name:'MD_Claude', type:'frame', width:256, height:60, layout:'vertical', gap:8,
      children:[
        { id:'dash:mdch', name:'MD_Claude_H', type:'frame', width:256, height:16, layout:'horizontal', justifyContent:'space-between',
          children:[
            { id:'dash:mdcn', name:'MD_Claude_N', type:'text', content:'Claude', fontFamily:'Inter', fontSize:13, fontWeight:500, fill:c.textPrimary },
            { id:'dash:mdcp', name:'MD_Claude_P', type:'text', content:'40%', fontFamily:'Inter', fontSize:13, fontWeight:500, fill:'#d97706' },
          ]},
        { id:'dash:mdcb', name:'MD_Claude_B', type:'frame', width:256, height:8,
          children:[
            { id:'dash:mdcbb', name:'MD_Claude_BB', type:'rectangle', width:256, height:8, fill:c.surfaceBorder, cornerRadius:4 },
            { id:'dash:mdcbf', name:'MD_Claude_BF', type:'rectangle', width:102, height:8, fill:'#d97706', cornerRadius:4 },
          ]},
      ]},
  ],
};

const dashboard = {
  id:'page:dash', name:'Dashboard', type:'frame',
  width:1200, height:800, fill:c.surface,
  children:[
    // Sidebar
    { id:'dash:sidebar', name:'Sidebar', type:'frame', width:SBW, height:800, fill:c.surfaceAlt,
      children:[
        { id:'dash:logo-area', name:'LogoArea', type:'frame', width:SBW, height:HH, layout:'horizontal', gap:10, padding:[0,16,0,16], alignItems:'center',
          children:[
            { id:'dash:logo-icon', name:'LogoIcon', type:'rectangle', width:20, height:20, fill:c.accentBlue, cornerRadius:5 },
            { id:'dash:logo-text', name:'LogoText', type:'text', content:'OpenCode Usage Float', fontFamily:'Inter', fontSize:13, fontWeight:600, fill:c.textPrimary },
          ]},
        { id:'dash:sd1', name:'Div1', type:'rectangle', width:208, height:1, x:16, y:52, fill:c.surfaceBorder },
        { id:'dash:menu1', name:'MenuDashboard', type:'ref', ref:'comp:si', x:10, y:68, descendants:{ 'si:label':{ content:'Dashboard' }, 'si:icon':{ fill:c.accentBlue } } },
        { id:'dash:menu2', name:'MenuUsage', type:'ref', ref:'comp:si', x:10, y:108, descendants:{ 'si:label':{ content:'Usage History' } } },
        { id:'dash:menu3', name:'MenuModels', type:'ref', ref:'comp:si', x:10, y:148, descendants:{ 'si:label':{ content:'Models' } } },
        { id:'dash:sd2', name:'Div2', type:'rectangle', width:208, height:1, x:16, y:200, fill:c.surfaceBorder },
        { id:'dash:menu4', name:'MenuSettings', type:'ref', ref:'comp:si', x:10, y:216, descendants:{ 'si:label':{ content:'Settings' } } },
      ]},
    // Header
    { id:'dash:header', name:'Header', type:'frame', width:CW, height:HH, x:CX, fill:c.surface, layout:'horizontal', justifyContent:'space-between', alignItems:'center', padding:[0,20,0,24],
      children:[
        { id:'dash:hl', name:'HLeft', type:'frame', width:200, height:HH, layout:'horizontal', gap:10, alignItems:'center',
          children:[
            { id:'dash:hl-text', name:'HLText', type:'text', content:'OpenCode GO', fontFamily:'Inter', fontSize:16, fontWeight:600, fill:c.textPrimary },
          ]},
        { id:'dash:hr', name:'HRight', type:'frame', width:100, height:HH, layout:'horizontal', gap:4, alignItems:'center', justifyContent:'flex-end',
          children:[
            { id:'dash:set-btn', name:'SetBtn', type:'frame', width:28, height:28, cornerRadius:6,
              children:[
                { id:'dash:set-gear', name:'Gear', type:'ellipse', width:14, height:14, x:7, y:7, fill:'none', stroke:{ align:'inside', thickness:2, fill:c.textSecondary } },
              ]},
            { id:'dash:min-btn', name:'MinBtn', type:'frame', width:28, height:28, cornerRadius:6,
              children:[
                { id:'dash:min-line', name:'MinLn', type:'rectangle', width:10, height:2, fill:c.textSecondary, x:9, y:13, cornerRadius:1 },
              ]},
            { id:'dash:close-btn', name:'CloseBtn', type:'frame', width:28, height:28, cornerRadius:6,
              children:[
                { id:'dash:close-x', name:'CloseX', type:'text', content:X, fontFamily:'Inter', fontSize:14, fontWeight:400, fill:c.textSecondary, x:14, y:14, textAlign:'center', textAlignVertical:'center' },
              ]},
          ]},
      ]},
    // Content area
    { id:'dash:content', name:'Content', type:'frame', width:CW, height:CH, x:CX, y:CY, fill:c.surface, layout:'vertical', gap:20, padding:[24,24,24,24],
      children:[ planCard, quotaGrid,
        { id:'dash:br', name:'BottomRow', type:'frame', width:912, height:280, layout:'horizontal', gap:16,
          children:[ tokenUsage, modelUsage ]},
      ]},
  ],
};

// ============================================================
// PAGE 3 — Settings (480×440)
// ============================================================
const settings = {
  id:'page:settings', name:'Settings', type:'frame',
  width:480, height:440, fill:c.surfaceAlt, cornerRadius:12,
  effect:{ type:'drop-shadow', color:'rgba(0,0,0,0.5)', offset:{x:0,y:16}, blur:48 },
  children:[
    // Title bar
    { id:'set:tb', name:'TitleBar', type:'frame', width:480, height:48, layout:'horizontal', justifyContent:'space-between', alignItems:'center', padding:[0,16,0,20],
      children:[
        { id:'set:tt', name:'Title', type:'text', content:'Settings', fontFamily:'Inter', fontSize:16, fontWeight:600, fill:c.textPrimary },
        { id:'set:tc', name:'CloseBtn', type:'text', content:X, fontFamily:'Inter', fontSize:14, fontWeight:400, fill:c.textTertiary },
      ]},
    { id:'set:d1', name:'Div1', type:'rectangle', width:480, height:1, y:48, fill:c.surfaceBorder },
    // General
    { id:'set:sg', name:'SectionGeneral', type:'frame', width:480, height:110, y:49, layout:'vertical', gap:8, padding:[16,20,16,20],
      children:[
        { id:'set:sgh', name:'SGHeader', type:'text', content:'General', fontFamily:'Inter', fontSize:13, fontWeight:600, fill:c.textSecondary },
        { id:'set:sga', name:'SGAutoStart', type:'frame', width:440, height:32, layout:'horizontal', justifyContent:'space-between', alignItems:'center',
          children:[
            { id:'set:sg-al', name:'SGASLabel', type:'text', content:'Launch at startup', fontFamily:'Inter', fontSize:13, fontWeight:400, fill:c.textPrimary },
            { id:'set:sg-at', name:'SGASToggle', type:'frame', width:36, height:20, cornerRadius:10, fill:c.accentBlue,
              children:[
                { id:'set:sg-ak', name:'SGASKnob', type:'ellipse', width:16, height:16, x:18, y:2, fill:'#ffffff' },
              ]},
          ]},
        { id:'set:sgr', name:'SGRefresh', type:'frame', width:440, height:32, layout:'horizontal', gap:12, alignItems:'center',
          children:[
            { id:'set:sgrl', name:'SGRLabel', type:'text', content:'Auto refresh', fontFamily:'Inter', fontSize:13, fontWeight:400, fill:c.textPrimary },
            { id:'set:sgr5', name:'SGR5', type:'frame', width:40, height:24, cornerRadius:4, fill:c.accentBlue, layout:'horizontal', padding:[0,8,0,8], alignItems:'center',
              children:[
                { id:'set:sgr5t', name:'SGR5T', type:'text', content:'5m', fontFamily:'Inter', fontSize:12, fontWeight:500, fill:'#ffffff' },
              ]},
            { id:'set:sgr30', name:'SGR30', type:'frame', width:44, height:24, cornerRadius:4, fill:c.surfaceHover, layout:'horizontal', padding:[0,8,0,8], alignItems:'center',
              children:[
                { id:'set:sgr30t', name:'SGR30T', type:'text', content:'30m', fontFamily:'Inter', fontSize:12, fontWeight:500, fill:c.textSecondary },
              ]},
            { id:'set:sgr60', name:'SGR60', type:'frame', width:44, height:24, cornerRadius:4, fill:c.surfaceHover, layout:'horizontal', padding:[0,8,0,8], alignItems:'center',
              children:[
                { id:'set:sgr60t', name:'SGR60T', type:'text', content:'60m', fontFamily:'Inter', fontSize:12, fontWeight:500, fill:c.textSecondary },
              ]},
          ]},
      ]},
    { id:'set:d2', name:'Div2', type:'rectangle', width:480, height:1, y:159, fill:c.surfaceBorder },
    // Display
    { id:'set:sd', name:'SectionDisplay', type:'frame', width:480, height:110, y:160, layout:'vertical', gap:8, padding:[16,20,16,20],
      children:[
        { id:'set:sdh', name:'SDHeader', type:'text', content:'Display', fontFamily:'Inter', fontSize:13, fontWeight:600, fill:c.textSecondary },
        { id:'set:sdf', name:'SDFloat', type:'frame', width:440, height:32, layout:'horizontal', justifyContent:'space-between', alignItems:'center',
          children:[
            { id:'set:sdfl', name:'SDFLabel', type:'text', content:'Floating widget', fontFamily:'Inter', fontSize:13, fontWeight:400, fill:c.textPrimary },
            { id:'set:sdft', name:'SDFToggle', type:'frame', width:36, height:20, cornerRadius:10, fill:c.accentBlue,
              children:[
                { id:'set:sdfk', name:'SDFKnob', type:'ellipse', width:16, height:16, x:18, y:2, fill:'#ffffff' },
              ]},
          ]},
        { id:'set:sdt', name:'SDTheme', type:'frame', width:440, height:32, layout:'horizontal', justifyContent:'space-between', alignItems:'center',
          children:[
            { id:'set:sdtl', name:'SDTLabel', type:'text', content:'Theme', fontFamily:'Inter', fontSize:13, fontWeight:400, fill:c.textPrimary },
            { id:'set:sdtv', name:'SDTValue', type:'frame', width:80, height:28, cornerRadius:4, fill:c.surfaceHover, layout:'horizontal', padding:[0,10,0,10], alignItems:'center', gap:4,
              children:[
                { id:'set:sdtt', name:'SDTText', type:'text', content:'Dark', fontFamily:'Inter', fontSize:12, fontWeight:500, fill:c.textPrimary },
                { id:'set:sdta', name:'SDTArrow', type:'text', content:'>', fontFamily:'Inter', fontSize:14, fontWeight:400, fill:c.textTertiary },
              ]},
          ]},
      ]},
    { id:'set:d3', name:'Div3', type:'rectangle', width:480, height:1, y:270, fill:c.surfaceBorder },
    // Privacy
    { id:'set:sp', name:'SectionPrivacy', type:'frame', width:480, height:80, y:271, layout:'vertical', gap:8, padding:[16,20,16,20],
      children:[
        { id:'set:sph', name:'SPHeader', type:'text', content:'Privacy', fontFamily:'Inter', fontSize:13, fontWeight:600, fill:c.textSecondary },
        { id:'set:spb', name:'SPBadge', type:'frame', width:440, height:32, layout:'horizontal', gap:8, alignItems:'center',
          children:[
            { id:'set:spl', name:'SPLock', type:'rectangle', width:14, height:16, fill:c.textTertiary, cornerRadius:2 },
            { id:'set:spt', name:'SPText', type:'text', content:'Local data only - No cloud sync', fontFamily:'Inter', fontSize:12, fontWeight:400, fill:c.textSecondary },
          ]},
      ]},
  ],
};

// ============================================================
// ASSEMBLE & WRITE
// ============================================================
// Components page (contains reusable components)
const compPage = {
  id:'page:comps', name:'Components', type:'frame',
  width: 1200, height: 800, fill: c.surface,
  children: comps,
};

const doc = {
  version: '1',
  children: [
    compPage,
    floatWidget,
    dashboard,
    settings,
  ],
};

fs.writeFileSync('D:/projects/Usage-Float/opencode-usage-float.pen', JSON.stringify(doc), 'utf8');
console.log('Written opencode-usage-float.pen');
