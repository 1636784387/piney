<script lang="ts">
    import * as Dialog from "$lib/components/ui/dialog";
    import { Button } from "$lib/components/ui/button";
    import { Input } from "$lib/components/ui/input";
    import { Label } from "$lib/components/ui/label";
    import { Switch } from "$lib/components/ui/switch";
    import { Loader2, FileJson, AlertCircle, RotateCcw, Regex, Plus, Import } from "lucide-svelte";
    import { toast } from "svelte-sonner";
    import { cn } from "$lib/utils";
    import { convertJsonlToTxt, scanTags } from "$lib/utils/exportUtils";
    import RegexItem from "$lib/components/character/regex/RegexItem.svelte";
    import { ScrollArea } from "$lib/components/ui/scroll-area";

    let { open = $bindable(false), history, cardId, onUpdate } = $props();

    // Strip extension for display
    // svelte-ignore state_referenced_locally
    let name = $state(history.display_name.replace(/\.(txt|jsonl)$/i, ''));
    let isProcessing = $state(false);
    let step: 'main' | 'tags' | 'regex' = $state('main');

    // Tag Reselection State
    let rawSource = "";
    let availableTags: string[] = $state([]);
    let selectedTags: string[] = $state([]);

    // Regex State
    let regexScripts: any[] = $state([]);

    $effect(() => {
        if (open) {
            // Strip extension for display whenever dialog opens
            name = history.display_name.replace(/\.(txt|jsonl)$/i, '');
            step = 'main';
            // Parse regex scripts
            try {
                regexScripts = JSON.parse(history.regex_scripts || "[]");
            } catch {
                regexScripts = [];
            }
        }
    });

    async function fetchSource() {
        isProcessing = true;
        try {
            const token = localStorage.getItem("auth_token");
            const res = await fetch(`${API_BASE}/api/cards/${cardId}/history/${history.id}/content?source=true`, {
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

    // Regex Utils
     function generateUUID() {
        return 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, function(c) {
            var r = Math.random() * 16 | 0, v = c == 'x' ? r : (r & 0x3 | 0x8);
            return v.toString(16);
        });
    }

    function addScript() {
        // ... (keep generated ID logic if needed internally or remove if fully unused, but user might want 'add' back later. keeping for safety but unused)
        // ... (omitted for brevity in replacement if not modifying)
    }

    async function updateRegexToBackend(scripts: any[]) {
        const token = localStorage.getItem("auth_token");
        const payload = { regex_scripts: JSON.stringify(scripts) };
        const res = await fetch(`${API_BASE}/api/cards/${cardId}/history/${history.id}`, {
            method: 'PATCH',
            headers: {
                'Content-Type': 'application/json',
                ...(token ? { Authorization: `Bearer ${token}` } : {}),
            },
            body: JSON.stringify(payload)
        });
        if (!res.ok) throw new Error("Failed to auto-save regex");
    }

    async function handleImportRegex(e: Event) {
        const input = e.target as HTMLInputElement;
        if (!input.files || input.files.length === 0) return;
        const file = input.files[0];
        
        try {
            const text = await file.text();
            let data = JSON.parse(text);
            let scripts: any[] = [];
            
            if (Array.isArray(data)) {
                scripts = data;
            } else if (data.extensions && Array.isArray(data.extensions.regex_scripts)) {
                scripts = data.extensions.regex_scripts;
            } else if (data.data?.extensions?.regex_scripts && Array.isArray(data.data.extensions.regex_scripts)) {
                 scripts = data.data.extensions.regex_scripts;
            } else {
                 toast.error("未识别到有效的正则脚本配置");
                 return;
            }
            
            // Normalize scripts
            scripts = scripts.map(s => ({
                ...s,
                id: s.id || generateUUID()
            }));

            // Immediate Save
            await updateRegexToBackend(scripts);
            
            // Update local state and parent
            regexScripts = scripts;
            // Update the history prop too to prevent 'dirty' logic in parent re-saving old value if we didn't update it?
            // Since we updated backend, the 'history' prop is stale until we call onUpdate.
            history.regex_scripts = JSON.stringify(scripts); 
            onUpdate();

            toast.success(`成功导入并保存 ${scripts.length} 条规则`);
        } catch (error) {
            console.error(error);
            toast.error("导入或保存失败");
        } finally {
            input.value = "";
        }
    }

    function deleteScript(id: string) {
        regexScripts = regexScripts.filter(s => s.id !== id);
    }

    async function handleSave() {
        isProcessing = true;
        try {
            const token = localStorage.getItem("auth_token");
            
            // 1. If in 'tags' mode, convert and upload content (Only for .txt target)
            if (step === 'tags') {
                const txt = convertJsonlToTxt(rawSource, selectedTags);
                const txtBlob = new Blob([txt], { type: "text/plain;charset=utf-8" });
                
                const formData = new FormData();
                formData.append('file', txtBlob, history.file_name); 

                const resContent = await fetch(`${API_BASE}/api/cards/${cardId}/history/${history.id}/content`, {
                    method: 'PUT',
                    headers: token ? { Authorization: `Bearer ${token}` } : {},
                    body: formData
                });
                if (!resContent.ok) throw new Error("Failed to update content");
            }

            // 2. Update metadata (Name, Regex)
            const payload: any = {};
            if (name !== history.display_name) {
                payload.display_name = name;
            }
            // Always check/update regex if we modified it? Or just compare?
            // Simple: just send it if JSON string different.
            const newRegexStr = JSON.stringify(regexScripts);
            if (newRegexStr !== (history.regex_scripts || "[]")) {
                payload.regex_scripts = newRegexStr;
            }

            if (Object.keys(payload).length > 0) {
                const resMeta = await fetch(`${API_BASE}/api/cards/${cardId}/history/${history.id}`, {
                    method: 'PATCH',
                    headers: {
                        'Content-Type': 'application/json',
                        ...(token ? { Authorization: `Bearer ${token}` } : {}),
                    },
                    body: JSON.stringify(payload)
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
    
    // API BASE fallback
    import { API_BASE } from "$lib/api";

</script>

<Dialog.Root bind:open={open}>
    <Dialog.Content class="sm:max-w-[600px] max-h-[90vh] flex flex-col">
        <Dialog.Header>
            <Dialog.Title>编辑记录</Dialog.Title>
        </Dialog.Header>

        <div class="py-4 space-y-6 flex-1 overflow-y-auto px-1">
            {#if step === 'main'}
                <div class="space-y-4">
                    <div class="space-y-2">
                        <Label>记录名称</Label>
                        <Input bind:value={name} placeholder="请输入名称" />
                    </div>

                    <!-- Config Items -->
                    {#if history.format === 'jsonl'}
                        <div class="space-y-2">
                             <Label>聊天记录正则</Label>
                             <div class="flex items-center justify-between p-3 border rounded-md bg-muted/30">
                                <div class="flex items-center gap-2 text-sm">
                                    <Regex class="h-4 w-4 text-purple-500" />
                                    <span>{regexScripts.length} 个规则</span>
                                </div>
                                <Button variant="outline" size="sm" onclick={() => step = 'regex'}>
                                    配置
                                </Button>
                             </div>
                             <p class="text-xs text-muted-foreground">
                                 仅对此聊天记录生效的显示规则 (应用顺序优先于角色卡正则)
                             </p>
                        </div>
                    {/if}

                    <div class="space-y-2">
                        <Label>源文件操作</Label>
                        {#if history.source_file_name && history.format !== 'jsonl'}
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
                                可以通过源文件重新生成显示内容
                            </p>
                        {:else if history.format === 'jsonl'}
                             <div class="p-3 border rounded-md bg-muted/20 text-sm text-muted-foreground">
                                当前为随风模式 (Jsonl)，直接读取源文件，支持正则配置。
                             </div>
                        {:else}
                             <div class="p-3 border rounded-md bg-muted/50 text-sm text-muted-foreground text-center">
                                无关联的源文件
                             </div>
                        {/if}
                    </div>
                </div>
            {:else if step === 'tags'}
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
                     </div>
                </div>
            {:else if step === 'regex'}
                  <!-- Regex Editor UI -->
                  <div class="flex items-center justify-between mb-2">
                      <h4 class="text-sm font-medium">正则列表</h4>
                      <input 
                            type="file" 
                            id="regex-import" 
                            class="hidden" 
                            accept=".json" 
                            onchange={handleImportRegex}
                      />
                      <Button size="sm" variant="outline" onclick={() => document.getElementById('regex-import')?.click()}>
                          <Import class="h-4 w-4 mr-1" /> 导入配置
                      </Button>
                  </div>
                  <ScrollArea class="h-[300px] pr-4 border rounded-md">
                      <div class="space-y-2 p-2">
                          {#if regexScripts.length === 0}
                              <div class="text-center text-muted-foreground text-sm py-8">暂无规则</div>
                          {:else}
                              {#each regexScripts as script, i (script.id)}
                                  <div class="flex items-center justify-between p-3 border rounded-md bg-card shadow-sm">
                                      <div class="flex items-center gap-3 min-w-0">
                                          <div class="flex flex-col min-w-0">
                                              <span class={cn("text-sm font-medium truncate", script.disabled && "text-muted-foreground line-through")}>
                                                  {script.scriptName || "未命名规则"}
                                              </span>
                                          </div>
                                      </div>
                                      <div class="flex items-center gap-2">
                                          <Switch 
                                              checked={!script.disabled} 
                                              onCheckedChange={(v) => {
                                                  regexScripts[i].disabled = !v;
                                              }}
                                              class="scale-90"
                                          />
                                      </div>
                                  </div>
                              {/each}
                          {/if}
                      </div>
                  </ScrollArea>
            {/if}
        </div>

        <Dialog.Footer>
            {#if step === 'main'}
                <Button variant="outline" onclick={() => open = false} disabled={isProcessing}>取消</Button>
                <Button onclick={handleSave} disabled={isProcessing}>
                    {#if isProcessing}<Loader2 class="mr-2 h-4 w-4 animate-spin" />{/if}
                    保存修改
                </Button>
            {:else}
                <Button variant="outline" onclick={() => step = 'main'} disabled={isProcessing}>返回</Button>
            {/if}
        </Dialog.Footer>
    </Dialog.Content>
</Dialog.Root>
