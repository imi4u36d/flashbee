use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LocalePreference {
    #[default]
    System,
    En,
    Zh,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lang {
    En,
    Zh,
}

impl Lang {
    pub fn from_preference(preference: LocalePreference) -> Self {
        match preference {
            LocalePreference::En => Lang::En,
            LocalePreference::Zh => Lang::Zh,
            LocalePreference::System => {
                let tag = sys_locale::get_locale().unwrap_or_default();
                if tag.to_ascii_lowercase().starts_with("zh") {
                    Lang::Zh
                } else {
                    Lang::En
                }
            }
        }
    }

    pub fn code(self) -> &'static str {
        match self {
            Lang::En => "en",
            Lang::Zh => "zh",
        }
    }
}

macro_rules! dictionary {
    ($( $key:literal => $en:literal , $zh:literal ; )*) => {
        pub fn tr(lang: Lang, key: &str) -> &'static str {
            match (lang, key) {
                $( (Lang::En, $key) => $en, (Lang::Zh, $key) => $zh, )*
                _ => "",
            }
        }
    };
}

dictionary! {
    "common.close" => "Close", "关闭";
    "common.cancel" => "Cancel", "取消";
    "common.remove" => "Remove", "移除";
    "common.add" => "Add", "添加";
    "common.create" => "Create", "创建";
    "common.save" => "Save", "保存";
    "common.done" => "Done", "完成";
    "common.back" => "Back", "返回";
    "common.retry" => "Try again", "重试";
    "common.loading" => "Loading…", "加载中…";
    "common.confirm" => "Confirm", "确认";
    "common.delete" => "Delete", "删除";
    "common.refresh" => "Refresh", "刷新";
    "common.copy" => "Copy", "复制";
    "common.copied" => "Copied", "已复制";
    "common.current" => "current", "当前";
    "common.clean" => "clean", "干净";
    "common.local" => "local", "本地";
    "common.staged" => "Staged", "已暂存";
    "common.unstaged" => "Changes", "更改";
    "common.workingTree" => "working tree", "工作区";
    "common.system" => "System", "系统";
    "common.justNow" => "just now", "刚刚";
    "common.untracked" => "untracked", "未跟踪";
    "common.binary" => "binary", "二进制";
    "common.none" => "None", "无";
    "common.files" => "files", "个文件";

    "app.name" => "FlashBee", "FlashBee";
    "app.tagline" => "A calm place for every change.", "每一处改动，都有安静的位置。";
    "app.repoRefreshed" => "Repository refreshed · {count} changed", "仓库已刷新 · {count} 处变更";
    "app.refreshed" => "Repositories refreshed", "仓库已刷新";
    "app.loadFailed" => "Could not load changes for “{name}”.", "无法加载「{name}」的变更。";
    "app.actionFailed" => "Could not complete that action.", "无法完成该操作。";

    "sidebar.personalWorkspace" => "Personal workspace", "个人工作区";
    "sidebar.sampleWorkspace" => "Sample workspace", "示例工作区";
    "sidebar.repositories" => "Repositories", "仓库";
    "sidebar.addRepository" => "Add repository", "添加仓库";
    "sidebar.newWorkspace" => "New workspace", "新建工作区";
    "sidebar.deleteWorkspace" => "Delete workspace", "删除工作区";
    "sidebar.keepOneWorkspace" => "At least one workspace is required", "至少需要保留一个工作区";
    "sidebar.noRepositories" => "No repositories yet", "还没有仓库";
    "sidebar.workspaces" => "Workspaces", "工作区";
    "sidebar.collapse" => "Collapse sidebar", "收起侧边栏";
    "sidebar.resize" => "Drag to resize", "拖动调整宽度";
    "sidebar.quickOpen" => "Quick open", "快速打开";
    "sidebar.localGit" => "Local Git", "本地 Git";
    "sidebar.autoRefresh" => "Refreshes when the window is focused", "窗口聚焦时自动刷新";
    "sidebar.workingTreeClean" => "Working tree clean", "工作区干净";
    "sidebar.upToDate" => "Everything is up to date", "所有内容均已同步";
    "sidebar.watchingOne" => "Watching 1 repository", "正在监视 1 个仓库";
    "sidebar.watchingMany" => "Watching {count} repositories", "正在监视 {count} 个仓库";
    "sidebar.linkedWorktree" => "Linked worktree", "关联工作树";
    "sidebar.worktreeCountOne" => "1 linked worktree", "1 个关联工作树";
    "sidebar.worktreeCountMany" => "{count} linked worktrees", "{count} 个关联工作树";
    "sidebar.repoCountOne" => "1 repository", "1 个仓库";
    "sidebar.repoCountMany" => "{count} repositories", "{count} 个仓库";
    "sidebar.overview" => "Overview", "总览";
    "sidebar.history" => "History", "历史";
    "sidebar.fetchAll" => "Fetch all", "全部抓取";
    "sidebar.reviewProgress" => "{done} of {total} reviewed", "已审阅 {done} / {total}";
    "sidebar.filterPlaceholder" => "Filter files…", "筛选文件…";
    "sidebar.onlyUnreviewed" => "Only unreviewed", "只看未审阅";
    "sidebar.hideWhitespace" => "Hide whitespace-only", "隐藏纯空白变更";
    "sidebar.hideGenerated" => "Hide generated files", "隐藏生成文件";
    "sidebar.noMatchingFiles" => "No files match this filter", "没有匹配的文件";

    "repo.add" => "Add repository", "添加仓库";
    "repo.worktree" => "worktree", "工作树";
    "repo.actions" => "Repository actions", "仓库操作";
    "repo.remove" => "Remove repository", "移除仓库";
    "repo.removeTitle" => "Remove repository?", "移除仓库？";
    "repo.removeDescription" => "“{name}” will be removed from this workspace. Files on disk stay untouched.", "「{name}」将从工作区移除，磁盘上的文件不会删除。";
    "repo.removed" => "Repository “{name}” removed", "已移除仓库「{name}」";
    "repo.addTitle" => "Add repository", "添加仓库";
    "repo.addDescription" => "Choose a folder on your Mac. Git branches are read automatically.", "选择一个文件夹，FlashBee 会自动读取 Git 分支。";
    "repo.chooseFolder" => "Choose folder", "选择文件夹";
    "repo.browse" => "Browse…", "浏览…";
    "repo.chooseHint" => "Choose a local Git repository. Branches and the current checkout are read automatically.", "选择一个本地 Git 仓库，分支和当前检出的内容会自动读取。";
    "repo.orPath" => "or enter a local path", "或输入本地路径";
    "repo.inspectPath" => "Continue", "继续";
    "repo.inspecting" => "Reading repository…", "正在读取仓库…";
    "repo.loadingChanges" => "Loading repository changes…", "正在加载仓库变更…";
    "repo.pathExists" => "This repository is already in the workspace.", "这个仓库已在工作区中。";
    "repo.notGit" => "This folder is not a Git repository.", "这个文件夹不是 Git 仓库。";
    "repo.notFound" => "This folder does not exist.", "这个文件夹不存在。";
    "repo.notDirectory" => "Select a folder, not a file.", "请选择文件夹，而不是文件。";
    "repo.inspectFailed" => "Could not read this repository.", "无法读取这个仓库。";
    "repo.gitUnavailable" => "Git is not installed or not available on PATH.", "Git 未安装或不在 PATH 中。";
    "repo.created" => "Repository “{name}” added", "已添加仓库「{name}」";
    "repo.createdMany" => "Added {count} repositories", "已添加 {count} 个仓库";
    "repo.nameLabel" => "Repository name", "仓库名称";
    "repo.pathLabel" => "Local path", "本地路径";
    "repo.branchLabel" => "Branch", "分支";
    "repo.pathPlaceholder" => "~/code/flashbee", "~/code/flashbee";
    "repo.namePlaceholder" => "flashbee", "flashbee";
    "repo.pathRequired" => "Enter the repository path.", "请输入仓库路径。";
    "repo.groupNotFound" => "No repositories found in this folder.", "这个文件夹中没有找到仓库。";
    "repo.discoveryHint" => "Found {count} repositories. Choose the ones to add.", "找到 {count} 个仓库，选择要添加的项。";
    "repo.addSelected" => "Add {count}", "添加 {count} 个";
    "repo.selectAll" => "Select all", "全选";
    "repo.clearSelection" => "Clear", "清空";
    "repo.scanning" => "Scanning for repositories…", "正在扫描仓库…";
    "repo.initHere" => "Initialize Git repository", "初始化 Git 仓库";
    "repo.initHint" => "Initializing creates a .git folder so FlashBee can track changes here.", "初始化会创建 .git 文件夹，之后 FlashBee 就能跟踪这里的变更。";
    "repo.initFailed" => "Could not initialize a Git repository in this folder.", "无法在这个文件夹中初始化 Git 仓库。";
    "repo.revealInFinder" => "Reveal in Finder", "在访达中显示";
    "repo.openInEditor" => "Open in Editor", "在编辑器中打开";
    "repo.openInDefaultApp" => "Open in Default App", "用默认应用打开";
    "repo.openInEditorAtLine" => "Open at line {line}", "在编辑器第 {line} 行打开";
    "repo.copyPath" => "Copy path", "复制路径";
    "repo.switchBranch" => "Compare with branch", "对比分支";
    "repo.jumpToFile" => "Jump to file", "跳转到文件";
    "repo.refresh" => "Refresh changes", "刷新变更";
    "repo.settings" => "Settings", "设置";
    "repo.openSettings" => "Open settings", "打开设置";
    "repo.sync" => "Sync with remote", "与远端同步";
    "repo.syncBlocked" => "Finish the merge before pulling or pushing", "完成合并后才能拉取或推送";
    "repo.pull" => "Pull", "拉取";
    "repo.push" => "Push", "推送";
    "repo.fetch" => "Fetch", "抓取";
    "repo.fetchAll" => "Fetch all repositories", "抓取所有仓库";
    "repo.pullTitle" => "Pull from remote", "从远端拉取";
    "repo.pushTitle" => "Push commits…", "推送提交…";
    "repo.fetchTitle" => "Fetch from remote", "从远端抓取";
    "repo.pulled" => "Pulled the latest changes from the remote", "已从远端拉取最新提交";
    "repo.pushed" => "Pushed local commits to the remote", "本地提交已推送到远端";
    "repo.fetched" => "Fetched the latest refs", "已抓取最新引用";
    "repo.fetchFailed" => "Fetch failed: {message}", "抓取失败：{message}";
    "repo.pullConflicts" => "Pull stopped · {count} files have conflicts", "拉取已停止 · {count} 个文件存在冲突";
    "repo.pullFailed" => "Pull failed: {message}", "拉取失败：{message}";
    "repo.pushFailed" => "Push failed: {message}", "推送失败：{message}";
    "repo.noRemote" => "No remote is configured for this repository.", "这个仓库还没有配置远端地址。";
    "repo.noUpstream" => "This branch has no upstream. Push once to link it to the remote.", "当前分支没有跟踪远端分支。先推送一次即可建立关联。";
    "repo.detachedHead" => "A detached HEAD cannot be pushed.", "分离头指针状态无法推送。";
    "repo.remoteSettings" => "Remote settings…", "远端设置…";
    "repo.remoteSaved" => "Remote address saved", "远端地址已保存";
    "repo.loadFailed" => "Could not load changes for “{name}”.", "无法加载「{name}」的变更。";
    "repo.refreshFailed" => "Could not refresh this repository.", "无法刷新这个仓库。";

    "branch.title" => "Branches", "分支";
    "branch.switch" => "Switch to", "切换到";
    "branch.current" => "Current branch", "当前分支";
    "branch.new" => "New branch…", "新建分支…";
    "branch.newTitle" => "Create a branch", "新建分支";
    "branch.newDescription" => "The new branch starts at the current commit.", "新分支将从当前提交开始。";
    "branch.nameLabel" => "Branch name", "分支名称";
    "branch.namePlaceholder" => "feature/…", "feature/…";
    "branch.nameRequired" => "Enter a branch name.", "请输入分支名称。";
    "branch.nameInvalid" => "Enter a valid branch name.", "请输入合法的分支名称。";
    "branch.nameExists" => "A branch with this name already exists.", "同名分支已存在。";
    "branch.created" => "Created branch “{name}”", "已创建分支「{name}」";
    "branch.switched" => "Switched to “{name}”", "已切换到「{name}」";
    "branch.switchFailed" => "Could not switch branches: {message}", "无法切换分支：{message}";
    "branch.dirtyTitle" => "Uncommitted changes", "存在未提交的改动";
    "branch.dirtyDescription" => "“{name}” has {count} changed files. Stash them before switching?", "「{name}」有 {count} 个文件已改动。是否先暂存这些改动再切换？";
    "branch.stashAndSwitch" => "Stash and switch", "暂存并切换";
    "branch.switchAnyway" => "Switch without stashing", "直接切换";
    "branch.detached" => "Detached HEAD", "分离头指针";
    "branch.local" => "Local branches", "本地分支";
    "branch.remote" => "Remote branches", "远端分支";
    "branch.noBranches" => "No branches found", "没有找到分支";
    "branch.stashed" => "Changes stashed before switching", "切换前已暂存改动";
    "branch.manage" => "Manage branches…", "管理分支…";
    "branch.tabBranches" => "Branches", "分支";
    "branch.count" => "{count} branches", "{count} 个分支";
    "branch.manageTitle" => "Branches", "分支管理";
    "branch.filterPlaceholder" => "Filter branches…", "筛选分支…";
    "branch.startPlaceholder" => "Start point (e.g. main)", "起点（如 main）";
    "branch.createAndSwitch" => "Create & switch", "创建并切换";
    "branch.createOnly" => "Create only", "仅创建";
    "branch.rename" => "Rename", "重命名";
    "branch.renamed" => "Renamed to “{name}”", "已重命名为「{name}」";
    "branch.delete" => "Delete", "删除";
    "branch.deleteTitle" => "Delete branch?", "删除分支？";
    "branch.deleteDescription" => "Delete “{name}”? This cannot be undone.", "删除「{name}」？此操作无法撤销。";
    "branch.deleteUnmergedDescription" => "“{name}” has commits that are not merged. Force deleting discards them.", "「{name}」包含未合并的提交。强制删除会丢弃这些提交。";
    "branch.deleted" => "Deleted branch “{name}”", "已删除分支「{name}」";
    "branch.merge" => "Merge", "合并";
    "branch.mergeTitle" => "Merge branch?", "合并分支？";
    "branch.mergeDescription" => "Merge “{source}” into “{current}”.", "将「{source}」合并到「{current}」。";
    "branch.merged" => "Branch merged", "分支已合并";
    "branch.mergeConflicts" => "Merge stopped · {count} files have conflicts", "合并已停止 · {count} 个文件存在冲突";
    "branch.openWorktree" => "Open worktree", "打开工作树";
    "branch.mergedIntoCurrent" => "merged", "已合并";
    "branch.notMerged" => "unmerged", "未合并";
    "branch.worktreeOwner" => "Checked out in a worktree", "已在工作树中检出";
    "branch.lastCommit" => "Last commit", "最后提交";
    "branch.managerHint" => "Switch, create, merge, rename, or delete branches. Selecting a branch checked out elsewhere opens its worktree.", "切换、新建、合并、重命名或删除分支。选择已在其他工作树检出的分支会打开对应工作树。";
    "branch.searchPlaceholder" => "Filter branches…", "筛选分支…";
    "branch.noMatches" => "No branches match this filter", "没有匹配的分支";
    "branch.currentWorktree" => "this worktree", "当前工作树";
    "branch.actions" => "Branch actions", "分支操作";

    "stash.title" => "Stashes", "暂存列表";
    "stash.description" => "Saved working tree changes for {name}.", "「{name}」保存的工作区改动。";
    "stash.empty" => "No stashed changes", "没有暂存的改动";
    "stash.save" => "Stash changes…", "暂存改动…";
    "stash.saveTitle" => "Stash changes", "暂存改动";
    "stash.saveDescription" => "Save the working tree so the branch can move freely.", "保存当前工作区，让分支可以自由切换。";
    "stash.messageLabel" => "Message (optional)", "说明（可选）";
    "stash.messagePlaceholder" => "Work in progress", "进行中的工作";
    "stash.includeUntracked" => "Include untracked files", "包含未跟踪的文件";
    "stash.saveAction" => "Stash", "暂存";
    "stash.saved" => "Stashed {count} changes", "已暂存 {count} 处改动";
    "stash.nothing" => "No local changes to stash", "没有可暂存的本地改动";
    "stash.apply" => "Restore", "恢复";
    "stash.drop" => "Drop", "丢弃";
    "stash.applied" => "Stash restored", "已恢复暂存";
    "stash.dropped" => "Stash dropped", "已丢弃暂存";
    "stash.applyFailed" => "Could not restore this stash.", "无法恢复这个暂存。";
    "stash.dropFailed" => "Could not drop this stash.", "无法丢弃这个暂存。";
    "stash.onBranch" => "on {branch}", "位于 {branch}";
    "stash.unknownBranch" => "detached", "分离头指针";
    "worktree.manage" => "Manage worktrees…", "管理工作树…";
    "worktree.title" => "Worktrees", "工作树";
    "worktree.description" => "Checkouts attached to {name}.", "「{name}」关联的检出目录。";
    "worktree.add" => "Add worktree", "新建工作树";
    "worktree.addTitle" => "Add worktree", "新建工作树";
    "worktree.branch" => "Branch", "分支";
    "worktree.branchPlaceholder" => "branch or new branch", "分支名或新分支名";
    "worktree.branchRequired" => "Choose or enter a branch.", "请选择或输入分支。";
    "worktree.path" => "Path", "路径";
    "worktree.pathPlaceholder" => "/Users/you/code/project-worktree", "/Users/你/code/project-worktree";
    "worktree.pathRequired" => "Enter a path for the worktree.", "请输入工作树路径。";
    "worktree.startPlaceholder" => "Start point (for a new branch)", "起点（新分支）";
    "worktree.newBranch" => "Create a new branch", "创建新分支";
    "worktree.create" => "Create worktree", "创建工作树";
    "worktree.created" => "Worktree for “{branch}” created at {path}", "已在 {path} 为「{branch}」创建工作树";
    "worktree.remove" => "Remove", "移除";
    "worktree.removeTitle" => "Remove worktree?", "移除工作树？";
    "worktree.removeDescription" => "Remove the checkout at “{path}”? Its branch stays in the repository.", "移除「{path}」的检出目录？分支会保留在仓库中。";
    "worktree.removeDirtyDescription" => "“{path}” has uncommitted changes. Force removing discards them permanently.", "「{path}」存在未提交改动，强制移除会永久丢弃这些改动。";
    "worktree.removed" => "Worktree removed", "已移除工作树";
    "worktree.prune" => "Prune missing", "清理失效工作树";
    "worktree.pruned" => "Pruned {count} worktree records", "已清理 {count} 条失效工作树记录";
    "worktree.empty" => "No linked worktrees", "没有关联工作树";
    "worktree.main" => "main checkout", "主检出";
    "worktree.current" => "current", "当前";
    "worktree.detached" => "detached", "分离头指针";
    "worktree.changed" => "{count} changed", "{count} 个改动";
    "worktree.conflicts" => "{count} conflicts", "{count} 个冲突";
    "worktree.open" => "Open", "打开";
    "worktree.loading" => "Opening worktree at {path}…", "正在打开工作树 {path}…";
    "worktree.noBranches" => "Create a branch first", "请先创建分支";
    "worktree.useExisting" => "Existing branch", "已有分支";
    "remote.editing" => "editing", "编辑中";

    "diff.original" => "Original", "原始";
    "diff.modified" => "Modified", "修改后";
    "diff.unifiedHeader" => "Unified diff", "统一差异";
    "diff.oldGutter" => "old", "旧";
    "diff.newGutter" => "new", "新";
    "diff.lineChanges" => "Line changes", "行变更";
    "diff.layout" => "Diff layout", "Diff 布局";
    "diff.split" => "Split", "分栏";
    "diff.unified" => "Unified", "统一";
    "diff.splitTitle" => "Side-by-side (S)", "并排 (S)";
    "diff.inlineTitle" => "Inline (S)", "行内 (S)";
    "diff.previousTitle" => "Previous change", "上一处变更";
    "diff.nextTitle" => "Next change", "下一处变更";
    "diff.previous" => "Previous change", "上一处变更";
    "diff.next" => "Next change", "下一处变更";
    "diff.changeNav" => "Change navigation", "变更区域导航";
    "diff.copyPath" => "Copy file path", "复制文件路径";
    "diff.copyRelativePath" => "Copy relative path", "复制相对路径";
    "diff.pathCopied" => "File path copied", "已复制文件路径";
    "diff.copyFailed" => "Could not copy path", "无法复制路径";
    "diff.noChanges" => "No changes in this file", "这个文件没有变更";
    "diff.binaryTitle" => "Binary file", "二进制文件";
    "diff.binaryDescription" => "FlashBee cannot show a line-by-line diff for this file.", "FlashBee 无法为这个文件显示逐行差异。";
    "diff.lfsTitle" => "Git LFS pointer", "Git LFS 指针";
    "diff.lfsDescription" => "This file is stored with Git LFS. The diff shows the pointer, not the real content.", "这个文件由 Git LFS 存储，diff 显示的是指针而不是真实内容。";
    "diff.truncated" => "This diff was shortened for performance.", "为了性能，这个 diff 已被截断。";
    "diff.imageTitle" => "Image comparison", "图片对比";
    "diff.imageBefore" => "Before", "修改前";
    "diff.imageAfter" => "After", "修改后";
    "diff.imageMissing" => "No previous version", "没有旧版本";
    "diff.wordDiff" => "Word-level highlights", "词级高亮";
    "diff.openAtLine" => "Open at line", "定位到行";
    "diff.copyAsMarkdown" => "Copy as PR description", "复制为 PR 描述";
    "diff.copyMarkdownDone" => "Copied a Markdown summary of this change", "已复制本次改动的 Markdown 摘要";
    "diff.stageFile" => "Stage file", "暂存文件";
    "diff.unstageFile" => "Unstage file", "取消暂存";
    "diff.discardFile" => "Discard changes", "丢弃改动";
    "diff.stageHunk" => "Stage hunk", "暂存此段";
    "diff.unstageHunk" => "Unstage hunk", "取消暂存此段";
    "diff.discardHunk" => "Discard hunk", "丢弃此段";
    "diff.markReviewed" => "Mark reviewed", "标记为已读";
    "diff.unmarkReviewed" => "Mark unreviewed", "标记为未读";
    "diff.reviewed" => "Reviewed", "已审阅";
    "diff.staged" => "staged", "已暂存";
    "diff.unstaged" => "unstaged", "未暂存";
    "diff.stageAll" => "Stage all", "全部暂存";
    "diff.unstageAll" => "Unstage all", "全部取消暂存";
    "diff.discardTitle" => "Discard changes?", "丢弃改动？";
    "diff.discardDescription" => "The changes to “{name}” will be lost. This cannot be undone.", "「{name}」的改动将丢失，且无法撤销。";
    "diff.discardHunkDescription" => "This hunk in “{name}” will be restored to its previous state. This cannot be undone.", "「{name}」中的这一段将恢复到之前的状态，且无法撤销。";
    "diff.discard" => "Discard", "丢弃";
    "diff.discarded" => "Changes discarded", "已丢弃改动";
    "diff.stageFailed" => "Could not stage this change.", "无法暂存这项改动。";
    "diff.unstageFailed" => "Could not unstage this change.", "无法取消暂存这项改动。";
    "diff.discardFailed" => "Could not discard this change.", "无法丢弃这项改动。";

    "blame.justNow" => "just now", "刚刚";
    "blame.minutesAgo" => "{count} min ago", "{count} 分钟前";
    "blame.hoursAgo" => "{count} h ago", "{count} 小时前";
    "blame.daysAgo" => "{count} d ago", "{count} 天前";
    "blame.monthsAgo" => "{count} mo ago", "{count} 个月前";
    "blame.yearsAgo" => "{count} y ago", "{count} 年前";
    "blame.loading" => "Reading commit…", "正在读取提交…";
    "blame.uncommitted" => "Not committed yet", "尚未提交";
    "blame.uncommittedHint" => "This line only exists in the working tree.", "这一行只存在于工作区。";
    "blame.unavailable" => "Commit info unavailable", "提交信息不可用";
    "blame.unavailableHint" => "This line could not be matched to a commit.", "无法将这一行匹配到提交。";
    "blame.unknownAuthor" => "Unknown author", "未知作者";
    "blame.noSummary" => "No commit message", "没有提交说明";

    "clean.eyebrow" => "All caught up", "全部处理完毕";
    "clean.title" => "Working tree clean", "工作区干净";
    "clean.description" => "{project} has no pending changes. Every file is committed.", "「{project}」没有待处理的改动，所有文件都已提交。";
    "clean.branch" => "Branch", "分支";
    "clean.status" => "Status", "状态";
    "clean.lastChecked" => "Last checked", "上次检查";
    "clean.check" => "Check for changes", "检查变更";

    "commit.title" => "Commit changes", "提交变更";
    "commit.description" => "{branch} · {count} staged files", "{branch} · {count} 个已暂存文件";
    "commit.messageLabel" => "Commit message", "提交说明";
    "commit.messagePlaceholder" => "Describe this change", "描述这次改动";
    "commit.messageRequired" => "Write a commit message first.", "请先填写提交说明。";
    "commit.nothingStaged" => "Stage at least one change before committing.", "请先暂存至少一项改动。";
    "commit.amend" => "Amend last commit", "修补上一次提交";
    "commit.amendHint" => "Replaces the previous commit instead of adding a new one.", "替换上一次提交，而不是新增一次提交。";
    "commit.commit" => "Commit", "提交";
    "commit.commitAndPush" => "Commit & push", "提交并推送";
    "commit.committing" => "Committing…", "正在提交…";
    "commit.committed" => "Committed {hash} · {count} files", "已提交 {hash} · {count} 个文件";
    "commit.failed" => "Commit failed: {message}", "提交失败：{message}";
    "commit.noStagedTitle" => "Nothing staged", "没有已暂存的改动";
    "commit.noStagedDescription" => "Stage files or hunks first, then write a message.", "先暂存文件或代码段，再填写提交说明。";
    "commit.pushAfter" => "Opening push…", "正在打开推送…";
    "commit.showStaged" => "Staged", "已暂存";
    "commit.showUnstaged" => "Unstaged", "未暂存";
    "commit.recent" => "Recently committed", "最近提交";
    "commit.includeAll" => "Stage all and commit", "暂存全部并提交";

    "settings.preferences" => "Preferences", "偏好设置";
    "settings.title" => "Settings", "设置";
    "settings.appearance" => "Appearance", "外观";
    "settings.appearanceDescription" => "Choose a look, or let FlashBee follow your Mac.", "选择外观，或让 FlashBee 跟随系统。";
    "settings.theme" => "Theme", "主题";
    "settings.system" => "System", "系统";
    "settings.systemDescription" => "Auto · currently {theme}", "自动 · 当前为{theme}";
    "settings.light" => "Daylight", "日光";
    "settings.lightDescription" => "Bright and crisp", "明亮清晰";
    "settings.dark" => "Midnight", "午夜";
    "settings.darkDescription" => "Low-light focus", "低光专注";
    "settings.language" => "Language", "语言";
    "settings.languageDescription" => "Choose how FlashBee speaks.", "选择 FlashBee 的界面语言。";
    "settings.languageAuto" => "Auto", "自动";
    "settings.languageAutoDescription" => "Auto · currently {language}", "自动 · 当前为{language}";
    "settings.languageEnglish" => "English", "English";
    "settings.languageChinese" => "中文", "中文";
    "settings.typography" => "Typography", "排版";
    "settings.typographyDescription" => "Tune the type that carries your code.", "调整界面与代码的字号。";
    "settings.interfaceSize" => "Interface size", "界面字号";
    "settings.codeSize" => "Code size", "代码字号";
    "settings.autoFetch" => "Auto fetch", "自动抓取";
    "settings.autoFetchDescription" => "Refresh remote refs in the background every few minutes.", "每隔几分钟在后台刷新远端引用。";
    "settings.restore" => "Restore defaults", "恢复默认";
    "settings.done" => "Done", "完成";
    "settings.resetToast" => "Preferences restored to defaults", "偏好设置已恢复默认";
    "settings.saved" => "Preferences are saved on this Mac", "偏好设置保存在这台 Mac 上";

    "push.title" => "Push commits to {name}", "将提交推送到 {name}";
    "push.description" => "{branch} → {target}", "{branch} → {target}";
    "push.loading" => "Reading commits…", "正在读取提交…";
    "push.loadFailed" => "Could not read the commits to push.", "无法读取待推送的提交。";
    "push.noCommits" => "Nothing to push", "没有待推送的提交";
    "push.noCommitsHint" => "The remote already has every commit on this branch.", "远端已经包含当前分支的所有提交。";
    "push.commitList" => "Commits to push", "待推送的提交";
    "push.detail" => "Commit changes", "提交改动";
    "push.commitsCount" => "{count} commits", "{count} 个提交";
    "push.noSelection" => "No commit selected", "未选择提交";
    "push.noSelectionHint" => "Select a commit to review its changes before pushing.", "选择一个提交以在推送前查看改动。";
    "push.noFiles" => "This commit has no file changes.", "该提交没有文件变更。";
    "push.filesChanged" => "{count} files changed", "{count} 个文件有变更";
    "push.selectAll" => "Select all commits", "全选提交";
    "push.options" => "Push options", "推送选项";
    "push.forcePush" => "Force push", "强制推送";
    "push.forceWarning" => "Force push rewrites the remote branch (--force-with-lease).", "强制推送会重写远端分支（--force-with-lease）。";
    "push.tags" => "Push tags", "推送标签";
    "push.allTags" => "All tags", "所有标签";
    "push.noTags" => "No tags", "没有标签";
    "push.push" => "Push", "推送";
    "push.pushing" => "Pushing…", "正在推送…";
    "push.pushed" => "Pushed local commits to the remote", "本地提交已推送到远端";
    "push.failed" => "Push failed: {message}", "推送失败：{message}";
    "push.newBranch" => "This branch is not on the remote yet. Pushing creates {target}.", "当前分支尚未推送到远端，推送后将创建 {target}。";
    "push.behind" => "The remote has {count} commits this branch does not have. Pull before pushing.", "远端有 {count} 个当前分支没有的提交，请先拉取。";
    "push.selectHint" => "Select at least one commit or enable push tags.", "请至少选择一个提交或勾选推送标签。";
    "push.noSubject" => "No commit message", "没有提交说明";
    "push.partialHint" => "Pushing {count} commits up to {hash}; {newer} newer commits stay local.", "将推送截至 {hash} 的 {count} 个提交，更新的 {newer} 个提交保留在本地。";
    "push.staleSelection" => "The selected commit is no longer on this branch. Refresh and try again.", "所选提交已不在当前分支上，请刷新后重试。";

    "remote.title" => "Remote repository", "远端仓库";
    "remote.description" => "Choose where “{name}” pulls from and pushes to.", "设置「{name}」的拉取和推送地址。";
    "remote.current" => "Configured remotes", "已配置的远端";
    "remote.add" => "Add remote", "添加远端";
    "remote.loading" => "Reading remotes…", "正在读取远端…";
    "remote.empty" => "No remote yet. Add one below to enable pull and push.", "还没有远端。添加后即可拉取和推送。";
    "remote.nameLabel" => "Remote name", "远端名称";
    "remote.urlLabel" => "Remote URL", "远端地址";
    "remote.hint" => "Saving an existing name updates its URL. Credentials are managed by Git itself.", "保存同名项会更新地址。凭据由 Git 自身管理。";
    "remote.save" => "Add remote", "添加远端";
    "remote.update" => "Save address", "保存地址";
    "remote.saveFailed" => "Could not save the remote address.", "无法保存远端地址。";
    "remote.invalidName" => "Use letters, numbers, dots, dashes, or underscores.", "只能使用字母、数字、点、短横线或下划线。";
    "remote.urlRequired" => "Enter the remote URL.", "请输入远端地址。";

    "conflict.title" => "Merge conflicts", "合并冲突";
    "conflict.badge" => "conflict", "冲突";
    "conflict.mergeBadge" => "merge", "合并";
    "conflict.mergeTitle" => "Merge ready to finish", "合并已准备好完成";
    "conflict.resolveCount" => "Resolve conflicts ({count})", "解决冲突（{count}）";
    "conflict.countChip" => "{count} files", "{count} 个文件";
    "conflict.progress" => "{resolved} of {total} resolved", "已解决 {resolved} / {total}";
    "conflict.files" => "Conflicted files", "冲突文件";
    "conflict.mergeInProgress" => "Merge in progress", "正在合并";
    "conflict.loading" => "Reading conflict sides…", "正在读取冲突两侧…";
    "conflict.loadFailed" => "Could not read this conflict.", "无法读取这个冲突。";
    "conflict.ours" => "Ours", "我方";
    "conflict.theirs" => "Theirs", "对方";
    "conflict.left" => "Left", "左侧";
    "conflict.right" => "Right", "右侧";
    "conflict.unifiedHeader" => "Conflict diff", "冲突差异";
    "conflict.emptySide" => "This side deleted the file", "这一侧删除了文件";
    "conflict.useOursFile" => "Use entire left file", "使用整个左侧文件";
    "conflict.useTheirsFile" => "Use entire right file", "使用整个右侧文件";
    "conflict.save" => "Save and resolve", "保存并解决";
    "conflict.saved" => "Resolved {name}", "已解决 {name}";
    "conflict.saveFailed" => "Could not save this resolution.", "无法保存解决结果。";
    "conflict.allResolved" => "Every conflict is resolved", "所有冲突都已解决";
    "conflict.allResolvedDescription" => "Finish the merge to create the merge commit, or abort to return to the previous state.", "完成合并以创建合并提交，或中止并返回之前的状态。";
    "conflict.completeMerge" => "Complete merge", "完成合并";
    "conflict.abortMerge" => "Abort merge", "中止合并";
    "conflict.abortTitle" => "Abort this merge?", "中止这次合并？";
    "conflict.abortDescription" => "All conflict resolutions will be discarded and the working tree returns to its state before the merge.", "所有冲突解决结果都会被丢弃，工作区将恢复到合并前。";
    "conflict.abortConfirm" => "Abort merge", "中止合并";
    "conflict.mergeCompleted" => "Merge completed", "合并已完成";
    "conflict.mergeAborted" => "Merge aborted", "合并已中止";
    "conflict.completeFailed" => "Could not complete the merge.", "无法完成合并。";
    "conflict.abortFailed" => "Could not abort the merge.", "无法中止合并。";
    "conflict.later" => "Resolve later", "稍后解决";
    "conflict.block" => "Conflict {index}", "冲突 {index}";
    "conflict.autoHint" => "can auto-merge", "可自动合并";
    "conflict.unresolved" => "{count} conflicts unresolved", "{count} 个冲突未解决";
    "conflict.unresolvedOne" => "1 conflict unresolved", "1 个冲突未解决";
    "conflict.allBlocksResolved" => "All conflicts resolved in this file", "这个文件中的冲突已全部解决";
    "conflict.bothDeletedDescription" => "Both sides deleted this file. Keeping the deletion resolves the conflict.", "双方都删除了这个文件。保留删除即可解决冲突。";
    "conflict.banner" => "{count} files have unresolved conflicts", "{count} 个文件存在未解决的冲突";
    "conflict.bannerHint" => "The merge stays paused until every conflict is resolved", "在解决所有冲突之前，合并会保持暂停";
    "conflict.bannerComplete" => "Conflicts resolved · merge is ready to finish", "冲突已解决 · 合并可以完成";
    "conflict.bannerReadyHint" => "Create the merge commit when you are ready", "准备好后创建合并提交";
    "conflict.open" => "Resolve conflicts", "解决冲突";
    "conflict.completeShort" => "Complete merge", "完成合并";
    "conflict.binaryTitle" => "Binary file conflict", "二进制文件冲突";
    "conflict.binaryDescription" => "This file cannot be compared line by line. Choose the version to keep.", "这个文件无法逐行比较，请选择要保留的版本。";
    "conflict.fileLevelTitle" => "File-level conflict", "文件级冲突";
    "conflict.fileLevelDescription" => "One side kept this file and the other deleted it. Choose the version to keep.", "一侧保留了这个文件，另一侧删除了它。请选择要保留的版本。";
    "conflict.keepDeleted" => "Keep deletion", "保留删除";
    "conflict.kindBothModified" => "both modified", "双方修改";
    "conflict.kindBothAdded" => "both added", "双方新增";
    "conflict.kindDeletedByUs" => "deleted on the left", "左侧删除";
    "conflict.kindDeletedByThem" => "deleted on the right", "右侧删除";
    "conflict.kindAddedByUs" => "added on the left", "左侧新增";
    "conflict.kindAddedByThem" => "added on the right", "右侧新增";

    "workspace.emptyTitle" => "Start with a repository", "从一个仓库开始";
    "workspace.emptyDescription" => "This workspace is empty. Add a repository to start reviewing changes.", "这个工作区还是空的。添加一个仓库开始审阅变更。";
    "workspace.emptyCta" => "Add your first repository", "添加第一个仓库";
    "workspace.createTitle" => "New workspace", "新建工作区";
    "workspace.createDescription" => "Create a place to group related repositories.", "创建一个用于归类相关仓库的空间。";
    "workspace.nameLabel" => "Workspace name", "工作区名称";
    "workspace.namePlaceholder" => "Client projects", "客户项目";
    "workspace.nameRequired" => "Enter a workspace name.", "请输入工作区名称。";
    "workspace.nameExists" => "A workspace with this name already exists.", "同名工作区已存在。";
    "workspace.created" => "Workspace “{name}” created", "已创建工作区「{name}」";
    "workspace.deleteTitle" => "Delete workspace?", "删除工作区？";
    "workspace.deleteDescription" => "“{name}” will be removed from FlashBee. Files on disk stay untouched.", "「{name}」将从 FlashBee 移除，磁盘上的文件不会删除。";
    "workspace.deleted" => "Workspace “{name}” removed", "已删除工作区「{name}」";

    "overview.title" => "Workspace overview", "工作区总览";
    "overview.description" => "What needs your attention across {count} repositories.", "{count} 个仓库中需要你关注的内容。";
    "overview.attention" => "Needs attention", "需要关注";
    "overview.allClear" => "Everything is in order", "一切井然有序";
    "overview.allClearHint" => "No conflicts, no pending changes, nothing waiting.", "没有冲突、没有待提交的改动、没有等待中的操作。";
    "overview.conflicts" => "{count} conflicts", "{count} 个冲突";
    "overview.merge" => "merge in progress", "正在合并";
    "overview.dirty" => "{count} changed files", "{count} 个文件已改动";
    "overview.staged" => "{count} staged", "{count} 个已暂存";
    "overview.ahead" => "{count} ahead", "领先 {count}";
    "overview.behind" => "{count} behind", "落后 {count}";
    "overview.detached" => "detached HEAD", "分离头指针";
    "overview.clean" => "clean", "干净";
    "overview.open" => "Open repository", "打开仓库";
    "overview.fetchAll" => "Fetch everything", "抓取全部";
    "overview.refreshAll" => "Refresh all", "刷新全部";
    "overview.lastSeen" => "checked {time}", "{time}检查";

    "history.title" => "History", "历史";
    "history.description" => "Read-only log for {name}", "「{name}」的只读提交历史";
    "history.all" => "All commits", "全部提交";
    "history.file" => "File history", "文件历史";
    "history.compare" => "Compare", "对比";
    "history.base" => "Base", "基准";
    "history.target" => "Target", "目标";
    "history.loading" => "Reading history…", "正在读取历史…";
    "history.empty" => "No commits found", "没有找到提交";
    "history.emptyFile" => "No commits touch this file", "没有提交涉及这个文件";
    "history.loadMore" => "Load more", "加载更多";
    "history.detail" => "Commit details", "提交详情";
    "history.selectCommit" => "Select a commit to read its changes.", "选择一个提交以查看改动。";
    "history.compareAction" => "Compare refs", "对比引用";
    "history.clearFile" => "All files", "所有文件";
    "history.author" => "by {author}", "作者 {author}";
    "history.parent" => "parent", "父提交";
    "history.merge" => "merge", "合并";
    "history.today" => "today", "今天";
    "history.thisWeek" => "this week", "本周";
    "history.older" => "older", "更早";

    "search.title" => "Search", "搜索";
    "search.placeholder" => "Search repositories, commits, files, code…", "搜索仓库、提交、文件、代码…";
    "search.repositories" => "Repositories", "仓库";
    "search.changedFiles" => "Changed files", "变更文件";
    "search.commits" => "Commits", "提交";
    "search.code" => "Code", "代码";
    "search.noMatches" => "No matches found", "没有找到匹配项";
    "search.noMatchesHint" => "Try another repository, commit message, file name, or code snippet.", "试试其他仓库、提交说明、文件名或代码片段。";
    "search.searching" => "Searching…", "正在搜索…";
    "search.allRepositories" => "All repositories", "所有仓库";
    "search.inRepo" => "in {name}", "在 {name}";
    "search.open" => "Open", "打开";
    "search.line" => "line {line}", "第 {line} 行";
    "search.empty" => "Type to search the whole workspace", "输入内容以搜索整个工作区";

    "palette.close" => "Close quick open", "关闭快速打开";
    "palette.navigate" => "navigate", "移动";
    "palette.open" => "open", "打开";
    "palette.closeKey" => "close", "关闭";
    "review.markReviewed" => "Mark reviewed", "标记为已读";
    "review.progress" => "{done} / {total} reviewed", "已审阅 {done} / {total}";
    "review.reset" => "Reset review", "重置审阅状态";
    "review.filterUnreviewed" => "Unreviewed only", "只看未审阅";
    "review.hideWhitespace" => "Whitespace only", "纯空白";
    "review.generated" => "generated", "生成文件";

    "toast.error" => "Something went wrong", "出了点问题";
    "toast.saved" => "Saved", "已保存";
    "toast.copied" => "Copied to clipboard", "已复制到剪贴板";
    "toast.fetchAllDone" => "Fetched {count} repositories", "已抓取 {count} 个仓库";
    "toast.fetchAllPartial" => "Fetched {done} of {total} repositories", "已抓取 {done} / {total} 个仓库";
    "toast.reviewReset" => "Review progress cleared", "审阅进度已清除";
}

pub fn t(lang: Lang, key: &str) -> &'static str {
    tr(lang, key)
}

/// Formats a small set of placeholders without pulling in a template engine.
pub fn fill(template: &str, vars: &[(&str, &str)]) -> String {
    let mut value = template.to_string();
    for (key, replacement) in vars {
        value = value.replace(&format!("{{{key}}}"), replacement);
    }
    value
}
