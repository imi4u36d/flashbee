import react from '@vitejs/plugin-react'
import { repositoryApiPlugin } from './server/repositoryApi.ts'
import { defineConfig } from 'vite'

// https://vite.dev/config/
export default defineConfig({
  plugins: [react(), repositoryApiPlugin()],
})
