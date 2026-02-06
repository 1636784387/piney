<script lang="ts">
    import { onMount } from "svelte";
    import { goto } from "$app/navigation";
    import { Button } from "$lib/components/ui/button/index.js";
    import * as Card from "$lib/components/ui/card/index.js";
    import { readTextFile, BaseDirectory } from "@tauri-apps/plugin-fs";
    import { appDataDir } from "@tauri-apps/api/path";

    let logs = $state("");
    let loading = $state(true);
    let error = $state("");
    let copied = $state(false);
    let dataDir = $state("");

    onMount(async () => {
        await loadLogs();
    });

    async function loadLogs() {
        loading = true;
        error = "";
        try {
            // 获取 app data 目录路径
            const appData = await appDataDir();
            dataDir = appData;
            
            // 读取 startup.log 文件
            const content = await readTextFile("startup.log", {
                baseDir: BaseDirectory.AppData
            });
            logs = content || "（日志文件为空）";
        } catch (e: any) {
            console.error("读取日志失败:", e);
            error = `读取日志失败: ${e.message || e}`;
            logs = "";
        } finally {
            loading = false;
        }
    }

    async function copyLogs() {
        try {
            const fullText = `=== Piney 启动日志 ===\n数据目录: ${dataDir}\n\n${logs}`;
            await navigator.clipboard.writeText(fullText);
            copied = true;
            setTimeout(() => {
                copied = false;
            }, 2000);
        } catch (e) {
            console.error("复制失败:", e);
        }
    }

    function goBack() {
        goto("/sign-up");
    }
</script>

<div class="min-h-svh w-full p-4 md:p-8">
    <Card.Root class="max-w-4xl mx-auto">
        <Card.Header>
            <div class="flex items-center justify-between">
                <div>
                    <Card.Title class="flex items-center gap-2">
                        🔧 调试日志
                    </Card.Title>
                    <Card.Description>
                        startup.log 启动日志 - 用于诊断问题
                    </Card.Description>
                </div>
                <div class="flex gap-2">
                    <Button variant="outline" size="sm" onclick={loadLogs} disabled={loading}>
                        {loading ? "加载中..." : "刷新"}
                    </Button>
                    <Button variant="outline" size="sm" onclick={goBack}>
                        返回
                    </Button>
                </div>
            </div>
        </Card.Header>
        <Card.Content>
            <!-- 数据目录信息 -->
            {#if dataDir}
                <div class="mb-4 p-3 bg-muted rounded-md">
                    <div class="text-xs text-muted-foreground mb-1">数据目录</div>
                    <code class="text-sm break-all">{dataDir}</code>
                </div>
            {/if}

            <!-- 错误提示 -->
            {#if error}
                <div class="mb-4 p-3 bg-destructive/10 text-destructive rounded-md text-sm">
                    {error}
                </div>
            {/if}

            <!-- 日志内容 -->
            <div class="relative">
                <pre class="p-4 bg-black text-green-400 rounded-md overflow-auto max-h-[60vh] text-xs font-mono whitespace-pre-wrap">{#if loading}正在加载日志...{:else if logs}{logs}{:else}暂无日志内容{/if}</pre>
                
                <!-- 复制按钮 -->
                <Button 
                    variant="secondary" 
                    size="sm" 
                    class="absolute top-2 right-2"
                    onclick={copyLogs}
                    disabled={loading || !logs}
                >
                    {copied ? "已复制 ✓" : "一键复制"}
                </Button>
            </div>

            <!-- 提示信息 -->
            <div class="mt-4 text-xs text-muted-foreground">
                <p>💡 提示：如果遇到登录问题，请复制日志并发送给开发者以便诊断。</p>
            </div>
        </Card.Content>
    </Card.Root>
</div>
