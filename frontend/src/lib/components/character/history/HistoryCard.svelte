<script lang="ts">
    import { FileText, FileJson, Trash2, BookOpen, Edit } from "lucide-svelte";
    import { Button } from "$lib/components/ui/button";
    import { Progress } from "$lib/components/ui/progress";
    import * as Tooltip from "$lib/components/ui/tooltip";
    import { cn } from "$lib/utils";
    import { goto } from "$app/navigation";
    import EditHistoryDialog from "./EditHistoryDialog.svelte";

    let { history, cardId, onDelete, onUpdate } = $props();

    let isEditOpen = $state(false);

    function formatSize(bytes: number) {
        if (bytes === 0) return "0 B";
        const k = 1024;
        const m = k * k;
        if (bytes >= m) {
            return (bytes / m).toFixed(2) + " MB";
        }
        return (bytes / k).toFixed(2) + " KB";
    }

    function goToReader() {
        goto(`/characters/${history.card_id}/history?history_id=${history.id}`);
    }

    function handleUpdate() {
        onUpdate(history.id, null);
    }
</script>

<div class="group relative flex flex-col justify-between rounded-lg border bg-card p-4 transition-all hover:shadow-md">
    <div class="flex items-start justify-between gap-3">
        <div class="flex items-center gap-2 overflow-hidden">
            <div class={cn(
                "flex h-8 w-8 shrink-0 items-center justify-center rounded-md border",
                history.format === "jsonl" ? "bg-blue-50 text-blue-600 border-blue-200" : "bg-orange-50 text-orange-600 border-orange-200"
            )}>
                 <!-- Always show TXT icon effectively since we convert, but keep format check if legacy -->
                {#if history.format === "jsonl"}
                    <FileJson class="h-4 w-4" />
                {:else}
                    <FileText class="h-4 w-4" />
                {/if}
            </div>
            
            <div class="flex flex-col overflow-hidden">
                <span class="truncate text-sm font-medium" title={history.display_name}>
                    {history.display_name}
                </span>
                <div class="flex items-center gap-2 text-xs text-muted-foreground">
                    <span class="font-mono">{history.format.toUpperCase()}</span>
                    <span>•</span>
                    <span>{formatSize(history.file_size)}</span>
                </div>
            </div>
        </div>

        <div class="flex items-center gap-1 opacity-100 sm:opacity-0 sm:group-hover:opacity-100 transition-opacity">
            <Tooltip.Root>
                <Tooltip.Trigger>
                    <Button variant="ghost" size="icon" class="h-8 w-8 text-muted-foreground hover:text-primary" onclick={() => isEditOpen = true}>
                        <Edit class="h-4 w-4" />
                    </Button>
                </Tooltip.Trigger>
                <Tooltip.Content>编辑记录</Tooltip.Content>
            </Tooltip.Root>

            <Tooltip.Root>
                <Tooltip.Trigger>
                    <Button variant="ghost" size="icon" class="h-8 w-8 text-muted-foreground hover:text-destructive" onclick={() => onDelete(history.id)}>
                        <Trash2 class="h-4 w-4" />
                    </Button>
                </Tooltip.Trigger>
                <Tooltip.Content>删除记录</Tooltip.Content>
            </Tooltip.Root>
        </div>
    </div>

    <div class="mt-4 space-y-2">
        <div class="flex items-center justify-between text-xs text-muted-foreground">
            <span>阅读进度</span>
            <span>{history.progress}%</span>
        </div>
        <Progress value={history.progress} class="h-1.5" />
    </div>

    <div class="mt-4 flex items-center justify-end">
        <Button size="sm" class="w-full gap-2 transition-transform active:scale-95" onclick={goToReader}>
            <BookOpen class="h-4 w-4" />
            开始阅读
        </Button>
    </div>
</div>

<EditHistoryDialog 
    bind:open={isEditOpen} 
    {history} 
    {cardId} 
    onUpdate={handleUpdate} 
/>
