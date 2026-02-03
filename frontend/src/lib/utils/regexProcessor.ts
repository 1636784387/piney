export interface RegexScript {
    id: string;
    scriptName: string;
    findRegex: string;
    replaceString: string;
    trimStrings?: string[];
    placement?: number[];
    disabled?: boolean;
    markdownOnly?: boolean;
    promptOnly?: boolean;
    runOnEdit?: boolean;
    substituteRegex?: number;
    minDepth?: number | null;
    maxDepth?: number | null;
}

interface ProcessOptions {
    isMarkdown?: boolean;
    isPrompt?: boolean;
    noWrap?: boolean;
}

export function processContentWithScripts(content: string, scripts: RegexScript[], options: ProcessOptions = {}): string {
    if (!content) return "";
    let processed = content;

    for (const script of scripts) {
        if (script.disabled) continue;

        if (script.markdownOnly && !options.isMarkdown) continue;
        if (script.promptOnly && !options.isPrompt) continue;

        try {

            // Fallback for SillyTavern format (regex / replace)
            // @ts-ignore
            let pattern = script.findRegex || script.regex;
            let flags = "g";

            if (!pattern) continue;

            const trimmed = pattern.trim();
            // Robust parsing: Find the LAST slash that separates pattern and flags
            // Standard JS Regex literal syntax: /pattern/flags
            if (trimmed.startsWith("/") && trimmed.lastIndexOf("/") > 0) {
                const lastSlashIndex = trimmed.lastIndexOf("/");
                const extractedPattern = trimmed.substring(1, lastSlashIndex);
                const extractedFlags = trimmed.substring(lastSlashIndex + 1);
                const validFlags = extractedFlags.split('').filter((c: string) => "gimsuy".includes(c)).join('');

                pattern = extractedPattern;
                flags = validFlags || "g";
            }

            const regex = new RegExp(pattern, flags);

            // @ts-ignore
            let replacement = script.replaceString || script.replace || "";

            replacement = replacement
                .replace(/\\n/g, '\n')
                .replace(/\\r/g, '\r')
                .replace(/\\t/g, '\t')
                .replace(/\\"/g, '"');
            if (replacement.trim() && !options.noWrap) {
                replacement = `<piney_render>\n${replacement}\n</piney_render>`;
            }

            processed = processed.replace(regex, replacement);
        } catch (e) {
            console.warn(`Failed to apply regex script ${script.scriptName}:`, e);
        }
    }

    return processed;
}
