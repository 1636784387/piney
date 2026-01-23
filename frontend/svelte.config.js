import adapter from '@sveltejs/adapter-static';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';

/** @type {import('@sveltejs/kit').Config} */
const config = {
	preprocess: vitePreprocess(),

	// 过滤掉 a11y 警告（减少构建日志）
	onwarn: (warning, handler) => {
		// 忽略 a11y 相关警告
		if (warning.code.startsWith('a11y_')) return;
		// 忽略自闭合标签警告
		if (warning.code === 'element_invalid_self_closing_tag') return;
		// 其他警告正常处理
		handler(warning);
	},

	kit: {
		adapter: adapter({
			pages: 'build',
			assets: 'build',
			fallback: 'index.html',
			precompress: false,
			strict: true
		}),
		alias: {
			$components: 'src/components',
			$lib: 'src/lib'
		}
	}
};

export default config;
