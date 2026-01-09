<script lang="ts">
    import { onMount, onDestroy, tick } from "svelte";
    import { page } from "$app/stores";
    import { 
        Loader2, ArrowLeft, Settings, 
        ChevronLeft, ChevronRight, ChevronsLeft, ChevronsRight 
    } from "lucide-svelte";
    import { Button } from "$lib/components/ui/button";
    import { Input } from "$lib/components/ui/input";
    import { Progress } from "$lib/components/ui/progress";
    import * as Popover from "$lib/components/ui/popover";
    import { Label } from "$lib/components/ui/label";
    import { Slider } from "$lib/components/ui/slider";
    import { toast } from "svelte-sonner";
    import { goto } from "$app/navigation";
    import { cn } from "$lib/utils";
    import { breadcrumbs } from "$lib/stores/breadcrumb";
    import { Checkbox } from "$lib/components/ui/checkbox";
    import HTMLRender from "$lib/components/render/HTMLRender.svelte";
    import { processContentWithScripts, type RegexScript } from "$lib/utils/regexProcessor";
    import { isFrontend } from "$lib/utils/renderUtils";
    import { smartFilterTags, detectTags, sortTags, processTagNewlines } from "$lib/utils/tagFilter";
    import { API_BASE } from "$lib/api";

    const historyId = $page.url.searchParams.get("history_id");
    const cardId = $page.params.id;

    // State
    let isLoading = $state(true);
    let title = $state("阅读模式");
    let characterName = $state("");
    let isTxtFormat = $state(false); // Detect format
    
    // Tag Settings
    let availableTags = $state(new Set<string>());
    let filterTags = $state<string[]>([]); // Tags to HIDE (Unchecked in UI)
    let newlineTags = $state<string[]>([]); // Tags to apply Newline conversion (Checked in UI)
    
    // Regex State
    let cardRegex: RegexScript[] = $state([]);
    let chatRegex: RegexScript[] = $state([]);
    
    // Pagination
    let currentPage = $state(1);
    let totalPages = $state(1);
    let jumpPage = $state(1);
    
    // Progress
    let globalProgress = $state(0);
    
    // Content
    interface ChatMessage {
        floor: number;
        name: string;
        content: string;
    }
    let floors = $state<ChatMessage[]>([]);
    
    // Settings
    let textBrightness = $state(100);
    let savedScrollRatio = 0; // Temp store for restoration
    
    // Auto-save timer
    let saveTimeout: any;

    // Cache for preloading next page
    let cacheData = $state<{
        page: number;
        floors: ChatMessage[];
        totalPages: number;
    } | null>(null);

    async function loadMetadata() {
        cacheData = null; // Reset cache on new load
        if (!cardId) return;
        const token = localStorage.getItem("auth_token");
        
        // 1. Get Character Name & Regex
        try {
            const cardRes = await fetch(`${API_BASE}/api/cards/${cardId}`, {
                headers: token ? { Authorization: `Bearer ${token}` } : {},
            });
            if (cardRes.ok) {
                const card = await cardRes.json();
                characterName = card.name;
                // Parse card regex
                try {
                    // Check structure. Usually card.data.extensions.regex_scripts
                    const data = typeof card.data === 'string' ? JSON.parse(card.data) : card.data;
                    if (data?.extensions?.regex_scripts) {
                        cardRegex = data.extensions.regex_scripts;
                    }
                } catch (e) { console.error("Failed to parse card regex", e); }
            }
        } catch {}

        // 2. Get History Metadata & Regex
        try {
            const res = await fetch(`${API_BASE}/api/cards/${cardId}/history`, {
                headers: token ? { Authorization: `Bearer ${token}` } : {},
            });
            if (res.ok) {
                const list = await res.json();
                const item = list.find((h: any) => h.id === historyId);
                if (item) {
                    title = item.display_name;
                    currentPage = item.current_page || 1;
                    globalProgress = item.progress || 0;
                    // Detect TXT format
                    isTxtFormat = item.format === 'txt' || item.file_name?.toLowerCase().endsWith('.txt');
                    
                    if (item.reading_settings) {
                        try {
                            const settings = JSON.parse(item.reading_settings);
                            if (settings.brightness) textBrightness = settings.brightness;
                            if (settings.scroll_ratio) savedScrollRatio = settings.scroll_ratio;
                            if (settings.tag_filters) filterTags = settings.tag_filters;
                            if (settings.newline_tags) newlineTags = settings.newline_tags;
                        } catch {}
                    }
                    // Parse chat regex
                    if (item.regex_scripts) {
                         try {
                             chatRegex = JSON.parse(item.regex_scripts);
                         } catch (e) { console.error("Failed to parse chat regex", e); }
                    }
                }
            }
        } catch {}
        
        updateBreadcrumbs();
    }
    
    function updateBreadcrumbs() {
        breadcrumbs.set([
            { label: "角色库", href: "/characters" },
            { label: characterName || "角色", href: `/characters/${cardId}` },
            { label: "聊天记录", href: `/characters/${cardId}?tab=chat` },
            { label: title },
        ]);
    }

    async function loadPage(p: number, restoreScroll = false) {
        if (!historyId || !cardId) return;

        // Use cached data if available
        if (cacheData && cacheData.page === p) {
            totalPages = cacheData.totalPages;
            currentPage = cacheData.page;
            floors = cacheData.floors;
            jumpPage = currentPage;
            cacheData = null; // Clear cache
        } else {
            isLoading = true;
            try {
                const token = localStorage.getItem("auth_token");
                const res = await fetch(`${API_BASE}/api/cards/${cardId}/history/${historyId}/content?page=${p}`, {
                    headers: token ? { Authorization: `Bearer ${token}` } : {},
                });
                
                if (!res.ok) throw new Error("加载失败");
                
                const data = await res.json();
                
                totalPages = data.total_pages;
                currentPage = data.current_page;
                floors = data.floors;
                jumpPage = currentPage;
            } catch (e) {
                console.error(e);
                toast.error("加载内容失败");
                isLoading = false;
                return;
            } finally {
                isLoading = false;
            }
        }

        // Common post-load logic (from cache or fresh fetch)
        try {
            // Detect tags in loaded content
            if (!isTxtFormat) {
                floors.forEach(f => {
                    const t = detectTags(f.content);
                    t.forEach(ti => availableTags.add(ti));
                });
                availableTags = new Set(availableTags);
            }

            await tick(); // Wait for DOM update
            
            // Restore scroll position with a slight delay to ensure layout
            if (restoreScroll && savedScrollRatio > 0) {
                setTimeout(() => {
                    const container = document.querySelector('.overflow-y-auto');
                    if (container) {
                        const { scrollHeight, clientHeight } = container;
                        container.scrollTop = (scrollHeight - clientHeight) * savedScrollRatio;
                    }
                }, 100);
            } else if (!restoreScroll) {
                 const container = document.querySelector('.overflow-y-auto');
                 if (container) container.scrollTop = 0;
            }

            // Trigger preload of NEXT page
            if (currentPage < totalPages) {
                setTimeout(() => preloadPage(currentPage + 1), 500);
            }

        } catch (e) {
            console.error(e);
        }
    }

    async function preloadPage(p: number) {
        if (!historyId || !cardId || p > totalPages) return;
        if (cacheData && cacheData.page === p) return;

        try {
            const token = localStorage.getItem("auth_token");
            const res = await fetch(`${API_BASE}/api/cards/${cardId}/history/${historyId}/content?page=${p}`, {
                headers: token ? { Authorization: `Bearer ${token}` } : {},
            });
            
            if (res.ok) {
                const data = await res.json();
                cacheData = {
                    page: p,
                    floors: data.floors,
                    totalPages: data.total_pages
                };
                
                // Also scan tags for the preloaded page to keep the menu updated
                if (!isTxtFormat) {
                    data.floors.forEach((f: any) => {
                        const t = detectTags(f.content);
                        t.forEach((ti: string) => availableTags.add(ti));
                    });
                    availableTags = new Set(availableTags);
                }
            }
        } catch (e) {
            // Silently fail preload
        }
    }

    function handlePageChange(newPage: number) {
        if (newPage < 1 || newPage > totalPages) return;
        savedScrollRatio = 0; 
        currentPage = newPage;
        saveProgress();
        loadPage(newPage, false);
    }

    function handleScroll(e: Event) {
        const target = e.target as HTMLElement;
        const { scrollTop, scrollHeight, clientHeight } = target;
        
        // Update saved ratio immediately
        let ratio = 0;
        if (scrollHeight > clientHeight) {
            ratio = scrollTop / (scrollHeight - clientHeight);
        }
        savedScrollRatio = ratio;
        
        // Calculate Global Progress
        const totalRaw = ((currentPage - 1) + ratio) / totalPages;
        globalProgress = Math.min(100, Math.round(totalRaw * 100));
        
        // Debounce Save
        clearTimeout(saveTimeout);
        saveTimeout = setTimeout(() => {
            saveProgress();
        }, 1000);
    }

    async function saveProgress() {
        if (!historyId || !cardId) return;
        try {
            const token = localStorage.getItem("auth_token");
            const settings = JSON.stringify({ 
                brightness: textBrightness,
                scroll_ratio: savedScrollRatio,
                tag_filters: filterTags,
                newline_tags: newlineTags
            });
            
            await fetch(`${API_BASE}/api/cards/${cardId}/history/${historyId}`, {
                method: 'PATCH',
                headers: { 
                    'Content-Type': 'application/json',
                    ...(token ? { Authorization: `Bearer ${token}` } : {}),
                },
                body: JSON.stringify({ 
                    progress: globalProgress,
                    current_page: currentPage,
                    reading_settings: settings
                })
            });
        } catch {}
    }
    

    
    // Debounced save for brightness
    function handleBrightnessChange(val: number[]) {
        textBrightness = val[0];
        triggerSave();
    }
    
    function triggerSave() {
        clearTimeout(saveTimeout);
        saveTimeout = setTimeout(() => {
            saveProgress();
        }, 1000);
    }
    
    function toggleTagFilter(tag: string, checked: boolean) {
        // UI: Checked = Visible. 
        // Logic: filterTags stores HIDDEN.
        // So Checked(true) -> Remove from filterTags.
        // Unchecked(false) -> Add to filterTags.
        if (checked) {
            filterTags = filterTags.filter(t => t !== tag);
        } else {
            if(!filterTags.includes(tag)) filterTags = [...filterTags, tag];
        }
        triggerSave();
    }
    
    function toggleNewlineTag(tag: string, checked: boolean) {
        // UI: Checked = Enabled.
        if (checked) {
            if(!newlineTags.includes(tag)) newlineTags = [...newlineTags, tag];
        } else {
             newlineTags = newlineTags.filter(t => t !== tag);
        }
        triggerSave();
    }
    
    function processText(text: string): string {
        // 1. Normalize line endings
        let res = text.replace(/\r\n/g, '\n').replace(/\r/g, '\n');

        // IF TXT FORMAT: Skip all regex and formatting
        if (isTxtFormat) {
            // Escape HTML chars to prevent XSS only
            res = res
                .replace(/&/g, '&amp;')
                .replace(/</g, '&lt;')
                .replace(/>/g, '&gt;');
            // Preserve whitespace with simple CSS in the rendering part, but here just return text
            // We return plain text, the container will allow pre-wrap
            return res;
        }

        // 2. Smart Tag Filtering (Before Regex)
        // Handles unclosed tags: <thought>... (deletes to next tag), </thought>... (deletes from start)
        res = smartFilterTags(res, filterTags);
        
        // New: Process Tag Newlines
        res = processTagNewlines(res, newlineTags);

        // 3. Apply User Regex Scripts (Chat -> Card)
        res = processContentWithScripts(res, chatRegex);
        res = processContentWithScripts(res, cardRegex);
        
        // 4. Collapse excessive empty lines (3+ newlines -> 2)
        res = res.replace(/(\n\s*){3,}/g, '\n\n');
        
        // 5. Process triple-backtick code blocks BEFORE DOM parsing
        // Match ```language\ncontent\n``` or just ```content```
        res = res.replace(/```(\w*)\n?([\s\S]*?)```/g, (match, lang, content) => {
            // Check if content is HTML/frontend code
            if (isFrontend(content)) {
                // Render HTML directly (unwrap from code block)
                return content;
            }
            // Otherwise, create a proper code block
            const langClass = lang ? ` class="language-${lang}"` : '';
            const escapedContent = content
                .replace(/&/g, '&amp;')
                .replace(/</g, '&lt;')
                .replace(/>/g, '&gt;');
            return `<pre><code${langClass}>${escapedContent}</code></pre>`;
        });
        
        // 5. Check if result is frontend code - if so, skip sanitization to preserve scripts
        // The iframe sandbox will safely contain any scripts
        // Frontend HTML handles its own formatting, don't modify it
        if (isFrontend(res)) {
            return res;
        }
        
        // 6. For non-frontend content, use DOM-based sanitizer
        const container = document.createElement('div');
        container.innerHTML = res;
        
        // 7. Remove dangerous elements (only for non-frontend content)
        container.querySelectorAll('script, iframe, object, embed, form').forEach(el => el.remove());
        container.querySelectorAll('*').forEach(el => {
            // Remove event handlers and javascript: links
            Array.from(el.attributes).forEach(attr => {
                if (attr.name.startsWith('on') || 
                    (attr.name === 'href' && attr.value.toLowerCase().startsWith('javascript:'))) {
                    el.removeAttribute(attr.name);
                }
            });
        });
        
        // 8. Process text nodes for Markdown-like formatting
        processTextNodes(container);
        
        // 9. Get HTML and collapse excessive <br> tags
        let html = container.innerHTML;
        // Replace 2+ consecutive <br> (with optional whitespace) to single <br>
        html = html.replace(/(<br\s*\/?>\s*){2,}/gi, '<br>');
        
        return html;
    }
    
    /**
     * Process text nodes for Markdown formatting (SillyReader style)
     */
    function processTextNodes(element: HTMLElement): void {
        const walker = document.createTreeWalker(element, NodeFilter.SHOW_TEXT, null);
        const textNodes: Text[] = [];
        while (walker.nextNode()) textNodes.push(walker.currentNode as Text);
        
        textNodes.forEach(node => {
            const parent = node.parentNode as HTMLElement;
            if (!parent) return;
            
            // Skip processing inside pre, code, script, style
            if (parent.closest('pre, code, script, style')) return;
            
            let text = node.textContent || '';
            let hasChanges = false;
            
            // Inline code: `code` (but NOT triple backticks - those are handled earlier)
            // Use negative lookbehind/ahead to avoid matching ``` patterns
            if (text.includes('`') && !text.includes('```')) {
                text = text.replace(/(?<!`)`([^`]+)`(?!`)/g, '<code>$1</code>');
                hasChanges = true;
            }
            
            // Strikethrough: ~~text~~
            if (text.includes('~~')) {
                text = text.replace(/~~(.+?)~~/g, '<del>$1</del>');
                hasChanges = true;
            }
            
            // Bold + Italic: ***text***
            if (text.includes('*')) {
                text = text.replace(/\*\*\*(.+?)\*\*\*/g, '<strong><em>$1</em></strong>');
                // Bold: **text**
                text = text.replace(/\*\*(.+?)\*\*/g, '<strong>$1</strong>');
                // Italic: *text* (negative lookbehind for < to avoid matching HTML)
                text = text.replace(/(?<![<\\])\*([^*\n]+)\*(?![>])/g, '<em>$1</em>');
                hasChanges = true;
            }
            
            // Quotes: "text" or 「text」 or 『text』
            if (text.includes('"') || text.includes('「') || text.includes('『')) {
                text = text.replace(/"([^"]+)"/g, '<q>$1</q>');
                text = text.replace(/「([^」]+)」/g, '<q>$1</q>');
                text = text.replace(/『([^』]+)』/g, '<q>$1</q>');
                hasChanges = true;
            }
            
            // Newlines -> <br>
            if (text.includes('\n')) {
                text = text.replace(/\n/g, '<br>');
                hasChanges = true;
            }
            
            if (hasChanges) {
                const span = document.createElement('span');
                span.innerHTML = text;
                parent.replaceChild(span, node);
            }
        });
    }

    onMount(async () => {
        await loadMetadata();
        await loadPage(currentPage, true);

        // Keyboard Navigation
        window.addEventListener('keydown', handleKeydown);
        // Iframe Navigation Message listener
        window.addEventListener('message', handleMessage);
    });

    onDestroy(() => {
        if (typeof window !== 'undefined') {
            window.removeEventListener('keydown', handleKeydown);
            window.removeEventListener('message', handleMessage);
        }
    });

    function handleMessage(event: MessageEvent) {
        if (event.data?.type === 'TH_NAVIGATE') {
            if (event.data.key === 'ArrowLeft' || event.data.swipe === 'right') {
                if (currentPage > 1) handlePageChange(currentPage - 1);
            } else if (event.data.key === 'ArrowRight' || event.data.swipe === 'left') {
                if (currentPage < totalPages) handlePageChange(currentPage + 1);
            }
        }
    }

    function handleKeydown(e: KeyboardEvent) {
        if (e.key === 'ArrowLeft') {
            if (currentPage > 1) handlePageChange(currentPage - 1);
        } else if (e.key === 'ArrowRight' || e.key.toLowerCase() === 'd') {
            if (currentPage < totalPages) handlePageChange(currentPage + 1);
        }
    }

    // Touch Navigation (SillyReader Style)
    let touchStartX = 0;
    let touchStartY = 0;
    let touchStartTime = 0;

    function handleTouchStart(e: TouchEvent) {
        touchStartX = e.changedTouches[0].clientX;
        touchStartY = e.changedTouches[0].clientY;
        touchStartTime = Date.now();
    }

    function handleTouchEnd(e: TouchEvent) {
        const touchEndX = e.changedTouches[0].clientX;
        const touchEndY = e.changedTouches[0].clientY;
        const diffX = touchEndX - touchStartX;
        const diffY = touchEndY - touchStartY;
        const duration = Date.now() - touchStartTime;

        // SillyReader Logic: < 300ms, > 50px distance, Horizontal > Vertical * 1.5
        if (duration < 300 && Math.abs(diffX) > 50 && Math.abs(diffX) > Math.abs(diffY) * 1.5) {
            if (diffX > 0) {
                // Swipe Right -> Prev Page
                if (currentPage > 1) handlePageChange(currentPage - 1);
            } else {
                // Swipe Left -> Next Page
                if (currentPage < totalPages) handlePageChange(currentPage + 1);
            }
        }
    }
</script>

<div class="flex flex-col h-screen bg-background" style="filter: brightness({textBrightness}%);">
    <!-- Header -->
    <header class="flex items-center gap-4 border-b px-6 py-3 bg-card/80 backdrop-blur-sm sticky top-0 z-10 justify-between">
        <div class="flex items-center gap-2 md:gap-4 flex-1 min-w-0 mr-2">
            <Button variant="ghost" size="icon" onclick={() => goto(`/characters/${cardId}?tab=chat`)} class="shrink-0">
                <ArrowLeft class="h-5 w-5" />
            </Button>
            <div class="flex flex-col min-w-0 flex-1 md:block">
                <div class="flex items-center gap-2">
                    <h1 class="text-base md:text-lg font-semibold truncate">{title}</h1>
                    <span class="md:hidden text-xs text-muted-foreground font-mono shrink-0 whitespace-nowrap">{globalProgress}%</span>
                </div>
                <div class="hidden md:flex items-center gap-2 text-xs text-muted-foreground">
                    <span class="font-mono">进度 {globalProgress}%</span>
                    <span>•</span>
                    <span>第 {currentPage} 页 / 共 {totalPages} 页</span>
                </div>
            </div>
        </div>
        <div class="flex items-center gap-2">
             <div class="w-32 mr-4 hidden md:block"><Progress value={globalProgress} class="h-2" /></div>
             <Popover.Root>
                <Popover.Trigger><Button variant="ghost" size="icon"><Settings class="h-5 w-5" /></Button></Popover.Trigger>
                <Popover.Content class="w-80">
                    <div class="grid gap-4">
                        <div class="space-y-2"><h4 class="font-medium leading-none">阅读设置</h4></div>
                        
                        {#if availableTags.size > 0}
                            <div class="grid gap-2 border-b pb-4">
                                <Label>标签显示</Label>
                                <p class="text-xs text-muted-foreground">选择的标签内容将会显示</p>
                                <div class="grid grid-cols-2 gap-2 max-h-40 overflow-y-auto">
                                    {#each sortTags(availableTags) as tag}
                                        <div class="flex items-center space-x-2">
                                            <Checkbox 
                                                id={`filter-${tag}`} 
                                                checked={!filterTags.includes(tag)} 
                                                onCheckedChange={(v) => toggleTagFilter(tag, !!v)} 
                                            />
                                            <Label for={`filter-${tag}`} class="text-xs font-normal cursor-pointer leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70">{tag}</Label>
                                        </div>
                                    {/each}
                                </div>
                            </div>
                            
                            <div class="grid gap-2 border-b pb-4">
                                <Label>标签分行</Label>
                                <p class="text-xs text-muted-foreground">选择的标签内容将会很好的分行显示，建议content和status必选（若有）</p>
                                <div class="grid grid-cols-2 gap-2 max-h-40 overflow-y-auto">
                                    {#each sortTags(availableTags) as tag}
                                        <div class="flex items-center space-x-2">
                                            <Checkbox 
                                                id={`newline-${tag}`} 
                                                checked={newlineTags.includes(tag)} 
                                                onCheckedChange={(v) => toggleNewlineTag(tag, !!v)} 
                                            />
                                            <Label for={`newline-${tag}`} class="text-xs font-normal cursor-pointer leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70">{tag}</Label>
                                        </div>
                                    {/each}
                                </div>
                            </div>
                        {/if}

                        <div class="grid gap-2"><Label>文字亮度 (深色模式)</Label><div class="flex items-center gap-4"><Slider value={[textBrightness]} max={100} step={5} onValueChange={handleBrightnessChange} class="flex-1"/><span class="w-8 text-sm text-right">{textBrightness}%</span></div></div>
                    </div>
                </Popover.Content>
             </Popover.Root>
        </div>
    </header>

    <!-- Content -->
    <div 
        class="flex-1 overflow-y-auto p-2 md:p-8 bg-muted/10 outline-none touch-pan-y" 
        onscroll={handleScroll}
        ontouchstart={handleTouchStart}
        ontouchend={handleTouchEnd}
        role="region"
        tabindex="0"
        aria-label="Chat History Content"
    >
        <div class="mx-auto max-w-3xl space-y-4">
            {#if isLoading}
                <div class="flex h-64 items-center justify-center"><Loader2 class="h-8 w-8 animate-spin text-primary" /></div>
            {:else}
                {#each floors as floor, i}
                    <div class={cn("rounded-lg border p-3 md:p-6 shadow-sm", i % 2 === 0 ? "bg-card" : "bg-card/50")}>
                        <div class="flex items-center justify-between mb-4 pb-2 border-b border-border/50">
                            <span class="font-mono text-xs text-muted-foreground bg-muted px-2 py-0.5 rounded">#{floor.floor}</span>
                            <span class="font-semibold opacity-90">{floor.name}</span>
                        </div>

                        <div class="leading-relaxed text-foreground/90 w-full min-h-[100px]">
                            <!-- SillyReader-style: direct HTML rendering with simple styling -->
                            {#if isTxtFormat}
                                <!-- Optimization: Render TXT directly without iframe for performance -->
                                <div class="p-2 md:p-4 whitespace-pre-wrap font-sans text-base leading-relaxed break-words text-foreground">{@html processText(floor.content)}</div>
                            {:else}
                                <HTMLRender content={`
                                    <style>
                                        html, body { 
                                            background: transparent !important;
                                            margin: 0;
                                            padding: 0;
                                            line-height: 1.8; 
                                            color: inherit; /* Inherit color from parent keys or set by class */
                                        }
                                        /* Default Light Mode Text */
                                        html:not(.dark) body { color: #333; }

                                        /* Dark Mode Text */
                                        html.dark body { color: #e0e0e0; }
                                        
                                        /* Inline Code */
                                        code { 
                                            background: rgba(128,128,128,0.2); 
                                            padding: 2px 6px; 
                                            border-radius: 4px; 
                                            font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace;
                                            font-size: 0.9em;
                                        }
                                        
                                        /* Block Code (Pre) */
                                        pre {
                                            background: #f5f5f5;
                                            border: 1px solid #e5e5e5;
                                            border-radius: 8px;
                                            padding: 1rem;
                                            margin: 1rem 0;
                                            overflow-x: auto;
                                            font-size: 0.9em;
                                            line-height: 1.5;
                                        }
                                        html.dark pre {
                                            background: #1e1e1e;
                                            border-color: #333;
                                        }
                                        @media (prefers-color-scheme: dark) {
                                            /* pre { background: #1e1e1e; border-color: #333; } Disabled */
                                        }
                                        
                                        /* Code inside Pre resets inline styles */
                                        pre code {
                                            background: transparent;
                                            padding: 0;
                                            border-radius: 0;
                                            color: inherit;
                                            font-size: inherit;
                                            white-space: pre;
                                        }
                                        q { color: #2e7d32; }
                                        html.dark q { color: #99cc99; }
                                        q::before { content: '"'; }
                                        q::after { content: '"'; }
                                        em { color: #b8860b; font-style: italic; }
                                        html.dark em { color: #ffcc00; }
                                        strong { color: #c62828; font-weight: bold; }
                                        html.dark strong { color: #ff9966; }
                                        del { color: #888888; text-decoration: line-through; }
                                    </style>
                                    <div style="padding: 0.5rem;">${processText(floor.content)}</div>
                                `} />
                            {/if}
                        </div>
                    </div>
                {/each}

                <!-- Pagination Controls -->
                <div class="flex flex-col md:flex-row items-center justify-center gap-4 py-8 mt-8">
                     <div class="flex items-center gap-2">
                         <Button variant="outline" size="icon" disabled={currentPage <= 1} onclick={() => handlePageChange(1)}>
                             <ChevronsLeft class="h-4 w-4" />
                         </Button>
                         <Button variant="outline" size="icon" disabled={currentPage <= 1} onclick={() => handlePageChange(currentPage - 1)}>
                             <ChevronLeft class="h-4 w-4" />
                         </Button>
                         
                         <div class="flex items-center gap-2 mx-2">
                             <Input 
                                type="number" 
                                class="w-16 text-center" 
                                min={1} 
                                max={totalPages}
                                bind:value={jumpPage} 
                                onkeydown={(e) => {
                                    if(e.key === 'Enter') handlePageChange(jumpPage);
                                }}
                             />
                             <span class="text-sm text-muted-foreground">/ {totalPages}</span>
                         </div>

                         <Button variant="outline" size="icon" disabled={currentPage >= totalPages} onclick={() => handlePageChange(currentPage + 1)}>
                             <ChevronRight class="h-4 w-4" />
                         </Button>
                         <Button variant="outline" size="icon" disabled={currentPage >= totalPages} onclick={() => handlePageChange(totalPages)}>
                             <ChevronsRight class="h-4 w-4" />
                         </Button>
                     </div>
                </div>
            {/if}
        </div>
    </div>
</div>
