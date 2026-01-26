<script lang="ts">
    // 快速回复卡片组件
    import { FileJson, Trash2, Download, Edit, MoreVertical } from "lucide-svelte";
    import { Button } from "$lib/components/ui/button";
    import * as DropdownMenu from "$lib/components/ui/dropdown-menu";
    import { cn } from "$lib/utils";
    import { toast } from "svelte-sonner";
    import { API_BASE } from "$lib/api";
    import RenameQuickReplyDialog from "./RenameQuickReplyDialog.svelte";

    let { quickReply, cardId, onDelete, onUpdate } = $props();

    let isRenameOpen = $state(false);

    function formatSize(bytes: number) {
        if (bytes === 0) return "0 B";
        const k = 1024;
        const m = k * k;
        if (bytes >= m) {
            return (bytes / m).toFixed(2) + " MB";
        }
        return (bytes / k).toFixed(2) + " KB";
    }

    function formatDate(dateStr: string) {
        const date = new Date(dateStr);
        return date.toLocaleDateString('zh-CN', {
            year: 'numeric',
            month: '2-digit',
            day: '2-digit'
        });
    }

    async function handleExport() {
        try {
            const token = localStorage.getItem("auth_token");
            const res = await fetch(`${API_BASE}/api/cards/${cardId}/quick_reply/${quickReply.id}/export`, {
                headers: token ? { Authorization: `Bearer ${token}` } : {},
            });
            
            if (!res.ok) throw new Error("导出失败");
            
            const blob = await res.blob();
            const url = window.URL.createObjectURL(blob);
            const a = document.createElement("a");
            a.href = url;
            a.download = `${quickReply.display_name}.json`;
            document.body.appendChild(a);
            a.click();
            window.URL.revokeObjectURL(url);
            document.body.removeChild(a);
            
            toast.success("导出成功");
        } catch (e) {
            console.error(e);
            toast.error("导出失败");
        }
    }

    function handleDelete() {
        if (confirm("确定要删除这个快速回复吗？删除后不可恢复。")) {
            onDelete(quickReply.id);
        }
    }
</script>

<div class="group relative flex flex-col justify-between rounded-lg border bg-card p-4 transition-all hover:shadow-md">
    <!-- JSON 角标 -->
    <div class="absolute top-2 right-2">
        <span class="inline-flex items-center rounded-full bg-blue-100 px-2 py-0.5 text-[10px] font-medium text-blue-700 dark:bg-blue-900/30 dark:text-blue-400">
            JSON
        </span>
    </div>

    <div class="flex items-start gap-3 pr-12">
        <div class={cn(
            "flex h-10 w-10 shrink-0 items-center justify-center rounded-md border",
            "bg-blue-50 text-blue-600 border-blue-200 dark:bg-blue-950 dark:text-blue-400 dark:border-blue-800"
        )}>
            <FileJson class="h-5 w-5" />
        </div>
        
        <div class="flex flex-col overflow-hidden flex-1 min-w-0">
            <span 
                class="truncate text-sm font-medium" 
                title={quickReply.display_name}
            >
                {quickReply.display_name}
            </span>
            <div class="flex items-center gap-2 text-xs text-muted-foreground mt-1">
                <span>{formatSize(quickReply.file_size)}</span>
                <span>•</span>
                <span>{formatDate(quickReply.created_at)}</span>
            </div>
        </div>
    </div>

    <!-- 操作按钮 -->
    <div class="mt-4 flex items-center justify-end gap-2">
        <DropdownMenu.Root>
            <DropdownMenu.Trigger>
                {#snippet child({ props })}
                    <Button variant="outline" size="sm" class="gap-2" {...props}>
                        <MoreVertical class="h-4 w-4" />
                        操作
                    </Button>
                {/snippet}
            </DropdownMenu.Trigger>
            <DropdownMenu.Content align="end">
                <DropdownMenu.Item onclick={() => isRenameOpen = true} class="gap-2 cursor-pointer">
                    <Edit class="h-4 w-4" />
                    重命名
                </DropdownMenu.Item>
                <DropdownMenu.Item onclick={handleExport} class="gap-2 cursor-pointer">
                    <Download class="h-4 w-4" />
                    导出
                </DropdownMenu.Item>
                <DropdownMenu.Separator />
                <DropdownMenu.Item onclick={handleDelete} class="gap-2 cursor-pointer text-destructive focus:text-destructive">
                    <Trash2 class="h-4 w-4" />
                    删除
                </DropdownMenu.Item>
            </DropdownMenu.Content>
        </DropdownMenu.Root>
    </div>
</div>

<RenameQuickReplyDialog 
    bind:open={isRenameOpen} 
    {quickReply}
    {cardId}
    {onUpdate}
/>

