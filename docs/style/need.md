我现在需要实现一个新的功能：前端样式

## 功能描述

该功能是通过AI的能力生成可以在sillytavern中使用的html代码片段，支持html/css/js。用户可以提供示例文本，描述想要根据示例文本生成的样式效果，AI输出正则、世界书内容、代码。同时需要支持将代码渲染出来，用户可以选择"交互式修改"功能，点选渲染内容中的元素，与AI对话，修改选中部分的元素。

## 功能入口

侧边栏的"前端样式"即是功能入口。

## 前端设计

顶栏+两栏式设计。

### 顶栏

标题：皮皮美化生成器
副标题：前端样式生成系统，一键生成样式、正则和世界书。

右上角增加一个"新建"、"保存"和"样式库"功能，生成的内容可以保存到样式库内。

点击"新建"后，清空现有内容，重新创建新的内容；
点击"保存"后，需要对当前的样式进行命名（dialog弹窗命名），然后才能存入样式库。

样式库为shadcn-svelte的"sheet"样式库，请参考"角色库"页面的"筛选功能"，就是是使用sheet组件创建的。

### 左侧栏（控制台）

左侧栏的上方是一个输入框（textarea），用户可以输入原始文本（也可以不输入）。
下方是对话记录 和 输入框及发送按钮。
对话记录缓存在浏览器中。

### 右侧栏（AI输出和交互式编辑）

右侧顶部是三个按钮，左上角为"预览"和"AI输出"。右上角为"修改模式"

默认显示"预览"，"修改模式"按钮，仅在"预览"中可用。

预览tab中，会显示AI生成的内容的实时预览（支持HTML/CSS/JS），在预览中，点击"修改模式"，屏蔽原有渲染结果中的所有交互，用户可以点击选择页面中的元素，选择后，浮窗显示"添加到对话框"，该元素将会被选中并添加到对话框中，但是以选中的元素的最大标签对现实，以#开头，例如`#div`。然后用户可以和AI对话，精准地修改选中部分的内容。

在AI输出tab中，一共显示三个内容：
- 正则：配套的正则
- 样式代码（替换为）：即生成的代码内容
- 世界书条目：生成的世界书条目内容，包含世界书名称和世界书内容（参考图中未实现）（在sillytavern中应用该条目，以控制ai每次输出的内容都能被正则替换为正确的样式）
- 原始文本：即用户输入的原始文本，如果用户没有输入，则AI需要输出

## 后端设计

美化库需要存入数据库，每个美化内容包含：
- 名称：用户保存时输入的名称
- 样式代码
- 正则
- 世界书条目（世界书名称和世界书内容）
- 原始文本
请规划数据库表结构

## Prompt构建

请先构建一个简单的prompt，需要同时生成代码、正则、世界书条目。

请先列好可能使用的变量，例如：

- {{user_request}}：用户输入的内容
- {{original_text}}：原始文本
- {{source_code}}：AI生成的完整代码
- {{code_snippet}}：用户交互式选中的代码片段
- {{worldinfo}}：世界书内容

注意：是需要支持多轮会话的，所以也需要在对话时同时传递上下文，第二轮对话开始（AI已经生成了代码），就附加{{source_code}}到构建的prompt中，如果用户交互式选择了部分代码，就需要{{code_snippet}}附加到prompt中。注意{{code_snippet}}一般会和{{user_request}}一起出现，例如用户可能会："`#div`将它改为红色"，请处理好prompt的构建。

### 推荐的prompt（供参考）

该prompt适合在首轮对话中使用

```
# You are an Expert SillyTavern Frontend & Lore Architect.
Your task is to build a "World Info" and "Frontend Interaction" solution based on the provided {{original_text}} and {{user_request}}.

### INPUT DATA
- **Original Text (`{{original_text}}`):** - IF NOT EMPTY: You MUST extract key fields (Name, Stats, Level, etc.) from here to structure the World Info.
  - IF EMPTY: You must invent logical fields based on the {{user_request}} (e.g., if requesting a "Health Bar", invent an HP field).
- **User Request (`{{user_request}}`):** Defines the visual style (e.g., "Cyberpunk", "Minimalist") and functional requirements.

### LOGIC GATES (Tag Selection)
1. **Scenario A: Status Monitoring** (Status Bars, Dashboards, HUDs, Info Panels)
   - **Wrapper:** You MUST use: `<details><summary>TITLE</summary><statublock>CONTENT</statublock></details>`
   - **Regex Target:** Match content strictly inside `<statublock>...</statublock>`.

2. **Scenario B: Decorative Effects** (Speech Bubbles, Cards, Narrative FX)
   - **Wrapper:** You MUST use: `<piney>CONTENT</piney>`
   - **Regex Target:** Match content strictly inside `<piney>...</piney>`.

### EXECUTION TASKS

1. **Design World Info (Lorebook)**
   - Create an instruction that forces the AI to output data in the format selected by the Logic Gates.
   - **Context:** Use `{{user}}` for user, `{{char}}` for character.
   - **Mapping:** Incorporate all fields found in `{{original_text}}`.
   - **Rules:** Define logic for how values change (e.g., "Decrease HP on damage").

2. **Create Regex Script**
   - Write a JavaScript-compatible Regex to capture variables from the World Info format.
   - **Capturing:** Use groups (`$1`, `$2`...) for dynamic data.
   - **Multiline:** MUST support `[\s\S]*?` to handle multi-line data blocks safely.

3. **Engineer Frontend Code (HTML/CSS/JS)**
   - **Aesthetics:** strictly follow the style described in `{{user_request}}`.
   - **Quality:** Write **Production-Grade** code. You are STRONGLY encouraged to use JS for interactivity (animations, toggle states, dynamic progress bars).
   - **Scope Safety:** You MUST use unique or random IDs/Class names (e.g., `.hud-container-x99`) to prevent style conflicts across the chat log.
   - **Integration:** Embed regex capturing groups (`$1`, `$2`...) into the HTML structure or JS logic.

### OUTPUT FORMAT
Return ONLY a raw JSON object (minified, no markdown):
{
  "worldinfo": {
    "key": "Entry Name",
    "content": "Instruction text for the AI, including format and rules"
  },
  "regex": "The regex pattern string (double escape backslashes)",
  "html": "The full HTML/CSS/JS string (properly escaped for JSON)"
}
```