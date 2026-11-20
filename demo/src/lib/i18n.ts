import { createContext, useContext } from 'react'

export type Locale = 'en' | 'zh'
export type LocalePreference = 'system' | Locale

const en = {
  'common.local': 'local',
  'common.clean': 'clean',
  'common.close': 'Close',
  'common.workingTree': 'working tree',
  'common.staged': 'staged',
  'common.unstaged': 'unstaged',
  'common.current': 'current',
  'common.system': 'System',

  'sidebar.collapse': 'Collapse sidebar',
  'sidebar.personalWorkspace': 'Personal workspace',
  'sidebar.allRepositories': 'All repositories',
  'sidebar.repositories': 'Repositories',
  'sidebar.addRepository': 'Add repository',
  'sidebar.staged': 'Staged',
  'sidebar.changes': 'Changes',
  'sidebar.workingTreeClean': 'Working tree clean',
  'sidebar.upToDate': 'Everything is up to date',
  'sidebar.quickOpen': 'Quick open',
  'sidebar.watchingOne': 'Watching 1 repository',
  'sidebar.watchingMany': 'Watching {count} repositories',

  'repo.private': 'private',
  'repo.openSidebar': 'Open sidebar',
  'repo.switchBranch': 'Switch branch',
  'repo.jumpToFile': 'Jump to file',
  'repo.refresh': 'Refresh changes',
  'repo.settings': 'Settings',
  'repo.openSettings': 'Open settings',
  'repo.openProfile': 'Open profile',

  'diff.original': 'Original',
  'diff.modified': 'Modified',
  'diff.unifiedHeader': 'Unified diff',
  'diff.oldGutter': 'old',
  'diff.newGutter': 'new',
  'diff.lineChanges': 'Line changes',
  'diff.layout': 'Diff layout',
  'diff.splitTitle': 'Side-by-side (S)',
  'diff.inlineTitle': 'Inline (S)',
  'diff.split': 'Split',
  'diff.unified': 'Unified',
  'diff.previousTitle': 'Previous file (K)',
  'diff.nextTitle': 'Next file (J)',
  'diff.previous': 'Previous file',
  'diff.next': 'Next file',
  'diff.copyPath': 'Copy file path',
  'diff.pathCopied': 'File path copied to clipboard',
  'diff.copyFailed': 'Could not copy path',

  'clean.eyebrow': 'All caught up',
  'clean.title': 'Working tree clean',
  'clean.description':
    '{project} has no pending changes. Every file is committed and your local branch is in sync.',
  'clean.repository': 'Repository',
  'clean.branch': 'Branch',
  'clean.lastChecked': 'Last checked',
  'clean.justNow': 'just now',
  'clean.check': 'Check for changes',

  'palette.close': 'Close quick open',
  'palette.dialog': 'Quick open',
  'palette.placeholder': 'Search repositories, branches, files…',
  'palette.searchAria': 'Search repositories, branches, and files',
  'palette.repositories': 'Repositories',
  'palette.changedFiles': 'Changed files',
  'palette.changed': '{count} changed',
  'palette.noMatches': 'No matches found',
  'palette.noMatchesHint': 'Try a repository name, branch, or file path.',
  'palette.brand': 'FlashBee quick open',
  'palette.navigate': 'navigate',
  'palette.open': 'open',
  'palette.closeKey': 'close',
  'palette.status': 'Status:',

  'settings.preferences': 'Preferences',
  'settings.title': 'Settings',
  'settings.close': 'Close settings',
  'settings.appearance': 'Appearance',
  'settings.appearanceDescription':
    'Choose a look, or let FlashBee follow your Mac.',
  'settings.systemIs': 'System is {theme}',
  'settings.theme': 'Theme',
  'settings.system': 'System',
  'settings.systemDescription': 'Auto · currently {theme}',
  'settings.light': 'Daylight',
  'settings.lightDescription': 'Bright and crisp',
  'settings.dark': 'Midnight',
  'settings.darkDescription': 'Low-light focus',
  'settings.lightWord': 'light',
  'settings.darkWord': 'dark',
  'settings.language': 'Language',
  'settings.languageDescription': 'Choose how FlashBee speaks.',
  'settings.interfaceLanguage': 'Interface language',
  'settings.interfaceLanguageDescription':
    'Switches all app labels and controls.',
  'settings.languageAria': 'Interface language',
  'settings.languageAuto': 'Auto',
  'settings.languageAutoDescription': 'Auto · currently {language}',
  'settings.languageEnglish': 'English',
  'settings.languageChinese': '中文',
  'settings.typography': 'Typography',
  'settings.typographyDescription': 'Tune the type that carries your code.',
  'settings.interfaceFont': 'Interface font',
  'settings.interfaceFontDescription': 'Used for menus, labels, and metadata.',
  'settings.interfaceSize': 'Interface size',
  'settings.interfaceSizeDescription': 'Scales menus, labels, and controls.',
  'settings.codeFont': 'Code font',
  'settings.codeFontDescription': 'Applies to diff code and file paths.',
  'settings.codeSize': 'Code size',
  'settings.codeSizeDescription': 'Adjust line density without zooms.',
  'settings.livePreview': 'Live preview',
  'settings.previewSentence':
    'Every change, every repository — in one calm place.',
  'settings.previewComment': '// review the whole workspace',
  'settings.restore': 'Restore defaults',
  'settings.done': 'Done',
  'settings.saved': 'Preferences are saved on this device',
  'settings.resetToast': 'Preferences restored to defaults',
  'settings.themeAria': 'Theme',
  'settings.uiFontAria': 'Interface font',
  'settings.codeFontAria': 'Code font',
  'settings.uiSizeAria': 'Interface font size',
  'settings.codeSizeAria': 'Code font size',


  'common.cancel': 'Cancel',
  'common.remove': 'Remove',
  'common.add': 'Add',
  'common.create': 'Create',

  'sidebar.workspaces': 'Workspaces',
  'sidebar.workspaceRepoCountOne': '1 repository',
  'sidebar.workspaceRepoCountMany': '{count} repositories',
  'sidebar.newWorkspace': 'New workspace',
  'sidebar.deleteWorkspace': 'Delete current workspace',
  'sidebar.keepOneWorkspace': 'At least one workspace is required',
  'sidebar.noRepositories': 'No repositories yet',

  'repo.add': 'Add repository',
  'repo.actions': 'Repository actions',
  'repo.remove': 'Remove repository',
  'repo.removeTitle': 'Remove repository?',
  'repo.removeDescription':
    '“{name}” will be removed from this workspace. The folder on disk will not be deleted.',
  'repo.removed': 'Repository “{name}” removed',
  'repo.addTitle': 'Add repository',

  'repo.branches': 'Branches',
  'repo.noBranches': 'No branches found',
  'repo.chooseFolder': 'Choose folder',
  'repo.browse': 'Browse…',
  'repo.chooseHint':
    'Choose a local Git repository. Branches and the current checkout are read automatically.',
  'repo.orPath': 'or enter a local path',
  'repo.inspectPath': 'Continue',
  'repo.pickingFolder': 'Waiting for folder selection…',
  'repo.inspecting': 'Reading repository…',
  'repo.loadingChanges': 'Loading repository changes…',
  'repo.pathExists': 'This repository is already in the workspace.',
  'repo.notGit': 'This folder is not a Git repository.',
  'repo.notFound': 'This folder does not exist.',
  'repo.notDirectory': 'Select a folder, not a file.',
  'repo.pickerUnsupported':
    'Native folder selection is unavailable here. Enter a path instead.',
  'repo.inspectFailed': 'Could not read this repository.',
  'repo.serviceUnavailable': 'The local repository service is unavailable.',
  'repo.changeFolder': 'Choose another folder',
  'repo.currentBranchHint': 'Defaults to the current branch.',

  'repo.addDescription':
    'Choose a folder on your Mac. FlashBee reads its Git branches and current checkout automatically.',
  'repo.nameLabel': 'Repository name',
  'repo.namePlaceholder': 'e.g. nova-web',
  'repo.nameRequired': 'Enter a repository name.',
  'repo.nameExists': 'This repository is already in the workspace.',
  'repo.pathLabel': 'Local path',
  'repo.pathPlaceholder': '~/studio/nova-web',
  'repo.pathRequired': 'Enter the repository path.',
  'repo.branchLabel': 'Branch',
  'repo.branchPlaceholder': 'main',
  'repo.created': 'Repository “{name}” added',

  'workspace.emptyEyebrow': 'Empty workspace',
  'workspace.emptyTitle': 'Start with a repository',
  'workspace.emptyDescription':
    'This workspace is empty. Add a repository to start reviewing changes.',
  'workspace.emptyCta': 'Add your first repository',
  'workspace.createTitle': 'New workspace',
  'workspace.createDescription':
    'Create a place to group related repositories.',
  'workspace.nameLabel': 'Workspace name',
  'workspace.namePlaceholder': 'e.g. Client projects',
  'workspace.nameRequired': 'Enter a workspace name.',
  'workspace.nameExists': 'A workspace with this name already exists.',
  'workspace.created': 'Workspace “{name}” created',
  'workspace.deleteTitle': 'Delete workspace?',
  'workspace.deleteDescription':
    '“{name}” will be removed from FlashBee. Files on disk stay untouched.',
  'workspace.deleted': 'Workspace “{name}” removed',

  'app.title': 'FlashBee — Multi-project Git viewer',
  'app.repoRefreshed': 'Repository refreshed · {count} changed',
  'repo.loadFailed': 'Could not load changes for “{name}”.',
  'repo.refreshFailed': 'Could not refresh this repository.',
  'app.refreshed': 'Repositories refreshed · 12 files changed',
  'app.srSummary':
    '{project}, {path}, {count} total line changes across the workspace.',
  'app.srClean': 'Working tree clean',
} as const

