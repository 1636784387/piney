<script lang="ts">
    import { cn } from "$lib/utils.js";
    import type { Snippet } from "svelte";
    import type { HTMLAttributes } from "svelte/elements";

    interface Props extends HTMLAttributes<HTMLSpanElement> {
        placeholder?: string;
        children?: Snippet;
        ref?: HTMLSpanElement | null;
        class?: string;
    }

    let {
        ref = $bindable(null),
        class: className,
        placeholder,
        children,
        ...restProps
    }: Props = $props();
</script>

<span
    bind:this={ref}
    class={cn("data-[placeholder]:text-muted-foreground", className)}
    data-slot="select-value"
    {...restProps}
>
    {#if children}
        {@render children()}
    {:else if placeholder}
        <span data-placeholder>{placeholder}</span>
    {/if}
</span>
