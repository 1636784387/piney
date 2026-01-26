<script lang="ts">
    // 导入小剧场对话框
    import * as Dialog from "$lib/components/ui/dialog";
    import { Button } from "$lib/components/ui/button";
    import { Upload, FileText, Loader2, AlertCircle } from "lucide-svelte";
    import { toast } from "svelte-sonner";
    import { cn } from "$lib/utils";
    import { API_BASE } from "$lib/api";

    let { open = $bindable(false), onSuccess } = $props();

    let dragActive = $state(false);
    let isUploading = $state(false);
    let selectedFile: File | null = $state(null);

    $effect(() => {
        if (!open) {
            reset();
        }
    });

    function reset() {
        selectedFile = null;
        isUploading = false;
    }

    function validateFile(file: File): boolean {
        const ext = '.' + file.name.split('.').pop()?.toLowerCase();
        if (ext !== '.txt') {
            toast.warning("仅支持 TXT 格式文件");
            return false;
        }
        if (file.size > 50 * 1024 * 1024) {
            toast.warning("文件大小不能超过 50MB");
            return false;
        }
        return true;
    }

    function handleDrag(e: DragEvent) {
        e.preventDefault();
        e.stopPropagation();
        if (e.type === "dragenter" || e.type === "dragover") {
            dragActive = true;
        } else if (e.type === "dragleave") {
            dragActive = false;
        }
    }

    function handleDrop(e: DragEvent) {
        e.preventDefault();
        e.stopPropagation();
        dragActive = false;
        if (e.dataTransfer?.files && e.dataTransfer.files.length > 0) {
            const file = e.dataTransfer.files[0];
            if (validateFile(file)) {
                selectedFile = file;
            }
        }
    }

    function handleFileSelect(e: Event) {
        const input = e.target as HTMLInputElement;
        if (input.files && input.files.length > 0) {
            const file = input.files[0];
            if (validateFile(file)) {
                selectedFile = file;
            }
        }
        input.value = "";
    }

    async function handleUpload() {
        if (!selectedFile) return;
        isUploading = true;

        try {
            const formData = new FormData();
            formData.append('file', selectedFile);

            const token = localStorage.getItem("auth_token");
            const res = await fetch(`${API_BASE}/api/theaters/import`, {
                method: 'POST',
                headers: {
                   ...(token ? { Authorization: `Bearer ${token}` } : {}),
                },
                body: formData
            });

            if (!res.ok) {
                const errText = await res.text();
                throw new Error(errText || "导入失败");
            }

            const result = await res.json();
            toast.success(`导入完成：共 ${result.total} 个，成功 ${result.success} 个，失败 ${result.failed} 个`);
            onSuccess?.();
            open = false;
        } catch (error: any) {
            console.error(error);
            toast.error(error.message || "导入失败，请重试");
        } finally {
            isUploading = false;
        }
    }

    function formatSize(bytes: number) {
        if (bytes === 0) return "0 B";
        const k = 1024;
        const m = k * k;
        if (bytes >= m) {
            return (bytes / m).toFixed(2) + " MB";
        }
        return (bytes / k).toFixed(2) + " KB";
    }
</script>

<Dialog.Root bind:open={open}>
    <Dialog.Content class="sm:max-w-[550px] [&>button.absolute]:hidden">
        <Dialog.Header>
            <Dialog.Title>导入小剧场</Dialog.Title>
            <Dialog.Description>
                <div class="flex items-start gap-2 text-amber-600 dark:text-amber-400 mt-2">
                    <AlertCircle class="h-4 w-4 shrink-0 mt-0.5" />
                    <div class="text-xs space-y-1">
                        <p>仅支持 TXT 格式，每个小剧场格式如下：</p>
                        <pre class="bg-muted p-2 rounded text-[10px] leading-relaxed overflow-x-auto">### Title
Title: [标题]
Category: [分类]
Desc: [简介]

[Content]</pre>
                        <p>多个小剧场之间用空行分隔</p>
                    </div>
                </div>
            </Dialog.Description>
        </Dialog.Header>

        <div class="grid gap-4 py-4">
            {#if !selectedFile}
                 <!-- 拖拽上传区域 -->
                 <div
                     role="button"
                     tabindex="0"
                     class={cn(
                         "relative flex flex-col items-center justify-center rounded-lg border-2 border-dashed p-8 transition-colors text-center cursor-pointer h-40",
                         dragActive ? "border-primary bg-primary/5" : "border-muted-foreground/25 hover:border-primary/50 hover:bg-muted/50",
                         isUploading && "pointer-events-none opacity-50"
                     )}
                     ondragenter={handleDrag}
                     ondragleave={handleDrag}
                     ondragover={handleDrag}
                     ondrop={handleDrop}
                     onclick={() => document.getElementById('theater-file-upload')?.click()}
                     onkeydown={(e) => e.key === 'Enter' && document.getElementById('theater-file-upload')?.click()}
                 >
                     <input 
                         id="theater-file-upload" 
                         type="file" 
                         class="hidden" 
                         accept=".txt" 
                         onchange={handleFileSelect}
                     />
                     
                     <div class="flex h-12 w-12 items-center justify-center rounded-full bg-muted shadow-sm mb-4">
                         <Upload class="h-6 w-6 text-muted-foreground" />
                     </div>
                     
                     <div class="space-y-1">
                         <p class="text-sm font-medium">点击或拖拽文件到这里</p>
                         <p class="text-xs text-muted-foreground">仅支持 .txt 格式</p>
                     </div>
                 </div>
            {:else}
                 <!-- 已选择文件预览 -->
                 <div class="flex items-center justify-between p-4 border rounded-md bg-muted/20">
                     <div class="flex items-center gap-3">
                        <div class="flex h-10 w-10 shrink-0 items-center justify-center rounded-md bg-amber-50 text-amber-600 border border-amber-200">
                            <FileText class="h-5 w-5" />
                        </div>
                        <div class="flex flex-col overflow-hidden">
                            <span class="truncate text-sm font-medium" title={selectedFile.name}>
                                {selectedFile.name}
                            </span>
                            <span class="text-xs text-muted-foreground">
                                {formatSize(selectedFile.size)}
                            </span>
                        </div>
                     </div>
                     <Button variant="ghost" size="sm" onclick={reset} disabled={isUploading}>
                        更换文件
                     </Button>
                 </div>
            {/if}
        </div>

        <Dialog.Footer>
            {#if !selectedFile}
                 <Button variant="outline" onclick={() => open = false}>取消</Button>
            {:else}
                 <Button onclick={handleUpload} disabled={isUploading}>
                     {#if isUploading}
                         <Loader2 class="mr-2 h-4 w-4 animate-spin" />
                         导入中...
                     {:else}
                         开始导入
                     {/if}
                 </Button>
            {/if}
        </Dialog.Footer>
    </Dialog.Content>
</Dialog.Root>
