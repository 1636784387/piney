import { AiFeature } from './types';

export const PROMPT_TEMPLATES = {
    [AiFeature.OVERVIEW]: `请深入分析以下角色卡数据：

[角色元数据]
Name: {{name}}
Description: {{description}}

[详细设定]
Personality: {{personality}}
First Message: {{first_mes}}
Creator Notes: {{creator_notes}}

[任务与约束]
{{task_instruction}}

[回复格式]
请严格仅返回 JSON，不要使用代码块：
{{response_format}}
`
};

export const SYSTEM_PROMPTS = {
    [AiFeature.OVERVIEW]: `你是SillyTavern角色卡侧写师。
请深度解析角色卡数据，精准提取核心要素（性格特征、语言风格、世界观、剧情钩子等），生成客观的角色设定摘要。
严禁文学创作，仅返回无Markdown标记的纯JSON格式。`
};
