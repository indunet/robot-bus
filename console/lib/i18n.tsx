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
  navBotWindows: 'OPEN BOT SIM',
  botSimWindowTitle: 'BOT SIM · bot_viz',
  botTeleopWindowTitle: 'BOT SIM · bot_viz',
  botPoseTitle: 'POSE',
  botTeleopTitle: 'CONTROL',
  botCapabilitiesTitle: 'CAPABILITIES',
  botCapKeyboard: 'Keyboard teleop',
  botCapPointNav: 'Point navigate',
  botCapPointNavHelp: 'Click to place goal, drag to set heading, then Execute',
  botCapMultiNav: 'Multi-waypoint',
  botCapMultiNavHelp: 'Click waypoints; drag the last one for heading, then Execute',
  botCapSoon: 'soon',
  botNavGo: 'Execute',
  botNavClear: 'Clear',
  botNavUndo: 'Undo',
  botNavCancel: 'Cancel',
  botNavIdle: 'Idle',
  botNavRunning: 'Navigating… {p}%',
  botNavDone: 'Arrived',
  botNavFailed: 'Nav failed',
  botNavWaypoints: '{n} waypoints',
  botNavProgress: 'Progress',
  botNavTheta: 'θ',
  botNavDragHint: 'Drag to aim heading',
  botReset: 'Reset',
  botResetHint: 'Snap robot back to world center',
  botResetting: 'Resetting…',
  botConnecting: 'CONNECTING',
  botOnline: 'ONLINE',
  botWaitingPose: 'WAITING FOR POSE',
  botPublishError: 'PUBLISH ERROR',
  botBusLabel: 'BUS',
  botSimLabel: 'SIM',
  botBusOk: 'OK',
  botBusError: 'ERROR',
  botSimOnline: 'ONLINE',
  botSimOffline: 'OFFLINE',
  botSimHint: 'Starting bot_sim…',
  botSimStarting: 'Acquiring sim session…',
  botBusHint: 'Broker unreachable — start robot_bus_broker',
  botSharedHint: 'Shared world — teleop is last-writer-wins',
  botViewers: '{n} viewers',
  botArrowHelp: 'Click the panel, then use Arrow / WASD keys',
  botPublish: 'PUB',
  botSubscribe: 'SUB',
  botViewerHint: 'Viz/ops panel — physics runs in broker bot_sim',

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
  connecting: 'CONNECTING',
  loading: 'LOADING',

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
  throughputSub: 'Last {n}m msg/s',
  throughputSeries: 'MSG/S',
  svcRateTitle: 'SERVICE CALLS',
  svcRateSub: 'Last {n}m calls/s',
  svcRateSeries: 'CALLS/S',
  actRateTitle: 'ACTION RUNS',
  actRateSub: 'Last {n}m runs/s',
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
  colTime: 'TIME',
  colLevel: 'LEVEL',
  colSource: 'SOURCE',
  colMessage: 'MESSAGE',
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
  navBotSim: '机器人仿真',
  navBotSimShort: 'BOT',
  navBotWindows: '打开机器人仿真',
  botSimWindowTitle: '机器人仿真 · bot_viz',
  botTeleopWindowTitle: '机器人仿真 · bot_viz',
  botPoseTitle: '位姿',
  botTeleopTitle: '控制',
  botCapabilitiesTitle: '能力调度',
  botCapKeyboard: '键盘遥控',
  botCapPointNav: '单点导航',
  botCapPointNavHelp: '点击放置目标，拖拽设朝向，再点执行',
  botCapMultiNav: '多途经点',
  botCapMultiNavHelp: '点击添加途经点；拖拽最后一点设朝向，再点执行',
  botCapSoon: '待定',
  botNavGo: '执行',
  botNavClear: '清空',
  botNavUndo: '撤销',
  botNavCancel: '取消',
  botNavIdle: '空闲',
  botNavRunning: '导航中… {p}%',
  botNavDone: '已到达',
  botNavFailed: '导航失败',
  botNavWaypoints: '{n} 个途经点',
  botNavProgress: '进度',
  botNavTheta: 'θ',
  botNavDragHint: '拖拽设置朝向',
  botReset: '复位',
  botResetHint: '将机器人拉回世界中心',
  botResetting: '复位中…',
  botConnecting: '连接中',
  botOnline: '在线',
  botWaitingPose: '等待位姿',
  botPublishError: '发布错误',
  botBusLabel: '总线',
  botSimLabel: '仿真',
  botBusOk: '正常',
  botBusError: '异常',
  botSimOnline: '在线',
  botSimOffline: '离线',
  botSimHint: '正在启动 bot_sim…',
  botSimStarting: '正在申请仿真会话…',
  botBusHint: 'Broker 不可达 — 请启动 robot_bus_broker',
  botSharedHint: '共享世界 — 遥控后写覆盖',
  botViewers: '{n} 人在看',
  botArrowHelp: '点击面板，然后使用方向键 / WASD',
  botPublish: '发布',
  botSubscribe: '订阅',
  botViewerHint: '可视化/操作面板 — 物理由 broker 内 bot_sim 运行',

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
  connecting: '连接中',
  loading: '加载中',

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
  throughputSub: '近 {n} 分钟 msg/s',
  throughputSeries: 'MSG/S',
  svcRateTitle: 'SERVICE CALLS',
  svcRateSub: '近 {n} 分钟 calls/s',
  svcRateSeries: 'CALLS/S',
  actRateTitle: 'ACTION RUNS',
  actRateSub: '近 {n} 分钟 runs/s',
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
  colTime: '时间',
  colLevel: '级别',
  colSource: '来源',
  colMessage: '内容',
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
