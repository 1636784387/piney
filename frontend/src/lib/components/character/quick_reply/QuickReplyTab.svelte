<script lang="ts">
    // 快速回复 Tab 组件
    import { onMount } from "svelte";
    import { Button } from "$lib/components/ui/button";
    import { Plus, MessageSquareQuote } from "lucide-svelte";
    import { toast } from "svelte-sonner";
    import QuickReplyCard from "./QuickReplyCard.svelte";
    import ImportQuickReplyDialog from "./ImportQuickReplyDialog.svelte";
    import { Skeleton } from "$lib/components/ui/skeleton";
    import { API_BASE } from "$lib/api";

    let { cardId } = $props();

    let quickReplies: any[] = $state([]);
    let isLoading = $state(true);
    let isImportOpen = $state(false);

    async function loadQuickReplies() {
        if (!cardId) return;
        try {
            const token = localStorage.getItem("auth_token");
            const res = await fetch(`${API_BASE}/api/cards/${cardId}/quick_reply`, {
                headers: token ? { Authorization: `Bearer ${token}` } : {},
            });
            if (res.ok) {
                quickReplies = await res.json();
            }
        } catch (e) {
            console.error(e);
            toast.error("加载快速回复失败");
        } finally {
            isLoading = false;
        }
    }

    onMount(() => {
        loadQuickReplies();
    });

    function handleDelete(id: string) {
        // 乐观更新
        const prev = [...quickReplies];
        quickReplies = quickReplies.filter(qr => qr.id !== id);

        const token = localStorage.getItem("auth_token");
        fetch(`${API_BASE}/api/cards/${cardId}/quick_reply/${id}`, { 
            method: 'DELETE',
            headers: token ? { Authorization: `Bearer ${token}` } : {},
        })
            .then(res => {
                if (!res.ok) throw new Error();
                toast.success("已删除");
            })
            .catch(() => {
                quickReplies = prev;
                toast.error("删除失败");
            });
    }

    function handleUpdate(id: string, payload: any) {
        if (payload === null) {
            // 刷新列表
            loadQuickReplies();
            return;
        }

        // 乐观更新
        const idx = quickReplies.findIndex(qr => qr.id === id);
        if (idx === -1) return;
        
        const prev = quickReplies[idx];
        quickReplies[idx] = { ...prev, ...payload };
        
        const token = localStorage.getItem("auth_token");
        fetch(`${API_BASE}/api/cards/${cardId}/quick_reply/${id}`, {
            method: 'PATCH',
            headers: { 
                'Content-Type': 'application/json',
                ...(token ? { Authorization: `Bearer ${token}` } : {}),
            },
            body: JSON.stringify(payload)
        }).catch(() => {
            quickReplies[idx] = prev;
            toast.error("更新失败");
        });
    }

    function onImport(newItems: any[]) {
        // 将新导入的项添加到列表开头
        quickReplies = [...newItems, ...quickReplies];
    }
</script>

<div class="flex flex-col h-full min-h-[400px]">
    {#if isLoading}
        <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
            {#each Array(3) as _}
                <Skeleton class="h-[120px] w-full rounded-lg" />
            {/each}
        </div>
    {:else if quickReplies.length === 0}
        <div class="flex flex-col items-center justify-center flex-1 border-2 border-dashed rounded-lg bg-muted/10 m-2">
            <div class="flex h-20 w-20 items-center justify-center rounded-full bg-muted/50 mb-6">
                <MessageSquareQuote class="h-10 w-10 text-muted-foreground" />
            </div>
            <h3 class="text-xl font-semibold mb-2">还没有导入快速回复（QR）...</h3>
            <p class="text-muted-foreground text-sm mb-6 max-w-sm text-center">
                您可以导入 JSON 格式的快速回复文件，便于管理和使用。
            </p>
            <Button size="lg" onclick={() => isImportOpen = true}>
                <Plus class="mr-2 h-5 w-5" />
                导入快速回复
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
            {#each quickReplies as qr (qr.id)}
                <QuickReplyCard 
                    quickReply={qr} 
                    {cardId}
                    onDelete={handleDelete}
                    onUpdate={handleUpdate}
                />
            {/each}
        </div>
    {/if}

    <ImportQuickReplyDialog 
        bind:open={isImportOpen} 
        {cardId} 
        {onImport} 
    />
</div>
