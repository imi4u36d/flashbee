import { createHighlighterCore } from 'shiki/core'
import { createJavaScriptRegexEngine } from 'shiki/engine/javascript'
import theme from 'shiki/themes/vesper.mjs'
import langBash from 'shiki/langs/bash.mjs'
import langC from 'shiki/langs/c.mjs'
import langCpp from 'shiki/langs/cpp.mjs'
import langCsharp from 'shiki/langs/csharp.mjs'
import langCss from 'shiki/langs/css.mjs'
import langGo from 'shiki/langs/go.mjs'
import langHtml from 'shiki/langs/html.mjs'
import langIni from 'shiki/langs/ini.mjs'
import langJava from 'shiki/langs/java.mjs'
import langJavascript from 'shiki/langs/javascript.mjs'
import langJsx from 'shiki/langs/jsx.mjs'
import langJson from 'shiki/langs/json.mjs'
import langKotlin from 'shiki/langs/kotlin.mjs'
import langLess from 'shiki/langs/less.mjs'
import langMarkdown from 'shiki/langs/markdown.mjs'
import langPhp from 'shiki/langs/php.mjs'
import langPython from 'shiki/langs/python.mjs'
import langRuby from 'shiki/langs/ruby.mjs'
import langRust from 'shiki/langs/rust.mjs'
import langScss from 'shiki/langs/scss.mjs'
import langSql from 'shiki/langs/sql.mjs'
import langSvelte from 'shiki/langs/svelte.mjs'
import langSwift from 'shiki/langs/swift.mjs'
import langToml from 'shiki/langs/toml.mjs'
import langTsx from 'shiki/langs/tsx.mjs'
import langTypescript from 'shiki/langs/typescript.mjs'
import langVue from 'shiki/langs/vue.mjs'
import langWgsl from 'shiki/langs/wgsl.mjs'
import langXml from 'shiki/langs/xml.mjs'
import langYaml from 'shiki/langs/yaml.mjs'

import type { ResolvedTheme } from './settings'

type Highlighter = Awaited<ReturnType<typeof createHighlighterCore>>

let highlighter: Highlighter | null = null
const htmlCache = new Map<string, string>()

const flashbeeNightTheme = {
  ...theme,
  name: 'flashbee-night',
  type: 'dark' as const,
  colors: {
    ...theme.colors,
    'editor.background': '#00000000',
    'editor.foreground': '#d9dfeb',
  },
  tokenColors: [
    {
      scope: ['comment', 'punctuation.definition.comment'],
      settings: { foreground: '#66727f', fontStyle: 'italic' },
    },
    {
      scope: [
        'keyword',
        'storage',
        'storage.type',
        'storage.modifier',
        'keyword.control',
        'keyword.operator.new',
      ],
      settings: { foreground: '#c7a6ee' },
    },
    {
      scope: [
        'string',
        'string.quoted',
        'string.template',
        'constant.other.symbol',
        'markup.inline.raw',
      ],
      settings: { foreground: '#8fd3a8' },
    },
    {
      scope: ['constant.numeric', 'constant.language', 'constant.character'],
      settings: { foreground: '#f5c77e' },
    },
    {
      scope: [
        'entity.name.function',
        'support.function',
        'meta.function-call',
        'variable.function',
      ],
      settings: { foreground: '#82cdea' },
    },
    {
      scope: [
        'entity.name.type',
        'entity.name.class',
        'support.class',
        'support.type',
        'entity.other.inherited-class',
      ],
      settings: { foreground: '#f0c98f' },
    },
    {
      scope: [
        'variable',
        'variable.other',
        'variable.parameter',
        'meta.object-literal.key',
        'object.key',
      ],
      settings: { foreground: '#d9dfeb' },
    },
    {
      scope: ['variable.other.constant', 'support.constant', 'entity.name.constant'],
      settings: { foreground: '#f5a489' },
    },
    {
      scope: [
        'entity.name.tag',
        'meta.tag',
        'punctuation.definition.tag',
        'support.class.component',
      ],
      settings: { foreground: '#ef8ba0' },
    },
    {
      scope: [
        'entity.other.attribute-name',
        'support.type.property-name',
        'variable.other.property',
      ],
      settings: { foreground: '#8dc7ea' },
    },
    {
      scope: ['punctuation', 'meta.brace', 'keyword.operator'],
      settings: { foreground: '#7f8b9c' },
    },
    {
      scope: ['markup.heading', 'entity.name.section', 'markup.bold'],
      settings: { foreground: '#fbba63', fontStyle: 'bold' },
    },
    {
      scope: ['markup.underline.link', 'string.other.link'],
      settings: { foreground: '#82cdea' },
    },
    {
      scope: ['invalid', 'invalid.illegal'],
      settings: { foreground: '#f27f92' },
    },
    {
      scope: ['meta.diff.header', 'meta.diff.range'],
      settings: { foreground: '#a8b3c2' },
    },
  ],
}


