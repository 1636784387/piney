export enum AiFeature {
    OVERVIEW = 'overview', // 概览
    // DIAGNOSIS = 'diagnosis', 
    // TAG_GENERATION = 'tag_generation'
}

export interface PromptVariables {
    name: string;
    description: string;
    personality: string;
    first_mes: string;
    creator_notes: string;
    task_instruction?: string;
    response_format?: string;
    [key: string]: string | undefined; // Allow flexible keys
}
