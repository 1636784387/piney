<script lang="ts">
    import { settings } from "$lib/stores/settings.svelte"; 
    import { auth } from "$lib/stores/auth.svelte";
    import { goto } from "$app/navigation";
    import { page } from "$app/stores";

    $effect(() => {
        if (
            auth.authenticated && 
            $settings.loaded &&
            $settings.user_agreement_accepted === false && 
            $page.url.pathname !== '/agreement'
        ) {
            goto('/agreement');
        }
    });
</script>