// ============================================================================
// Constants
// ============================================================================

const CONFIG = {
    LOG_MAX_ENTRIES: 100,
    CAP_BITS_COUNT: 16,
    SYSTEM_READ_SCOPE: '_type:_type',
    SYSTEM_READ_BIT: 0x2000,
};

// ============================================================================
// State
// ============================================================================

let apiUrl = '';
let connected = false;
let currentViewer = 'user:root';
let authToken = localStorage.getItem('capbit_token') || null;
let viewerCanSeeSystem = false;
let modalBits = 0;
let selectedBits = 0;

const known = {
    types: [],
    entities: [],
    grants: [],
    capabilities: [],
    delegations: [],
    capLabels: []
};

// ============================================================================
// DOM Helpers
// ============================================================================

const $ = (id) => document.getElementById(id);
const $$ = (sel) => document.querySelectorAll(sel);

const setHtml = (id, html) => { const el = $(id); if (el) el.innerHTML = html; };
const setValue = (id, val) => { const el = $(id); if (el) el.value = val; };
const getValue = (id, trim = true) => { const el = $(id); return el ? (trim ? el.value.trim() : el.value) : ''; };

const entitySelectHtml = (placeholder = '-- Select entity --') =>
    `<option value="">${placeholder}</option>` +
    known.entities.map(e => `<option value="${e.id}">${e.id}</option>`).join('');

const setEntitySelects = (...ids) => {
    const html = entitySelectHtml();
    ids.forEach(id => setHtml(id, html));
};

const showError = (msg) => {
    log(`Error: ${msg}`, 'error');
    alert(msg);
};

const renderListOrEmpty = (containerId, items, emptyIcon, emptyMsg, itemRenderer) => {
    if (!items.length) {
        setHtml(containerId, `<div class="empty"><div class="empty-icon">${emptyIcon}</div><p>${emptyMsg}</p></div>`);
        return;
    }
    setHtml(containerId, items.map(itemRenderer).join(''));
};

// ============================================================================
// UI Helpers
// ============================================================================

const chip = (entityId) => {
    const [type, id] = entityId.split(':');
    return `<span class="chip ${type}"><span class="type">${type}:</span>${id}</span>`;
};

const arrow = (text = '→') => `<span class="arrow">${text}</span>`;

const badge = (text, className = 'relation-badge') =>
    `<span class="${className}">${text}</span>`;

const capBadge = (mask) =>
    `<span class="cap-badge">0x${mask.toString(16).padStart(4, '0')}</span>`;

const systemIcon = () => '<span title="System internal" class="system-icon">⚙</span>';

// ============================================================================
// Template Data (Declarative)
// ============================================================================

const SYSTEM_CAP_BITS = [
    { bit: 0, label: 'type-create' },
    { bit: 1, label: 'type-delete' },
    { bit: 2, label: 'entity-create' },
    { bit: 3, label: 'entity-delete' },
    { bit: 4, label: 'grant-read' },
    { bit: 5, label: 'grant-write' },
    { bit: 6, label: 'grant-delete' },
    { bit: 7, label: 'cap-read' },
    { bit: 8, label: 'cap-write' },
    { bit: 9, label: 'cap-delete' },
    { bit: 10, label: 'delegate-read' },
    { bit: 11, label: 'delegate-write' },
    { bit: 12, label: 'delegate-delete' },
    { bit: 13, label: 'system-read' },
];

const TEMPLATES = {
    startup: {
        bootstrap: { root_id: 'root' },
        capLabels: [
            { scope: '_type:_type', bits: SYSTEM_CAP_BITS },
            { scope: '_type:resource', bits: [
                { bit: 0, label: 'enter' },
                { bit: 1, label: 'print' },
                { bit: 2, label: 'fax' },
                { bit: 3, label: 'safe' },
                { bit: 4, label: 'server' },
                { bit: 5, label: 'can-grant' },
            ]}
        ],
        entities: [
            { type: 'user', ids: ['alice', 'bob', 'charlie', 'dana'] },
            { type: 'resource', ids: ['hq-office'] }
        ],
        capabilities: [
            { scope: 'resource:hq-office', defs: [
                ['visitor', 0x01],
                ['employee', 0x07],
                ['manager', 0x0F],
                ['owner', 0x3F],
                ['delegator', 0x0800]
            ]}
        ],
        grants: [
            ['user:alice', 'owner', 'resource:hq-office'],
            ['user:bob', 'employee', 'resource:hq-office'],
            ['user:charlie', 'visitor', 'resource:hq-office'],
            ['user:alice', 'delegator', 'resource:hq-office'],
        ],
        delegations: [
            ['user:dana', 'resource:hq-office', 'user:alice']
        ],
        summary: [
            'Created: alice, bob, charlie, dana + hq-office',
            'Roles: visitor=0x01, employee=0x07, manager=0x0F, owner=0x3F',
            'alice: owner (0x3F), bob: employee (0x07), charlie: visitor (0x01)',
            'dana: inherits owner from alice',
            'Test: bob on hq-office with 0x04 (fax) -> ALLOWED'
        ]
    },
    saas: {
        bootstrap: { root_id: 'admin' },
        capLabels: [
            { scope: '_type:_type', bits: SYSTEM_CAP_BITS },
            { scope: '_type:app', bits: [
                { bit: 0, label: 'read' },
                { bit: 1, label: 'write' },
                { bit: 2, label: 'delete' },
                { bit: 3, label: 'bulk' },
                { bit: 4, label: 'webhooks' },
                { bit: 5, label: 'export' },
                { bit: 6, label: 'admin' },
                { bit: 7, label: 'unlimited' },
            ]},
            { scope: '_type:team', bits: [
                { bit: 0, label: 'view-org' },
                { bit: 1, label: 'invite' },
                { bit: 2, label: 'billing' },
                { bit: 3, label: 'settings' },
            ]}
        ],
        entities: [
            { type: 'team', ids: ['acme', 'globex'] },
            { type: 'user', ids: ['alice', 'bob', 'charlie', 'dana', 'contractor'] },
            { type: 'app', ids: ['api-gateway', 'dashboard', 'analytics'] }
        ],
        capabilities: [
            { scope: 'app:api-gateway', defs: [
                ['basic', 0x03],
                ['pro', 0x1F],
                ['enterprise', 0xFF],
                ['delegate', 0x0800]
            ]},
            { scope: 'app:dashboard', defs: [
                ['viewer', 0x01],
                ['analyst', 0x07],
                ['manager', 0x0F]
            ]},
            { scope: 'team:acme', defs: [
                ['member', 0x01],
                ['admin', 0x03],
                ['owner', 0x0F]
            ]},
            { scope: 'team:globex', defs: [
                ['member', 0x01],
                ['admin', 0x03],
                ['owner', 0x0F]
            ]}
        ],
        grants: [
            ['user:alice', 'enterprise', 'app:api-gateway'],
            ['user:alice', 'manager', 'app:dashboard'],
            ['user:alice', 'owner', 'team:acme'],
            ['user:alice', 'delegate', 'app:api-gateway'],
            ['user:bob', 'pro', 'app:api-gateway'],
            ['user:bob', 'analyst', 'app:dashboard'],
            ['user:bob', 'member', 'team:acme'],
            ['user:charlie', 'basic', 'app:api-gateway'],
            ['user:charlie', 'viewer', 'app:dashboard'],
            ['user:charlie', 'owner', 'team:globex'],
            ['user:dana', 'member', 'team:globex'],
        ],
        delegations: [
            ['user:contractor', 'app:api-gateway', 'user:alice']
        ],
        summary: [
            'Created: alice, bob, charlie, dana, contractor + apps + teams',
            'alice: enterprise (api), manager (dash), owner (acme)',
            'bob: pro (api), analyst (dash), member (acme)',
            'charlie: basic (api), viewer (dash), owner (globex)',
            'contractor: inherits enterprise from alice on api-gateway'
        ]
    }
};

