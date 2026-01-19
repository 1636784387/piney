<script lang="ts">
    import { onMount } from "svelte";
    import * as Dialog from "$lib/components/ui/dialog";
    import { Button } from "$lib/components/ui/button";
    import { Badge } from "$lib/components/ui/badge";
    import { ScrollArea } from "$lib/components/ui/scroll-area";
    import { Separator } from "$lib/components/ui/separator";
    import {
        Stethoscope,
        Loader2,
        CheckCircle,
        AlertCircle,
        History,
        ChevronRight,
        FileText,
        Lightbulb,
        AlertTriangle,
        Trash2,
        ArrowLeft,
        Clock
    } from "lucide-svelte";
    import { cn } from "$lib/utils";
    import {
        startDiagnosis,
        stopDiagnosis,
        getDoctorHistory,
        deleteDoctorHistory,
        doctorTasks,
        type DoctorReport,
        type DoctorHistoryItem,
    } from "$lib/ai/doctor";
    import { marked } from "marked";

    interface Props {
        open: boolean;
        cardId: string;
        onClose: () => void;
    }

    let { open = $bindable(), cardId, onClose }: Props = $props();

    // Global Store State
    let activeTask = $derived($doctorTasks[cardId] || { status: 'idle', message: '', report: null });
    let isTaskRunning = $derived(activeTask.status === 'analyzing');

    // UI View State
    type Tab = 'current' | 'history';
    let activeTab = $state<Tab>('current');

    // History Detail State
    let selectedHistoryItem = $state<DoctorHistoryItem | null>(null);
    let historyReport = $state<DoctorReport | null>(null);

    // Auto-switch to current tab if task starts
    $effect(() => {
        if (isTaskRunning) {
            activeTab = 'current';
        }
    });

    // Auto-load history on open or tab switch
    $effect(() => {
        if (open || activeTab === 'history') {
            loadHistory();
        }
    });

    // marked config
    marked.setOptions({ breaks: true, gfm: true });

    function renderMarkdown(text: string): string {
        if (!text) return "";
        return marked.parse(text) as string;
    }

    let history: DoctorHistoryItem[] = $state([]);

    async function loadHistory() {
        try {
            history = await getDoctorHistory(cardId);
        } catch (e) {
            console.error(e);
        }
    }

    async function handleDeleteHistory(id: string, e: MouseEvent) {
        e.stopPropagation();
        if(!confirm("确定删除此记录？")) return;
        try {
            await deleteDoctorHistory(id);
            history = history.filter(item => item.id !== id);
            if (selectedHistoryItem?.id === id) {
                selectedHistoryItem = null;
                historyReport = null;
            }
        } catch (e) {
            alert("删除失败");
        }
    }

    function handleStartDiagnosis() {
        activeTab = 'current';
        startDiagnosis(cardId);
    }

    function handleCancel() {
        stopDiagnosis(cardId);
    }

    function selectHistoryItem(item: DoctorHistoryItem) {
        if (item.final_report) {
            try {
                historyReport = JSON.parse(item.final_report);
                selectedHistoryItem = item;
            } catch (e) {
                console.error("解析报告失败", e);
            }
        }
    }

    function clearHistorySelection() {
        selectedHistoryItem = null;
        historyReport = null;
    }

    function getConclusionStyle(conclusion: string) {
        if (conclusion.includes("通过")) return "bg-green-500/10 text-green-600 border-green-500/30";
        if (conclusion.includes("重构")) return "bg-red-500/10 text-red-600 border-red-500/30";
        return "bg-amber-500/10 text-amber-600 border-amber-500/30";
    }
</script>

<style>
    /* Markdown Styles */
    :global(.md-content) { line-height: 1.6; }
    :global(.md-content p) { margin-bottom: 0.5em; }
    :global(.md-content strong) { font-weight: 600; }
    :global(.md-content em) { font-style: italic; }
    :global(.md-content ul) { padding-left: 1.5em; list-style-type: disc; margin: 0.5em 0; }
    :global(.md-content ol) { padding-left: 1.5em; list-style-type: decimal; margin: 0.5em 0; }
    :global(.md-content code) { background: hsl(var(--muted)); padding: 0.2em 0.4em; border-radius: 0.25em; font-size: 0.9em; font-family: ui-monospace, monospace; }
</style>

