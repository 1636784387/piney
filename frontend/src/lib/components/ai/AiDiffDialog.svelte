<script lang="ts">
    import * as Dialog from "$lib/components/ui/dialog";
    import { Button } from "$lib/components/ui/button";
    import { ScrollArea } from "$lib/components/ui/scroll-area";
    import { cn } from "$lib/utils";
    import { Check, X, ArrowRight } from "lucide-svelte";

    let {
        open = $bindable(false),
        original = "",
        optimized = "",
        title = "AI 建议对比",
        onApply,
        onCancel
    } = $props<{
        open: boolean;
        original: string;
        optimized: string;
        title?: string;
        onApply: () => void;
        onCancel: () => void;
    }>();

    function handleApply() {
        onApply();
        open = false;
    }

    function handleCancel() {
        if (onCancel) onCancel();
        open = false;
    }
</script>

<Dialog.Root bind:open>
    <Dialog.Content 
        class="!max-w-none !w-[90vw] md:!w-[80vw] h-[90vh] flex flex-col p-0 gap-0 overflow-hidden" 
        overlayClass="bg-black/10"
    >
        <Dialog.Header class="p-6 pb-4 flex-shrink-0 border-b">
            <Dialog.Title>{title}</Dialog.Title>
            <Dialog.Description>
                请检查 AI 生成的建议。点击“应用”将替换当前内容（需点击页面保存按钮以持久化）。
            </Dialog.Description>
        </Dialog.Header>

        <div class="flex-1 min-h-0 flex flex-col md:flex-row divide-y md:divide-y-0 md:divide-x">
            <!-- Original -->
            <div class="flex-1 min-h-0 flex flex-col bg-muted/10">
                <div class="px-4 py-2 text-xs font-medium text-muted-foreground uppercase tracking-wider border-b bg-muted/20 flex-shrink-0">
                    当前内容
                </div>
                <div class="flex-1 overflow-y-auto p-4 whitespace-pre-wrap font-mono text-sm leading-relaxed text-muted-foreground/80">
                    {original}
                </div>
            </div>

            <!-- New -->
            <div class="flex-1 min-h-0 flex flex-col bg-blue-50/50 dark:bg-blue-900/10">
                <div class="px-4 py-2 text-xs font-medium text-primary uppercase tracking-wider border-b bg-blue-100/20 dark:bg-blue-900/20 flex items-center justify-between flex-shrink-0">
                    <span>AI 建议内容</span>
                    <span class="text-[10px] bg-primary/10 text-primary px-1.5 py-0.5 rounded">New</span>
                </div>
                <div class="flex-1 overflow-y-auto p-4 whitespace-pre-wrap font-mono text-sm leading-relaxed text-foreground">
                    {optimized}
                </div>
            </div>
        </div>

        <Dialog.Footer class="p-6 pt-4 border-t bg-muted/5 flex-shrink-0 flex items-center justify-end gap-2">
            <Button variant="outline" onclick={handleCancel}>
                <X class="w-4 h-4 mr-2" />
                放弃
            </Button>
            <Button onclick={handleApply}>
                <Check class="w-4 h-4 mr-2" />
                应用更改
            </Button>
        </Dialog.Footer>
    </Dialog.Content>
</Dialog.Root>