// ============================================================================
// Logging
// ============================================================================

function log(msg, type = '') {
    const logDiv = $('log');
    const time = new Date().toLocaleTimeString();
    const entry = document.createElement('div');
    entry.className = `log-entry ${type}`;
    entry.innerHTML = `<span class="time">${time}</span>${msg}`;
    logDiv.insertBefore(entry, logDiv.firstChild);
    while (logDiv.children.length > CONFIG.LOG_MAX_ENTRIES) {
        logDiv.removeChild(logDiv.lastChild);
    }
}

function logRequest(method, endpoint, body = null) {
    let msg = `<span class="method">${method}</span> ${endpoint}`;
    if (body) msg += ` <span style="color: var(--text-muted)">${JSON.stringify(body)}</span>`;
    log(msg, 'request');
}

function logResponse(success, data) {
    log(success ? `✓ ${JSON.stringify(data)}` : `✗ ${data}`, success ? 'success' : 'error');
}

function clearLog() {
    setHtml('log', '');
    log('Log cleared', 'info');
}

function copyLogs() {
    const entries = [...$$('#log .log-entry')].reverse();
    const text = entries.map(e => e.textContent).join('\n');
    navigator.clipboard?.writeText(text)
        .then(() => log('Logs copied!', 'success'))
        .catch(() => fallbackCopy(text)) || fallbackCopy(text);
}

function fallbackCopy(text) {
    const textarea = document.createElement('textarea');
    textarea.value = text;
    textarea.style.cssText = 'position:fixed;opacity:0';
    document.body.appendChild(textarea);
    textarea.select();
    document.execCommand('copy');
    document.body.removeChild(textarea);
    log('Logs copied!', 'success');
}

// ============================================================================
// API
// ============================================================================

async function api(method, endpoint, body = null) {
    logRequest(method, endpoint, body);
    try {
        const headers = { 'Content-Type': 'application/json' };
        if (authToken) headers['Authorization'] = `Bearer ${authToken}`;
        const opts = { method, headers };
        if (body) opts.body = JSON.stringify(body);
        const res = await fetch(`${apiUrl}${endpoint}`, opts);
        const json = await res.json();
        if (json.success) {
            logResponse(true, json.data);
            return { ok: true, data: json.data };
        }
        logResponse(false, json.error);
        return { ok: false, error: json.error };
    } catch (e) {
        logResponse(false, e.message);
        return { ok: false, error: e.message };
    }
}

// ============================================================================
// Connection & Auth
// ============================================================================

async function checkConnection() {
    apiUrl = '';
    log(`Connecting to ${window.location.origin}...`, 'info');
    const result = await api('GET', '/status');
    const dot = $('status-dot');
    if (result.ok) {
        connected = true;
        dot.classList.add('connected');
        dot.title = result.data.bootstrapped
            ? `Connected (root: ${result.data.root_entity})`
            : 'Connected (not bootstrapped)';
        updateBootstrapUI(result.data.bootstrapped, result.data.root_entity);
        await checkAuth();
        await loadData();
    } else {
        connected = false;
        dot.classList.remove('connected');
    }
}

function updateBootstrapUI(bootstrapped, rootEntity) {
    const form = $('bootstrap-form');
    const status = $('bootstrap-status');
    if (bootstrapped) {
        form.style.display = 'none';
        status.innerHTML = `System bootstrapped. Root: <strong>${rootEntity}</strong>`;
        status.style.color = 'var(--success)';
    } else {
        form.style.display = 'block';
        status.textContent = 'System not yet bootstrapped. Create a root user to get started.';
    }
}

async function checkAuth() {
    if (!authToken) { updateAuthUI(false, null); return; }
    const result = await api('GET', '/me');
    if (result.ok) {
        currentViewer = result.data.entity;
        updateAuthUI(true, result.data.entity);
    } else {
        authToken = null;
        localStorage.removeItem('capbit_token');
        localStorage.removeItem('capbit_entity');
        updateAuthUI(false, null);
    }
}

function updateAuthUI(loggedIn, entity) {
    const authStatus = $('auth-status');
    const logoutBtn = $('logout-btn');
    const loggedOutDiv = $('auth-logged-out');
    const loggedInDiv = $('auth-logged-in');
    const setPasswordCard = $('set-password-card');

    if (loggedIn) {
        authStatus.textContent = entity;
        authStatus.style.color = 'var(--success)';
        logoutBtn.style.display = 'block';
        loggedOutDiv.style.display = 'none';
        loggedInDiv.style.display = 'block';
        setPasswordCard.style.display = 'block';
        $('logged-in-entity').textContent = entity;
        $('current-token').textContent = authToken.substring(0, 20) + '...';
    } else {
        authStatus.textContent = 'Not logged in';
        authStatus.style.color = 'var(--text-muted)';
        logoutBtn.style.display = 'none';
        loggedOutDiv.style.display = 'block';
        loggedInDiv.style.display = 'none';
        setPasswordCard.style.display = 'none';
    }
}

async function doLogin() {
    const token = getValue('login-token');
    if (!token) return showError('Enter a token');
    authToken = token;
    const result = await api('GET', '/me');
    if (result.ok) {
        localStorage.setItem('capbit_token', token);
        localStorage.setItem('capbit_entity', result.data.entity);
        currentViewer = result.data.entity;
        updateAuthUI(true, result.data.entity);
        log(`Logged in as ${result.data.entity}`, 'success');
        setValue('login-token', '');
        await loadData();
    } else {
        authToken = null;
        showError('Invalid token');
    }
}

async function doPasswordLogin() {
    let entityId = getValue('login-entity');
    const password = getValue('login-password', false);
    if (!entityId) return showError('Enter your username');
    if (!entityId.includes(':')) entityId = `user:${entityId}`;
    const result = await api('POST', '/login', { entity_id: entityId, password });
    if (result.ok) {
        authToken = result.data.token;
        localStorage.setItem('capbit_token', authToken);
        localStorage.setItem('capbit_entity', result.data.root_entity);
        currentViewer = result.data.root_entity;
        updateAuthUI(true, result.data.root_entity);
        log(`Logged in as ${result.data.root_entity}`, 'success');
        setValue('login-entity', '');
        setValue('login-password', '');
        await loadData();
    } else {
        showError('Login failed: ' + (result.error || 'Invalid credentials'));
    }
}

async function doSetPassword() {
    let entityId = getValue('set-password-entity');
    const password = getValue('set-password-value', false);
    if (!entityId || !password) return showError('Enter username and password');
    if (!entityId.includes(':')) entityId = `user:${entityId}`;
    const result = await api('POST', '/password', { entity_id: entityId, password });
    if (result.ok) {
        log(`Password set for ${entityId}`, 'success');
        setValue('set-password-entity', '');
        setValue('set-password-value', '');
    } else {
        showError('Failed: ' + (result.error || 'Unknown error'));
    }
}

