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
  navTank: 'TANK',
  navTankShort: 'TANK',
  navTankWindows: 'OPEN TANK',
  navDocs: 'DOCS',
  navDocsShort: 'DOC',
  docsTitle: 'DOCS',
  docsSub: 'how to use',
  tankWindowTitle: 'TANK · tank_viz',
  tankTeleopWindowTitle: 'TANK · tank_viz',
  tankPoseTitle: 'POSE',
  tankTeleopTitle: 'CONTROL',
  tankCapabilitiesTitle: 'CAPABILITIES',
  tankCapKeyboard: 'Keyboard teleop',
  tankCapPointNav: 'Point navigate',
  tankCapPointNavHelp: 'Click to place goal, drag to set heading, then Execute',
  tankCapMultiNav: 'Multi-waypoint',
  tankCapMultiNavHelp: 'Click waypoints; drag the last one for heading, then Execute',
  tankCapReset: 'Reset',
  tankCapSoon: 'soon',
  tankNavGo: 'Execute',
  tankNavClear: 'Clear',
  tankNavUndo: 'Undo',
  tankNavCancel: 'Cancel',
  tankNavIdle: 'Idle',
  tankNavRunning: 'Navigating… {p}%',
  tankNavDone: 'Arrived',
  tankNavFailed: 'Nav failed',
  tankNavWaypoints: '{n} waypoints',
  tankNavProgress: 'Progress',
  tankNavTheta: 'θ',
  tankNavDragHint: 'Drag to aim heading',
  tankReset: 'Reset',
  tankResetHint: 'Snap tank back to world center',
  tankResetting: 'Resetting…',
  tankConnecting: 'CONNECTING',
  tankWaitingPose: 'WAITING FOR POSE',
  tankPublishError: 'PUBLISH ERROR',
  tankBusLabel: 'BUS',
  tankLabel: 'TANK',
  tankBusOk: 'OK',
  tankBusError: 'ERROR',
  tankOnline: 'ONLINE',
  tankOffline: 'OFFLINE',
  tankHint: 'Starting tank…',
  tankStarting: 'Acquiring sim session…',
  tankBusHint: 'Broker unreachable — start robot_bus_broker',
  tankSharedHint: 'Shared world — teleop is last-writer-wins',
  tankViewers: '{n} viewers',
  tankArrowHelp: 'Click the panel, then use the arrow keys',
  tankPublish: 'PUB',
  tankSubscribe: 'SUB',
  tankSrv: 'SRV',
  tankAct: 'ACT',
  tankViewerHint: 'Viz/ops panel — physics runs in broker tank',

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
  brokerWs: 'WS RPC',
  brokerMsgXSub: 'MSG XSUB',
  brokerMsgXPub: 'MSG XPUB',
  brokerSvcFE: 'SVC  FE',
  brokerSvcBE: 'SVC  BE',
  brokerActFE: 'ACT  FE',
  brokerActBE: 'ACT  BE',
  listen: 'OK',
  down: 'DOWN',
  connecting: 'CONNECTING',
  loading: 'LOADING',
  statusOnline: 'ONLINE',
  statusDegraded: 'DEGRADED',
  statusOffline: 'OFFLINE',

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

  nameFilterPlaceholder: 'Filter by name',
  nameFilterAria: 'Filter by name',
  nameFilterClear: 'Clear filter',
  nameFilterEmpty: 'No matches',
  nameFilterHits: '{n}/{total}',

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
  filterDebug: 'DEBUG',
  filterInfo: 'INFO',
  filterWarn: 'WARN',
  filterError: 'ERROR',
  chartWindow: '{n}m',
  chartWindowAria: 'Chart window',
  unitMsgS: 'msg/s',
  unitCallsS: 'calls/s',
  unitRunsS: 'runs/s',

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
  uptimeShortDh: '{d}d {h}h',
  uptimeShortHm: '{h}h {m}m',
  uptimeShortMs: '{m}m {s}s',

  topologyTitle: 'TOPOLOGY',
  topologySub: '{n} nodes · {e} edges',
  topologyEmpty: 'No registered endpoints yet',
  topologyNode: 'NODE',
  topologyInput: 'SUBSCRIBE',
  topologyOutput: 'PUBLISH',
  topologyNoPorts: 'No ports registered',
  topologyService: 'SERVICE',
  topologyAction: 'ACTION',
  topologySrv: 'SRV',
  topologyCli: 'CLI',
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
  navTank: '坦克',
  navTankShort: '坦克',
  navTankWindows: '打开坦克',
  navDocs: '文档',
  navDocsShort: '文档',
  docsTitle: '文档',
  docsSub: '使用说明',
  tankWindowTitle: '坦克 · tank_viz',
  tankTeleopWindowTitle: '坦克 · tank_viz',
  tankPoseTitle: '位姿',
  tankTeleopTitle: '控制',
  tankCapabilitiesTitle: '能力调度',
  tankCapKeyboard: '键盘遥控',
  tankCapPointNav: '单点导航',
  tankCapPointNavHelp: '点击放置目标，拖拽设朝向，再点执行',
  tankCapMultiNav: '多途经点',
  tankCapMultiNavHelp: '点击添加途经点；拖拽最后一点设朝向，再点执行',
  tankCapReset: '复位',
  tankCapSoon: '待定',
  tankNavGo: '执行',
  tankNavClear: '清空',
  tankNavUndo: '撤销',
  tankNavCancel: '取消',
  tankNavIdle: '空闲',
  tankNavRunning: '导航中… {p}%',
  tankNavDone: '已到达',
  tankNavFailed: '导航失败',
  tankNavWaypoints: '{n} 个途经点',
  tankNavProgress: '进度',
  tankNavTheta: 'θ',
  tankNavDragHint: '拖拽设置朝向',
  tankReset: '复位',
  tankResetHint: '将坦克拉回世界中心',
  tankResetting: '复位中…',
  tankConnecting: '连接中',
  tankWaitingPose: '等待位姿',
  tankPublishError: '发布错误',
  tankBusLabel: '总线',
  tankLabel: '坦克',
  tankBusOk: '正常',
  tankBusError: '异常',
  tankOnline: '在线',
  tankOffline: '离线',
  tankHint: '正在启动 tank…',
  tankStarting: '正在申请仿真会话…',
  tankBusHint: 'Broker 不可达 — 请启动 robot_bus_broker',
  tankSharedHint: '共享世界 — 遥控后写覆盖',
  tankViewers: '{n} 人在看',
  tankArrowHelp: '点击面板，然后使用方向键',
  tankPublish: '发布',
  tankSubscribe: '订阅',
  tankSrv: '服务',
  tankAct: '动作',
  tankViewerHint: '可视化/操作面板 — 物理由 broker 内 tank 运行',

  metricMsgS: '消息/秒',
  metricBw: '带宽',
  metricSvcS: '服务/秒',
  metricActS: '动作/秒',
  metricTotal: '总计',
  metricUptime: '运行',
  metricPid: 'PID',
  metricErr: '错误',

  brokerTitle: '代理',
  brokerSub: '端点健康',
  brokerWs: 'WebSocket',
  brokerMsgXSub: '发布接入',
  brokerMsgXPub: '订阅接入',
  brokerSvcFE: '服务前端',
  brokerSvcBE: '服务后端',
  brokerActFE: '动作前端',
  brokerActBE: '动作后端',
  listen: '正常',
  down: '断开',
  connecting: '连接中',
  loading: '加载中',
  statusOnline: '在线',
  statusDegraded: '降级',
  statusOffline: '离线',

  statsTitle: '快照',
  statsSub: '实时计数',
  statMsgS: '消息/秒',
  statBw: '带宽',
  statTopics: '话题数',
  statTopicsActive: '{n} 活跃',
  statServices: '服务数',
  statWorkers: '{n} 个工作端',
  statSvcS: '调用/秒',
  statActions: '动作数',
  statActiveGoals: '{n} 进行中',
  statActS: '运行/秒',
  statTotalMsgs: '消息总计',
  statUptime: '运行时间',

  throughputTitle: '话题吞吐',
  throughputSub: '近 {n} 分钟 消息/秒',
  throughputSeries: '消息/秒',
  svcRateTitle: '服务调用',
  svcRateSub: '近 {n} 分钟 调用/秒',
  svcRateSeries: '调用/秒',
  actRateTitle: '动作运行',
  actRateSub: '近 {n} 分钟 运行/秒',
  actRateSeries: '运行/秒',

  nameFilterPlaceholder: '按名称过滤',
  nameFilterAria: '按名称过滤',
  nameFilterClear: '清除过滤',
  nameFilterEmpty: '无匹配',
  nameFilterHits: '{n}/{total}',

  topicsTitle: '话题',
  topicsActive: '{n} 活跃',
  colTopic: '话题',
  colType: '类型',
  colMsgS: '消息/秒',
  colBandwidth: '带宽',
  colTotal: '总计',
  colPub: '发布',
  colSub: '订阅',
  colLastSeen: '最近',

  servicesTitle: '服务',
  servicesKnown: '{n} 已知',
  servicesEmpty: '暂无服务',
  colService: '服务',
  colCalls: '调用',
  colCallsS: '调用/秒',
  colWorkers: '工作端',
  colErr: '错误',
  colP99: '平均ms',
  colLast: '最近',

  actionsTitle: '动作',
  actionsKnown: '{n} 已知',
  actionsEmpty: '暂无动作',
  colAction: '动作',
  colRuns: '运行',
  colRunsS: '运行/秒',
  colActive: '进行中',
  colAvgDur: '平均ms',

  eventsTitle: '事件',
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
  filterDebug: '调试',
  filterInfo: '信息',
  filterWarn: '警告',
  filterError: '错误',
  chartWindow: '{n}分',
  chartWindowAria: '图表时间窗',
  unitMsgS: '消息/秒',
  unitCallsS: '调用/秒',
  unitRunsS: '运行/秒',

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
  uptimeShortDh: '{d}天 {h}时',
  uptimeShortHm: '{h}时 {m}分',
  uptimeShortMs: '{m}分 {s}秒',

  topologyTitle: '拓扑',
  topologySub: '{n} 节点 · {e} 边',
  topologyEmpty: '暂无已登记端点',
  topologyNode: '节点',
  topologyInput: '订阅',
  topologyOutput: '发布',
  topologyNoPorts: '暂无已注册端口',
  topologyService: '服务',
  topologyAction: '动作',
  topologySrv: '服务端',
  topologyCli: '客户端',
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

export type I18nValue = {
  locale: Locale
  setLocale: (locale: Locale) => void
  t: (key: MessageKey, vars?: Record<string, string | number>) => string
  dateLocale: string
  /** English labels are stored uppercase; skip CSS uppercase in Chinese. */
  labelCase: string
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
      labelCase: locale === 'en' ? 'uppercase' : '',
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
