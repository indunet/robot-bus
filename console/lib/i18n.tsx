'use client'

import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
  type ReactNode,
} from 'react'

export type Locale = 'en' | 'zh'

const STORAGE_KEY = 'robot-bus-console-locale'
const DEFAULT_LOCALE: Locale = 'en'

type MessageKey = keyof typeof en

const en = {
  navOverview: 'OVERVIEW',
  navOverviewShort: 'OVE',
  navTopics: 'TOPICS',
  navTopicsShort: 'TOP',
  navServices: 'SERVICES',
  navServicesShort: 'SVC',
  navActions: 'ACTIONS',
  navActionsShort: 'ACT',
  navEvents: 'EVENTS',
  navEventsShort: 'EVT',
  navTopology: 'TOPOLOGY',
  navTopologyShort: 'TOPO',
  navBotSim: 'BOT SIM',
  navBotSimShort: 'BOT',
  navBotWindows: 'OPEN BOT SIM + TELEOP',
  botSimWindowTitle: 'BOT SIM NODE · bot_sim_node',
  botTeleopWindowTitle: 'BOT TELEOP NODE · bot_teleop_node',
  botPoseTitle: 'POSE',
  botTeleopTitle: 'TELEOP',
  botConnecting: 'CONNECTING',
  botOnline: 'ONLINE',
  botPublishError: 'PUBLISH ERROR',
  botArrowHelp: 'Click here, then use the Arrow keys',
  botPublish: 'PUB',
  botSubscribe: 'SUB',

  metricMsgS: 'MSG/S',
  metricBw: 'BW',
  metricSvcS: 'SVC/S',
  metricActS: 'ACT/S',
  metricTotal: 'TOTAL',
  metricUptime: 'UPTIME',
  metricPid: 'PID',
  metricErr: 'ERR',

  brokerTitle: 'BROKER',
  brokerSub: 'Endpoint health',
  listen: 'LISTEN',
  down: 'DOWN',

  statsTitle: 'SNAPSHOT',
  statsSub: 'Live counters',
  statMsgS: 'MSG/S',
  statBw: 'BANDWIDTH',
  statTopics: 'TOPICS',
  statTopicsActive: '{n} active',
  statServices: 'SERVICES',
  statWorkers: '{n} workers',
  statSvcS: 'CALLS/S',
  statActions: 'ACTIONS',
  statActiveGoals: '{n} active',
  statActS: 'RUNS/S',
  statTotalMsgs: 'TOTAL MSGS',
  statUptime: 'UPTIME',

  throughputTitle: 'TOPIC THROUGHPUT',
  throughputSub: 'Last 60s msg/s',
  throughputSeries: 'MSG/S',
  svcRateTitle: 'SERVICE CALLS',
  svcRateSub: 'Last 60s calls/s',
  svcRateSeries: 'CALLS/S',
  actRateTitle: 'ACTION RUNS',
  actRateSub: 'Last 60s runs/s',
  actRateSeries: 'RUNS/S',

  topicsTitle: 'TOPICS',
  topicsActive: '{n} active',
  colTopic: 'TOPIC',
  colType: 'TYPE',
  colMsgS: 'MSG/S',
  colBandwidth: 'BANDWIDTH',
  colTotal: 'TOTAL',
  colPub: 'PUB',
  colSub: 'SUB',
  colLastSeen: 'LAST SEEN',

  servicesTitle: 'SERVICES',
  servicesKnown: '{n} known',
  servicesEmpty: 'No services yet',
  colService: 'SERVICE',
  colCalls: 'CALLS',
  colCallsS: 'CALLS/S',
  colWorkers: 'WORKERS',
  colErr: 'ERR',
  colP99: 'AVG ms',
  colLast: 'LAST',

  actionsTitle: 'ACTIONS',
  actionsKnown: '{n} known',
  actionsEmpty: 'No actions yet',
  colAction: 'ACTION',
  colRuns: 'RUNS',
  colRunsS: 'RUNS/S',
  colActive: 'ACTIVE',
  colAvgDur: 'AVG ms',

  eventsTitle: 'EVENTS',
  eventsSub: 'Live event stream',
  eventsEmpty: 'No log entries',
  eventsCount: '{n} entries',
  eventsPaused: 'Paused',
  eventsLive: 'Live',
  pause: 'PAUSE',
  resume: 'RESUME',
  clear: 'CLR',
  filterAll: 'ALL',

  langEn: 'EN',
  langZh: '中文',
  langSwitch: 'Language',

  ageMs: '{n}ms ago',
  ageSec: '{n}s ago',
  ageMin: '{n}m ago',
  ageHour: '{n}h ago',
  ageDash: '—',

  uptimeDhms: '{d}d {h}h {m}m {s}s',
  uptimeHms: '{h}h {m}m {s}s',
  uptimeMs: '{m}m {s}s',

  topologyTitle: 'TOPOLOGY',
  topologySub: '{n} nodes · {e} edges',
  topologyEmpty: 'No registered endpoints yet',
  topologyNode: 'NODE',
  topologyInput: 'INPUT',
  topologyOutput: 'OUTPUT',
  topologyNoPorts: 'No ports registered',
} as const