async function doLogout() {
    authToken = null;
    localStorage.removeItem('capbit_token');
    localStorage.removeItem('capbit_entity');
    viewerCanSeeSystem = false;
    currentViewer = '';
    known.entities = [];
    known.grants = [];
    known.capabilities = [];
    known.delegations = [];
    known.capLabels = [];
    updateAuthUI(false, null);
    renderAll();
    log('Logged out', 'info');
    const status = await api('GET', '/status');
    if (status.ok) updateBootstrapUI(status.data.bootstrapped, status.data.root_entity);
}

function doCopyToken() {
    if (authToken) {
        navigator.clipboard.writeText(authToken).then(() => log('Token copied!', 'success')).catch(() => alert('Token: ' + authToken));
    }
}

async function loadData() {
    const [types, entities, grants, caps, labels, delegs] = await Promise.all([
        api('GET', '/types'),
        api('GET', '/entities'),
        api('GET', '/grants'),
        api('GET', '/capabilities'),
        api('GET', '/cap-labels'),
        api('GET', '/delegations')
    ]);
    if (types.ok) known.types = types.data;
    if (entities.ok) known.entities = entities.data.map(e => ({ id: e.id, type: e.entity_type }));
    if (grants.ok) known.grants = grants.data.map(g => ({ seeker: g.seeker, relation: g.relation, scope: g.scope }));
    if (caps.ok) known.capabilities = caps.data.map(c => ({ scope: c.scope, relation: c.relation, cap_mask: c.cap_mask }));
    if (labels.ok) known.capLabels = labels.data.map(l => ({ scope: l.scope, bit: l.bit, label: l.label }));
    if (delegs.ok) known.delegations = delegs.data.map(d => ({ seeker: d.seeker, scope: d.scope, delegate: d.source }));
    await updateViewerPermission();
}

// ============================================================================
// Actions
// ============================================================================

async function doBootstrap() {
    const rootId = document.getElementById('bootstrap-root').value.trim();
    if (!rootId) return alert('Enter a root ID');
    const password = document.getElementById('bootstrap-password').value;
    const body = { root_id: rootId };
    if (password) body.password = password;
    const result = await api('POST', '/bootstrap', body);
    if (result.ok) {
        authToken = result.data.token;
        localStorage.setItem('capbit_token', authToken);
        localStorage.setItem('capbit_entity', result.data.root_entity);
        currentViewer = result.data.root_entity;
        log(`Logged in as ${currentViewer}`, 'success');
        await checkConnection();
    }
}

async function doReset() {
    if (!confirm('Reset entire database? This cannot be undone.')) return;
    const result = await api('POST', '/reset');
    if (result.ok) {
        authToken = null;
        localStorage.removeItem('capbit_token');
        localStorage.removeItem('capbit_entity');
        known.types = [];
        known.entities = [];
        known.grants = [];
        known.capabilities = [];
        known.delegations = [];
        known.capLabels = [];
        clearLog();
        log('Database reset complete', 'success');
        renderAll();
        await checkConnection();
    }
}

// ============================================================================
// Query Functions (11 items)
// ============================================================================

// 1. Types - show entities of type
function qType() {
    const type = getValue('q-type');
    const div = $('q-types-result');
    if (!type) { div.innerHTML = ''; return; }
    const entities = known.entities.filter(e => e.type === type);
    if (entities.length === 0) { div.innerHTML = '<p class="text-muted mt-1">No entities of this type</p>'; return; }
    div.innerHTML = `<div class="list mt-1">${entities.map(e => `<div class="list-item">${chip(e.id)}</div>`).join('')}</div>`;
}

// 2. Entities - show details
function qEntity() {
    const entity = getValue('q-entity');
    const div = $('q-entities-result');
    if (!entity) { div.innerHTML = ''; return; }

    // Get memberships, groups, sharing for this entity
    const memberships = known.grants.filter(g => g.seeker === entity);
    const groups = known.capabilities.filter(c => c.scope === entity);
    const sharesRecv = known.delegations.filter(d => d.seeker === entity);
    const sharesGiven = known.delegations.filter(d => d.delegate === entity);

    let html = '<div class="mt-1">';
    if (memberships.length > 0) {
        html += `<p class="text-muted text-sm">Members of:</p><div class="list">${memberships.map(g =>
            `<div class="list-item">${badge(g.relation)} on ${chip(g.scope)}</div>`).join('')}</div>`;
    }
    if (groups.length > 0) {
        html += `<p class="text-muted text-sm mt-1">Groups defined:</p><div class="list">${groups.map(g =>
            `<div class="list-item">${badge(g.relation)} ${capBadge(g.cap_mask)}</div>`).join('')}</div>`;
    }
    if (sharesRecv.length > 0) {
        html += `<p class="text-muted text-sm mt-1">Shares received:</p><div class="list">${sharesRecv.map(d =>
            `<div class="list-item delegation-item">from ${chip(d.delegate)} on ${chip(d.scope)}</div>`).join('')}</div>`;
    }
    if (sharesGiven.length > 0) {
        html += `<p class="text-muted text-sm mt-1">Shares given:</p><div class="list">${sharesGiven.map(d =>
            `<div class="list-item delegation-item">to ${chip(d.seeker)} on ${chip(d.scope)}</div>`).join('')}</div>`;
    }
    if (memberships.length === 0 && groups.length === 0 && sharesRecv.length === 0 && sharesGiven.length === 0) {
        html += '<p class="text-muted">No data for this entity</p>';
    }
    html += '</div>';
    div.innerHTML = html;
}

// 3. Actions by Type
function qActionsByType() {
    const scope = getValue('q-act-type');
    const div = $('q-actions-result');
    if (!scope) { div.innerHTML = ''; return; }
    const actions = known.capLabels.filter(l => l.scope === scope).sort((a, b) => a.bit - b.bit);
    if (actions.length === 0) { div.innerHTML = '<p class="text-muted mt-1">No actions defined</p>'; return; }
    div.innerHTML = `<div class="list mt-1">${actions.map(a =>
        `<div class="list-item"><span class="cap-badge cap-badge-success">bit${a.bit}</span> <strong>${a.label}</strong> ${capBadge(1 << a.bit)}</div>`
    ).join('')}</div>`;
}

// 4. Groups by Entity
function qGroupsByEntity() {
    const entity = getValue('q-grp-ent');
    const div = $('q-groups-ent-result');
    if (!entity) { div.innerHTML = ''; return; }
    const groups = known.capabilities.filter(c => c.scope === entity);
    if (groups.length === 0) { div.innerHTML = '<p class="text-muted mt-1">No groups defined</p>'; return; }
    div.innerHTML = `<div class="list mt-1">${groups.map(g =>
        `<div class="list-item">${badge(g.relation)} ${capBadge(g.cap_mask)}</div>`
    ).join('')}</div>`;
}

