<script lang="ts">
    import { marked } from "marked";
    import HTMLRender from "$lib/components/render/HTMLRender.svelte";
    import { isFrontend } from "$lib/utils/renderUtils";

    let { content = "" } = $props<{ content: string }>();

    // Token interface from marked
    interface Token {
        type: string;
        raw: string;
        text?: string;
        lang?: string;
        tokens?: Token[];
        [key: string]: any;
    }

    // Configure marked locally if needed, but assuming global or default is fine.
    // marked.lexer splits text into tokens.
    let tokens: Token[] = $derived(marked.lexer(content));
    
    // Helper to check if a code token is 'frontend'
    function checkFrontend(token: Token): { isFrontend: boolean, content: string } {
        // Case 1: Code Block
        if (token.type === 'code') {
            const text = token.text || "";
            if (isFrontend(text)) return { isFrontend: true, content: text };
        }
        
        // Case 2: Raw HTML containing <pre> block (Legacy/ST style raw html fallback)
        if (token.type === 'html') {
            const text = token.text || "";
            // Simple regex to find <pre> content. 
            // Note: This matches the FIRST pre block if multiple exist in one HTML token.
            // ST behavior maps ALL pre blocks. 
            // Since we are iterating tokens, one HTML token usually is one block.
            // But if it contains multiple PREs, we might need to split? 
            // For now, let's assume one main block or strict extraction.
            const match = /<pre[^>]*>([\s\S]*?)<\/pre>/i.exec(text);
            if (match && isFrontend(match[1])) {
                // If the HTML token is primarily just this wrapper, we replace it.
                // If it has other text, we might lose it? 
                // ST replaces the PRE tag with the iframe wrapper.
                // Here we replace the WHOLE token if we return true?
                // Ideally we should split the token. But complexity increases.
                // Let's assume the token is the block.
                return { isFrontend: true, content: match[1] }; 
            }
        }
        
        return { isFrontend: false, content: "" };
    }
    
    // Group tokens: Consecutive non-frontend tokens are grouped for the parser
    // This allows marked to handle list continuity and other block relations better,
    // and potentially performance.
    let groupedBlocks = $derived.by(() => {
        const blocks: { isFrontend: boolean, content: string | Token[] }[] = [];
        let currentGroup: Token[] = [];
        
        for (const token of tokens) {
            const check = checkFrontend(token);
            if (check.isFrontend) {
                // Flush current group
                if (currentGroup.length > 0) {
                    blocks.push({ isFrontend: false, content: [...currentGroup] });
                    currentGroup = [];
                }
                // Add frontend block
                blocks.push({ isFrontend: true, content: check.content });
            } else {
                currentGroup.push(token);
            }
        }
        // Flush remaining
        if (currentGroup.length > 0) {
            blocks.push({ isFrontend: false, content: currentGroup });
        }
        return blocks;
    });

    function renderTokens(tokens: Token[]): string {
        // marked.parser signature takes tokens array
        // We need to cast or trust the layout.
        return marked.parser(tokens as any);
    }
</script>

<div class="prose dark:prose-invert max-w-none message-content">
    {#each groupedBlocks as block}
        {#if block.isFrontend}
            <div class="my-4">
                 <HTMLRender content={block.content as string} />
            </div>
        {:else}
            {@html renderTokens(block.content as Token[])}
        {/if}
    {/each}
</div>

<style>
    /* Add basic markdown styles here OR rely on Tailwind Typography (prose) if available.
       Since I added 'prose' class, I assume tailwind typography plugin might be used.
       If not, I should add the styles manually here.
       I will add manual styles as fallback akin to what I had before.
    */
    :global(.message-content p) { margin-bottom: 1em; }
    :global(.message-content img) { max-width: 100%; height: auto; display: block; margin: 0.5em 0; }
    :global(.message-content pre) { background-color: rgb(241 245 249); padding: 1em; border-radius: 0.375rem; overflow-x: auto; margin: 0.5em 0; border: 1px solid rgb(226 232 240); }
    :global(.message-content code) { background-color: rgb(241 245 249); padding: 0.2em 0.4em; border-radius: 0.25rem; font-family: monospace; font-size: 0.9em; }
    :global(.message-content pre code) { background-color: transparent; padding: 0; border: none; }
    /* Dark mode */
    :global(.dark .message-content pre) { background-color: rgb(30 41 59); border-color: rgb(51 65 85); }
    :global(.dark .message-content code) { background-color: rgb(30 41 59); color: rgb(226 232 240); }
</style>
