import { createI18n } from 'vue-i18n'
import zhCN from './locales/zh-CN'
import enUS from './locales/en-US'

// 定义支持的语言类型
export type SupportedLocale = 'zh-CN' | 'en-US'

// 创建i18n实例
const i18n = createI18n({
  legacy: false, // 使用组合式API
  locale: 'zh-CN', // 默认语言
  fallbackLocale: 'en-US', // 回退语言
  messages: {
    'zh-CN': zhCN,
    'en-US': enUS
  }
})

export default i18n 