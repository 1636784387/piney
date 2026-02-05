/**
 * 角色库页面预加载
 */

import { API_BASE } from '$lib/api';
import type { PageLoad } from './$types';

export const load: PageLoad = async ({ fetch, url }) => {
    const token = typeof localStorage !== 'undefined' ? localStorage.getItem('auth_token') : null;

    // 从 URL 参数获取筛选条件
    const categoryId = url.searchParams.get('category_id') || '';
    const search = url.searchParams.get('search') || '';
    const page = url.searchParams.get('page') || '1';
    const pageSize = url.searchParams.get('page_size') || '20';
    const sort = url.searchParams.get('sort') || 'updated_at';
    const order = url.searchParams.get('order') || 'desc';

    // 构建请求参数
    const params = new URLSearchParams();
    if (categoryId) params.set('category_id', categoryId);
    if (search) params.set('search', search);
    params.set('page', page);
    params.set('page_size', pageSize);
    params.set('sort', sort);
    params.set('order', order);

    try {
        // 并行加载分类、分页卡片和标签统计
        const [categoriesRes, cardsRes, tagStatsRes] = await Promise.all([
            fetch(`${API_BASE}/api/categories`, {
                headers: token ? { Authorization: `Bearer ${token}` } : {}
            }),
            fetch(`${API_BASE}/api/cards/all?${params.toString()}`, {
                headers: token ? { Authorization: `Bearer ${token}` } : {}
            }),
            fetch(`${API_BASE}/api/cards/stats/tags`, {
                headers: token ? { Authorization: `Bearer ${token}` } : {}
            })
        ]);

        const categories = categoriesRes.ok ? await categoriesRes.json() : [];
        const cardsData = cardsRes.ok ? await cardsRes.json() : { items: [], total: 0, page: 1, page_size: 20, total_pages: 1 };
        const tagStats = tagStatsRes.ok ? await tagStatsRes.json() : { tags: {}, total_cards: 0 };

        return {
            categories,
            cards: cardsData.items,
            pagination: {
                total: cardsData.total,
                page: cardsData.page,
                pageSize: cardsData.page_size,
                totalPages: cardsData.total_pages
            },
            tagStats: tagStats.tags,
            totalCards: tagStats.total_cards,
            preloaded: true
        };
    } catch (error) {
        console.error('预加载失败:', error);
        return {
            categories: [],
            cards: [],
            pagination: { total: 0, page: 1, pageSize: 20, totalPages: 1 },
            tagStats: {},
            totalCards: 0,
            preloaded: false
        };
    }
};

// 禁用 SSR，在客户端加载
export const ssr = false;
