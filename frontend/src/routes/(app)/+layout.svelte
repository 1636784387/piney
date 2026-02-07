<script lang="ts">
    import AppSidebar from "$lib/components/app-sidebar.svelte";
    import * as Sidebar from "$lib/components/ui/sidebar/index.js";
    import * as Breadcrumb from "$lib/components/ui/breadcrumb/index.js";
    import { Separator } from "$lib/components/ui/separator/index.js";
    import { SidebarTrigger } from "$lib/components/ui/sidebar/index.js";
    import { breadcrumbs } from "$lib/stores/breadcrumb";
    import UserAgreementModal from "$lib/components/UserAgreementModal.svelte";
    import MoonIcon from "@lucide/svelte/icons/moon";
    import SunIcon from "@lucide/svelte/icons/sun";
    import { toggleMode, mode } from "mode-watcher";
    import { Button } from "$lib/components/ui/button";
    import { settings } from "$lib/stores/settings.svelte";

    import { onMount } from "svelte";

    let { children } = $props();

    function handleModeToggle(e: MouseEvent) {
        // ... (unchanged)
        toggleMode();
        // ...
    }

    onMount(() => {
        // 每次页面加载（包括刷新）时清除重启标志，确保 401 跳转机制恢复正常
        if (typeof localStorage !== 'undefined') {
            localStorage.removeItem('is_restarting');
        }
    });
</script>

<Sidebar.Provider>
    <AppSidebar />
    <Sidebar.Inset>
        <header
            class="flex h-auto min-h-12 pt-[env(safe-area-inset-top)] shrink-0 items-center gap-2 border-b border-border transition-[width,height] ease-linear group-has-data-[collapsible=icon]/sidebar-wrapper:h-auto"
        >
            <div class="flex items-center gap-2 px-4">
                <SidebarTrigger class="-ml-1" />
                <Separator orientation="vertical" class="mr-2 h-4" />
                <Breadcrumb.Root>
                    <Breadcrumb.List>
                        {#each $breadcrumbs as item, i}
                            {#if i > 0}
                                <Breadcrumb.Separator class="hidden md:block" />
                            {/if}
                            <Breadcrumb.Item class={i < $breadcrumbs.length - 1 ? "hidden md:block" : ""}>
                                {#if item.href && i < $breadcrumbs.length - 1}
                                    <Breadcrumb.Link href={item.href}
                                        >{item.label}</Breadcrumb.Link
                                    >
                                {:else}
                                    <Breadcrumb.Page
                                        >{item.label}</Breadcrumb.Page
                                    >
                                {/if}
                            </Breadcrumb.Item>
                        {/each}
                    </Breadcrumb.List>
                </Breadcrumb.Root>
            </div>
            <div class="ml-auto px-3">
            <Button variant="ghost" size="icon" onclick={handleModeToggle}>
                <SunIcon class="h-[1.2rem] w-[1.2rem] rotate-0 scale-100 transition-all dark:-rotate-90 dark:scale-0" />
                <MoonIcon class="absolute h-[1.2rem] w-[1.2rem] rotate-90 scale-0 transition-all dark:rotate-0 dark:scale-100" />
                <span class="sr-only">Toggle theme</span>
            </Button>
        </div>
    </header>
        <div class="flex flex-1 flex-col gap-4 p-4 pt-0 pb-[calc(1rem+env(safe-area-inset-bottom))]">
            {@render children()}
        </div>
    </Sidebar.Inset>

    <UserAgreementModal />
</Sidebar.Provider>
