# 酒馆助手 HTML/CSS/JS 渲染实现详解

## 一、渲染检测机制

**文件**: `src/util/is_frontend.ts:1-3`

```typescript
export function isFrontend(content: string): boolean {
  return ['html>', '<head>', '<body'].some(tag => content.includes(tag));
}
```

通过检查代码块内容是否包含 `html>`、`<head>` 或 `<body>` 标签来判断是否为前端代码。

---

## 二、整体渲染流程

```
聊天消息渲染完成
       ↓
message.ts:render$mes() 查找符合条件的前端代码块
       ↓
用 <div class="TH-render"> 包装代码块
       ↓
Render.vue 使用 Teleport 将 Iframe 组件渲染到包装元素内
       ↓
Iframe.vue 创建 iframe 并加载内容
       ↓
iframe 内脚本调整高度并 postMessage 通知父窗口
```

---

## 三、消息代码块处理

**文件**: `src/store/iframe_runtimes/message.ts:8-28`

```typescript
function render$mes($mes: JQuery<HTMLElement>, reload_memo: string): Runtime[] {
  return _($mes.toArray())
    .map(div => {
      const message_id = Number($(div).attr('mesid'));
      const $element = $(div)
        .find('pre')
        .filter((_index, pre) => isFrontend($(pre).text()))  // 检测是否前端代码
        .map((_index, pre) => {
          const $pre = $(pre);
          const $possible_div = $pre.parent('div.TH-render');
          if ($possible_div.length > 0) {
            return $possible_div[0];
          }
          $pre.wrap('<div class="TH-render">');  // 用 div 包装
          return $pre.parent('div.TH-render')[0];
        });
      return { message_id, reload_memo, elements: $element.toArray() };
    })
    .filter(({ elements }) => elements.length > 0)
    .value();
}
```

---

## 四、iframe 组件实现

**文件**: `src/panel/render/Iframe.vue`

### 4.1 模板

```vue
<template>
  <iframe
    :id="prefixed_id"
    :name="prefixed_id"
    ref="iframe_ref"
    loading="lazy"
    v-bind="src_prop"
    class="w-full"
    frameborder="0"
    @load="onLoad"
  />
</template>
```

### 4.2 创建 iframe 内容

```typescript
// 从 <pre><code> 中提取文本内容
const content = createSrcContent($pre.find('code').text(), props.useBlobUrl);

// 两种模式：srcdoc 或 Blob URL
if (!props.useBlobUrl) {
  return { srcdoc: content };
}
return { src: URL.createObjectURL(new Blob([content], { type: 'text/html' })) };
```

### 4.3 高度调整监听

```typescript
useEventListener('message', event => {
  if (event?.data?.type === 'TH_ADJUST_IFRAME_HEIGHT' && event?.data?.iframe_name === iframe_ref.value?.id) {
    iframe_ref.value!.style.height = `${event.data.height}px`;
  }
});
useEventListener(window, 'resize', () => {
  iframe_ref.value?.contentWindow?.postMessage({ type: 'TH_UPDATE_VIEWPORT_HEIGHT' }, '*');
});
```

---

## 五、iframe 内容构建

**文件**: `src/panel/render/iframe.ts:78-104`

```typescript
export function createSrcContent(content: string, use_blob_url: boolean) {
  content = replaceVhInContent(content);  // VH 单位转换

  return `
<html>
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
${use_blob_url ? `<base href="${window.location.origin}"/>` : ''}
<style>
*,*::before,*::after{box-sizing:border-box;}
html,body{margin:0!important;padding:0;overflow:hidden!important;max-width:100%!important;}
.user_avatar,.user-avatar{background-image:url('${getUserAvatarPath()}')}
.char_avatar,.char-avatar{background-image:url('${getCharAvatarPath()}')}
</style>
${third_party}  <!-- 注入第三方库 -->
<script src="${predefine_url}"></script>
<script src="https://testingcf.jsdelivr.net/npm/vue/dist/vue.runtime.global.prod.min.js"></script>
<script src="${adjust_viewport_url}"></script>
<script src="${adjust_iframe_height_url}"></script>
</head>
<body>
${content}  <!-- 用户代码放在 body 内 -->
</body>
</html>
`;
}
```

---

## 六、VH 单位转换

**文件**: `src/panel/render/iframe.ts:5-75`

解决 iframe 内 `100vh` 计算问题，将 `vh` 转换为 `var(--TH-viewport-height)` CSS 变量：

```typescript
function replaceVhInContent(content: string): string {
  // 检测是否有使用 vh 单位
  const has_css_min_vh = /min-height\s*:\s*[^;{}]*\d+(?:\.\d+)?vh/gi.test(content);
  // ... 更多检测正则

  const convertVhToVariable = (value: string) =>
    value.replace(/(\d+(?:\.\d+)?)vh\b/gi, (match, value) => {
      const parsed = parseFloat(value);
      if (parsed === 100) {
        return `var(--TH-viewport-height)`;
      }
      return `calc(var(--TH-viewport-height) * ${parsed / 100})`;
    });

  // 替换 CSS 声明块中的 min-height
  content = content.replace(
    /(min-height\s*:\s*)([^;]*?\d+(?:\.\d+)?vh)(?=\s*[;}])/gi,
    (_m, prefix, value) => `${prefix}${convertVhToVariable(value)}`,
  );

  // ... 更多替换场景：行内 style、JavaScript 设置等
  return content;
}
```

