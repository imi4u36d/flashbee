import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import '@fontsource-variable/inter'
import '@fontsource-variable/jetbrains-mono'
import './index.css'
import App from './App.tsx'
import { installContextMenuPolicy } from './lib/contextMenu'
import {
  hydrateNativeStore,
  invokeNative,
  isNative,
  markNativeEnvironment,
  watchFullscreen,
} from './lib/native'
import { applyBootSettings } from './lib/settings'

async function boot() {
  await hydrateNativeStore()
  markNativeEnvironment()
  applyBootSettings()

  createRoot(document.getElementById('root')!).render(
    <StrictMode>
      <App />
    </StrictMode>,
  )

  if (isNative) {
    watchFullscreen()
    installContextMenuPolicy()
    await invokeNative('app_ready').catch(() => undefined)
  }
}

void boot()