const zh: Record<MessageKey, string> = {
  navOverview: '概览',
  navOverviewShort: '概览',
  navTopics: '话题',
  navTopicsShort: '话题',
  navServices: '服务',
  navServicesShort: '服务',
  navActions: '动作',
  navActionsShort: '动作',
  navEvents: '事件',
  navEventsShort: '事件',
  navTopology: '拓扑',
  navTopologyShort: '拓扑',
  navBotSim: '小机器人仿真',
  navBotSimShort: 'BOT',
  navBotWindows: '打开小机器人仿真与遥控浮窗',
  botSimWindowTitle: '小机器人仿真节点 · bot_sim_node',
  botTeleopWindowTitle: '小机器人遥控节点 · bot_teleop_node',
  botPoseTitle: '位姿',
  botTeleopTitle: '遥控',
  botConnecting: '连接中',
  botOnline: '在线',
  botPublishError: '发布错误',
  botArrowHelp: '点击此处，然后使用键盘方向键',
  botPublish: '发布',
  botSubscribe: '订阅',

  metricMsgS: 'MSG/S',
  metricBw: '带宽',
  metricSvcS: 'SVC/S',
  metricActS: 'ACT/S',
  metricTotal: '总计',
  metricUptime: '运行',
  metricPid: 'PID',
  metricErr: '错误',

  brokerTitle: 'BROKER',
  brokerSub: '端点健康',
  listen: 'LISTEN',
  down: 'DOWN',

  statsTitle: 'SNAPSHOT',
  statsSub: '实时计数',
  statMsgS: 'MSG/S',
  statBw: '带宽',
  statTopics: '话题数',
  statTopicsActive: '{n} 活跃',
  statServices: '服务数',
  statWorkers: '{n} workers',
  statSvcS: 'CALLS/S',
  statActions: '动作数',
  statActiveGoals: '{n} 进行中',
  statActS: 'RUNS/S',
  statTotalMsgs: '消息总计',
  statUptime: '运行时间',

  throughputTitle: 'TOPIC THROUGHPUT',
  throughputSub: '近 60s msg/s',
  throughputSeries: 'MSG/S',
  svcRateTitle: 'SERVICE CALLS',
  svcRateSub: '近 60s calls/s',
  svcRateSeries: 'CALLS/S',
  actRateTitle: 'ACTION RUNS',
  actRateSub: '近 60s runs/s',
  actRateSeries: 'RUNS/S',

  topicsTitle: 'TOPICS',
  topicsActive: '{n} 活跃',
  colTopic: 'TOPIC',
  colType: 'TYPE',
  colMsgS: 'MSG/S',
  colBandwidth: '带宽',
  colTotal: '总计',
  colPub: 'PUB',
  colSub: 'SUB',
  colLastSeen: '最近',

  servicesTitle: 'SERVICES',
  servicesKnown: '{n} 已知',
  servicesEmpty: '暂无服务',
  colService: 'SERVICE',
  colCalls: '调用',
  colCallsS: 'CALLS/S',
  colWorkers: 'Worker',
  colErr: '错误',
  colP99: '平均 ms',
  colLast: '最近',

  actionsTitle: 'ACTIONS',
  actionsKnown: '{n} 已知',
  actionsEmpty: '暂无动作',
  colAction: 'ACTION',
  colRuns: '运行',
  colRunsS: 'RUNS/S',
  colActive: '进行中',
  colAvgDur: '平均 ms',

  eventsTitle: 'EVENTS',
  eventsSub: '实时日志流',
  eventsEmpty: '无日志条目',
  eventsCount: '{n} 条目',
  eventsPaused: '已暂停',
  eventsLive: '实时',
  pause: '暂停',
  resume: '继续',
  clear: '清空',
  filterAll: '全部',

  langEn: 'EN',
  langZh: '中文',
  langSwitch: '语言',

  ageMs: '{n}ms 前',
  ageSec: '{n}s 前',
  ageMin: '{n}m 前',
  ageHour: '{n}h 前',
  ageDash: '—',

  uptimeDhms: '{d}天 {h}时 {m}分 {s}秒',
  uptimeHms: '{h}时 {m}分 {s}秒',
  uptimeMs: '{m}分 {s}秒',

  topologyTitle: '拓扑',
  topologySub: '{n} 节点 · {e} 边',
  topologyEmpty: '暂无已登记端点',
  topologyNode: '节点',
  topologyInput: '输入',
  topologyOutput: '输出',
  topologyNoPorts: '暂无已注册端口',
}