export type TranslationKey = keyof typeof en

const zh: Record<TranslationKey, string> = {
  'common.local': '本地',
  'common.clean': '干净',
  'common.close': '关闭',
  'common.workingTree': '工作区',
  'common.staged': '已暂存',
  'common.unstaged': '未暂存',
  'common.current': '当前',
  'common.system': '系统',

  'sidebar.collapse': '收起侧边栏',
  'sidebar.personalWorkspace': '个人工作区',
  'sidebar.allRepositories': '所有仓库',
  'sidebar.repositories': '仓库',
  'sidebar.addRepository': '添加仓库',
  'sidebar.staged': '已暂存',
  'sidebar.changes': '更改',
  'sidebar.workingTreeClean': '工作区干净',
  'sidebar.upToDate': '所有内容均已同步',
  'sidebar.quickOpen': '快速打开',
  'sidebar.watchingOne': '正在监视 1 个仓库',
  'sidebar.watchingMany': '正在监视 {count} 个仓库',

  'repo.private': '私有',
  'repo.openSidebar': '打开侧边栏',
  'repo.switchBranch': '切换分支',
  'repo.jumpToFile': '跳转到文件',
  'repo.refresh': '刷新变更',
  'repo.settings': '设置',
  'repo.openSettings': '打开设置',
  'repo.openProfile': '打开个人资料',

  'diff.original': '原始',
  'diff.modified': '修改后',
  'diff.unifiedHeader': '统一差异',
  'diff.oldGutter': '旧',
  'diff.newGutter': '新',
  'diff.lineChanges': '行变更',
  'diff.layout': 'Diff 布局',
  'diff.splitTitle': '并排 (S)',
  'diff.inlineTitle': '行内 (S)',
  'diff.split': '分栏',
  'diff.unified': '统一',
  'diff.previousTitle': '上一个文件 (K)',
  'diff.nextTitle': '下一个文件 (J)',
  'diff.previous': '上一个文件',
  'diff.next': '下一个文件',
  'diff.copyPath': '复制文件路径',
  'diff.pathCopied': '文件路径已复制到剪贴板',
  'diff.copyFailed': '无法复制路径',


  'clean.eyebrow': '一切就绪',
  'clean.title': '工作区干净',
  'clean.description':
    '{project} 没有待处理的更改。所有文件均已提交，本地分支保持同步。',
  'clean.repository': '仓库',
  'clean.branch': '分支',
  'clean.lastChecked': '最近检查',
  'clean.justNow': '刚刚',
  'clean.check': '检查变更',

  'palette.close': '关闭快速打开',
  'palette.dialog': '快速打开',
  'palette.placeholder': '搜索仓库、分支或文件…',
  'palette.searchAria': '搜索仓库、分支和文件',
  'palette.repositories': '仓库',
  'palette.changedFiles': '变更文件',
  'palette.changed': '{count} 项变更',
  'palette.noMatches': '未找到匹配项',
  'palette.noMatchesHint': '试试仓库名、分支名或文件路径。',
  'palette.brand': 'FlashBee 快速打开',
  'palette.navigate': '导航',
  'palette.open': '打开',
  'palette.closeKey': '关闭',
  'palette.status': '状态：',

  'settings.preferences': '偏好设置',
  'settings.title': '设置',
  'settings.close': '关闭设置',
  'settings.appearance': '外观',
  'settings.appearanceDescription': '选择一种外观，或让 FlashBee 跟随你的 Mac。',
  'settings.systemIs': '系统当前为{theme}',
  'settings.theme': '主题',
  'settings.system': '跟随系统',
  'settings.systemDescription': '自动 · 当前{theme}',
  'settings.light': '日光',
  'settings.lightDescription': '明亮清爽',
  'settings.dark': '午夜',
  'settings.darkDescription': '暗光专注',
  'settings.lightWord': '浅色',
  'settings.darkWord': '深色',
  'settings.language': '语言',
  'settings.languageDescription': '选择 FlashBee 的界面语言。',
  'settings.interfaceLanguage': '界面语言',
  'settings.interfaceLanguageDescription': '切换应用中的所有标签和控件。',
  'settings.languageAria': '界面语言',
  'settings.languageAuto': '自动',
  'settings.languageAutoDescription': '自动 · 当前{language}',
  'settings.languageEnglish': 'English',
  'settings.languageChinese': '中文',
  'settings.typography': '字体',
  'settings.typographyDescription': '调整阅读代码时的字体和字号。',
  'settings.interfaceFont': '界面字体',
  'settings.interfaceFontDescription': '用于菜单、标签和元信息。',
  'settings.interfaceSize': '界面字号',
  'settings.interfaceSizeDescription': '缩放菜单、标签和控件。',
  'settings.codeFont': '代码字体',
  'settings.codeFontDescription': '应用于 Diff 代码和文件路径。',
  'settings.codeSize': '代码字号',
  'settings.codeSizeDescription': '调整代码行密度，无需缩放页面。',
  'settings.livePreview': '实时预览',
  'settings.previewSentence': '所有变更，所有仓库，尽在一处。',
  'settings.previewComment': '// 审阅整个工作区',
  'settings.restore': '恢复默认',
  'settings.done': '完成',
  'settings.saved': '偏好设置已保存在本设备',
  'settings.resetToast': '已恢复默认偏好设置',
  'settings.themeAria': '主题',
  'settings.uiFontAria': '界面字体',
  'settings.codeFontAria': '代码字体',
  'settings.uiSizeAria': '界面字号',
  'settings.codeSizeAria': '代码字号',


  'common.cancel': '取消',
  'common.remove': '移除',
  'common.add': '添加',
  'common.create': '创建',

  'sidebar.workspaces': '工作区',
  'sidebar.workspaceRepoCountOne': '1 个仓库',
  'sidebar.workspaceRepoCountMany': '{count} 个仓库',
  'sidebar.newWorkspace': '新建工作区',
  'sidebar.deleteWorkspace': '删除当前工作区',
  'sidebar.keepOneWorkspace': '至少需要保留一个工作区',
  'sidebar.noRepositories': '暂无仓库',

  'repo.add': '添加仓库',
  'repo.actions': '仓库操作',
  'repo.remove': '移除仓库',
  'repo.removeTitle': '移除仓库？',
  'repo.removeDescription':
    '将从当前工作区移除“{name}”，磁盘上的文件夹不会被删除。',
  'repo.removed': '已移除仓库“{name}”',
  'repo.addTitle': '添加仓库',
  'repo.branches': '分支',
  'repo.noBranches': '未找到分支',
  'repo.chooseFolder': '选择文件夹',
  'repo.browse': '浏览…',
  'repo.chooseHint': '选择一个本地 Git 仓库，会自动读取分支和当前检出状态。',
  'repo.orPath': '或输入本地路径',
  'repo.inspectPath': '继续',
  'repo.pickingFolder': '等待选择文件夹…',
  'repo.inspecting': '正在读取仓库…',
  'repo.loadingChanges': '正在加载仓库变更…',
  'repo.pathExists': '该仓库已存在于当前工作区。',
  'repo.notGit': '该文件夹不是 Git 仓库。',
  'repo.notFound': '该文件夹不存在。',
  'repo.notDirectory': '请选择文件夹，而不是文件。',
  'repo.pickerUnsupported': '当前环境不支持原生文件夹选择，请输入路径。',
  'repo.inspectFailed': '无法读取该仓库。',
  'repo.serviceUnavailable': '本地仓库服务不可用。',
  'repo.changeFolder': '选择其他文件夹',
  'repo.currentBranchHint': '默认使用当前分支。',
  'repo.addDescription':
    '选择一个本地文件夹，FlashBee 会自动读取它的 Git 分支和当前检出状态。',
  'repo.nameLabel': '仓库名称',
  'repo.namePlaceholder': '例如：nova-web',
  'repo.nameRequired': '请输入仓库名称。',
  'repo.nameExists': '该仓库已存在于当前工作区。',
  'repo.pathLabel': '本地路径',
  'repo.pathPlaceholder': '~/studio/nova-web',
  'repo.pathRequired': '请输入仓库路径。',
  'repo.branchLabel': '分支',
  'repo.branchPlaceholder': 'main',
  'repo.created': '已添加仓库“{name}”',

  'workspace.emptyEyebrow': '空工作区',
  'workspace.emptyTitle': '从一个仓库开始',
  'workspace.emptyDescription': '这个工作区还没有仓库。添加一个仓库开始审查变更。',
  'workspace.emptyCta': '添加第一个仓库',
  'workspace.createTitle': '新建工作区',
  'workspace.createDescription': '创建一个工作区，用来归类相关仓库。',
  'workspace.nameLabel': '工作区名称',
  'workspace.namePlaceholder': '例如：客户项目',
  'workspace.nameRequired': '请输入工作区名称。',
  'workspace.nameExists': '已存在同名工作区。',
  'workspace.created': '已创建工作区“{name}”',
  'workspace.deleteTitle': '删除工作区？',
  'workspace.deleteDescription':
    '将从 FlashBee 中移除“{name}”，磁盘上的文件不会受到影响。',
  'workspace.deleted': '已删除工作区“{name}”',

  'app.title': 'FlashBee — 多项目 Git 查看器',
  'app.repoRefreshed': '仓库已刷新 · {count} 项变更',
  'repo.loadFailed': '无法加载“{name}”的变更。',
  'repo.refreshFailed': '无法刷新该仓库。',
  'app.refreshed': '仓库已刷新 · 12 个文件有变更',
  'app.srSummary': '{project}，{path}，工作区共 {count} 行变更。',
  'app.srClean': '工作区干净',
}

const messages: Record<Locale, Record<TranslationKey, string>> = { en, zh }

export function resolveLocale(preference: LocalePreference): Locale {
  if (preference !== 'system') return preference
  const language = navigator.languages?.[0] ?? navigator.language ?? 'en'
  return language.toLowerCase().startsWith('zh') ? 'zh' : 'en'
}

export function localeName(locale: Locale) {
  return locale === 'zh' ? '中文' : 'English'
}

export type Translate = (
  key: TranslationKey,
  variables?: Record<string, string | number>,
) => string

export function createTranslator(locale: Locale): Translate {
  return (key, variables) => {
    let value = messages[locale][key]
    if (!variables) return value
    for (const [name, replacement] of Object.entries(variables)) {
      value = value.replaceAll(`{${name}}`, String(replacement))
    }
    return value
  }
}

export interface I18nContextValue {
  locale: Locale
  t: Translate
}

export const I18nContext = createContext<I18nContextValue | null>(null)

export function useI18n() {
  const context = useContext(I18nContext)
  if (!context) {
    throw new Error('useI18n must be used inside I18nProvider')
  }
  return context
}
