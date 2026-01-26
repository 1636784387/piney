<script lang="ts">
    // 小剧场卡片组件
    import { Copy, Trash2 } from "lucide-svelte";
    import { Button } from "$lib/components/ui/button";
    import * as AlertDialog from "$lib/components/ui/alert-dialog";
    import { cn } from "$lib/utils";
    import { toast } from "svelte-sonner";

    let { theater, isSelected = false, isSelectionMode = false, onEdit, onDelete, onSelect } = $props();

    let deleteDialogOpen = $state(false);

    // 莫兰迪色系 - 10种浅色背景 + 对应文字颜色
    const categoryColors = [
        { bg: "bg-rose-100", text: "text-rose-700" },
        { bg: "bg-sky-100", text: "text-sky-700" },
        { bg: "bg-emerald-100", text: "text-emerald-700" },
        { bg: "bg-amber-100", text: "text-amber-700" },
        { bg: "bg-violet-100", text: "text-violet-700" },
        { bg: "bg-teal-100", text: "text-teal-700" },
        { bg: "bg-orange-100", text: "text-orange-700" },
        { bg: "bg-indigo-100", text: "text-indigo-700" },
        { bg: "bg-pink-100", text: "text-pink-700" },
        { bg: "bg-lime-100", text: "text-lime-700" },
    ];

    // 根据分类名称获取颜色（根据字符串哈希值选择）
    function getCategoryColor(category: string) {
        let hash = 0;
        for (let i = 0; i < category.length; i++) {
            hash = category.charCodeAt(i) + ((hash << 5) - hash);
        }
        const index = Math.abs(hash) % categoryColors.length;
        return categoryColors[index];
    }

    async function handleCopy(e: MouseEvent) {
        e.stopPropagation();
        try {
            await navigator.clipboard.writeText(theater.content);
            toast.success("内容已复制到剪贴板");
        } catch (err) {
            toast.error("复制失败");
        }
    }

    function handleCardClick() {
        if (isSelectionMode) {
            onSelect?.(theater.id);
        } else {
            onEdit?.(theater);
        }
    }

    function handleDeleteClick(e: MouseEvent) {
        e.stopPropagation();
        deleteDialogOpen = true;
    }

    function confirmDelete() {
        onDelete?.(theater.id);
        deleteDialogOpen = false;
    }
</script>

<div
    class={cn(
        "relative flex flex-col rounded-lg border bg-card shadow-sm transition-all hover:shadow-md hover:border-primary/30 cursor-pointer",
        isSelected && "ring-2 ring-primary bg-primary/5"
    )}
    role="button"
    tabindex="0"
    onclick={handleCardClick}
    onkeydown={(e) => e.key === 'Enter' && handleCardClick()}
>
    <!-- 多选复选框 -->
    {#if isSelectionMode}
        <div class="absolute top-2 left-2 z-10">
            <input
                type="checkbox"
                class="h-4 w-4 rounded border-gray-300 text-primary accent-primary focus:ring-primary shadow-sm cursor-pointer"
                checked={isSelected}
                onclick={(e) => e.stopPropagation()}
                onchange={() => onSelect?.(theater.id)}
            />
        </div>
    {/if}

    <!-- 主内容区域 -->
    <div class={cn("p-3 flex-1", isSelectionMode && "pl-8")}>
        <!-- 标题行 -->
        <div class="flex items-start justify-between gap-2 mb-1.5">
            <h3 class="font-medium text-sm line-clamp-1 flex-1" title={theater.title}>
                {theater.title}
            </h3>
            {#if theater.category && theater.category !== "未分类"}
                {@const color = getCategoryColor(theater.category)}
                <span class={cn("text-[10px] px-1.5 py-0.5 rounded-md shrink-0 font-medium", color.bg, color.text)}>
                    {theater.category}
                </span>
            {/if}
        </div>

        <!-- 简介 -->
        <p class="text-xs text-muted-foreground line-clamp-2" title={theater.desc}>
            {theater.desc}
        </p>
    </div>

    <!-- 操作按钮 - 始终在底部 -->
    <div class="flex items-center justify-between px-2 py-1.5 border-t bg-muted/30">
        <!-- 左侧：删除 -->
        <Button
            variant="ghost"
            size="icon"
            class="h-7 w-7 text-muted-foreground hover:text-muted-foreground"
            title="删除"
            onclick={handleDeleteClick}
        >
            <Trash2 class="h-3.5 w-3.5" />
        </Button>
        
        <!-- 右侧：复制 -->
        <Button
            variant="ghost"
            size="icon"
            class="h-7 w-7"
            title="复制内容"
            onclick={handleCopy}
        >
            <Copy class="h-3.5 w-3.5" />
        </Button>
    </div>
</div>

<!-- 删除确认对话框 -->
<AlertDialog.Root bind:open={deleteDialogOpen}>
    <AlertDialog.Content onclick={(e: MouseEvent) => e.stopPropagation()}>
        <AlertDialog.Header>
            <AlertDialog.Title>确定要删除吗？</AlertDialog.Title>
            <AlertDialog.Description>
                将删除小剧场「{theater.title}」，此操作不可恢复。
            </AlertDialog.Description>
        </AlertDialog.Header>
        <AlertDialog.Footer>
            <AlertDialog.Cancel>取消</AlertDialog.Cancel>
            <AlertDialog.Action
                class="bg-destructive !text-destructive-foreground hover:bg-destructive/90"
                onclick={confirmDelete}
            >
                确认删除
            </AlertDialog.Action>
        </AlertDialog.Footer>
    </AlertDialog.Content>
</AlertDialog.Root>

