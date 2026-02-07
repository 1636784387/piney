import { writable, get } from 'svelte/store';
import { toast } from "svelte-sonner";
import { API_BASE } from "$lib/api";

type UploadState = {
    isUploading: boolean;
    isComplete: boolean;
    progress: {
        total: number;
        current: number;
        success: number;
        failed: number;
    };
    currentFileName: string;
};

const initialState: UploadState = {
    isUploading: false,
    isComplete: false,
    progress: { total: 0, current: 0, success: 0, failed: 0 },
    currentFileName: ""
};

function createUploadStore() {
    const { subscribe, set, update } = writable<UploadState>(initialState);

    return {
        subscribe,
        reset: () => set(initialState),
        startUpload: async (files: FileList | File[], onComplete?: () => void) => {
            const state = get({ subscribe });
            if (state.isUploading) {
                toast.error("已有上传任务正在进行中");
                return;
            }

            const fileArray = Array.from(files);
            const total = fileArray.length;
            if (total === 0) return;

            update(s => ({
                isUploading: true,
                isComplete: false,
                progress: { total, current: 0, success: 0, failed: 0 },
                currentFileName: ""
            }));

            // 进度提示 Toast (仅显示一次)
            // const toastId = toast.loading(`正在上传 ${total} 张图片... 请勿关闭页面`, { duration: Infinity });
            const toastId = undefined;

            const CONCURRENCY = 3;
            const queue = [...fileArray];

            const processNext = async () => {
                const file = queue.shift();
                if (!file) return;

                update(s => ({
                    ...s,
                    progress: {
                        ...s.progress,
                        current: s.progress.current + 1
                    },
                    currentFileName: file.name
                }));

                const formData = new FormData();
                formData.append("files", file);

                try {
                    const token = localStorage.getItem("auth_token");
                    const res = await fetch(`${API_BASE}/api/images`, {
                        method: "POST",
                        headers: token ? { Authorization: `Bearer ${token}` } : {},
                        body: formData,
                    });

                    if (res.ok) {
                        update(s => ({
                            ...s,
                            progress: {
                                ...s.progress,
                                success: s.progress.success + 1
                            }
                        }));
                    } else {
                        update(s => ({
                            ...s,
                            progress: {
                                ...s.progress,
                                failed: s.progress.failed + 1
                            }
                        }));
                        console.error(`Failed to upload ${file.name}`);
                    }
                } catch (e) {
                    update(s => ({
                        ...s,
                        progress: {
                            ...s.progress,
                            failed: s.progress.failed + 1
                        }
                    }));
                    console.error(`Error uploading ${file.name}`, e);
                }
            };

            const pool = Array(Math.min(CONCURRENCY, total)).fill(null).map(async () => {
                while (queue.length > 0) {
                    await processNext();
                }
            });

            await Promise.all(pool);

            const finalState = get({ subscribe });
            update(s => ({ ...s, isUploading: false, isComplete: true, currentFileName: "" }));

            if (finalState.progress.failed > 0) {
                toast.error(`上传完成: ${finalState.progress.success} 成功, ${finalState.progress.failed} 失败`);
            } else {
                toast.success(`全部 ${finalState.progress.success} 张图片上传成功`);
            }

            if (onComplete) onComplete();

            // 3秒后重置完成状态
            setTimeout(() => {
                update(s => ({ ...s, isComplete: false }));
            }, 3000);
        }
    };
}

export const galleryUpload = createUploadStore();
