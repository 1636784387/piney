import { goto } from '$app/navigation';
import { page } from '$app/stores';
import { get } from 'svelte/store';

import { getApiBase } from '$lib/api';

class AuthStore {
    authenticated = $state(false);
    initialized = $state(false);
    loading = $state(true);
    username = $state<string | null>(null);
    avatar = $state<string>("");

    async init() {
        try {
            const token = localStorage.getItem('auth_token');
            const headers: HeadersInit = {};
            if (token) {
                headers['Authorization'] = `Bearer ${token}`;
            }

            const res = await fetch(`${getApiBase()}/api/auth/status`, {
                headers
            });

            if (res.ok) {
                const data = await res.json();
                this.initialized = data.initialized;

                this.authenticated = !!token;
                this.username = data.username;

                if (this.authenticated) {
                    this.fetchAvatar();
                }
            }
        } catch (e) {
            console.error("Auth check failed", e);
        } finally {
            this.loading = false;
            this.checkRedirect();
        }
    }

    checkRedirect() {
        if (this.loading) return;

        const path = window.location.pathname;

        if (!this.initialized) {
            if (path !== '/sign-up') goto('/sign-up');
            return;
        }

        if (this.initialized && !this.authenticated) {
            if (path !== '/login' && path !== '/sign-up') goto('/login');
            return;
        }

        if (this.initialized && this.authenticated) {
            if (path === '/login' || path === '/sign-up') goto('/');
            return;
        }
    }

    async login(username: string, token: string) {
        localStorage.setItem('auth_token', token);
        this.authenticated = true;
        this.username = username;
        this.fetchAvatar();
        this.checkRedirect();
    }

    async setup(token: string) {
        localStorage.setItem('auth_token', token);
        this.initialized = true;
        this.authenticated = true;
        this.fetchAvatar();
        this.checkRedirect();
    }

    logout() {
        localStorage.removeItem('auth_token');
        this.authenticated = false;
        this.username = null;
        goto('/login');
    }
    async fetchAvatar() {
        try {
            const token = localStorage.getItem('auth_token');
            const headers: HeadersInit = {};
            if (token) {
                headers['Authorization'] = `Bearer ${token}`;
            }
            const res = await fetch(`${getApiBase()}/api/settings`, { headers });
            if (res.ok) {
                const settings = await res.json();
                if (settings.avatar) {
                    this.avatar = settings.avatar;
                }
            }
        } catch (e) {
            console.error("Failed to fetch avatar", e);
        }
    }
}

export const auth = new AuthStore();
