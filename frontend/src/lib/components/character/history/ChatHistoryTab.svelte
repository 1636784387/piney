<script lang="ts">
    import { onMount } from "svelte";
    import { Button } from "$lib/components/ui/button";
    import { Plus, MessageSquareDashed } from "lucide-svelte";
    import { toast } from "svelte-sonner";
    import HistoryCard from "./HistoryCard.svelte";
    import ImportHistoryDialog from "./ImportHistoryDialog.svelte";
    import { Skeleton } from "$lib/components/ui/skeleton";
    import { API_BASE } from "$lib/api";

    let { cardId } = $props();

    let histories: any[] = $state([]);
    let isLoading = $state(true);
    let isImportOpen = $state(false);

    async function loadHistories() {
        if (!cardId) return;
        try {
            const token = localStorage.getItem("auth_token");
            const res = await fetch(`${API_BASE}/api/cards/${cardId}/history`, {
                headers: token ? { Authorization: `Bearer ${token}` } : {},
            });
            if (res.ok) {
                histories = await res.json();
            }
        } catch (e) {
            console.error(e);
            toast.error("加载聊天记录失败");
        } finally {
            isLoading = false;
        }
    }

    onMount(() => {
        loadHistories();
    });

    function handleDelete(id: string) {
        if (!confirm("确定要删除这条记录吗？删除后不可恢复。")) return;
        
        const prev = [...histories];
        histories = histories.filter(h => h.id !== id);

        const token = localStorage.getItem("auth_token");
        fetch(`${API_BASE}/api/cards/${cardId}/history/${id}`, { 
            method: 'DELETE',
            headers: token ? { Authorization: `Bearer ${token}` } : {},
        })
            .then(res => {
                if (!res.ok) throw new Error();
                toast.success("已删除");
            })
            .catch(() => {
                histories = prev;
                toast.error("删除失败");
            });
    }

    function handleUpdate(id: string, payload: any) {
        if (payload === null) {
            loadHistories();
            return;
        }
        const idx = histories.findIndex(h => h.id === id);
        if (idx === -1) return;
        
        const prev = histories[idx];
        histories[idx] = { ...prev, ...payload };
        
        const token = localStorage.getItem("auth_token");
        fetch(`${API_BASE}/api/cards/${cardId}/history/${id}`, {
            method: 'PATCH',
            headers: { 
                'Content-Type': 'application/json',
                ...(token ? { Authorization: `Bearer ${token}` } : {}),
            },
            body: JSON.stringify(payload)
        }).catch(() => {
            histories[idx] = prev;
            toast.error("更新失败");
        });
    }

    function onImport(newItems: any[]) {
        histories = [...newItems, ...histories];
    }
</script>

<div class="flex flex-col h-full min-h-[400px]">
    {#if isLoading}
        <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
            {#each Array(3) as _}
                <Skeleton class="h-[140px] w-full rounded-lg" />
            {/each}
        </div>
    {:else if histories.length === 0}
        <div class="flex flex-col items-center justify-center flex-1 border-2 border-dashed rounded-lg bg-muted/10 m-2">
            <div class="flex h-20 w-20 items-center justify-center rounded-full bg-muted/50 mb-6">
                <MessageSquareDashed class="h-10 w-10 text-muted-foreground" />
            </div>
            <h3 class="text-xl font-semibold mb-2">还没有导入聊天记录...</h3>
            <p class="text-muted-foreground text-sm mb-6 max-w-sm text-center">
                您可以导入 SillyTavern 导出的 .jsonl 文件或普通的 .txt 文本记录，方便在网页端阅读。
            </p>
            <Button size="lg" onclick={() => isImportOpen = true}>
                <Plus class="mr-2 h-5 w-5" />
                导入聊天记录
            </Button>
        </div>
    {:else}
        <div class="flex justify-end mb-4">
            <Button variant="outline" onclick={() => isImportOpen = true} class="gap-2 border-primary bg-background text-foreground hover:bg-primary/10">
                <Plus class="mr-2 h-4 w-4" />
                导入更多
            </Button>
        </div>
        
        <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4 pb-10">
            {#each histories as history (history.id)}
                <HistoryCard 
                    {history} 
                    {cardId}
                    onDelete={handleDelete}
                    onUpdate={handleUpdate}
                />
            {/each}
        </div>
    {/if}

    <ImportHistoryDialog 
        bind:open={isImportOpen} 
        {cardId} 
        {onImport} 
    />
</div>
