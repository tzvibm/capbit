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

const SYSTEM_READ_SCOPE = '_type:_type';
const SYSTEM_READ_BIT = 0x2000;

const known = {
    entities: [],
    grants: [],
    capabilities: [],
    delegations: [],
    capLabels: []
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

const systemIcon = () => '<span title="System internal" style="opacity: 0.6; margin-right: 0.25rem;">⚙</span>';

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
    const logDiv = document.getElementById('log');
    const time = new Date().toLocaleTimeString();
    const entry = document.createElement('div');
    entry.className = `log-entry ${type}`;
    entry.innerHTML = `<span class="time">${time}</span>${msg}`;
    logDiv.insertBefore(entry, logDiv.firstChild);
    while (logDiv.children.length > 100) {
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
    document.getElementById('log').innerHTML = '';
    log('Log cleared', 'info');
}

function copyLogs() {
    const logDiv = document.getElementById('log');
    const entries = [...logDiv.querySelectorAll('.log-entry')].reverse();
    const text = entries.map(e => e.textContent).join('\n');
    if (navigator.clipboard) {
        navigator.clipboard.writeText(text).then(() => log('Logs copied!', 'success')).catch(() => fallbackCopy(text));
    } else {
        fallbackCopy(text);
    }
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
    if (result.ok) {
        connected = true;
        document.getElementById('status-dot').classList.add('connected');
        document.getElementById('status-dot').title = result.data.bootstrapped
            ? `Connected (root: ${result.data.root_entity})`
            : 'Connected (not bootstrapped)';
        updateBootstrapUI(result.data.bootstrapped, result.data.root_entity);
        await checkAuth();
        await loadData();
    } else {
        connected = false;
        document.getElementById('status-dot').classList.remove('connected');
    }
}

function updateBootstrapUI(bootstrapped, rootEntity) {
    const form = document.getElementById('bootstrap-form');
    const status = document.getElementById('bootstrap-status');
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
    const authStatus = document.getElementById('auth-status');
    const logoutBtn = document.getElementById('logout-btn');
    const loggedOutDiv = document.getElementById('auth-logged-out');
    const loggedInDiv = document.getElementById('auth-logged-in');
    const setPasswordCard = document.getElementById('set-password-card');

    if (loggedIn) {
        authStatus.textContent = entity;
        authStatus.style.color = 'var(--success)';
        logoutBtn.style.display = 'block';
        loggedOutDiv.style.display = 'none';
        loggedInDiv.style.display = 'block';
        setPasswordCard.style.display = 'block';
        document.getElementById('logged-in-entity').textContent = entity;
        document.getElementById('current-token').textContent = authToken.substring(0, 20) + '...';
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
    const token = document.getElementById('login-token').value.trim();
    if (!token) return alert('Enter a token');
    authToken = token;
    const result = await api('GET', '/me');
    if (result.ok) {
        localStorage.setItem('capbit_token', token);
        localStorage.setItem('capbit_entity', result.data.entity);
        currentViewer = result.data.entity;
        updateAuthUI(true, result.data.entity);
        log(`Logged in as ${result.data.entity}`, 'success');
        document.getElementById('login-token').value = '';
        await loadData();
    } else {
        authToken = null;
        alert('Invalid token');
    }
}

async function doPasswordLogin() {
    let entityId = document.getElementById('login-entity').value.trim();
    const password = document.getElementById('login-password').value;
    if (!entityId) return alert('Enter your username');
    // Auto-prepend "user:" if not a full entity ID
    if (!entityId.includes(':')) entityId = `user:${entityId}`;
    const result = await api('POST', '/login', { entity_id: entityId, password });
    if (result.ok) {
        authToken = result.data.token;
        localStorage.setItem('capbit_token', authToken);
        localStorage.setItem('capbit_entity', result.data.root_entity);
        currentViewer = result.data.root_entity;
        updateAuthUI(true, result.data.root_entity);
        log(`Logged in as ${result.data.root_entity}`, 'success');
        document.getElementById('login-entity').value = '';
        document.getElementById('login-password').value = '';
        await loadData();
    } else {
        alert('Login failed: ' + (result.error || 'Invalid credentials'));
    }
}

async function doSetPassword() {
    let entityId = document.getElementById('set-password-entity').value.trim();
    const password = document.getElementById('set-password-value').value;
    if (!entityId || !password) return alert('Enter username and password');
    // Auto-prepend "user:" if not a full entity ID
    if (!entityId.includes(':')) entityId = `user:${entityId}`;
    const result = await api('POST', '/password', { entity_id: entityId, password });
    if (result.ok) {
        log(`Password set for ${entityId}`, 'success');
        document.getElementById('set-password-entity').value = '';
        document.getElementById('set-password-value').value = '';
    } else {
        alert('Failed: ' + (result.error || 'Unknown error'));
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
    const [entities, grants, caps, labels] = await Promise.all([
        api('GET', '/entities'),
        api('GET', '/grants'),
        api('GET', '/capabilities'),
        api('GET', '/cap-labels')
    ]);
    if (entities.ok) known.entities = entities.data.map(e => ({ id: e.id, type: e.entity_type }));
    if (grants.ok) known.grants = grants.data.map(g => ({ seeker: g.seeker, relation: g.relation, scope: g.scope }));
    if (caps.ok) known.capabilities = caps.data.map(c => ({ scope: c.scope, relation: c.relation, cap_mask: c.cap_mask }));
    if (labels.ok) known.capLabels = labels.data.map(l => ({ scope: l.scope, bit: l.bit, label: l.label }));
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

async function doCheckAccess() {
    const subject = document.getElementById('check-subject').value.trim();
    const object = document.getElementById('check-object').value.trim();
    const required = parseInt(document.getElementById('check-cap').value);
    if (!subject || !object) return alert('Fill subject and object');
    const result = await api('POST', '/check', { subject, object, required });
    const resultDiv = document.getElementById('check-result');
    if (result.ok) {
        const { allowed, effective, effective_string, required_string } = result.data;
        resultDiv.className = `result ${allowed ? 'allowed' : 'denied'}`;
        resultDiv.innerHTML = `
            <div class="result-icon">${allowed ? '✓' : '✗'}</div>
            <div class="result-text">${allowed ? 'ACCESS ALLOWED' : 'ACCESS DENIED'}</div>
            <div class="result-detail">
                Required: 0x${required.toString(16).padStart(4, '0')} (${required_string})<br>
                Effective: 0x${effective.toString(16).padStart(4, '0')} (${effective_string})
            </div>
        `;
    } else {
        resultDiv.className = 'result denied';
        resultDiv.innerHTML = `<div class="result-icon">⚠</div><div class="result-text">ERROR</div><div class="result-detail">${result.error}</div>`;
    }
}

function showQueryMode(mode) {
    document.querySelectorAll('.seg-btn').forEach(btn => btn.classList.toggle('active', btn.dataset.mode === mode));
    document.getElementById('query-check').classList.toggle('hidden', mode !== 'check');
    document.getElementById('query-accessible').classList.toggle('hidden', mode !== 'accessible');
    document.getElementById('query-subjects').classList.toggle('hidden', mode !== 'subjects');
}

async function doQueryAccessible() {
    const subject = document.getElementById('query-subject').value;
    if (!subject) return alert('Select a subject');
    const result = await api('POST', '/query/accessible', { subject });
    const resultDiv = document.getElementById('accessible-result');
    if (result.ok) {
        if (result.data.length === 0) {
            resultDiv.innerHTML = '<div class="empty" style="margin-top: 1rem;"><p>No access found</p></div>';
            return;
        }
        resultDiv.innerHTML = `<div class="list" style="margin-top: 1rem;">${result.data.map(e => `
            <div class="list-item">${chip(e.entity)} ${badge(e.relation)} ${capBadge(e.effective)}</div>
        `).join('')}</div>`;
    } else {
        resultDiv.innerHTML = `<div class="result denied"><div class="result-detail">${result.error}</div></div>`;
    }
}

async function doQuerySubjects() {
    const object = document.getElementById('query-object').value;
    if (!object) return alert('Select an object');
    const result = await api('POST', '/query/subjects', { object });
    const resultDiv = document.getElementById('subjects-result');
    if (result.ok) {
        if (result.data.length === 0) {
            resultDiv.innerHTML = '<div class="empty" style="margin-top: 1rem;"><p>No subjects have access</p></div>';
            return;
        }
        resultDiv.innerHTML = `<div class="list" style="margin-top: 1rem;">${result.data.map(e => `
            <div class="list-item">${chip(e.entity)} ${badge(e.relation)} ${capBadge(e.effective)}</div>
        `).join('')}</div>`;
    } else {
        resultDiv.innerHTML = `<div class="result denied"><div class="result-detail">${result.error}</div></div>`;
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
        await api('POST', '/reset');
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
        'cap-bit': 'Define Capability Bit',
        'relation': 'Define Relation',
        'grant': 'Create Grant',
        'delegate': 'Create Delegation'
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
    const select = document.getElementById('m-cap-scope');
    select.innerHTML = '<option value="">-- Select entity --</option>' +
        known.entities.map(e => `<option value="${e.id}">${e.id}</option>`).join('');
    document.getElementById('m-cap-relation').innerHTML = '<option value="">-- Select entity first --</option>';
    document.getElementById('m-cap-new-relation-group').classList.add('hidden');
    document.getElementById('m-cap-new-relation').value = '';
    modalBits = 0;
    document.getElementById('m-cap-value').textContent = '0x0000';
    document.getElementById('m-cap-labels').innerHTML =
        '<span style="color: var(--text-muted); font-size: 0.8rem;">Select an entity to see available capability bits</span>';
}

function updateRelationDropdown() {
    const scope = document.getElementById('m-cap-scope').value;
    const select = document.getElementById('m-cap-relation');
    if (!scope) {
        select.innerHTML = '<option value="">-- Select entity first --</option>';
        document.getElementById('m-cap-new-relation-group').classList.add('hidden');
        return;
    }
    const existingRelations = [...new Set(known.capabilities.filter(c => c.scope === scope).map(c => c.relation))];
    select.innerHTML = '<option value="">-- Select or add new --</option>' +
        existingRelations.map(r => `<option value="${r}">${r}</option>`).join('') +
        '<option value="__new__">+ Add new relation...</option>';
    document.getElementById('m-cap-new-relation-group').classList.add('hidden');
}

function handleRelationSelect() {
    const select = document.getElementById('m-cap-relation');
    const newGroup = document.getElementById('m-cap-new-relation-group');
    if (select.value === '__new__') {
        newGroup.classList.remove('hidden');
        document.getElementById('m-cap-new-relation').focus();
    } else {
        newGroup.classList.add('hidden');
        document.getElementById('m-cap-new-relation').value = '';
    }
}

function populateGrantFormDropdowns() {
    const entityOptions = '<option value="">-- Select entity --</option>' +
        known.entities.map(e => `<option value="${e.id}">${e.id}</option>`).join('');
    document.getElementById('m-grant-seeker').innerHTML = entityOptions;
    document.getElementById('m-grant-scope').innerHTML = entityOptions;
    document.getElementById('m-grant-relation').innerHTML = '<option value="">-- Select scope first --</option>';
}

function updateGrantRelationDropdown() {
    const scope = document.getElementById('m-grant-scope').value;
    const select = document.getElementById('m-grant-relation');
    if (!scope) {
        select.innerHTML = '<option value="">-- Select scope first --</option>';
        return;
    }
    const relations = [...new Set(known.capabilities.filter(c => c.scope === scope).map(c => c.relation))];
    if (relations.length === 0) {
        select.innerHTML = '<option value="">-- No relations defined --</option>';
    } else {
        select.innerHTML = '<option value="">-- Select relation --</option>' +
            relations.map(r => `<option value="${r}">${r}</option>`).join('');
    }
}

function populateDelegateFormDropdowns() {
    const entityOptions = '<option value="">-- Select entity --</option>' +
        known.entities.map(e => `<option value="${e.id}">${e.id}</option>`).join('');
    document.getElementById('m-deleg-seeker').innerHTML = entityOptions;
    document.getElementById('m-deleg-scope').innerHTML = entityOptions;
    document.getElementById('m-deleg-source').innerHTML = entityOptions;
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
    const scope = document.getElementById('m-label-scope').value;
    const container = document.getElementById('bit-status');
    const usedBits = known.capLabels.filter(l => l.scope === scope).map(l => ({ bit: l.bit, label: l.label })).sort((a, b) => a.bit - b.bit);
    if (usedBits.length === 0) {
        container.innerHTML = '<span style="color: var(--success);">All bits (0-15) available</span>';
    } else {
        const usedList = usedBits.map(b => `<span style="color: var(--danger);">${b.bit}:${b.label}</span>`).join(', ');
        const availableBits = [];
        for (let i = 0; i < 16; i++) {
            if (!usedBits.find(b => b.bit === i)) availableBits.push(i);
        }
        container.innerHTML = `
            <div style="margin-bottom: 0.25rem;"><strong>Used:</strong> ${usedList}</div>
            <div><strong>Available:</strong> <span style="color: var(--success);">${availableBits.join(', ')}</span></div>
        `;
        if (availableBits.length > 0) document.getElementById('m-label-bit').value = availableBits[0];
    }
}

// Modal form handlers
async function doCreateTypeModal() {
    const typeName = document.getElementById('m-type-name').value.trim();
    if (!typeName) return alert('Enter type name');
    const result = await api('POST', '/type', { type_name: typeName });
    if (result.ok) {
        known.entities.push({ id: `_type:${typeName}`, type: '_type' });
        document.getElementById('m-type-name').value = '';
        renderAll();
        closeModal();
    }
}

async function doCreateEntityModal() {
    const entityType = document.getElementById('m-entity-type').value;
    const id = document.getElementById('m-entity-id').value.trim();
    if (!id) return alert('Enter entity ID');
    const result = await api('POST', '/entity', { entity_type: entityType, id });
    if (result.ok) {
        known.entities.push({ id: `${entityType}:${id}`, type: entityType });
        document.getElementById('m-entity-id').value = '';
        renderAll();
        closeModal();
    }
}

async function doDefineCapLabelModal() {
    const scope = document.getElementById('m-label-scope').value;
    const bit = parseInt(document.getElementById('m-label-bit').value);
    const label = document.getElementById('m-label-name').value.trim();
    if (!label) return alert('Enter a label');
    const result = await api('POST', '/cap-label', { scope, bit, label });
    if (result.ok) {
        const existing = known.capLabels.findIndex(l => l.scope === scope && l.bit === bit);
        if (existing >= 0) known.capLabels[existing].label = label;
        else known.capLabels.push({ scope, bit, label });
        document.getElementById('m-label-name').value = '';
        renderAll();
        closeModal();
    }
}

async function doCreateCapabilityModal() {
    const scope = document.getElementById('m-cap-scope').value;
    const relationSelect = document.getElementById('m-cap-relation').value;
    const relation = relationSelect === '__new__' ? document.getElementById('m-cap-new-relation').value.trim() : relationSelect;
    if (!scope || !relation) return alert('Fill all fields');
    if (modalBits === 0) return alert('Select at least one capability bit');
    const result = await api('POST', '/capability', { scope, relation, cap_mask: modalBits });
    if (result.ok) {
        const existing = known.capabilities.findIndex(c => c.scope === scope && c.relation === relation);
        if (existing >= 0) known.capabilities[existing].cap_mask = modalBits;
        else known.capabilities.push({ scope, relation, cap_mask: modalBits });
        document.getElementById('m-cap-scope').value = '';
        document.getElementById('m-cap-relation').value = '';
        document.getElementById('m-cap-new-relation').value = '';
        document.getElementById('m-cap-new-relation-group').classList.add('hidden');
        modalBits = 0;
        renderAll();
        closeModal();
    }
}

async function doCreateGrantModal() {
    const seeker = document.getElementById('m-grant-seeker').value;
    const scope = document.getElementById('m-grant-scope').value;
    const relation = document.getElementById('m-grant-relation').value;
    if (!seeker || !relation || !scope) return alert('Fill all fields');
    const result = await api('POST', '/grant', { seeker, relation, scope });
    if (result.ok) {
        known.grants.push({ seeker, relation, scope });
        document.getElementById('m-grant-seeker').value = '';
        document.getElementById('m-grant-scope').value = '';
        document.getElementById('m-grant-relation').innerHTML = '<option value="">-- Select scope first --</option>';
        renderAll();
        closeModal();
    }
}

async function doCreateDelegationModal() {
    const seeker = document.getElementById('m-deleg-seeker').value;
    const scope = document.getElementById('m-deleg-scope').value;
    const delegate = document.getElementById('m-deleg-source').value;
    if (!seeker || !scope || !delegate) return alert('Fill all fields');
    const result = await api('POST', '/delegation', { seeker, scope, delegate });
    if (result.ok) {
        known.delegations.push({ seeker, scope, delegate });
        document.getElementById('m-deleg-seeker').value = '';
        document.getElementById('m-deleg-scope').value = '';
        document.getElementById('m-deleg-source').value = '';
        renderAll();
        closeModal();
    }
}

// ============================================================================
// UI Rendering
// ============================================================================

function showTab(name) {
    document.querySelectorAll('.tab').forEach(t => t.classList.remove('active'));
    document.querySelectorAll('.tab-content').forEach(t => t.classList.remove('active'));
    const tabContent = document.getElementById(`tab-${name}`);
    if (tabContent) tabContent.classList.add('active');
    const tabBtn = document.querySelector(`.tab[onclick="showTab('${name}')"]`);
    if (tabBtn) tabBtn.classList.add('active');
}

function toggleAccordion(section) {
    const content = document.getElementById(`section-${section}`);
    const arrow = document.querySelector(`.accordion[data-section="${section}"] .accordion-arrow`);
    const isOpen = content.classList.contains('open');
    content.classList.toggle('open');
    arrow.textContent = isOpen ? '▶' : '▼';
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

function isSystemEntity(id) {
    return id.startsWith('_type:') || id.startsWith('_system:');
}

function renderAll() {
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
    document.getElementById('count-entities').textContent = displayEntities.length;
    document.getElementById('count-capbits').textContent = known.capLabels.length;
    document.getElementById('count-relations').textContent = displayCaps.length;
    document.getElementById('count-grants').textContent = displayGrants.length;
    document.getElementById('count-delegations').textContent = known.delegations.length;
    const total = displayEntities.length + known.capLabels.length + displayCaps.length + displayGrants.length + known.delegations.length;
    const homeBtn = document.getElementById('tab-btn-home');
    if (homeBtn) homeBtn.textContent = total > 0 ? `Home (${total})` : 'Home';
}

function renderTestSelects() {
    const displayEntities = filterSystem(known.entities, 'id');
    const entityOptions = displayEntities.map(e => `<option value="${e.id}">${e.id}</option>`).join('');
    const placeholder = '<option value="">-- Select entity --</option>';
    ['check-subject', 'check-object', 'query-subject', 'query-object'].forEach(id => {
        document.getElementById(id).innerHTML = placeholder + entityOptions;
    });
    const displayCaps = filterSystem(known.capabilities, 'scope');
    const sortedCaps = [...displayCaps].sort((a, b) => a.scope !== b.scope ? a.scope.localeCompare(b.scope) : a.cap_mask - b.cap_mask);
    let capOptions = '<option value="0">0x0000 (ANY)</option>';
    sortedCaps.forEach(c => {
        const hex = '0x' + c.cap_mask.toString(16).padStart(4, '0').toUpperCase();
        capOptions += `<option value="${c.cap_mask}">${c.scope} → ${c.relation} (${hex})</option>`;
    });
    document.getElementById('check-cap').innerHTML = capOptions;
}

function renderEntities() {
    const list = document.getElementById('entity-list');
    const displayEntities = filterSystem(known.entities, 'id');
    if (displayEntities.length === 0) {
        list.innerHTML = '<div class="empty"><div class="empty-icon">📋</div><p>No entities yet</p></div>';
        return;
    }
    list.innerHTML = displayEntities.map(e => {
        const isSys = isSystemEntity(e.id);
        return `<div class="list-item"${isSys ? ' style="opacity: 0.7;"' : ''}>${isSys ? systemIcon() : ''}${chip(e.id)}</div>`;
    }).join('');
}

function renderPrimitiveCapabilities() {
    const container = document.getElementById('primitive-cap-list');
    if (known.capLabels.length === 0) {
        container.innerHTML = '<div class="empty"><div class="empty-icon">🔹</div><p>No primitive capabilities defined yet</p></div>';
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
            html += `<div class="list-item">${chip(scope)} <span class="cap-badge" style="background: var(--success); color: white;">bit${l.bit}</span> <span style="font-weight: 600;">${l.label}</span> ${capBadge(mask)}</div>`;
        });
    });
    container.innerHTML = html + '</div>';
}

function renderGrants() {
    const list = document.getElementById('grant-list');
    const displayGrants = filterSystem(known.grants, 'scope');
    if (displayGrants.length === 0) {
        list.innerHTML = '<div class="empty"><div class="empty-icon">🔗</div><p>No direct grants yet</p></div>';
        return;
    }
    list.innerHTML = displayGrants.map(g => {
        const isSys = isSystemEntity(g.scope);
        return `<div class="list-item"${isSys ? ' style="opacity: 0.7;"' : ''}>${isSys ? systemIcon() : ''}${chip(g.seeker)} ${arrow()} ${badge(g.relation)} ${arrow()} ${chip(g.scope)}</div>`;
    }).join('');
}

function renderDelegations() {
    const list = document.getElementById('delegation-list');
    if (known.delegations.length === 0) {
        list.innerHTML = '<div class="empty"><div class="empty-icon">↗️</div><p>No delegations yet</p></div>';
        return;
    }
    list.innerHTML = known.delegations.map(d => `
        <div class="list-item" style="background: rgba(245, 158, 11, 0.1);">
            ${chip(d.seeker)} ${arrow('inherits from')} ${chip(d.delegate)} ${arrow('on')} ${chip(d.scope)}
        </div>
    `).join('');
}

function renderCapabilities() {
    const list = document.getElementById('cap-list');
    const displayCaps = filterSystem(known.capabilities, 'scope');
    if (displayCaps.length === 0) {
        list.innerHTML = '<div class="empty"><div class="empty-icon">⚡</div><p>No grant relations defined yet</p></div>';
        return;
    }
    list.innerHTML = displayCaps.map(c => {
        const scopeType = c.scope.split(':')[0];
        const typeScope = `_type:${scopeType}`;
        const isSys = isSystemEntity(c.scope);
        const typeLabels = known.capLabels.filter(l => l.scope === typeScope);
        const bitLabels = [];
        for (let i = 0; i < 16; i++) {
            if (c.cap_mask & (1 << i)) {
                const label = typeLabels.find(l => l.bit === i);
                bitLabels.push(label ? label.label : `bit${i}`);
            }
        }
        const labelStr = bitLabels.length > 0 ? bitLabels.join(' + ') : '';
        return `
            <div class="list-item" style="flex-wrap: wrap;${isSys ? ' opacity: 0.7;' : ''}">
                ${isSys ? systemIcon() : ''}${chip(c.scope)} ${badge(c.relation)} ${capBadge(c.cap_mask)}
                ${labelStr ? `<span style="font-size: 0.7rem; color: var(--text-muted); width: 100%; margin-top: 0.25rem;">= ${labelStr}</span>` : ''}
            </div>
        `;
    }).join('');
}

// ============================================================================
// Initialize
// ============================================================================

function initBitSelector() {
    const container = document.getElementById('bit-selector');
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