---

## 七、第三方库注入

**文件**: `src/iframe/third_party_message.html`

```html
<link rel="stylesheet" href="https://testingcf.jsdelivr.net/npm/@fortawesome/fontawesome-free/css/all.min.css" />
<script src="/scripts/extensions/third-party/JS-Slash-Runner/lib/tailwindcss.min.js"></script>
<script src="https://testingcf.jsdelivr.net/npm/jquery/dist/jquery.min.js"></script>
<script src="https://testingcf.jsdelivr.net/npm/jquery-ui/dist/jquery-ui.min.js"></script>
<link rel="stylesheet" href="https://testingcf.jsdelivr.net/npm/jquery-ui/themes/base/theme.min.css" />
<script src="https://testingcf.jsdelivr.net/npm/jquery-ui-touch-punch"></script>
<script src="https://testingcf.jsdelivr.net/npm/vue/dist/vue.runtime.global.prod.min.js"></script>
<script src="https://testingcf.jsdelivr.net/npm/vue-router/dist/vue-router.global.prod.min.js"></script>
<script src="https://testingcf.jsdelivr.net/npm/pixi.js/dist/pixi.min.js"></script>
```

### 支持的内容

| 库 | 用途 |
|---|------|
| Font Awesome | 图标 |
| jQuery + jQuery UI | DOM 操作 |
| Vue 3 Runtime | 响应式 UI |
| Vue Router | 路由 |
| Pixi.js | 2D 图形/游戏 |
| TailwindCSS | 样式框架 |

---

## 八、全局变量注入

**文件**: `src/iframe/predefine.js`

```javascript
// 继承父窗口的 lodash
window._ = window.parent._;

// SillyTavern 上下文
Object.defineProperty(window, 'SillyTavern', {
  get: () => _.get(window.parent, 'SillyTavern').getContext(),
});

// iframe ID 缓存
const iframeId = window.frameElement?.id || window.name;
if (iframeId) {
  window.__TH_IFRAME_ID = iframeId;
  window.name = iframeId;
}

// 合并父窗口的全局对象
result = _(window);
result = result.merge(_.pick(window.parent, ['EjsTemplate', 'TavernHelper', 'YAML', 'showdown', 'toastr', 'z']));

// Mvu 状态管理
if (_.has(window.parent, 'Mvu')) {
  Object.defineProperty(window, 'Mvu', {
    get: () => _.get(window.parent, 'Mvu'),
    set: () => {},
  });
}
```

---

## 九、高度自动调整

**文件**: `src/iframe/adjust_iframe_height.js`

### 9.1 测量高度并通知父窗口

```javascript
function measureAndPost() {
  const body = document.body;
  const html = document.documentElement;

  let height = 0;
  if (IS_BLOB_MODE) {
    // Blob 模式：计算子元素高度
    const children = Array.from(body.children || []);
    // 排除绝对/固定定位元素
    for (const el of children) {
      if (position === 'absolute' || position === 'fixed') continue;
      // 计算元素边界...
    }
    height = total_height;
  } else {
    // srcdoc 模式
    height = body.scrollHeight;
  }

  window.parent.postMessage({ type: 'TH_ADJUST_IFRAME_HEIGHT', iframe_name: getIframeName(), height }, '*');
}
```

### 9.2 监听变化

```javascript
const resize_observer = new ResizeObserver(() => postIframeHeight());
resize_observer.observe(body);

if (IS_BLOB_MODE) {
  const mutation_observer = new MutationObserver(() => {
    resize_observer.disconnect();
    for (const element of body.children) {
      resize_observer.observe(element);
    }
    postIframeHeight();
  });
  mutation_observer.observe(body, { childList: true, subtree: true, attributes: true });
}
```

---

## 十、视口高度同步

**文件**: `src/iframe/adjust_viewport.js`

```javascript
// 设置 CSS 变量
$('html').css('--TH-viewport-height', `${window.parent.innerHeight}px`);

// 监听父窗口 resize
window.addEventListener('message', function (event) {
  if (event.data?.type === 'TH_UPDATE_VIEWPORT_HEIGHT') {
    $('html').css('--TH-viewport-height', `${window.parent.innerHeight}px`);
  }
});
```

---

## 十一、关键配置项

**文件**: `src/store/settings/global.ts`

```typescript
interface RenderSettings {
  enabled: boolean;                          // 启用渲染器
  collapse_code_block: 'all' | 'frontend_only' | 'none'; // 代码折叠模式
  use_blob_url: boolean;                     // 使用 Blob URL 渲染（便于调试）
  depth: number;                             // 渲染深度 (0 = 全部)
}
```

---

## 十二、复刻要点总结

1. **用 `<div class="TH-render">` 包装代码块**，用 `<Teleport>` 将 iframe 放入其中
2. **检测前端代码**：检查是否包含 `<head>`、`<body>` 或 `html>` 标签
3. **构建 iframe 内容**：注入基础样式、第三方库、全局变量
4. **VH 单位转换**：将 `100vh` 转为 `var(--TH-viewport-height)` CSS 变量
5. **高度自适应**：iframe 内用 ResizeObserver + MutationObserver 测量高度，postMessage 通知父窗口
6. **视口同步**：监听父窗口 resize，更新 CSS 变量