// 5. Groups by Role
function qGroupsByRole() {
    const role = getValue('q-grp-role');
    const div = $('q-groups-role-result');
    if (!role) { div.innerHTML = ''; return; }
    const groups = known.capabilities.filter(c => c.relation === role);
    if (groups.length === 0) { div.innerHTML = '<p class="text-muted mt-1">No entities with this role</p>'; return; }
    div.innerHTML = `<div class="list mt-1">${groups.map(g =>
        `<div class="list-item">${chip(g.scope)} ${capBadge(g.cap_mask)}</div>`
    ).join('')}</div>`;
}

// 6. Members: By Who
function qMembersByWho() {
    const who = getValue('q-mem-who');
    const div = $('q-mem-who-result');
    if (!who) { div.innerHTML = ''; return; }
    const members = known.grants.filter(g => g.seeker === who);
    if (members.length === 0) { div.innerHTML = '<p class="text-muted mt-1">No memberships</p>'; return; }
    div.innerHTML = `<div class="list mt-1">${members.map(g =>
        `<div class="list-item">${badge(g.relation)} ${arrow()} ${chip(g.scope)}</div>`
    ).join('')}</div>`;
}

// 7. Members: By What
function qMembersByWhat() {
    const what = getValue('q-mem-what');
    const div = $('q-mem-what-result');
    if (!what) { div.innerHTML = ''; return; }
    const members = known.grants.filter(g => g.scope === what);
    if (members.length === 0) { div.innerHTML = '<p class="text-muted mt-1">No members</p>'; return; }
    div.innerHTML = `<div class="list mt-1">${members.map(g =>
        `<div class="list-item">${chip(g.seeker)} ${arrow()} ${badge(g.relation)}</div>`
    ).join('')}</div>`;
}

// 8. Members: By Role
function qMembersByRole() {
    const role = getValue('q-mem-role');
    const div = $('q-mem-role-result');
    if (!role) { div.innerHTML = ''; return; }
    const members = known.grants.filter(g => g.relation === role);
    if (members.length === 0) { div.innerHTML = '<p class="text-muted mt-1">No grants with this role</p>'; return; }
    div.innerHTML = `<div class="list mt-1">${members.map(g =>
        `<div class="list-item">${chip(g.seeker)} ${arrow()} ${chip(g.scope)}</div>`
    ).join('')}</div>`;
}

// 9. Sharing: By Receiver
function qSharingByRecv() {
    const recv = getValue('q-sh-recv');
    const div = $('q-sh-recv-result');
    if (!recv) { div.innerHTML = ''; return; }
    const shares = known.delegations.filter(d => d.seeker === recv);
    if (shares.length === 0) { div.innerHTML = '<p class="text-muted mt-1">No shares received</p>'; return; }
    div.innerHTML = `<div class="list mt-1">${shares.map(d =>
        `<div class="list-item delegation-item">${chip(d.delegate)} ${arrow()} ${chip(d.scope)}</div>`
    ).join('')}</div>`;
}

// 10. Sharing: By Giver
function qSharingByGiver() {
    const giver = getValue('q-sh-giver');
    const div = $('q-sh-giver-result');
    if (!giver) { div.innerHTML = ''; return; }
    const shares = known.delegations.filter(d => d.delegate === giver);
    if (shares.length === 0) { div.innerHTML = '<p class="text-muted mt-1">No shares given</p>'; return; }
    div.innerHTML = `<div class="list mt-1">${shares.map(d =>
        `<div class="list-item delegation-item">${chip(d.seeker)} ${arrow('on')} ${chip(d.scope)}</div>`
    ).join('')}</div>`;
}

// 11. Sharing: By Resource
function qSharingByRes() {
    const res = getValue('q-sh-res');
    const div = $('q-sh-res-result');
    if (!res) { div.innerHTML = ''; return; }
    const shares = known.delegations.filter(d => d.scope === res);
    if (shares.length === 0) { div.innerHTML = '<p class="text-muted mt-1">No shares on this resource</p>'; return; }
    div.innerHTML = `<div class="list mt-1">${shares.map(d =>
        `<div class="list-item delegation-item">${chip(d.delegate)} ${arrow()} ${chip(d.seeker)}</div>`
    ).join('')}</div>`;
}

// 12. Members: Who + What → Roles
function qMemWhoWhat() {
    const who = getValue('q-mem-ww-who');
    const what = getValue('q-mem-ww-what');
    const div = $('q-mem-ww-result');
    if (!who || !what) { div.innerHTML = ''; return; }
    const roles = known.grants.filter(g => g.seeker === who && g.scope === what);
    if (roles.length === 0) { div.innerHTML = '<p class="text-muted mt-1">No roles (relationship exists but empty)</p>'; return; }
    div.innerHTML = `<div class="list mt-1">${roles.map(g =>
        `<div class="list-item">${badge(g.relation)}</div>`
    ).join('')}</div>`;
}

// 13. Members: Who + Role → What
function qMemWhoRole() {
    const who = getValue('q-mem-wr-who');
    const role = getValue('q-mem-wr-role');
    const div = $('q-mem-wr-result');
    if (!who || !role) { div.innerHTML = ''; return; }
    const whats = known.grants.filter(g => g.seeker === who && g.relation === role);
    if (whats.length === 0) { div.innerHTML = '<p class="text-muted mt-1">No resources with this role</p>'; return; }
    div.innerHTML = `<div class="list mt-1">${whats.map(g =>
        `<div class="list-item">${chip(g.scope)}</div>`
    ).join('')}</div>`;
}

// 14. Members: What + Role → Who
function qMemWhatRole() {
    const what = getValue('q-mem-xr-what');
    const role = getValue('q-mem-xr-role');
    const div = $('q-mem-xr-result');
    if (!what || !role) { div.innerHTML = ''; return; }
    const whos = known.grants.filter(g => g.scope === what && g.relation === role);
    if (whos.length === 0) { div.innerHTML = '<p class="text-muted mt-1">No one has this role</p>'; return; }
    div.innerHTML = `<div class="list mt-1">${whos.map(g =>
        `<div class="list-item">${chip(g.seeker)}</div>`
    ).join('')}</div>`;
}

// 15. Sharing: Receiver + Resource → Givers
function qShRecvRes() {
    const recv = getValue('q-sh-rr-recv');
    const res = getValue('q-sh-rr-res');
    const div = $('q-sh-rr-result');
    if (!recv || !res) { div.innerHTML = ''; return; }
    const givers = known.delegations.filter(d => d.seeker === recv && d.scope === res);
    if (givers.length === 0) { div.innerHTML = '<p class="text-muted mt-1">No givers found</p>'; return; }
    div.innerHTML = `<div class="list mt-1">${givers.map(d =>
        `<div class="list-item delegation-item">${chip(d.delegate)}</div>`
    ).join('')}</div>`;
}

// 16. Sharing: Receiver + Giver → Resources
function qShRecvGiver() {
    const recv = getValue('q-sh-rg-recv');
    const giver = getValue('q-sh-rg-giver');
    const div = $('q-sh-rg-result');
    if (!recv || !giver) { div.innerHTML = ''; return; }
    const resources = known.delegations.filter(d => d.seeker === recv && d.delegate === giver);
    if (resources.length === 0) { div.innerHTML = '<p class="text-muted mt-1">No resources shared</p>'; return; }
    div.innerHTML = `<div class="list mt-1">${resources.map(d =>
        `<div class="list-item delegation-item">${chip(d.scope)}</div>`
    ).join('')}</div>`;
}

