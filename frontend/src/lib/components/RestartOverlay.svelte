<script lang="ts">
    import { fade } from 'svelte/transition';
    import { Loader2 } from 'lucide-svelte';

    export let visible = false;
    export let message = "正在应用变更并重启服务...";
    export let subMessage = "请稍候，系统将在服务就绪后自动刷新页面";
</script>

{#if visible}
    <div 
        class="fixed inset-0 z-[9999] flex flex-col items-center justify-center bg-background/95 backdrop-blur-sm"
        transition:fade
    >
        <div class="flex flex-col items-center space-y-4 text-center p-6">
            <Loader2 class="h-12 w-12 animate-spin text-primary" />
            <h2 class="text-2xl font-bold tracking-tight">{message}</h2>
            <p class="text-muted-foreground">{subMessage}</p>
        </div>
    </div>
{/if}