const catalogs: Record<Locale, Record<MessageKey, string>> = { en, zh }

function readStoredLocale(): Locale {
  if (typeof window === 'undefined') return DEFAULT_LOCALE
  try {
    const raw = window.localStorage.getItem(STORAGE_KEY)
    if (raw === 'en' || raw === 'zh') return raw
  } catch {
    /* ignore */
  }
  return DEFAULT_LOCALE
}

function interpolate(template: string, vars?: Record<string, string | number>): string {
  if (!vars) return template
  return template.replace(/\{(\w+)\}/g, (_, key: string) =>
    vars[key] !== undefined ? String(vars[key]) : `{${key}}`,
  )
}

type I18nValue = {
  locale: Locale
  setLocale: (locale: Locale) => void
  t: (key: MessageKey, vars?: Record<string, string | number>) => string
  dateLocale: string
}

const I18nContext = createContext<I18nValue | null>(null)

export function LocaleProvider({ children }: { children: ReactNode }) {
  const [locale, setLocaleState] = useState<Locale>(DEFAULT_LOCALE)
  const [hydrated, setHydrated] = useState(false)

  useEffect(() => {
    setLocaleState(readStoredLocale())
    setHydrated(true)
  }, [])

  useEffect(() => {
    if (!hydrated) return
    document.documentElement.lang = locale === 'zh' ? 'zh-CN' : 'en'
    try {
      window.localStorage.setItem(STORAGE_KEY, locale)
    } catch {
      /* ignore */
    }
  }, [locale, hydrated])

  const setLocale = useCallback((next: Locale) => {
    setLocaleState(next)
  }, [])

  const t = useCallback(
    (key: MessageKey, vars?: Record<string, string | number>) =>
      interpolate(catalogs[locale][key] ?? en[key], vars),
    [locale],
  )

  const value = useMemo<I18nValue>(
    () => ({
      locale,
      setLocale,
      t,
      dateLocale: locale === 'zh' ? 'zh-CN' : 'en-US',
    }),
    [locale, setLocale, t],
  )

  return <I18nContext.Provider value={value}>{children}</I18nContext.Provider>
}

export function useI18n(): I18nValue {
  const ctx = useContext(I18nContext)
  if (!ctx) throw new Error('useI18n must be used within LocaleProvider')
  return ctx
}

export function fmtAgeLocalized(
  ts: number,
  t: I18nValue['t'],
): string {
  if (!ts) return t('ageDash')
  const age = Date.now() - ts
  if (age < 1000) return t('ageMs', { n: age })
  if (age < 60000) return t('ageSec', { n: Math.floor(age / 1000) })
  if (age < 3600000) return t('ageMin', { n: Math.floor(age / 60000) })
  return t('ageHour', { n: Math.floor(age / 3600000) })
}

export function fmtUptimeLocalized(
  s: number,
  t: I18nValue['t'],
): string {
  const d = Math.floor(s / 86400)
  const h = Math.floor((s % 86400) / 3600)
  const m = Math.floor((s % 3600) / 60)
  const ss = s % 60
  if (d > 0) return t('uptimeDhms', { d, h, m, s: ss })
  if (h > 0) return t('uptimeHms', { h, m, s: ss })
  return t('uptimeMs', { m, s: ss })
}
