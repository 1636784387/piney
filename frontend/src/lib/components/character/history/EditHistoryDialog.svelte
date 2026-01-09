<script lang="ts">
    import * as Dialog from "$lib/components/ui/dialog";
    import { Button } from "$lib/components/ui/button";
    import { Input } from "$lib/components/ui/input";
    import { Label } from "$lib/components/ui/label";
    import { Loader2, FileJson, AlertCircle, RotateCcw } from "lucide-svelte";
    import { toast } from "svelte-sonner";
    import { cn } from "$lib/utils";
    import { convertJsonlToTxt, scanTags } from "$lib/utils/exportUtils";

    let { open = $bindable(false), history, cardId, onUpdate } = $props();

    // Strip extension for display
    let name = $state(history.display_name.replace(/\.(txt|jsonl)$/i, ''));
    let isProcessing = $state(false);
    let step: 'main' | 'tags' = $state('main');

    // Tag Reselection State
    let rawSource = "";
    let availableTags: string[] = $state([]);
    let selectedTags: string[] = $state([]);

    $effect(() => {
        if (open) {
            // Strip extension for display whenever dialog opens
            name = history.display_name.replace(/\.(txt|jsonl)$/i, '');
            step = 'main';
        }
    });

    async function fetchSource() {
        isProcessing = true;
        try {
            const token = localStorage.getItem("auth_token");
            const res = await fetch(`http://localhost:9696/api/cards/${cardId}/history/${history.id}/content?source=true`, {
                headers: token ? { Authorization: `Bearer ${token}` } : {},
            });
            if (!res.ok) throw new Error("Failed to fetch source file");
            
            const text = await res.text();
            rawSource = text;
            availableTags = scanTags(text);
            selectedTags = [...availableTags]; // Default select all
            step = 'tags';
        } catch (e) {
            toast.error("无法获取源文件，可能源文件不存在");
        } finally {
            isProcessing = false;
        }
    }

    function toggleTag(tag: string) {
        if (selectedTags.includes(tag)) {
            selectedTags = selectedTags.filter(t => t !== tag);
        } else {
            selectedTags = [...selectedTags, tag];
        }
    }

    function toggleAllTags() {
        if (selectedTags.length === availableTags.length) {
            selectedTags = [];
        } else {
            selectedTags = [...availableTags];
        }
    }

    async function handleSave() {
        isProcessing = true;
        try {
            const token = localStorage.getItem("auth_token");
            
            // 1. If in 'tags' mode, convert and upload content
            if (step === 'tags') {
                const txt = convertJsonlToTxt(rawSource, selectedTags);
                const txtBlob = new Blob([txt], { type: "text/plain;charset=utf-8" });
                
                const formData = new FormData();
                formData.append('file', txtBlob, history.file_name); // Keep original filename for overwrite? Or does backend handle it? source_file not needed here as we don't update it.

                const resContent = await fetch(`http://localhost:9696/api/cards/${cardId}/history/${history.id}/content`, {
                    method: 'PUT',
                    headers: token ? { Authorization: `Bearer ${token}` } : {},
                    body: formData
                });
                if (!resContent.ok) throw new Error("Failed to update content");
            }

            // 2. Update metadata (Name)
            if (name !== history.display_name) {
                const resMeta = await fetch(`http://localhost:9696/api/cards/${cardId}/history/${history.id}`, {
                    method: 'PATCH',
                    headers: {
                        'Content-Type': 'application/json',
                        ...(token ? { Authorization: `Bearer ${token}` } : {}),
                    },
                    body: JSON.stringify({ display_name: name })
                });
                if (!resMeta.ok) throw new Error("Global update failed");
            }

            // Notify parent
            onUpdate();
            toast.success("更新成功");
            open = false;
        } catch (e) {
            console.error(e);
            toast.error("更新失败");
        } finally {
            isProcessing = false;
        }
    }
</script>

