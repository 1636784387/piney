/// <reference types="@sveltejs/kit" />

// See https://kit.svelte.dev/docs/types#app
// for information about these interfaces
declare global {
    namespace App {
        // interface Error {}
        // interface Locals {}
        // interface PageData {}
        // interface PageState {}
        // interface Platform {}
    }

    namespace svelteHTML {
        interface HTMLAttributes<T> {
            "onlongpress"?: (event: CustomEvent<any>) => void;
        }
    }
}

declare module "svelte/elements" {
    export interface HTMLAttributes<T> {
        "onlongpress"?: (event: CustomEvent<any>) => void;
    }
}

export { };
