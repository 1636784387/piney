<script lang="ts">
    import * as Dialog from "$lib/components/ui/dialog";
    import { Button } from "$lib/components/ui/button";
    import { Upload, X, Loader2, FileJson, FileText, ArrowRight, RotateCcw, AlertCircle } from "lucide-svelte";
    import { toast } from "svelte-sonner";
    import { cn } from "$lib/utils";
    import { convertJsonlToTxt, scanTags } from "$lib/utils/exportUtils";

    let { open = $bindable(false), onImport, cardId } = $props();

    let dragActive = $state(false);
    let isUploading = $state(false);

    // Single file logic
    let selectedFile: File | null = $state(null);
    let step: 'select' | 'preprocess' = $state('select');

    // Preprocessing state
    let rawContent = "";
    let availableTags: string[] = $state([]);
    let selectedTags: string[] = $state([]);

    $effect(() => {
        if (!open) {
            // Reset state on close
            reset();
        }
    });

    function reset() {
        selectedFile = null;
        step = 'select';
        rawContent = "";
        availableTags = [];
        selectedTags = [];
        isUploading = false;
    }

    function validateFile(file: File): boolean {
        const ext = '.' + file.name.split('.').pop()?.toLowerCase();
        if (!['.jsonl', '.txt'].includes(ext)) {
            toast.warning("仅支持 .jsonl 或 .txt 文件");
            return false;
        }
        if (file.size > 100 * 1024 * 1024) {
            toast.warning("文件大小不能超过 100MB");
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
            if (e.dataTransfer.files.length > 1) {
                toast.warning("仅支持单文件导入");
                return;
            }
            const file = e.dataTransfer.files[0];
            if (validateFile(file)) {
                processFile(file);
            }
        }
    }

    function handleFileSelect(e: Event) {
        const input = e.target as HTMLInputElement;
        if (input.files && input.files.length > 0) {
            const file = input.files[0];
            if (validateFile(file)) {
                processFile(file);
            }
        }
        input.value = "";
    }

    async function processFile(file: File) {
        selectedFile = file;
        const ext = '.' + file.name.split('.').pop()?.toLowerCase();

        if (ext === '.jsonl') {
            const text = await file.text();
            rawContent = text;
            availableTags = scanTags(text);
            selectedTags = [...availableTags]; // Default select all
            step = 'preprocess';
        } else {
            step = 'select'; // Ready to upload directly
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

    async function handleUpload() {
        if (!selectedFile) return;
        isUploading = true;

        try {
            const formData = new FormData();
            
            if (step === 'preprocess') {
                // Perform client-side conversion
                const txt = convertJsonlToTxt(rawContent, selectedTags);
                const txtBlob = new Blob([txt], { type: "text/plain;charset=utf-8" });
                const txtName = selectedFile.name.replace(/\.jsonl$/i, "") + ".txt";
                
                // Add converted file as 'file' (main)
                formData.append('file', txtBlob, txtName);
                // Add original as 'source_file'
                formData.append('source_file', selectedFile, selectedFile.name);
            } else {
                // Direct TXT upload
                formData.append('file', selectedFile);
            }

            const token = localStorage.getItem("auth_token");
            const res = await fetch(`http://localhost:9696/api/cards/${cardId}/history`, {
                method: 'POST',
                headers: {
                   ...(token ? { Authorization: `Bearer ${token}` } : {}),
                },
                body: formData
            });

            if (!res.ok) throw new Error("Upload failed");

            const saved = await res.json(); // API returns ChatHistoryDto now (not array)
            onImport([saved]); // Keep interface valid expecting array (or update onImport too)
            
            toast.success(`导入成功`);
            open = false;
        } catch (error) {
            console.error(error);
            toast.error("导入失败，请重试");
        } finally {
            isUploading = false;
        }
    }
</script>

<Dialog.Root bind:open={open}>
    <Dialog.Content class="sm:max-w-[600px]">
        <Dialog.Header>
            <Dialog.Title>
                {#if step === 'preprocess'}
                    导入预处理
                {:else}
                    导入聊天记录
                {/if}
            </Dialog.Title>
            <Dialog.Description>
                {#if step === 'preprocess'}
                    检测到标签对。请选择要保留的内容标签（未选中的将被丢弃）。
                    <br>
                    本功能参考了旅程 @Yellows 老师"聊天记录导出脚本"中的部分逻辑。<a href="https://discord.com/channels/1291925535324110879/1447232920598216846" class="underline">原贴地址</a>
                {:else}
                    支持 .jsonl (自动转换) 或 .txt 文件。单文件最大 100MB。
                {/if}
            </Dialog.Description>
        </Dialog.Header>

        <div class="grid gap-4 py-4">
            {#if step === 'select' && !selectedFile}
                 <!-- Drop Zone -->
                 <div
                     role="button"
                     tabindex="0"
                     class={cn(
                         "relative flex flex-col items-center justify-center rounded-lg border-2 border-dashed p-8 transition-colors text-center cursor-pointer h-48",
                         dragActive ? "border-primary bg-primary/5" : "border-muted-foreground/25 hover:border-primary/50 hover:bg-muted/50",
                         isUploading && "pointer-events-none opacity-50"
                     )}
                     ondragenter={handleDrag}
                     ondragleave={handleDrag}
                     ondragover={handleDrag}
                     ondrop={handleDrop}
                     onclick={() => document.getElementById('history-file-upload')?.click()}
                     onkeydown={(e) => e.key === 'Enter' && document.getElementById('history-file-upload')?.click()}
                 >
                     <input 
                         id="history-file-upload" 
                         type="file" 
                         class="hidden" 
                         accept=".jsonl,.txt" 
                         onchange={handleFileSelect}
                     />
                     
                     <div class="flex h-12 w-12 items-center justify-center rounded-full bg-muted shadow-sm mb-4">
                         <Upload class="h-6 w-6 text-muted-foreground" />
                     </div>
                     
                     <div class="space-y-1">
                         <p class="text-sm font-medium">点击或拖拽文件到这里</p>
                         <p class="text-xs text-muted-foreground">支持 .jsonl, .txt</p>
                     </div>
                 </div>
            {:else if step === 'preprocess'}
                 <!-- Tag Selection UI -->
                 <div class="space-y-4">
                     <div class="flex items-center justify-between">
                         <div class="flex items-center gap-2">
                            <FileJson class="h-4 w-4 text-blue-500" />
                            <span class="font-medium text-sm">{selectedFile?.name}</span>
                         </div>
                         <Button variant="ghost" size="sm" onclick={reset} disabled={isUploading}>重新选择</Button>
                     </div>

                     <div class="border rounded-md p-4 bg-muted/20 space-y-3 max-h-[300px] overflow-y-auto">
                        <div class="flex items-center justify-between">
                            <h4 class="text-sm font-semibold">检测到的标签</h4>
                            <Button variant="link" size="sm" class="h-auto p-0 text-xs" onclick={toggleAllTags}>
                                {selectedTags.length === availableTags.length ? '全不选' : '全选'}
                            </Button>
                        </div>

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
                             选中标签的内容将被保留（标签本身会被移除）。未选中的标签对此将被完整丢弃。普通文本不受影响。
                        </p>
                     </div>
                 </div>
            {:else}
                 <!-- Selected TXT Preview -->
                 <div class="flex items-center justify-between p-4 border rounded-md">
                     <div class="flex items-center gap-2">
                        <FileText class="h-4 w-4 text-orange-500" />
                        <span class="font-medium text-sm">{selectedFile?.name}</span>
                        <span class="text-xs text-muted-foreground">({(selectedFile!.size / 1024 / 1024).toFixed(2)} MB)</span>
                     </div>
                     <Button variant="ghost" size="sm" onclick={reset} disabled={isUploading}>
                        <X class="h-4 w-4" />
                     </Button>
                 </div>
            {/if}
        </div>

        <Dialog.Footer>
            {#if step === 'select' && !selectedFile}
                 <Button variant="outline" onclick={() => open = false}>取消</Button>
            {:else}
                 <Button onclick={handleUpload} disabled={isUploading}>
                     {#if isUploading}
                         <Loader2 class="mr-2 h-4 w-4 animate-spin" />
                         上传中...
                     {:else}
                         {#if step === 'preprocess'}
                             转换并导入
                         {:else}
                             开始导入
                         {/if}
                     {/if}
                 </Button>
            {/if}
        </Dialog.Footer>
    </Dialog.Content>
</Dialog.Root>