<Dialog.Root bind:open={open}>
    <Dialog.Content class="sm:max-w-[500px]">
        <Dialog.Header>
            <Dialog.Title>编辑记录</Dialog.Title>
        </Dialog.Header>

        <div class="py-4 space-y-6">
            {#if step === 'main'}
                <div class="space-y-2">
                    <Label>记录名称</Label>
                    <Input bind:value={name} placeholder="请输入名称" />
                </div>

                <div class="space-y-2">
                    <Label>源文件操作</Label>
                    {#if history.source_file_name}
                        <div class="flex items-center justify-between p-3 border rounded-md bg-muted/30">
                            <div class="flex items-center gap-2 text-sm">
                                <FileJson class="h-4 w-4 text-blue-500" />
                                <span class="truncate max-w-[200px]" title={history.source_file_name}>
                                    {history.source_file_name}
                                </span>
                            </div>
                            <Button variant="outline" size="sm" onclick={fetchSource} disabled={isProcessing}>
                                {#if isProcessing}
                                    <Loader2 class="mr-2 h-3 w-3 animate-spin" />
                                {/if}
                                <RotateCcw class="mr-2 h-3 w-3" />
                                重选标签
                            </Button>
                        </div>
                        <p class="text-xs text-muted-foreground">
                            可以通过源文件重新生成显示内容 (例如修改屏蔽的标签)
                        </p>
                    {:else}
                         <div class="p-3 border rounded-md bg-muted/50 text-sm text-muted-foreground text-center">
                            无关联的源文件 (JSONL)，无法重新处理
                         </div>
                    {/if}
                </div>
            {:else}
                <!-- Tag Reselection UI -->
                <div class="space-y-4">
                     <div class="flex items-center justify-between">
                         <h4 class="text-sm font-semibold">选择保留的标签</h4>
                         <Button variant="ghost" size="sm" class="h-auto p-0 text-xs" onclick={toggleAllTags}>
                            {selectedTags.length === availableTags.length ? '全不选' : '全选'}
                         </Button>
                     </div>

                     <div class="border rounded-md p-4 bg-muted/20 space-y-3 max-h-[300px] overflow-y-auto">
                        {#if availableTags.length === 0}
                            <p class="text-xs text-muted-foreground text-center py-4">未检测到任何 XML 标签对。</p>
                        {:else}
                            <div class="flex flex-wrap gap-2">
                                {#each availableTags as tag}
                                    <button 
                                        class={cn(
                                            "px-3 py-1 rounded-full text-xs border transition-colors",
                                            selectedTags.includes(tag) 
                                                ? "bg-primary text-primary-foreground border-primary" 
                                                : "bg-muted text-muted-foreground border-transparent hover:bg-muted/80"
                                        )}
                                        onclick={() => toggleTag(tag)}
                                    >
                                        {tag}
                                    </button>
                                {/each}
                            </div>
                        {/if}
                        <p class="text-xs text-muted-foreground bg-yellow-50 dark:bg-yellow-900/20 p-2 rounded text-yellow-600 dark:text-yellow-400">
                             <AlertCircle class="w-3 h-3 inline mr-1" />
                             注意：这将覆盖当前的文本内容。
                        </p>
                     </div>
                </div>
            {/if}
        </div>

        <Dialog.Footer>
            {#if step === 'tags'}
                <Button variant="outline" onclick={() => step = 'main'} disabled={isProcessing}>返回</Button>
            {:else}
                <Button variant="outline" onclick={() => open = false} disabled={isProcessing}>取消</Button>
            {/if}
            
            <Button onclick={handleSave} disabled={isProcessing}>
                {#if isProcessing}
                    <Loader2 class="mr-2 h-4 w-4 animate-spin" />
                    保存
                {:else}
                    {#if step === 'tags'}
                        覆盖并保存
                    {:else}
                        保存修改
                    {/if}
                {/if}
            </Button>
        </Dialog.Footer>
    </Dialog.Content>
</Dialog.Root>
