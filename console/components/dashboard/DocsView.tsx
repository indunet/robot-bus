'use client'

import { useMemo, useState, type ReactNode } from 'react'
import { BookOpen } from 'lucide-react'
import { PanelHeader } from './BrokerOverview'
import { useI18n, type Locale } from '@/lib/i18n'

type DocSection = {
  id: string
  title: string
  body: ReactNode
}

function Code({ children }: { children: string }) {
  return (
    <pre className="mt-2 mb-3 overflow-x-auto rounded border border-bus-border bg-[#121416] px-3 py-2 font-mono text-[11px] leading-relaxed text-bus-cyan/90 whitespace-pre">
      {children}
    </pre>
  )
}

function P({ children }: { children: ReactNode }) {
  return <p className="mb-2 font-mono text-[12px] leading-relaxed text-bus-text/85">{children}</p>
}

function Ul({ items }: { items: ReactNode[] }) {
  return (
    <ul className="mb-3 list-disc space-y-1 pl-5 font-mono text-[12px] leading-relaxed text-bus-text/85">
      {items.map((item, i) => (
        <li key={i}>{item}</li>
      ))}
    </ul>
  )
}

function H({ children }: { children: ReactNode }) {
  return (
    <h3 className="mb-2 mt-1 font-mono text-[11px] font-bold tracking-widest text-bus-cyan uppercase">
      {children}
    </h3>
  )
}