const flashbeeDayTheme = {
  ...theme,
  name: 'flashbee-day',
  type: 'light' as const,
  colors: {
    ...theme.colors,
    'editor.background': '#00000000',
    'editor.foreground': '#2c3542',
  },
  tokenColors: [
    {
      scope: ['comment', 'punctuation.definition.comment'],
      settings: { foreground: '#8a96a6', fontStyle: 'italic' },
    },
    {
      scope: [
        'keyword',
        'storage',
        'storage.type',
        'storage.modifier',
        'keyword.control',
        'keyword.operator.new',
      ],
      settings: { foreground: '#7645b8' },
    },
    {
      scope: [
        'string',
        'string.quoted',
        'string.template',
        'constant.other.symbol',
        'markup.inline.raw',
      ],
      settings: { foreground: '#247a55' },
    },
    {
      scope: ['constant.numeric', 'constant.language', 'constant.character'],
      settings: { foreground: '#a75a12' },
    },
    {
      scope: [
        'entity.name.function',
        'support.function',
        'meta.function-call',
        'variable.function',
      ],
      settings: { foreground: '#146b96' },
    },
    {
      scope: [
        'entity.name.type',
        'entity.name.class',
        'support.class',
        'support.type',
        'entity.other.inherited-class',
      ],
      settings: { foreground: '#8d5b09' },
    },
    {
      scope: [
        'variable',
        'variable.other',
        'variable.parameter',
        'meta.object-literal.key',
        'object.key',
      ],
      settings: { foreground: '#344052' },
    },
    {
      scope: ['variable.other.constant', 'support.constant', 'entity.name.constant'],
      settings: { foreground: '#b64a22' },
    },
    {
      scope: [
        'entity.name.tag',
        'meta.tag',
        'punctuation.definition.tag',
        'support.class.component',
      ],
      settings: { foreground: '#b23d5c' },
    },
    {
      scope: [
        'entity.other.attribute-name',
        'support.type.property-name',
        'variable.other.property',
      ],
      settings: { foreground: '#1a6f86' },
    },
    {
      scope: ['punctuation', 'meta.brace', 'keyword.operator'],
      settings: { foreground: '#758192' },
    },
    {
      scope: ['markup.heading', 'entity.name.section', 'markup.bold'],
      settings: { foreground: '#996b0a', fontStyle: 'bold' },
    },
    {
      scope: ['markup.underline.link', 'string.other.link'],
      settings: { foreground: '#2e6fb7' },
    },
    {
      scope: ['invalid', 'invalid.illegal'],
      settings: { foreground: '#c93452' },
    },
    {
      scope: ['meta.diff.header', 'meta.diff.range'],
      settings: { foreground: '#647184' },
    },
  ],
}

const highlighterInit = createHighlighterCore({
  themes: [flashbeeNightTheme, flashbeeDayTheme],
  langs: [
    langTsx,
    langTypescript,
    langJavascript,
    langJsx,
    langCss,
    langScss,
    langLess,
    langJson,
    langMarkdown,
    langPython,
    langGo,
    langIni,
    langToml,
    langRust,
    langWgsl,
    langHtml,
    langYaml,
    langBash,
    langSql,
    langJava,
    langKotlin,
    langSwift,
    langC,
    langCpp,
    langCsharp,
    langRuby,
    langPhp,
    langXml,
    langVue,
    langSvelte,
  ],
  engine: createJavaScriptRegexEngine(),
}).then((instance) => {
  highlighter = instance
  return instance
})

export function warmHighlighter() {
  return highlighterInit
}

const escapeHtml = (value: string) =>
  value
    .replaceAll('&', '&amp;')
    .replaceAll('<', '&lt;')
    .replaceAll('>', '&gt;')

const escapeAttr = (value: string) =>
  escapeHtml(value).replaceAll('"', '&quot;')

export function highlightLine(
  code: string,
  language: string,
  appTheme: ResolvedTheme = 'dark',
) {
  if (!code) return ''

  const themeName = appTheme === 'light' ? 'flashbee-day' : 'flashbee-night'
  const fallback = appTheme === 'light' ? '#2c3542' : '#d9dfeb'
  const cacheKey = `${appTheme}\u0000${language}\u0000${code}`
  const cached = htmlCache.get(cacheKey)
  if (cached !== undefined) return cached

  if (!highlighter) return escapeHtml(code)

  try {
    if (!highlighter.getLoadedLanguages().includes(language)) {
      return escapeHtml(code)
    }

    const result = highlighter.codeToTokens(code, {
      lang: language,
      theme: themeName,
    })

    const html = (result.tokens[0] ?? [])
      .map((token) => {
        const styles = [`color:${token.color ?? fallback}`]
        if (token.fontStyle && token.fontStyle & 1) styles.push('font-style:italic')
        if (token.fontStyle && token.fontStyle & 2) styles.push('font-weight:600')
        if (token.fontStyle && token.fontStyle & 4) styles.push('text-decoration:underline')
        return `<span style="${escapeAttr(styles.join(';'))}">${escapeHtml(token.content)}</span>`
      })
      .join('')

    htmlCache.set(cacheKey, html)
    return html
  } catch {
    return escapeHtml(code)
  }
}
