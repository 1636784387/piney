export function isFrontend(content: string): boolean {
    return ['html>', '<head>', '<body'].some(tag => content.includes(tag));
}

export function replaceVhInContent(content: string): string {
    // Basic detection
    const has_css_min_vh = /min-height\s*:\s*[^;{}]*\d+(?:\.\d+)?vh/gi.test(content);

    const convertVhToVariable = (value: string) =>
        value.replace(/(\d+(?:\.\d+)?)vh\b/gi, (match, val) => {
            const parsed = parseFloat(val);
            if (parsed === 100) {
                return `var(--TH-viewport-height)`;
            }
            return `calc(var(--TH-viewport-height) * ${parsed / 100})`;
        });

    // Replace in CSS min-height declarations
    let processed = content.replace(
        /(min-height\s*:\s*)([^;]*?\d+(?:\.\d+)?vh)(?=\s*[;}])/gi,
        (_m, prefix, value) => `${prefix}${convertVhToVariable(value)}`,
    );

    // Replace in CSS height declarations
    processed = processed.replace(
        /(height\s*:\s*)([^;]*?\d+(?:\.\d+)?vh)(?=\s*[;}])/gi,
        (_m, prefix, value) => `${prefix}${convertVhToVariable(value)}`,
    );

    return processed;
}

const third_party = `
<script src="https://testingcf.jsdelivr.net/npm/vue/dist/vue.global.prod.js"></script>
<script src="https://testingcf.jsdelivr.net/npm/jquery/dist/jquery.min.js"></script>
<script src="https://testingcf.jsdelivr.net/npm/jquery-ui/dist/jquery-ui.min.js"></script>
<link rel="stylesheet" href="https://testingcf.jsdelivr.net/npm/jquery-ui/themes/base/theme.min.css" />
<script src="https://testingcf.jsdelivr.net/npm/jquery-ui-touch-punch"></script>
<script src="https://testingcf.jsdelivr.net/npm/pixi.js/dist/pixi.min.js"></script>
<script src="https://cdn.tailwindcss.com"></script>
<script>
    // Suppress Tailwind CDN production warning
    try {
        const originalWarn = console.warn;
        console.warn = (...args) => {
            if (args[0] && typeof args[0] === 'string' && args[0].includes('cdn.tailwindcss.com')) return;
            originalWarn.apply(console, args);
        };
    } catch(e) {}
</script>
`;

const predefine_script = `
// Inherit lodash if available (mocking minimal needed)
window._ = window.parent._ || {
    get: (obj, path) => path.split('.').reduce((acc, part) => acc && acc[part], obj),
    has: (obj, path) => !!path.split('.').reduce((acc, part) => acc && acc[part], obj),
    pick: (obj, keys) => keys.reduce((acc, key) => { if (key in obj) acc[key] = obj[key]; return acc; }, {}),
    merge: (target, source) => Object.assign(target, source)
};

// Mock SillyTavern Context
window.SillyTavern = {
    getContext: () => ({
        characterId: window.parent?.cardId || 'unknown', // Example context
    })
};

const iframeId = window.frameElement?.id || window.name;
if (iframeId) {
    window.__TH_IFRAME_ID = iframeId;
    window.name = iframeId;
}
`;

const adjust_height_script = `
function getIframeName() {
    return window.name || window.__TH_IFRAME_ID;
}

function measureAndPost() {
    const body = document.body;
    let height = body.scrollHeight;
    
    // Check if height is 0 (can happen if content isn't rendered yet), maybe use offsetHeight as fallback
    if (height === 0) height = body.offsetHeight;

    window.parent.postMessage({ 
        type: 'TH_ADJUST_IFRAME_HEIGHT', 
        iframe_name: getIframeName(), 
        height 
    }, '*');
}

// Observe Resize
const resize_observer = new ResizeObserver(() => measureAndPost());
if (document.body) {
    resize_observer.observe(document.body);
    // Also observe mutations as fallback
    const mutation_observer = new MutationObserver(() => measureAndPost());
    mutation_observer.observe(document.body, { childList: true, subtree: true, attributes: true });
}

window.addEventListener('load', measureAndPost);
// Polling for safety in early render stages
setTimeout(measureAndPost, 50);
setTimeout(measureAndPost, 200);
setTimeout(measureAndPost, 500);
`;

const adjust_viewport_script = `
// Initialize viewport height
document.documentElement.style.setProperty('--TH-viewport-height', (window.parent.innerHeight) + 'px');

// Sync Dark Mode from parent
function syncTheme() {
    try {
        const isDark = window.parent.document.documentElement.classList.contains('dark');
        if (isDark) {
            document.documentElement.classList.add('dark');
            document.body.classList.add('dark');
        } else {
            document.documentElement.classList.remove('dark');
            document.body.classList.remove('dark');
        }
    } catch(e) {
        // Fallback to system preference if parent access fails
        if (window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)').matches) {
            document.documentElement.classList.add('dark');
        }
    }
}
syncTheme();
// Listen for changes on parent class attribute (optional, usually static per session, but good to have)
try {
    const observer = new MutationObserver(syncTheme);
    observer.observe(window.parent.document.documentElement, { attributes: true, attributeFilter: ['class'] });
} catch(e) {}

window.addEventListener('message', function (event) {
    if (event.data?.type === 'TH_UPDATE_VIEWPORT_HEIGHT') {
        document.documentElement.style.setProperty('--TH-viewport-height', (window.parent.innerHeight) + 'px');
    }
});
`;

export function createSrcContent(content: string, useBlobUrl: boolean): string {
    let processedContent = replaceVhInContent(content);

    return `
<html>
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
${useBlobUrl ? `<base href="${window.location.origin}"/>` : ''}
<style>
/* Reset and base styles */
*,*::before,*::after{box-sizing:border-box;}
html {
    background-color: transparent !important;
}
body {
    margin:0 !important;
    padding:0;
    overflow:hidden !important;
    max-width: 100% !important;
    white-space: pre-wrap;
    font-family: ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif;
    font-size: 14px;
    line-height: 1.6;
    color: #0f172a; /* Slate 900 for Light Mode */
    background-color: transparent !important;
}
/* Dark Mode Overrides */
html.dark body {
    color: #e2e8f0; /* Slate 200 for Dark Mode */
}
/* Allow user to override */
@media (prefers-color-scheme: dark) {
    /* Only apply if no parent class sync happened (fallback) */
    body:not(.dark) { color: #e2e8f0; }
}

/* Default avatars */
.user_avatar,.user-avatar{background-color: #ccc;}
.char_avatar,.char-avatar{background-color: #888;}
</style>

${third_party}

<script>
${predefine_script}
</script>

<script>
${adjust_viewport_script}
</script>
</head>
<body>
${processedContent}

<script>
${adjust_height_script}
</script>
</body>
</html>
`;
}