function buildSections(locale: Locale): DocSection[] {
  if (locale === 'zh') {
    return [
      {
        id: 'quickstart',
        title: '快速开始',
        body: (
          <>
            <P>先启动 broker，再用任意语言 SDK 连上总线即可收发消息。</P>
            <Code>{`cargo run --bin robot_bus_broker
# 默认 API / WebSocket / Console: http://127.0.0.1:15570`}</Code>
            <Ul
              items={[
                '本机控制台：打开上述地址，或 cd console && pnpm dev（代理 /api 到 broker）',
                'CLI 自省：cargo run --bin rbus -- topic list',
                '发现接口：GET /api/v1/discover',
              ]}
            />
          </>
        ),
      },
      {
        id: 'transports',
        title: '传输方式',
        body: (
          <>
            <P>日常高性能路径用 ZeroMQ；跨语言远程 / 浏览器用 WebSocket RPC。</P>
            <Ul
              items={[
                <>
                  <span className="text-bus-cyan">tcp / ipc / inproc</span> — 本地或同机进程；inproc
                  必须共享同一 Context
                </>,
                <>
                  <span className="text-bus-cyan">ws</span> — 远程唯一协议：多路复用 WebSocket（
                  <code className="text-bus-cyan">/ws</code>
                  ），一条连接多个 RPC
                </>,
              ]}
            />
            <Code>{`# Rust
Node::tcp("talker")
Node::ws("remote")                      // http://127.0.0.1:15570 → ws://…/ws
Node::ws_at("remote", "http://host:15570")`}</Code>
          </>
        ),
      },
      {
        id: 'pubsub',
        title: '话题 Pub / Sub',
        body: (
          <>
            <P>创建发布者 / 订阅者后 spin（或浏览器里 start）即可。</P>
            <Code>{`# Python
node = robot_bus.Node("pilot")
pub = node.create_publisher("/robot1/cmd")
node.create_subscription("/robot1/imu", on_imu)
node.spin()`}</Code>
            <Code>{`# TypeScript（浏览器入口 Node = WsNode）
const node = Node.ws("web")
const pub = node.createPublisher("/robot1/cmd")
await pub.publish(new TextEncoder().encode("go"))
node.createSubscription("/robot1/imu", (topic, payload) => { … })
node.start()`}</Code>
            <P>
              系统话题在 <code className="text-bus-cyan">/robot_bus/*</code>
              （status / topics / topology …），控制台会自动订阅。
            </P>
          </>
        ),
      },
      {
        id: 'service-action',
        title: 'Service / Action',
        body: (
          <>
            <H>Service</H>
            <P>请求–响应。Worker 连 backend，Client 连 frontend（或走 Node.ws）。</P>
            <Code>{`client = node.create_client("/add_two_ints")
resp = client.call(req_bytes, timeout=3.0)`}</Code>
            <H>Action</H>
            <P>
              长任务：send_goal 立刻返回 GoalHandle；feedback 走回调；result 单独等待。cancel
              为尽力而为。
            </P>
            <Code>{`handle = action.send_goal(goal, on_feedback=…)
handle.cancel()
result = handle.wait_result()`}</Code>
          </>
        ),
      },
      {
        id: 'console',
        title: '控制台怎么用',
        body: (
          <>
            <Ul
              items={[
                '概览 — broker 健康度与吞吐快照',
                '拓扑 — 已注册的 pub/sub 进程图',
                '话题 / 服务 / 动作 — 实时表与速率曲线',
                '事件 — broker 日志流',
                '坦克 — 打开进程内坦克仿真面板（键盘遥控 / 导航）',
              ]}
            />
            <P>
              拓扑与类型注册依赖 SDK 在 create_publisher / create_subscription 时上报；约 30s
              无刷新会过期。
            </P>
          </>
        ),
      },
      {
        id: 'tank',
        title: '坦克仿真',
        body: (
          <>
            <P>侧栏 TANK 会申请会话并拉起 broker 内物理节点：</P>
            <Ul
              items={[
                <>
                  <code className="text-bus-cyan">/robot_bus/tank/cmd_vel</code> — 速度指令
                </>,
                <>
                  <code className="text-bus-cyan">/robot_bus/tank/pose</code> — 位姿 20Hz
                </>,
                <>
                  <code className="text-bus-cyan">/robot_bus/tank/point_navigation</code> 等 —
                  导航 action
                </>,
                <>
                  <code className="text-bus-cyan">/robot_bus/tank/reset</code> — 复位到世界中心
                </>,
              ]}
            />
            <P>多人可同时观看；遥控后写覆盖（last-writer-wins）。</P>
            <P>
              生产环境可关闭：启动 broker 时加{' '}
              <code className="text-bus-cyan">--no-tank</code>
              （或绑定里的 <code className="text-bus-cyan">no_tank</code>
              ）。此时侧栏隐藏入口，<code className="text-bus-cyan">GET /api/v1/tank</code> 返回{' '}
              <code className="text-bus-cyan">enabled: false</code>，申请会话会被拒绝。
            </P>
          </>
        ),
      },
      {
        id: 'cli-api',
        title: 'CLI 与 HTTP',
        body: (
          <>
            <Code>{`cargo run --bin rbus -- topic list
cargo run --bin rbus -- service list
cargo run --bin rbus -- status

curl -s http://127.0.0.1:15570/api/v1/discover | jq .
curl -s http://127.0.0.1:15570/api/v1/tank`}</Code>
            <P>更完整的语言 API 见仓库 docs/ 目录（rust / python / typescript / cpp / java …）。</P>
          </>
        ),
      },
    ]
  }

  return [
    {
      id: 'quickstart',
      title: 'Quick start',
      body: (
        <>
          <P>Start the broker, then connect any language SDK.</P>
          <Code>{`cargo run --bin robot_bus_broker
# Default API / WebSocket / Console: http://127.0.0.1:15570`}</Code>
          <Ul
            items={[
              'Console UI: open that URL, or cd console && pnpm dev (proxies /api)',
              'Introspection CLI: cargo run --bin rbus -- topic list',
              'Discovery: GET /api/v1/discover',
            ]}
          />
        </>
      ),
    },
    {
      id: 'transports',
      title: 'Transports',
      body: (
        <>
          <P>Use ZeroMQ for local high throughput; WebSocket RPC for remote / browsers.</P>
          <Ul
            items={[
              <>
                <span className="text-bus-cyan">tcp / ipc / inproc</span> — same host / process;
                inproc requires a shared Context
              </>,
              <>
                <span className="text-bus-cyan">ws</span> — sole remote protocol: multiplexed
                WebSocket (<code className="text-bus-cyan">/ws</code>), one connection, many
                streams
              </>,
            ]}
          />
          <Code>{`# Rust
Node::tcp("talker")
Node::ws("remote")                      // http://127.0.0.1:15570 → ws://…/ws
Node::ws_at("remote", "http://host:15570")`}</Code>
        </>
      ),
    },
    {
      id: 'pubsub',
      title: 'Topics Pub / Sub',
      body: (
        <>
          <P>Create publishers / subscriptions, then spin (or start() in the browser).</P>
          <Code>{`# Python
node = robot_bus.Node("pilot")
pub = node.create_publisher("/robot1/cmd")
node.create_subscription("/robot1/imu", on_imu)
node.spin()`}</Code>
          <Code>{`# TypeScript (browser Node = WsNode)
const node = Node.ws("web")
const pub = node.createPublisher("/robot1/cmd")
await pub.publish(new TextEncoder().encode("go"))
node.createSubscription("/robot1/imu", (topic, payload) => { … })
node.start()`}</Code>
          <P>
            System topics live under <code className="text-bus-cyan">/robot_bus/*</code> (status /
            topics / topology …); the console subscribes automatically.
          </P>
        </>
      ),
    },
    {
      id: 'service-action',
      title: 'Service / Action',
      body: (
        <>
          <H>Service</H>
          <P>Request–response. Workers bind backend; clients use frontend (or Node.ws).</P>
          <Code>{`client = node.create_client("/add_two_ints")
resp = client.call(req_bytes, timeout=3.0)`}</Code>
          <H>Action</H>
          <P>
            Long-running goals: send_goal returns a GoalHandle immediately; feedback via callback;
            wait for result separately. cancel is best-effort.
          </P>
          <Code>{`handle = action.send_goal(goal, on_feedback=…)
handle.cancel()
result = handle.wait_result()`}</Code>
        </>
      ),
    },
    {
      id: 'console',
      title: 'Using this console',
      body: (
        <>
          <Ul
            items={[
              'Overview — broker health and throughput snapshot',
              'Topology — registered pub/sub process graph',
              'Topics / Services / Actions — live tables and rate charts',
              'Events — broker log stream',
              'Tank — open the in-process tank sim panel (teleop / nav)',
            ]}
          />
          <P>
            Topology and type registration happen when SDKs create publishers / subscriptions;
            endpoints expire after ~30s without refresh.
          </P>
        </>
      ),
    },
    {
      id: 'tank',
      title: 'Tank sim',
      body: (
        <>
          <P>Sidebar TANK acquires a session and starts the in-broker physics node:</P>
          <Ul
            items={[
              <>
                <code className="text-bus-cyan">/robot_bus/tank/cmd_vel</code> — velocity command
              </>,
              <>
                <code className="text-bus-cyan">/robot_bus/tank/pose</code> — pose at 20 Hz
              </>,
              <>
                <code className="text-bus-cyan">/robot_bus/tank/point_navigation</code> etc. — nav
                actions
              </>,
              <>
                <code className="text-bus-cyan">/robot_bus/tank/reset</code> — snap to world center
              </>,
            ]}
          />
          <P>Multiple viewers share one world; teleop is last-writer-wins.</P>
          <P>
            Disable in production with broker flag{' '}
            <code className="text-bus-cyan">--no-tank</code> (or binding{' '}
            <code className="text-bus-cyan">no_tank</code>). The sidebar hides the entry,{' '}
            <code className="text-bus-cyan">GET /api/v1/tank</code> returns{' '}
            <code className="text-bus-cyan">enabled: false</code>, and session acquire is rejected.
          </P>
        </>
      ),
    },
    {
      id: 'cli-api',
      title: 'CLI & HTTP',
      body: (
        <>
          <Code>{`cargo run --bin rbus -- topic list
cargo run --bin rbus -- service list
cargo run --bin rbus -- status

curl -s http://127.0.0.1:15570/api/v1/discover | jq .
curl -s http://127.0.0.1:15570/api/v1/tank`}</Code>
          <P>Full language APIs live under docs/ in the repo (rust / python / typescript / cpp / java …).</P>
        </>
      ),
    },
  ]
}

export default function DocsView() {
  const { t, locale } = useI18n()
  const sections = useMemo(() => buildSections(locale), [locale])
  const [active, setActive] = useState(sections[0]?.id ?? 'quickstart')

  const current = sections.find((s) => s.id === active) ?? sections[0]

  return (
    <div className="flex flex-1 min-h-0 gap-3">
      <aside className="w-44 shrink-0 flex flex-col rounded border border-bus-border bg-bus-panel overflow-hidden">
        <PanelHeader icon={<BookOpen size={14} />} title={t('docsTitle')} sub={t('docsSub')} />
        <nav className="flex-1 overflow-y-auto py-1">
          {sections.map((s) => (
            <button
              key={s.id}
              type="button"
              onClick={() => setActive(s.id)}
              className={`w-full text-left px-3 py-2 font-mono text-[11px] tracking-wide transition-colors ${
                active === s.id
                  ? 'bg-bus-cyan/15 text-bus-cyan border-l-2 border-bus-cyan'
                  : 'text-bus-muted hover:text-bus-text hover:bg-[#1f2428] border-l-2 border-transparent'
              }`}
            >
              {s.title}
            </button>
          ))}
        </nav>
      </aside>

      <section className="flex-1 min-w-0 flex flex-col rounded border border-bus-border bg-bus-panel overflow-hidden">
        <PanelHeader title={current?.title ?? t('docsTitle')} />
        <div className="flex-1 overflow-y-auto px-4 py-3">{current?.body}</div>
      </section>
    </div>
  )
}
