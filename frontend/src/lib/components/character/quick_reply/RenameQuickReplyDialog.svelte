<script lang="ts">
    // 重命名快速回复对话框
    import * as Dialog from "$lib/components/ui/dialog";
    import { Button } from "$lib/components/ui/button";
    import { Input } from "$lib/components/ui/input";
    import { Label } from "$lib/components/ui/label";
    import { Loader2 } from "lucide-svelte";
    import { toast } from "svelte-sonner";
    import { API_BASE } from "$lib/api";

    let { open = $bindable(false), quickReply, cardId, onUpdate } = $props();

    let name = $state("");
    let isProcessing = $state(false);

    $effect(() => {
        if (open && quickReply) {
            name = quickReply.display_name;
        }
    });

    async function handleSave() {
        if (!name.trim()) {
            toast.error("名称不能为空");
            return;
        }

        if (name.trim() === quickReply.display_name) {
            open = false;
            return;
        }

        isProcessing = true;
        try {
            const token = localStorage.getItem("auth_token");
            const res = await fetch(`${API_BASE}/api/cards/${cardId}/quick_reply/${quickReply.id}`, {
                method: 'PATCH',
                headers: {
                    'Content-Type': 'application/json',
                    ...(token ? { Authorization: `Bearer ${token}` } : {}),
                },
                body: JSON.stringify({ display_name: name.trim() })
            });

            if (!res.ok) throw new Error("更新失败");

            onUpdate(quickReply.id, { display_name: name.trim() });
            toast.success("重命名成功");
            open = false;
        } catch (e) {
            console.error(e);
            toast.error("重命名失败");
        } finally {
            isProcessing = false;
        }
    }

    function handleKeydown(e: KeyboardEvent) {
        if (e.key === 'Enter' && !isProcessing) {
            handleSave();
        }
    }
</script>

<Dialog.Root bind:open={open}>
    <Dialog.Content class="sm:max-w-[400px]">
        <Dialog.Header>
            <Dialog.Title>重命名快速回复</Dialog.Title>
        </Dialog.Header>

        <div class="py-4 space-y-4">
            <div class="space-y-2">
                <Label for="qr-name">名称</Label>
                <Input 
                    id="qr-name"
                    bind:value={name} 
                    placeholder="请输入名称" 
                    onkeydown={handleKeydown}
                />
            </div>
        </div>

        <Dialog.Footer>
            <Button variant="outline" onclick={() => open = false} disabled={isProcessing}>
                取消
            </Button>
            <Button onclick={handleSave} disabled={isProcessing}>
                {#if isProcessing}
                    <Loader2 class="mr-2 h-4 w-4 animate-spin" />
                {/if}
                保存
            </Button>
        </Dialog.Footer>
    </Dialog.Content>
</Dialog.Root>
