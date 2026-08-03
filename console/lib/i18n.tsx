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
  navRoutes: 'ROUTES',
  navRoutesShort: 'RTE',

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
  topologyHint:
    'Best-effort live pub/sub graph from client registration. Stale endpoints expire after ~30s.',
  topologyEmpty: 'No registered endpoints yet',

  routesTitle: 'ROS 2 BRIDGE',
  routesSub: '{r} topics · {s} services · {a} actions',
  routesHint:
    'ROS 2 and robot-bus are port cards. Click a port or wire to edit. Export YAML — does not hot-apply a running bridge.',
  routesImport: 'Import',
  routesPaste: 'Paste YAML',
  routesApplyPaste: 'Apply paste',
  routesExport: 'Export',
  routesCopy: 'Copy',
  routesExample: 'Load example',
  routesClear: 'Clear',
  routesAddTopic: 'Topic',
  routesAddService: 'Service',
  routesAddAction: 'Action',
  routesRemove: 'Delete',
  routesTopics: 'Topics',
  routesServices: 'Services',
  routesActions: 'Actions',
  routesInspector: 'Inspector',
  routesSelectHint: 'Click a port or wire to edit.',
  routesImported: 'YAML imported',
  routesExported: 'YAML downloaded',
  routesCopied: 'Copied to clipboard',
  routesCopyFailed: 'Clipboard copy failed',
  routesExampleLoaded: 'Example loaded',
  routesToggleYaml: 'YAML',
  routesBroker: 'Broker',
  routesRosSide: 'ROS graph side',
  routesEditTopic: 'Edit topic route',
  routesEditService: 'Edit service route',
  routesEditAction: 'Edit action route',
  routesFieldTransport: 'Transport',
  routesFieldHost: 'Host',
  routesFieldRosTopic: 'ROS topic',
  routesFieldBusTopic: 'Bus topic',
  routesFieldRosService: 'ROS service',
  routesFieldBusService: 'Bus service',
  routesFieldRosAction: 'ROS action',
  routesFieldBusAction: 'Bus action',
  routesFieldType: 'Message type',
  routesFieldDirection: 'Direction',

  colType: 'TYPE',
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
  navRoutes: '路由',
  navRoutesShort: '路由',

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
  topologyHint: '基于客户端登记的实时 pub/sub 图（尽力而为）。约 30s 无刷新的端点会过期。',
  topologyEmpty: '暂无已登记端点',

  routesTitle: 'ROS 2 桥接',
  routesSub: '{r} 话题 · {s} 服务 · {a} 动作',
  routesHint: '左侧 ROS、右侧 robot-bus，端口连线表示桥接。点端口或连线即可编辑。导出 YAML，不会热更新运行中的桥。',
  routesImport: '导入',
  routesPaste: '粘贴 YAML',
  routesApplyPaste: '应用粘贴',
  routesExport: '导出',
  routesCopy: '复制',
  routesExample: '加载示例',
  routesClear: '清空',
  routesAddTopic: '话题',
  routesAddService: '服务',
  routesAddAction: '动作',
  routesRemove: '删除',
  routesTopics: '话题',
  routesServices: '服务',
  routesActions: '动作',
  routesInspector: '属性',
  routesSelectHint: '点击端口或连线进行编辑。',
  routesImported: '已导入 YAML',
  routesExported: '已下载 YAML',
  routesCopied: '已复制到剪贴板',
  routesCopyFailed: '复制失败',
  routesExampleLoaded: '已加载示例',
  routesToggleYaml: 'YAML',
  routesBroker: 'Broker',
  routesRosSide: 'ROS 侧',
  routesEditTopic: '编辑话题路由',
  routesEditService: '编辑服务路由',
  routesEditAction: '编辑动作路由',
  routesFieldTransport: '传输方式',
  routesFieldHost: '主机',
  routesFieldRosTopic: 'ROS 话题',
  routesFieldBusTopic: 'Bus 话题',
  routesFieldRosService: 'ROS 服务',
  routesFieldBusService: 'Bus 服务',
  routesFieldRosAction: 'ROS 动作',
  routesFieldBusAction: 'Bus 动作',
  routesFieldType: '消息类型',
  routesFieldDirection: '方向',

  colType: '类型',
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