// 17. Sharing: Resource + Giver → Receivers
function qShResGiver() {
    const res = getValue('q-sh-xg-res');
    const giver = getValue('q-sh-xg-giver');
    const div = $('q-sh-xg-result');
    if (!res || !giver) { div.innerHTML = ''; return; }
    const receivers = known.delegations.filter(d => d.scope === res && d.delegate === giver);
    if (receivers.length === 0) { div.innerHTML = '<p class="text-muted mt-1">No receivers found</p>'; return; }
    div.innerHTML = `<div class="list mt-1">${receivers.map(d =>
        `<div class="list-item delegation-item">${chip(d.seeker)}</div>`
    ).join('')}</div>`;
}

function showQuery(type) {
    const queries = ['types', 'entities', 'actions', 'groups-ent', 'groups-role',
        'mem-who', 'mem-what', 'mem-role', 'mem-ww', 'mem-wr', 'mem-xr',
        'sh-recv', 'sh-giver', 'sh-res', 'sh-rr', 'sh-rg', 'sh-xg'];
    const content = $(`qf-${type}`);
    const isHidden = content.classList.contains('hidden');

    // Close all first
    queries.forEach(q => {
        const el = $(`qf-${q}`);
        if (el) el.classList.add('hidden');
    });

    // Open clicked one if it was closed
    if (isHidden) {
        content.classList.remove('hidden');
    }
}



// ============================================================================
// Templates (Declarative Runner)
// ============================================================================

async function runTemplate(name) {
    const t = TEMPLATES[name];
    if (!t) return;

    const status = await api('GET', '/status');
    if (status.ok && status.data.bootstrapped) {
        if (!confirm(`System is already bootstrapped (root: ${status.data.root_entity}).\n\nReset database and run "${name}" template?`)) return;
        log('Resetting database...', 'info');
        const resetResult = await api('POST', '/reset');
        if (!resetResult.ok) {
            showError('Reset failed: ' + (resetResult.error || 'Unknown error'));
            return;
        }
        // Logout after successful reset
        authToken = null;
        localStorage.removeItem('capbit_token');
        localStorage.removeItem('capbit_entity');
        known.types = [];
        known.entities = [];
        known.grants = [];
        known.capabilities = [];
        known.delegations = [];
        known.capLabels = [];
    } else {
        if (!confirm(`Run "${name}" template? This will create entities, grants, and capabilities.`)) return;
    }

    log(`--- Running template: ${name} ---`, 'info');

    // Bootstrap
    const bootRes = await api('POST', '/bootstrap', t.bootstrap);
    if (!bootRes.ok) return;
    authToken = bootRes.data.token;
    localStorage.setItem('capbit_token', authToken);
    localStorage.setItem('capbit_entity', bootRes.data.root_entity);
    currentViewer = bootRes.data.root_entity;

    // Track bootstrap entities
    const rootEntity = `user:${t.bootstrap.root_id}`;
    known.entities.push({ id: rootEntity, type: 'user' });
    ['_type', 'user', 'team', 'app', 'resource'].forEach(type => {
        known.entities.push({ id: `_type:${type}`, type: '_type' });
        known.grants.push({ seeker: rootEntity, relation: 'admin', scope: `_type:${type}` });
    });
    known.capabilities.push({ scope: '_type:_type', relation: 'admin', cap_mask: 0x3FFF });
    ['user', 'team', 'app', 'resource'].forEach(type => {
        known.capabilities.push({ scope: `_type:${type}`, relation: 'admin', cap_mask: 0x1FFC });
    });

    // Cap labels
    for (const group of t.capLabels || []) {
        for (const b of group.bits) {
            await api('POST', '/cap-label', { scope: group.scope, bit: b.bit, label: b.label });
            known.capLabels.push({ scope: group.scope, bit: b.bit, label: b.label });
        }
    }

    // Entities
    for (const group of t.entities || []) {
        for (const id of group.ids) {
            const r = await api('POST', '/entity', { entity_type: group.type, id });
            if (r.ok) known.entities.push({ id: `${group.type}:${id}`, type: group.type });
        }
    }

    // Capabilities
    for (const group of t.capabilities || []) {
        for (const [relation, mask] of group.defs) {
            await api('POST', '/capability', { scope: group.scope, relation, cap_mask: mask });
            known.capabilities.push({ scope: group.scope, relation, cap_mask: mask });
        }
    }

    // Grants
    for (const [seeker, relation, scope] of t.grants || []) {
        await api('POST', '/grant', { seeker, relation, scope });
        known.grants.push({ seeker, relation, scope });
    }

    // Delegations
    for (const [seeker, scope, delegate] of t.delegations || []) {
        await api('POST', '/delegation', { seeker, scope, delegate });
        known.delegations.push({ seeker, scope, delegate });
    }

    // Log summary
    log('--- Summary ---', 'info');
    (t.summary || []).forEach(s => log(s, 'info'));
    log(`--- Template "${name}" complete ---`, 'success');

    await loadData();
    renderAll();
    checkConnection();
}

// ============================================================================
// Modal
// ============================================================================

function toggleModal() {
    const overlay = document.getElementById('modal-overlay');
    const fab = document.getElementById('fab');
    if (overlay.classList.contains('open')) {
        closeModal();
    } else {
        overlay.classList.add('open');
        fab.classList.add('open');
        showMenu();
    }
}

function closeModal() {
    document.getElementById('modal-overlay').classList.remove('open');
    document.getElementById('fab').classList.remove('open');
}

function showMenu() {
    document.getElementById('modal-title').textContent = 'Create';
    document.getElementById('modal-menu').classList.remove('hidden');
    document.querySelectorAll('[id^="form-"]').forEach(f => f.classList.add('hidden'));
}

function showForm(name) {
    const titles = {
        'type': 'Create Type',
        'entity': 'Create Entity',
        'cap-bit': 'Define Action',
        'relation': 'Create Role',
        'grant': 'Add Member',
        'delegate': 'Add Inheritance'
    };
    document.getElementById('modal-title').textContent = titles[name] || 'Create';
    document.getElementById('modal-menu').classList.add('hidden');
    document.querySelectorAll('[id^="form-"]').forEach(f => f.classList.add('hidden'));
    document.getElementById(`form-${name}`).classList.remove('hidden');
    if (name === 'relation') populateRelationEntityDropdown();
    if (name === 'cap-bit') updateBitStatus();
    if (name === 'grant') populateGrantFormDropdowns();
    if (name === 'delegate') populateDelegateFormDropdowns();
}

function populateRelationEntityDropdown() {
    setHtml('m-cap-scope', entitySelectHtml());
    setHtml('m-cap-relation', '<option value="">-- Select entity first --</option>');
    $('m-cap-new-relation-group').classList.add('hidden');
    setValue('m-cap-new-relation', '');
    modalBits = 0;
    $('m-cap-value').textContent = '0x0000';
    setHtml('m-cap-labels', '<span class="text-muted text-sm">Select an entity to see available capability bits</span>');
}

function updateRelationDropdown() {
    const scope = getValue('m-cap-scope');
    if (!scope) {
        setHtml('m-cap-relation', '<option value="">-- Select entity first --</option>');
        $('m-cap-new-relation-group').classList.add('hidden');
        return;
    }
    const existingRelations = [...new Set(known.capabilities.filter(c => c.scope === scope).map(c => c.relation))];
    setHtml('m-cap-relation',
        '<option value="">-- Select or add new --</option>' +
        existingRelations.map(r => `<option value="${r}">${r}</option>`).join('') +
        '<option value="__new__">+ Add new relation...</option>');
    $('m-cap-new-relation-group').classList.add('hidden');
}

