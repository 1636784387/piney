
/**
 * Smart Tag Filtering Logic
 * Ported from SillyReader (sillyreaderfixpro.html)
 */

export function filterTagContent(text: string, tag: string): string {
    const tagLower = tag.toLowerCase();

    // Regex for Open and Close tags
    const openTagRegex = new RegExp(`<${tagLower}(?:\\s[^>]*)?>`, 'gi');
    const closeTagRegex = new RegExp(`</${tagLower}>`, 'gi');

    // Collect positions
    const openTags: { index: number, end: number, type: 'open' }[] = [];
    const closeTags: { index: number, end: number, type: 'close' }[] = [];
    let match;

    openTagRegex.lastIndex = 0;
    while ((match = openTagRegex.exec(text)) !== null) {
        openTags.push({ index: match.index, end: match.index + match[0].length, type: 'open' });
    }

    closeTagRegex.lastIndex = 0;
    while ((match = closeTagRegex.exec(text)) !== null) {
        closeTags.push({ index: match.index, end: match.index + match[0].length, type: 'close' });
    }

    if (openTags.length === 0 && closeTags.length === 0) {
        return text;
    }

    // Collect ALL tag start positions (to determine where an unclosed tag automatically ends)
    // Matches any XML-like tag start <TagName ...>
    const anyTagStartRegex = /<[a-zA-Z][a-zA-Z0-9_-]*(?:\s[^>]*)?>/gi;
    const allTagStartPositions: number[] = [];
    anyTagStartRegex.lastIndex = 0;
    while ((match = anyTagStartRegex.exec(text)) !== null) {
        allTagStartPositions.push(match.index);
    }

    // Sort all target tags by position
    const allTargetTags = [...openTags, ...closeTags].sort((a, b) => a.index - b.index);

    const removeRanges: { start: number, end: number }[] = [];
    const openStack: typeof openTags[0][] = [];

    // Set of current target tag open positions for quick lookup
    const currentTagOpenPositions = new Set(openTags.map(t => t.index));

    for (const tagObj of allTargetTags) {
        if (tagObj.type === 'open') {
            openStack.push(tagObj);
        } else {
            // Close tag
            if (openStack.length > 0) {
                // Matched pair: <tag>...</tag>
                const openTag = openStack.pop()!;
                removeRanges.push({ start: openTag.index, end: tagObj.end });
            } else {
                // Orphan close tag: 0...</tag>
                // "Delete from start of string to this close tag"
                removeRanges.push({ start: 0, end: tagObj.end });
            }
        }
    }

    // Handle remaining unclosed open tags
    // "Delete from <tag> to next tag start"
    for (const openTag of openStack) {
        let nextTagPos = text.length; // Default to end of string

        // Find the first tag start position that is AFTER this open tag
        // AND is not an instance of the SAME tag type (unless nested? SillyReader ignores nesting check here)
        // SillyReader logic: "!currentTagOpenPositions.has(pos)" 
        // implies "Next tag that is DIFFERENT from the one we are filtering"?
        // Wait, if we have <thought> A <thought> B.
        // openStack has both.
        // First <thought>: next tag is 2nd <thought>.
        // SillyReader logic excludes `currentTagOpenPositions` from being a "stopper".
        // So <thought> ... <thought> ... <other>
        // Start <thought> deletes until <other>.
        // This effectively merges them?

        for (const pos of allTagStartPositions) {
            if (pos > openTag.index && !currentTagOpenPositions.has(pos)) {
                nextTagPos = pos;
                break;
            }
        }
        removeRanges.push({ start: openTag.index, end: nextTagPos });
    }

    if (removeRanges.length === 0) return text;

    // Merge overlapping ranges
    removeRanges.sort((a, b) => a.start - b.start);
    const mergedRanges = [removeRanges[0]];

    for (let i = 1; i < removeRanges.length; i++) {
        const last = mergedRanges[mergedRanges.length - 1];
        const current = removeRanges[i];
        if (current.start <= last.end) {
            last.end = Math.max(last.end, current.end);
        } else {
            mergedRanges.push(current);
        }
    }

    // Reconstruct string
    let result = '';
    let lastCursor = 0;
    for (const range of mergedRanges) {
        // Ensure bounds
        const start = Math.max(0, range.start);
        const end = Math.min(text.length, range.end);

        if (start > lastCursor) {
            result += text.slice(lastCursor, start);
        }
        lastCursor = Math.max(lastCursor, end);
    }
    result += text.slice(lastCursor);

    return result;
}

export function smartFilterTags(text: string, tags: string[]): string {
    let res = text;
    for (const tag of tags) {
        res = filterTagContent(res, tag);
    }
    return res;
}

/**
 * Detect all XML-like tags in the text
 * LOGIC SYNCED WITH project's `frontend/src/lib/utils/exportUtils.ts` -> `scanTags`
 * 1. Matches VALID PAIRED tags only: <tag>...</tag>
 * 2. Removes Code Blocks (```...```), Inline Code (`...`), and Comments (<!-- ... -->) before scanning.
 * 3. Does NOT use a hardcoded HTML blocklist (consistent with export tool).
 */
export function detectTags(text: string): Set<string> {
    const tags = new Set<string>();

    // 1. Clean Content (Order matters: blocks -> inline -> comments)
    let content = text;
    content = content.replace(/```[\s\S]*?```/g, '');
    content = content.replace(/`[^`]*`/g, '');
    content = content.replace(/<!--[\s\S]*?-->/g, '');

    // 2. Strict Regex from exportUtils.ts
    const regex = /<([a-zA-Z0-9_\-\.]+)(?:\s[^>]*)?>[\s\S]*?<\/\1>/g;

    const matches = content.matchAll(regex);
    for (const m of matches) {
        tags.add(m[1]);
    }
    return tags;
}

/**
 * Sorts tags based on priority and then alphabetically
 */
export function sortTags(tags: string[] | Set<string>): string[] {
    const tagList = Array.isArray(tags) ? tags : Array.from(tags);
    const priority = ['content', 'status', 'small theater', 'small-theater', 'small_theater'];

    return tagList.sort((a, b) => {
        const idxA = priority.indexOf(a.toLowerCase());
        const idxB = priority.indexOf(b.toLowerCase());

        if (idxA !== -1 && idxB !== -1) return idxA - idxB;
        if (idxA !== -1) return -1;
        if (idxB !== -1) return 1;

        return a.localeCompare(b);
    });
}

/**
 * For specified tags, convert \n to <br> within their content
 * Preserves other HTML structure.
 */
export function processTagNewlines(text: string, tags: string[]): string {
    let res = text;
    for (const tag of tags) {
        const tagLower = tag.toLowerCase();
        // Regex to capture content of <tag>...</tag>
        // Use [\s\S] for dot-all logic
        const regex = new RegExp(`(<${tagLower}(?:\\s[^>]*)?>)([\\s\\S]*?)(<\/${tagLower}>)`, 'gi');
        res = res.replace(regex, (match, startTag, content, endTag) => {
            // Replace \n with <br> inside content
            // Note: This replaces newlines in nested tags too, but usually fine for display
            // We use <br> for visual line break
            const processedContent = content.replace(/\n/g, '<br>');
            return `${startTag}${processedContent}${endTag}`;
        });
    }
    return res;
}
