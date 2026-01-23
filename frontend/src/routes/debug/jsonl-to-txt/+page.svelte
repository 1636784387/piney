<script lang="ts">
    import { convertJsonlToTxt, scanTags } from "$lib/utils/exportUtils";
    import { Button } from "$lib/components/ui/button";
    import { toast } from "svelte-sonner";
    import { Badge } from "$lib/components/ui/badge";

    let files: FileList | null = null;
    let txtContent = "";
    let downloadUrl = "";
    let fileName = "";
    let fileContent = ""; // store raw content

    let availableTags: string[] = [];
    let selectedTags: string[] = [];
    let isProcessing = false;

    function handleFileChange(e: Event) {
        const input = e.target as HTMLInputElement;
        if (input.files && input.files.length > 0) {
            const file = input.files[0];
            fileName = file.name.replace(/\.jsonl$/i, "") + ".txt";
            const reader = new FileReader();
            reader.onload = (e) => {
                fileContent = e.target?.result as string;
                if (fileContent) {
                    // 1. Scan Tags
                    availableTags = scanTags(fileContent);
                    // Default: select all? or select none?
                    // User probably wants to keep everything by default usually, but for "filtering" maybe select all is safer start.
                    selectedTags = [...availableTags];
                    
                    processConversion();
                }
            };
            reader.readAsText(file);
        }
    }

    function toggleTag(tag: string) {
        if (selectedTags.includes(tag)) {
            selectedTags = selectedTags.filter(t => t !== tag);
        } else {
            selectedTags = [...selectedTags, tag];
        }
        processConversion();
    }

    function processConversion() {
        if (!fileContent) return;
        try {
            // Pass selectedTags to filter
            txtContent = convertJsonlToTxt(fileContent, selectedTags);
            
            const blob = new Blob([txtContent], { type: "text/plain;charset=utf-8" });
            if (downloadUrl) URL.revokeObjectURL(downloadUrl);
            downloadUrl = URL.createObjectURL(blob);
        } catch (err) {
            toast.error("转换失败：" + err);
        }
    }
</script>

<div class="p-8 space-y-8">
    <h1 class="text-2xl font-bold">JSONL 转 TXT 工具 (Debug)</h1>
    
    <div class="space-y-4 max-w-md">
        <label class="block text-sm font-medium">
            上传 JSONL 文件
            <input 
                type="file" 
                accept=".jsonl,.json" 
                onchange={handleFileChange}
                class="block w-full text-sm text-slate-500
                  file:mr-4 file:py-2 file:px-4
                  file:rounded-full file:border-0
                  file:text-sm file:font-semibold
                  file:bg-violet-50 file:text-violet-700
                  hover:file:bg-violet-100 mt-2"
            />
        </label>
    </div>

    {#if availableTags.length > 0}
        <div class="space-y-2">
            <h3 class="text-sm font-medium">检测到的标签 (点击切换)</h3>
            <div class="flex flex-wrap gap-2">
                {#each availableTags as tag}
                    <button 
                        class="px-3 py-1 rounded-full text-xs border transition-colors 
                        {selectedTags.includes(tag) ? 'bg-primary text-primary-foreground border-primary' : 'bg-muted text-muted-foreground border-transparent hover:bg-muted/80'}"
                        onclick={() => toggleTag(tag)}
                    >
                        {tag}
                    </button>
                {/each}
            </div>
            <p class="text-xs text-muted-foreground">
                未选中的标签对内的内容将被<b>丢弃</b>。不在标签对内的普通文本将始终保留。
            </p>
        </div>
    {/if}

    {#if txtContent}
        <div class="space-y-4">
            <div class="flex items-center gap-4">
                <h2 class="text-lg font-semibold">转换预览</h2>
                <a href={downloadUrl} download={fileName}>
                    <Button>下载 TXT</Button>
                </a>
            </div>
            
            <div class="p-4 bg-muted rounded-md whitespace-pre-wrap font-mono text-xs h-96 overflow-y-auto border">
                {txtContent}
            </div>
        </div>
    {/if}
</div>