function handleRelationSelect() {
    const val = getValue('m-cap-relation');
    const newGroup = $('m-cap-new-relation-group');
    if (val === '__new__') {
        newGroup.classList.remove('hidden');
        $('m-cap-new-relation').focus();
    } else {
        newGroup.classList.add('hidden');
        setValue('m-cap-new-relation', '');
    }
}

function populateGrantFormDropdowns() {
    setEntitySelects('m-grant-seeker', 'm-grant-scope');
    setHtml('m-grant-relation', '<option value="">-- Select scope first --</option>');
}

function updateGrantRelationDropdown() {
    const scope = getValue('m-grant-scope');
    if (!scope) {
        setHtml('m-grant-relation', '<option value="">-- Select scope first --</option>');
        return;
    }
    const relations = [...new Set(known.capabilities.filter(c => c.scope === scope).map(c => c.relation))];
    setHtml('m-grant-relation', relations.length === 0
        ? '<option value="">-- No relations defined --</option>'
        : '<option value="">-- Select relation --</option>' + relations.map(r => `<option value="${r}">${r}</option>`).join(''));
}

function populateDelegateFormDropdowns() {
    setEntitySelects('m-deleg-seeker', 'm-deleg-scope', 'm-deleg-source');
}

function toggleCapLabel(bit) {
    modalBits ^= (1 << bit);
    document.getElementById('m-cap-value').textContent = '0x' + modalBits.toString(16).padStart(4, '0');
}

function updateCapBitLabels() {
    const scope = document.getElementById('m-cap-scope').value;
    const container = document.getElementById('m-cap-labels');
    if (!scope || !scope.includes(':')) {
        container.innerHTML = '<span style="color: var(--text-muted); font-size: 0.8rem;">Select an entity to see available capability bits</span>';
        modalBits = 0;
        document.getElementById('m-cap-value').textContent = '0x0000';
        return;
    }
    const entityType = scope.split(':')[0];
    const typeScope = `_type:${entityType}`;
    const labels = known.capLabels.filter(l => l.scope === typeScope).sort((a, b) => a.bit - b.bit);
    modalBits = 0;
    document.getElementById('m-cap-value').textContent = '0x0000';
    if (labels.length === 0) {
        container.innerHTML = `
            <div style="padding: 0.75rem; background: var(--bg-input); border-radius: 6px; text-align: center;">
                <span style="color: var(--text-muted); font-size: 0.8rem;">No capability bits defined for <strong>${typeScope}</strong></span>
                <br><br>
                <button class="btn sm outline" type="button" onclick="showForm('cap-bit'); document.getElementById('m-label-scope').value = '${typeScope}';">
                    + Define Capability Bits
                </button>
            </div>
        `;
        return;
    }
    container.innerHTML = labels.map(l => {
        const mask = 1 << l.bit;
        return `
            <label style="display: flex; align-items: center; gap: 0.5rem; padding: 0.5rem; background: var(--bg-input); border-radius: 6px; cursor: pointer;">
                <input type="checkbox" class="cap-label-cb" data-bit="${l.bit}" onchange="toggleCapLabel(${l.bit})" style="width: auto;">
                <span style="flex: 1;"><strong>${l.label}</strong></span>
                <span style="font-family: monospace; font-size: 0.75rem; color: var(--text-muted);">bit ${l.bit} = 0x${mask.toString(16).padStart(2, '0')}</span>
            </label>
        `;
    }).join('');
}

function updateBitStatus() {
    const scope = getValue('m-label-scope');
    const usedBits = known.capLabels.filter(l => l.scope === scope).map(l => ({ bit: l.bit, label: l.label })).sort((a, b) => a.bit - b.bit);
    if (usedBits.length === 0) {
        setHtml('bit-status', '<span class="text-success">All bits (0-15) available</span>');
    } else {
        const usedList = usedBits.map(b => `<span class="text-danger">${b.bit}:${b.label}</span>`).join(', ');
        const availableBits = Array.from({ length: CONFIG.CAP_BITS_COUNT }, (_, i) => i).filter(i => !usedBits.find(b => b.bit === i));
        setHtml('bit-status', `
            <div class="mb-1"><strong>Used:</strong> ${usedList}</div>
            <div><strong>Available:</strong> <span class="text-success">${availableBits.join(', ')}</span></div>
        `);
        if (availableBits.length > 0) setValue('m-label-bit', availableBits[0]);
    }
}

// Modal form handlers
async function doCreateTypeModal() {
    const typeName = getValue('m-type-name');
    if (!typeName) return showError('Enter type name');
    const result = await api('POST', '/type', { type_name: typeName });
    if (result.ok) {
        known.entities.push({ id: `_type:${typeName}`, type: '_type' });
        setValue('m-type-name', '');
        renderAll();
        closeModal();
    }
}

async function doCreateEntityModal() {
    const entityType = getValue('m-entity-type');
    const id = getValue('m-entity-id');
    if (!id) return showError('Enter entity ID');
    const result = await api('POST', '/entity', { entity_type: entityType, id });
    if (result.ok) {
        known.entities.push({ id: `${entityType}:${id}`, type: entityType });
        setValue('m-entity-id', '');
        renderAll();
        closeModal();
    }
}

async function doDefineCapLabelModal() {
    const scope = getValue('m-label-scope');
    const bit = parseInt(getValue('m-label-bit'));
    const label = getValue('m-label-name');
    if (!label) return showError('Enter a label');
    const result = await api('POST', '/cap-label', { scope, bit, label });
    if (result.ok) {
        const existing = known.capLabels.findIndex(l => l.scope === scope && l.bit === bit);
        if (existing >= 0) known.capLabels[existing].label = label;
        else known.capLabels.push({ scope, bit, label });
        setValue('m-label-name', '');
        renderAll();
        closeModal();
    }
}

async function doCreateCapabilityModal() {
    const scope = getValue('m-cap-scope');
    const relationSelect = getValue('m-cap-relation');
    const relation = relationSelect === '__new__' ? getValue('m-cap-new-relation') : relationSelect;
    if (!scope || !relation) return showError('Fill all fields');
    if (modalBits === 0) return showError('Select at least one capability bit');
    const result = await api('POST', '/capability', { scope, relation, cap_mask: modalBits });
    if (result.ok) {
        const existing = known.capabilities.findIndex(c => c.scope === scope && c.relation === relation);
        if (existing >= 0) known.capabilities[existing].cap_mask = modalBits;
        else known.capabilities.push({ scope, relation, cap_mask: modalBits });
        setValue('m-cap-scope', '');
        setValue('m-cap-relation', '');
        setValue('m-cap-new-relation', '');
        $('m-cap-new-relation-group').classList.add('hidden');
        modalBits = 0;
        renderAll();
        closeModal();
    }
}

