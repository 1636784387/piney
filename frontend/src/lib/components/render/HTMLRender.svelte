<script lang="ts">
    import { onMount, onDestroy } from 'svelte';
    import { createSrcContent, isFrontend } from '$lib/utils/renderUtils';

    // Simple ID generator since strict UUID isn't required for iframe IDs
    function generateId() {
        return Math.random().toString(36).substring(2, 9);
    }

    let { content = '', useBlobUrl = false } = $props<{
        content: string,
        useBlobUrl?: boolean
    }>();

    let iframeRef: HTMLIFrameElement;
    let iframeId = `th-render-${generateId()}`;
    let srcifiedContent = $state('');
    let isDark = $state(false);

    // Watch for class changes on <html> element to detect dark mode
    let observer: MutationObserver;

    onMount(() => {
        // Initial check
        if (typeof document !== 'undefined') {
            isDark = document.documentElement.classList.contains('dark');
        }

        // Setup observer
        if (typeof window !== 'undefined') {
            observer = new MutationObserver((mutations) => {
                mutations.forEach((mutation) => {
                    if (mutation.type === 'attributes' && mutation.attributeName === 'class') {
                        isDark = document.documentElement.classList.contains('dark');
                    }
                });
            });
            
            observer.observe(document.documentElement, {
                attributes: true,
                attributeFilter: ['class']
            });
        }
    });

    onDestroy(() => {
        if (observer) observer.disconnect();
        if (typeof window !== 'undefined') {
            window.removeEventListener('message', handleMessage);
            window.removeEventListener('resize', handleResize);
        }
    });

    // Validated effect that depends on isDark state
    $effect(() => {
        // We just reference isDark here so the effect re-runs when it changes
        const currentDark = isDark; 
        if (content && isFrontend(content)) {
             srcifiedContent = createSrcContent(content, useBlobUrl, currentDark);
        } else {
             srcifiedContent = createSrcContent(content, useBlobUrl, currentDark);
        }
    });
    
    function handleMessage(event: MessageEvent) {
        if (event.data?.type === 'TH_ADJUST_IFRAME_HEIGHT' && event.data?.iframe_name === iframeId) {
            if (iframeRef) {
                iframeRef.style.height = `${event.data.height}px`;
            }
        }
    }

    function handleResize() {
        if (iframeRef && iframeRef.contentWindow) {
            iframeRef.contentWindow.postMessage({ type: 'TH_UPDATE_VIEWPORT_HEIGHT' }, '*');
        }
    }

    onMount(() => {
        window.addEventListener('message', handleMessage);
        window.addEventListener('resize', handleResize);
    });
    
    // Helper to derive Blob URL if mode is switched (simplified for this version to just use srcdoc for stability first)
    // If useBlobUrl is true, we should convert srcifiedContent to a blob url.
    let finalSrc = $derived(useBlobUrl ? URL.createObjectURL(new Blob([srcifiedContent], { type: 'text/html' })) : undefined);
    let finalSrcDoc = $derived(useBlobUrl ? undefined : srcifiedContent);

</script>

<div class="w-full relative min-h-[50px] TH-render" style="background:transparent;">
    <iframe
        bind:this={iframeRef}
        id={iframeId}
        name={iframeId}
        src={finalSrc}
        srcdoc={finalSrcDoc}
        class="w-full border-none overflow-hidden block"
        style="background:transparent;"
        sandbox="allow-scripts allow-popups allow-forms allow-same-origin allow-modals"
        loading="lazy"
        allowtransparency={true}
        title="Rendered Content"
    ></iframe>
</div>
