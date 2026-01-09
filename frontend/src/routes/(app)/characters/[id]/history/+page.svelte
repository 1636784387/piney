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

    const historyId = $page.url.searchParams.get("history_id");
    const cardId = $page.params.id;

    // State
    let isLoading = $state(true);
    let title = $state("阅读模式");
    let characterName = $state("");
    
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
    let floors: ChatMessage[] = $state([]);
    
    // Settings
    let textBrightness = $state(100);
    let savedScrollRatio = 0; // Temp store for restoration
    
    // Auto-save timer
    let saveTimeout: any;

    async function loadMetadata() {
        if (!cardId) return;
        const token = localStorage.getItem("auth_token");
        
        // 1. Get Character Name for Breadcrumb
        try {
            const cardRes = await fetch(`http://localhost:9696/api/cards/${cardId}`, {
                headers: token ? { Authorization: `Bearer ${token}` } : {},
            });
            if (cardRes.ok) {
                const card = await cardRes.json();
                characterName = card.name;
            }
        } catch {}

        // 2. Get History Metadata
        try {
            const res = await fetch(`http://localhost:9696/api/cards/${cardId}/history`, {
                headers: token ? { Authorization: `Bearer ${token}` } : {},
            });
            if (res.ok) {
                const list = await res.json();
                const item = list.find((h: any) => h.id === historyId);
                if (item) {
                    title = item.display_name;
                    currentPage = item.current_page || 1;
                    globalProgress = item.progress || 0;
                    if (item.reading_settings) {
                        try {
                            const settings = JSON.parse(item.reading_settings);
                            if (settings.brightness) textBrightness = settings.brightness;
                            if (settings.scroll_ratio) savedScrollRatio = settings.scroll_ratio;
                        } catch {}
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
        isLoading = true;
        try {
            const token = localStorage.getItem("auth_token");
            const res = await fetch(`http://localhost:9696/api/cards/${cardId}/history/${historyId}/content?page=${p}`, {
                headers: token ? { Authorization: `Bearer ${token}` } : {},
            });
            
            if (!res.ok) throw new Error("加载失败");
            
            const data = await res.json();
            
            totalPages = data.total_pages;
            currentPage = data.current_page;
            floors = data.floors;
            jumpPage = currentPage;

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

        } catch (e) {
            console.error(e);
            toast.error("加载内容失败");
        } finally {
            isLoading = false;
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
                scroll_ratio: savedScrollRatio 
            });
            
            await fetch(`http://localhost:9696/api/cards/${cardId}/history/${historyId}`, {
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
        clearTimeout(saveTimeout);
        saveTimeout = setTimeout(() => {
            saveProgress();
        }, 1000);
    }

    onMount(async () => {
        await loadMetadata();
        // Load with restoreScroll=true for the initial load
        await loadPage(currentPage, true);
    });
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
                    <!-- Mobile Progress (Inline) -->
                    <span class="md:hidden text-xs text-muted-foreground font-mono shrink-0 whitespace-nowrap">
                        {globalProgress}%
                    </span>
                </div>
                <!-- Desktop Progress (Subtitle) -->
                <div class="hidden md:flex items-center gap-2 text-xs text-muted-foreground">
                    <span class="font-mono">进度 {globalProgress}%</span>
                    <span>•</span>
                    <span>第 {currentPage} 页 / 共 {totalPages} 页</span>
                </div>
            </div>
        </div>

        <div class="flex items-center gap-2">
             <div class="w-32 mr-4 hidden md:block">
                 <Progress value={globalProgress} class="h-2" />
             </div>
        
             <!-- Settings Popover -->
             <Popover.Root>
                <Popover.Trigger>
                    <Button variant="ghost" size="icon">
                        <Settings class="h-5 w-5" />
                    </Button>
                </Popover.Trigger>
                <Popover.Content class="w-80">
                    <div class="grid gap-4">
                        <div class="space-y-2">
                            <h4 class="font-medium leading-none">阅读设置</h4>
                            <p class="text-xs text-muted-foreground">
                                调整阅读体验
                            </p>
                        </div>
                        <div class="grid gap-2">
                            <Label>文字亮度 (深色模式)</Label>
                            <div class="flex items-center gap-4">
                                <Slider 
                                    value={[textBrightness]} 
                                    max={100} 
                                    step={5} 
                                    onValueChange={handleBrightnessChange} 
                                    class="flex-1"
                                />
                                <span class="w-8 text-sm text-right">{textBrightness}%</span>
                            </div>
                        </div>
                    </div>
                </Popover.Content>
             </Popover.Root>
        </div>
    </header>

    <!-- Content -->
    <div 
        class="flex-1 overflow-y-auto p-4 md:p-8 bg-muted/10"
        onscroll={handleScroll}
    >
        <div class="mx-auto max-w-3xl space-y-4">
            {#if isLoading}
                <div class="flex h-64 items-center justify-center">
                    <Loader2 class="h-8 w-8 animate-spin text-primary" />
                </div>
            {:else}
                {#each floors as floor, i}
                    <div 
                        class={cn(
                            "rounded-lg border p-6 shadow-sm",
                            i % 2 === 0 ? "bg-card" : "bg-card/50" // Odd/Even styling
                        )}
                    >
                        <div class="flex items-center justify-between mb-4 pb-2 border-b border-border/50">
                            <span class="font-mono text-xs text-muted-foreground bg-muted px-2 py-0.5 rounded">
                                #{floor.floor}
                            </span>
                            <span class="font-semibold opacity-90">{floor.name}</span>
                        </div>
                        <div class="whitespace-pre-wrap leading-relaxed text-foreground/90">
                            {floor.content}
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