async function doCreateGrantModal() {
    const seeker = getValue('m-grant-seeker');
    const scope = getValue('m-grant-scope');
    const relation = getValue('m-grant-relation');
    if (!seeker || !relation || !scope) return showError('Fill all fields');
    const result = await api('POST', '/grant', { seeker, relation, scope });
    if (result.ok) {
        known.grants.push({ seeker, relation, scope });
        setValue('m-grant-seeker', '');
        setValue('m-grant-scope', '');
        setHtml('m-grant-relation', '<option value="">-- Select scope first --</option>');
        renderAll();
        closeModal();
    }
}

async function doCreateDelegationModal() {
    const seeker = getValue('m-deleg-seeker');
    const scope = getValue('m-deleg-scope');
    const delegate = getValue('m-deleg-source');
    if (!seeker || !scope || !delegate) return showError('Fill all fields');
    const result = await api('POST', '/delegation', { seeker, scope, delegate });
    if (result.ok) {
        known.delegations.push({ seeker, scope, delegate });
        setValue('m-deleg-seeker', '');
        setValue('m-deleg-scope', '');
        setValue('m-deleg-source', '');
        renderAll();
        closeModal();
    }
}

// ============================================================================
// Delete Operations
// ============================================================================

async function doDeleteEntity(entityId) {
    if (!confirm(`Delete ${entityId}? This will remove all its grants and inheritance.`)) return;
    const result = await api('POST', '/delete/entity', { entity_id: entityId });
    if (result.ok) {
        await loadData();
        log(`🗑️ Deleted entity: ${entityId}`, 'success');
        renderAll();
    }
}

async function doDeleteGrant(seeker, relation, scope) {
    if (!confirm(`Remove ${seeker} from ${relation} on ${scope}?`)) return;
    const result = await api('POST', '/delete/grant', { seeker, relation, scope });
    if (result.ok) {
        await loadData();
        log(`🗑️ Removed grant: ${seeker} → ${relation} → ${scope}`, 'success');
        renderAll();
    }
}

async function doDeleteDelegation(seeker, scope, delegate) {
    if (!confirm(`Remove inheritance: ${seeker} from ${delegate} on ${scope}?`)) return;
    const result = await api('POST', '/delete/delegation', { seeker, scope, delegate });
    if (result.ok) {
        await loadData();
        log(`🗑️ Removed inheritance: ${seeker} ← ${delegate} on ${scope}`, 'success');
        renderAll();
    }
}

async function doDeleteCapability(scope, relation) {
    if (!confirm(`Delete role "${relation}" on ${scope}?`)) return;
    const result = await api('POST', '/delete/capability', { scope, relation });
    if (result.ok) {
        await loadData();
        log(`🗑️ Deleted role: ${relation} on ${scope}`, 'success');
        renderAll();
    }
}

async function doDeleteType(typeName) {
    if (!confirm(`Delete type "${typeName}"? This will also delete _type:${typeName}.`)) return;
    const result = await api('POST', '/delete/type', { type_name: typeName });
    if (result.ok) {
        await loadData();
        log(`🗑️ Deleted type: ${typeName}`, 'success');
        renderAll();
    }
}

async function doDeleteCapLabel(scope, bit, label) {
    if (!confirm(`Delete action "${label}" (bit ${bit}) on ${scope}?`)) return;
    const result = await api('POST', '/delete/cap-label', { scope, bit });
    if (result.ok) {
        await loadData();
        log(`🗑️ Deleted action: ${label} (bit ${bit}) on ${scope}`, 'success');
        renderAll();
    }
}

async function doRenameEntity(entityId) {
    const [type, oldName] = entityId.split(':');
    const newName = prompt(`Rename ${entityId}\n\nEnter new name:`, oldName);
    if (!newName || newName === oldName) return;
    const result = await api('POST', '/rename/entity', { entity_id: entityId, new_name: newName });
    if (result.ok) {
        await loadData();
        log(`Renamed: ${entityId} → ${type}:${newName}`, 'success');
        renderAll();
    }
}

// ============================================================================
// UI Rendering
// ============================================================================

function showTab(name) {
    $$('.tab').forEach(t => t.classList.remove('active'));
    $$('.tab-content').forEach(t => t.classList.remove('active'));
    $(`tab-${name}`)?.classList.add('active');
    document.querySelector(`.tab[onclick="showTab('${name}')"]`)?.classList.add('active');
    // Close all dash sections when switching tabs
    $$('.dash-section').forEach(el => el.classList.add('hidden'));
}

function toggleDash(section) {
    const sections = ['types', 'entities', 'capbits', 'relations', 'grants', 'delegations'];
    const el = $(`dash-${section}`);
    const isHidden = el.classList.contains('hidden');

    // Close all and remove active
    sections.forEach(s => {
        $(`dash-${s}`)?.classList.add('hidden');
    });
    $$('.dash-stat').forEach(s => s.classList.remove('active'));

    // Open clicked if it was hidden
    if (isHidden) {
        el.classList.remove('hidden');
        // Mark stat as active
        event.currentTarget.classList.add('active');
        // Scroll section into view
        setTimeout(() => el.scrollIntoView({ behavior: 'smooth', block: 'nearest' }), 50);
    }
}


async function updateViewerPermission() {
    viewerCanSeeSystem = !!authToken;
    renderAll();
}

function filterSystem(items, scopeField = 'id') {
    if (viewerCanSeeSystem) return items;
    return items.filter(item => {
        const val = item[scopeField] || item.id || item.scope;
        return !val.startsWith('_type:') && !val.startsWith('_system:');
    });
}

const isSystemEntity = (id) => id.startsWith('_type:') || id.startsWith('_system:');

function renderAll() {
    renderTypes();
    renderEntities();
    renderPrimitiveCapabilities();
    renderCapabilities();
    renderGrants();
    renderDelegations();
    renderTestSelects();
    updateCounts();
}

function updateCounts() {
    const displayEntities = filterSystem(known.entities, 'id');
    const displayCaps = filterSystem(known.capabilities, 'scope');
    const displayGrants = filterSystem(known.grants, 'scope');

    const counts = {
        'count-types': known.types.length,
        'count-entities': displayEntities.length,
        'count-capbits': known.capLabels.length,
        'count-relations': displayCaps.length,
        'count-grants': displayGrants.length,
        'count-delegations': known.delegations.length
    };

    let total = 0;
    Object.entries(counts).forEach(([id, val]) => {
        const el = $(id);
        if (el) {
            el.textContent = val;
            el.classList.toggle('zero', val === 0);
        }
        total += val;
    });

    // Show/hide empty hint
    const hint = $('empty-hint');
    if (hint) hint.classList.toggle('hidden', total > 0);
}