<!-- Reusable Report Snippet -->
{#snippet ReportContent(report: DoctorReport)}
    <div class="space-y-6 pt-2 pb-6 px-1">
        <!-- 核心评估 -->
        <div class="p-6 bg-card rounded-xl border shadow-sm">
            <h3 class="text-base font-semibold mb-4 flex items-center gap-2">
                <FileText class="h-5 w-5 text-primary" />
                核心评估
            </h3>
            <div class="text-base text-muted-foreground leading-relaxed md-content">
                {@html renderMarkdown(report.core_assessment)}
            </div>
        </div>

        <!-- 详细维度 -->
        <div class="space-y-4">
            <h3 class="text-base font-semibold flex items-center gap-2">
                <AlertTriangle class="h-5 w-5 text-amber-500" />
                详细维度诊断
            </h3>
            {#each report.dimensions as dimension}
                <div class="p-4 bg-card rounded-xl border shadow-sm hover:border-primary/20 transition-colors">
                    <div class="mb-3">
                        <span class="font-bold text-base border-l-4 border-primary pl-2">
                            {dimension.name}
                        </span>
                    </div>
                    
                    <div class="space-y-3 text-sm">
                        <!-- 现状 -->
                        <div class="bg-blue-500/5 p-3 rounded-lg border border-blue-500/10">
                            <span class="text-xs font-semibold text-blue-600/80 mb-1 block">
                                当前表现
                            </span>
                            <div class="md-content text-foreground/90">
                                {@html renderMarkdown(dimension.status)}
                            </div>
                        </div>

                        <!-- 问题 -->
                        {#if dimension.issues && dimension.issues !== "无"}
                            <div class="bg-amber-500/10 p-3 rounded-lg border border-amber-500/20">
                                <span class="text-xs font-semibold text-amber-600 mb-1 block flex items-center gap-1">
                                    <AlertCircle class="h-3 w-3" /> 发现问题
                                </span>
                                <div class="md-content text-amber-900 dark:text-amber-100">
                                    {@html renderMarkdown(dimension.issues)}
                                </div>
                            </div>
                        {/if}

                        <!-- 建议 -->
                        {#if dimension.suggestions && dimension.suggestions !== "无"}
                            <div class="bg-green-500/10 p-3 rounded-lg border border-green-500/20">
                                <span class="text-xs font-semibold text-green-600 mb-1 block flex items-center gap-1">
                                    <Lightbulb class="h-3 w-3" /> 优化建议
                                </span>
                                <div class="md-content text-green-900 dark:text-green-100">
                                    {@html renderMarkdown(dimension.suggestions)}
                                </div>
                            </div>
                        {/if}
                    </div>
                </div>
            {/each}
        </div>

        <!-- 处方 -->
        {#if report.prescriptions && report.prescriptions.length > 0}
            <div class="p-6 bg-primary/5 rounded-xl border border-primary/20 shadow-sm">
                <h3 class="text-base font-semibold mb-4 flex items-center gap-2">
                    <Lightbulb class="h-5 w-5 text-primary" />
                    小皮医生的处方
                </h3>
                <ul class="space-y-3">
                    {#each report.prescriptions as prescription}
                        <li class="text-sm flex items-start gap-3">
                            <ChevronRight class="h-4 w-4 text-primary shrink-0 mt-1" />
                            <span class="md-content text-base">{@html renderMarkdown(prescription)}</span>
                        </li>
                    {/each}
                </ul>
            </div>
        {/if}

        <!-- 结论 (Bottom) -->
        <div class={cn("p-6 rounded-xl border text-center shadow-sm", getConclusionStyle(report.conclusion))}>
            <p class="font-semibold text-lg">
                诊断结论：{report.conclusion}
            </p>
        </div>
        
        <!-- Re-run Button (Only for Current View context ideally, but ok here) -->
        <!-- We can conditionally render this outside -->
    </div>
{/snippet}

<Dialog.Root bind:open>
    <Dialog.Content class="!max-w-none w-[95vw] h-[90vh] lg:w-[60vw] lg:h-[80vh] flex flex-col p-4 md:p-6 gap-0" overlayClass="bg-black/10">
        <!-- Header Zone (Fixed Height) -->
        <Dialog.Header class="flex-shrink-0 flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between space-y-0 pb-4 border-b mb-4">
            <div class="flex items-center gap-4">
                <div class="p-2 bg-primary/10 rounded-lg">
                    <Stethoscope class="h-6 w-6 text-primary" />
                </div>
                <div>
                    <Dialog.Title class="text-xl flex items-center gap-2">
                        小皮医生
                        <Badge variant="outline" class="text-[10px] border-primary/30 text-primary">Beta</Badge>
                    </Dialog.Title>
                    <Dialog.Description class="text-sm">
                        AI 智能诊断与优化建议
                    </Dialog.Description>
                </div>
            </div>

            <!-- Tabs Switcher -->
            <div class="flex items-center gap-1 bg-muted/50 p-1 rounded-lg w-full sm:w-auto">
                <button 
                    class={cn(
                        "flex items-center justify-center gap-2 px-4 py-1.5 rounded-md text-sm font-medium transition-all flex-1 sm:flex-none", 
                        activeTab === 'current' 
                            ? "bg-background shadow-sm text-foreground" 
                            : "text-muted-foreground hover:text-foreground hover:bg-muted"
                    )}
                    onclick={() => activeTab = 'current'}
                >
                    {#if isTaskRunning}
                        <Loader2 class="h-3.5 w-3.5 animate-spin text-primary" />
                    {:else}
                        <Stethoscope class="h-3.5 w-3.5" />
                    {/if}
                    当前任务
                </button>
                <button 
                    class={cn(
                        "flex items-center justify-center gap-2 px-4 py-1.5 rounded-md text-sm font-medium transition-all flex-1 sm:flex-none", 
                        activeTab === 'history' 
                            ? "bg-background shadow-sm text-foreground" 
                            : "text-muted-foreground hover:text-foreground hover:bg-muted"
                    )}
                    onclick={() => activeTab = 'history'}
                >
                    <History class="h-3.5 w-3.5" />
                    历史记录
                </button>
            </div>
        </Dialog.Header>

        <!-- Main Content (Flex-1 to fill space) -->
        <div class="flex-1 min-h-0 overflow-hidden relative flex flex-col rounded-lg border bg-background">
            
            {#if activeTab === 'current'}
                <!-- CURRENT TAB -->
                {#if isTaskRunning}
                    <!-- Loading View -->
                    <div class="flex flex-col items-center justify-center flex-1 h-full gap-6 bg-muted/5 animate-in fade-in duration-300">
                        <div class="relative">
                            <div class="w-16 h-16 rounded-full border-4 border-primary/20"></div>
                            <div class="absolute inset-0 w-16 h-16 rounded-full border-4 border-transparent border-t-primary animate-spin"></div>
                        </div>
                        <div class="text-center space-y-2">
                            <p class="text-lg font-medium animate-pulse text-primary">
                                {activeTask.message || "正在连接大那..."}
                            </p>
                            <p class="text-sm text-muted-foreground">
                                正在深入分析角色设定与世界书...
                            </p>
                        </div>
                        <Button variant="ghost" size="sm" onclick={handleCancel} class="mt-4 text-muted-foreground hover:text-destructive">
                            停止诊断
                        </Button>
                    </div>

                {:else if activeTask.status === 'complete' && activeTask.report}
                    <!-- Result View (Using ScrollArea) -->
                    <ScrollArea class="flex-1 h-full"> 
                         <div class="p-6">
                            {@render ReportContent(activeTask.report)}
                            <div class="pt-4 text-center">
                                <Button size="lg" onclick={handleStartDiagnosis} variant="outline" class="gap-2">
                                    <Stethoscope class="h-4 w-4" /> 重新诊断
                                </Button>
                            </div>
                        </div>
                    </ScrollArea>

                {:else if activeTask.status === 'error'}
                    <!-- Error View -->
                    <div class="flex flex-col items-center justify-center flex-1 h-full gap-4">
                        <AlertCircle class="h-12 w-12 text-destructive" />
                        <p class="text-lg font-medium text-destructive">{activeTask.message}</p>
                        <Button onclick={handleStartDiagnosis}>重试</Button>
                    </div>

                {:else}
                    <!-- Idle / Start View -->
                    <div class="flex flex-col items-center justify-center flex-1 h-full gap-8 bg-muted/5 animate-in zoom-in-95 duration-300">
                        <div class="p-6 bg-background rounded-full shadow-sm ring-1 ring-border">
                            <Stethoscope class="h-16 w-16 text-primary/80" />
                        </div>
                        <div class="text-center space-y-4 max-w-md px-4">
                            <h3 class="text-2xl font-bold tracking-tight">准备好开始了吗？</h3>
                            <p class="text-muted-foreground leading-relaxed">
                                小皮医生将全面分析您的角色设定，从逻辑一致性到世界观融合度，为您提供专业的优化建议。
                            </p>
                        </div>
                        <Button
                            class="gap-2 h-12 px-8 text-base shadow-lg hover:shadow-primary/25 transition-all w-48"
                            size="lg"
                            onclick={handleStartDiagnosis}
                        >
                            <Stethoscope class="h-5 w-5" />
                            开始诊断
                        </Button>
                    </div>
                {/if}

            {:else}
                <!-- HISTORY TAB -->
                {#if selectedHistoryItem && historyReport}
                    <!-- Detail View (Nested) -->
                    <div class="flex flex-col h-full">
                        <div class="flex-shrink-0 p-4 border-b bg-muted/20 flex items-center gap-3">
                            <Button variant="ghost" size="sm" class="gap-1 h-8" onclick={clearHistorySelection}>
                                <ArrowLeft class="h-4 w-4" /> 返回列表
                            </Button>
                            <div class="h-4 w-px bg-border"></div>
                            <span class="text-sm font-medium">诊断报告 ({selectedHistoryItem.created_at})</span>
                            <Badge variant={selectedHistoryItem.status === 'success' ? 'default' : 'destructive'} class="ml-auto">
                                {selectedHistoryItem.status}
                            </Badge>
                        </div>
                        
                        <ScrollArea class="flex-1 h-full">
                             <div class="p-6">
                                {@render ReportContent(historyReport)}
                             </div>
                        </ScrollArea>
                    </div>

                {:else}
                    <!-- List View -->
                    <div class="flex-1 flex flex-col bg-muted/5 h-full">
                         <div class="p-4 flex items-center justify-between text-sm text-muted-foreground border-b bg-background flex-shrink-0">
                            <span>共 {history.length} 条记录</span>
                            <Button variant="ghost" size="sm" class="h-6 text-xs" onclick={loadHistory}>刷新</Button>
                         </div>
                         <ScrollArea class="flex-1 h-full">
                             <div class="p-4 space-y-3">
                                {#if history.length === 0}
                                    <div class="flex flex-col items-center justify-center py-20 opacity-50">
                                        <History class="h-12 w-12 mb-2" />
                                        <p>暂无历史记录</p>
                                    </div>
                                {:else}
                                    {#each history as item}
                                        <button 
                                            class="w-full text-left bg-card hover:bg-accent/50 border rounded-xl p-4 transition-all flex items-center justify-between group"
                                            onclick={() => selectHistoryItem(item)}
                                        >
                                            <div class="flex items-center gap-4">
                                                <div class={cn(
                                                    "p-2 rounded-full",
                                                    item.status === 'success' ? "bg-green-500/10 text-green-600" : "bg-red-500/10 text-red-600"
                                                )}>
                                                    {#if item.status === 'success'}
                                                        <CheckCircle class="h-5 w-5" />
                                                    {:else}
                                                        <AlertCircle class="h-5 w-5" />
                                                    {/if}
                                                </div>
                                                <div>
                                                    <div class="font-medium flex items-center gap-2">
                                                        诊断报告
                                                        <span class="text-xs font-mono opacity-50">#{item.id.slice(0, 6)}</span>
                                                    </div>
                                                    <div class="text-xs text-muted-foreground flex items-center gap-3 mt-1">
                                                        <span class="flex items-center gap-1"><Clock class="h-3 w-3" /> {new Date(item.created_at).toLocaleString()}</span>
                                                    </div>
                                                </div>
                                            </div>
                                            <div class="flex items-center gap-3">
                                                 <Badge variant="outline">{item.status}</Badge>
                                                 <Button variant="ghost" size="icon" class="h-8 w-8 text-muted-foreground hover:text-destructive shrink-0" 
                                                    onclick={(e) => handleDeleteHistory(item.id, e)}
                                                 >
                                                    <Trash2 class="h-4 w-4" />
                                                 </Button>
                                                 <ChevronRight class="h-4 w-4 text-muted-foreground group-hover:translate-x-1 transition-transform" />
                                            </div>
                                        </button>
                                    {/each}
                                {/if}
                             </div>
                         </ScrollArea>
                    </div>
                {/if}
            {/if}
            
        </div>
        
        <!-- Footer -->
        <Dialog.Footer class="flex-shrink-0 mt-4 border-t pt-4">
             <Button variant="outline" onclick={onClose}>关闭窗口</Button>
        </Dialog.Footer>

    </Dialog.Content>
</Dialog.Root>