function renderTestSelects() {
    const displayEntities = filterSystem(known.entities, 'id');
    const entOpts = '<option value="">Select...</option>' + displayEntities.map(e => `<option value="${e.id}">${e.id}</option>`).join('');

    // Type selects
    const typeOpts = '<option value="">Select...</option>' + known.types.map(t => `<option value="${t}">${t}</option>`).join('');
    const typeOptsScope = '<option value="">Select...</option>' + known.types.map(t => `<option value="_type:${t}">${t}</option>`).join('');
    setHtml('q-type', typeOpts);
    setHtml('q-act-type', typeOptsScope);

    // Entity select
    setHtml('q-entity', entOpts);

    // Role selects (unique relation names from grants too)
    const roles = [...new Set([...known.capabilities.map(c => c.relation), ...known.grants.map(g => g.relation)])].sort();
    const roleOpts = '<option value="">Select...</option>' + roles.map(r => `<option value="${r}">${r}</option>`).join('');
    setHtml('q-grp-role', roleOpts);
    setHtml('q-mem-role', roleOpts);
    setHtml('q-mem-wr-role', roleOpts);
    setHtml('q-mem-xr-role', roleOpts);

    // Entity selects for queries
    ['q-grp-ent', 'q-mem-who', 'q-mem-what', 'q-sh-recv', 'q-sh-giver', 'q-sh-res',
     'q-mem-ww-who', 'q-mem-ww-what', 'q-mem-wr-who', 'q-mem-xr-what',
     'q-sh-rr-recv', 'q-sh-rr-res', 'q-sh-rg-recv', 'q-sh-rg-giver', 'q-sh-xg-res', 'q-sh-xg-giver'
    ].forEach(id => setHtml(id, entOpts));
}

function renderTypes() {
    if (known.types.length === 0) {
        setHtml('type-list', '<div class="empty"><div class="empty-icon">📁</div><p>No types yet</p><button class="btn sm mt-1" onclick="toggleModal(); showForm(\'type\');">+ Create Type</button></div>');
        return;
    }
    const html = '<div class="list">' + known.types.map(t =>
        `<div class="list-item${t.startsWith('_') ? ' system-item' : ''}">${t.startsWith('_') ? systemIcon() : ''}<span class="chip"><span class="type">${t}</span></span>${!t.startsWith('_') ? `<button class="del-btn" onclick="doDeleteType('${t}')">&times;</button>` : ''}</div>`
    ).join('') + '</div>';
    setHtml('type-list', html);
}

function renderEntities() {
    const displayEntities = filterSystem(known.entities, 'id');
    renderListOrEmpty('entity-list', displayEntities, '📋', 'No entities yet',
        e => `<div class="list-item${isSystemEntity(e.id) ? ' system-item' : ''}">${isSystemEntity(e.id) ? systemIcon() : ''}${chip(e.id)}${!isSystemEntity(e.id) ? `<button class="rename-btn" onclick="doRenameEntity('${e.id}')" title="Rename">✎</button><button class="del-btn" onclick="doDeleteEntity('${e.id}')">&times;</button>` : ''}</div>`);
}

function renderPrimitiveCapabilities() {
    if (known.capLabels.length === 0) {
        setHtml('primitive-cap-list', '<div class="empty"><div class="empty-icon">🔹</div><p>No primitive capabilities defined yet</p></div>');
        return;
    }
    const byScope = new Map();
    known.capLabels.forEach(l => {
        if (!byScope.has(l.scope)) byScope.set(l.scope, []);
        byScope.get(l.scope).push(l);
    });
    let html = '<div class="list">';
    byScope.forEach((labels, scope) => {
        labels.sort((a, b) => a.bit - b.bit);
        labels.forEach(l => {
            const mask = 1 << l.bit;
            html += `<div class="list-item">${chip(scope)} <span class="cap-badge cap-badge-success">bit${l.bit}</span> <strong>${l.label}</strong> ${capBadge(mask)}<button class="del-btn" onclick="doDeleteCapLabel('${scope}',${l.bit},'${l.label}')">&times;</button></div>`;
        });
    });
    setHtml('primitive-cap-list', html + '</div>');
}

function renderGrants() {
    const displayGrants = filterSystem(known.grants, 'scope');
    renderListOrEmpty('grant-list', displayGrants, '🔗', 'No direct grants yet',
        g => `<div class="list-item${isSystemEntity(g.scope) ? ' system-item' : ''}">${isSystemEntity(g.scope) ? systemIcon() : ''}${chip(g.seeker)} ${arrow()} ${badge(g.relation)} ${arrow()} ${chip(g.scope)}<button class="del-btn" onclick="doDeleteGrant('${g.seeker}','${g.relation}','${g.scope}')">&times;</button></div>`);
}

function renderDelegations() {
    renderListOrEmpty('delegation-list', known.delegations, '↗️', 'No delegations yet',
        d => `<div class="list-item delegation-item">${chip(d.seeker)} ${arrow('inherits from')} ${chip(d.delegate)} ${arrow('on')} ${chip(d.scope)}<button class="del-btn" onclick="doDeleteDelegation('${d.seeker}','${d.scope}','${d.delegate}')">&times;</button></div>`);
}

function renderCapabilities() {
    const displayCaps = filterSystem(known.capabilities, 'scope');
    if (displayCaps.length === 0) {
        setHtml('cap-list', '<div class="empty"><div class="empty-icon">⚡</div><p>No grant relations defined yet</p></div>');
        return;
    }
    setHtml('cap-list', displayCaps.map(c => {
        const typeScope = `_type:${c.scope.split(':')[0]}`;
        const isSys = isSystemEntity(c.scope);
        const typeLabels = known.capLabels.filter(l => l.scope === typeScope);
        const bitLabels = [];
        for (let i = 0; i < CONFIG.CAP_BITS_COUNT; i++) {
            if (c.cap_mask & (1 << i)) {
                const label = typeLabels.find(l => l.bit === i);
                bitLabels.push(label ? label.label : `bit${i}`);
            }
        }
        const labelStr = bitLabels.join(' + ');
        return `
            <div class="list-item cap-item${isSys ? ' system-item' : ''}">
                ${isSys ? systemIcon() : ''}${chip(c.scope)} ${badge(c.relation)} ${capBadge(c.cap_mask)}
                ${labelStr ? `<span class="cap-labels">= ${labelStr}</span>` : ''}
                ${!isSys ? `<button class="del-btn" onclick="doDeleteCapability('${c.scope}','${c.relation}')">&times;</button>` : ''}
            </div>`;
    }).join(''));
}

// ============================================================================
// Initialize
// ============================================================================

function initBitSelector() {
    const container = $('bit-selector');
    if (!container) return;
    container.innerHTML = '';
    for (let i = 0; i < 16; i++) {
        const btn = document.createElement('button');
        btn.type = 'button';
        btn.className = 'bit-btn';
        btn.dataset.bit = i;
        btn.textContent = i;
        btn.style.cssText = 'padding: 0.25rem; font-size: 0.65rem; border: 1px solid var(--border); background: var(--bg-input); color: var(--text-muted); border-radius: 4px; cursor: pointer;';
        btn.onclick = () => toggleBit(i);
        container.appendChild(btn);
    }
    updateBitDisplay();
}

function toggleBit(bit) {
    selectedBits ^= (1 << bit);
    updateBitDisplay();
}

function updateBitDisplay() {
    document.querySelectorAll('.bit-btn').forEach(btn => {
        const bit = parseInt(btn.dataset.bit);
        const isSet = (selectedBits & (1 << bit)) !== 0;
        btn.style.background = isSet ? 'var(--primary)' : 'var(--bg-input)';
        btn.style.color = isSet ? 'white' : 'var(--text-muted)';
    });
    const capValue = document.getElementById('cap-value');
    const capMaskInput = document.getElementById('cap-mask-input');
    if (capValue) capValue.textContent = '0x' + selectedBits.toString(16).padStart(4, '0');
    if (capMaskInput) capMaskInput.value = '0x' + selectedBits.toString(16).padStart(4, '0');
}

// Auto-connect on load
initBitSelector();
checkConnection();
