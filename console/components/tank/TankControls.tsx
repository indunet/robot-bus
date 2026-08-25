'use client'

import type { ButtonHTMLAttributes, ReactNode } from 'react'
import { useI18n } from '@/lib/i18n'
import {
  KEY_BUTTONS,
  type Capability,
  type Direction,
  type NavPhase,
  type Pose,
} from './model'

interface Props {
  compact: boolean
  capability: Capability
  busOk: boolean | null
  simOnline: boolean
  sessionReady: boolean
  viewers: number
  pressed: Set<Direction>
  velocity: { linear: number; angular: number }
  pointGoal: Pose | null
  waypoints: Pose[]
  navPhase: NavPhase
  navProgress: number
  navDetail: string
  resetting: boolean
  onSelectCapability: (id: Capability) => void
  onPress: (direction: Direction) => void
  onRelease: (direction: Direction) => void
  onRunPointNav: () => void
  onRunMultiNav: () => void
  onUndoWaypoint: () => void
  onClearWaypoints: () => void
  onCancelNav: () => void
  onReset: () => void
}

export default function TankControls({
  compact,
  capability,
  busOk,
  simOnline,
  sessionReady,
  viewers,
  pressed,
  velocity,
  pointGoal,
  waypoints,
  navPhase,
  navProgress,
  navDetail,
  resetting,
  onSelectCapability,
  onPress,
  onRelease,
  onRunPointNav,
  onRunMultiNav,
  onUndoWaypoint,
  onClearWaypoints,
  onCancelNav,
  onReset,
}: Props) {
  const { t } = useI18n()
  const busLabel =
    busOk === null ? t('tankConnecting') : busOk ? t('tankBusOk') : t('tankBusError')
  const simLabel = simOnline
    ? t('tankOnline')
    : sessionReady
      ? t('tankHint')
      : t('tankStarting')
  const hint =
    busOk === false
      ? t('tankBusHint')
      : !simOnline
        ? sessionReady
          ? t('tankHint')
          : t('tankStarting')
        : t('tankSharedHint')
  const navStatusLabel =
    navPhase === 'running'
      ? t('tankNavRunning', { p: navProgress })
      : navPhase === 'done'
        ? t('tankNavDone')
        : navPhase === 'failed'
          ? navDetail || t('tankNavFailed')
          : t('tankNavIdle')

  const capabilities: { id: Capability; label: string }[] = [
    { id: 'keyboard', label: t('tankCapKeyboard') },
    { id: 'point_nav', label: t('tankCapPointNav') },
    { id: 'multi_waypoint', label: t('tankCapMultiNav') },
    { id: 'reset', label: t('tankCapReset') },
  ]

  return (
    <aside
      className={`${compact ? 'bg-bus-bg/25 p-2' : 'bg-bus-panel p-4'} border border-bus-border rounded-sm font-mono shadow-[0_1px_0_rgb(255_255_255_/06%)_inset] flex flex-col min-h-0 overflow-auto`}
    >
      <div className="mb-3">
        <div className="grid grid-cols-1 gap-1.5 text-[9px]">
          <StatusChip
            label={t('tankBusLabel')}
            value={busLabel}
            tone={busOk === null ? 'cyan' : busOk ? 'green' : 'red'}
          />
          <StatusChip
            label={t('tankLabel')}
            value={simLabel}
            tone={simOnline ? 'green' : sessionReady ? 'cyan' : 'amber'}
          />
        </div>
        <p
          className={`mt-1.5 text-[9px] leading-4 ${
            !simOnline || busOk === false ? 'text-bus-amber' : 'text-bus-muted'
          }`}
        >
          {hint}
          {viewers > 0 ? ` · ${t('tankViewers', { n: viewers })}` : ''}
        </p>
      </div>

      <ul className="flex flex-col gap-1.5">
        {capabilities.map((item) => {
          const active = capability === item.id
          return (
            <CapabilityBlock
              key={item.id}
              label={item.label}
              active={active}
              onSelect={() => onSelectCapability(item.id)}
            >
              {active && item.id === 'keyboard' && (
                <KeyboardPad
                  help={t('tankArrowHelp')}
                  pressed={pressed}
                  velocity={velocity}
                  onPress={onPress}
                  onRelease={onRelease}
                />
              )}
              {active && item.id === 'point_nav' && (
                <PointNavPad
                  help={t('tankCapPointNavHelp')}
                  goLabel={t('tankNavGo')}
                  cancelLabel={t('tankNavCancel')}
                  progressLabel={t('tankNavProgress')}
                  thetaLabel={t('tankNavTheta')}
                  pointGoal={pointGoal}
                  navPhase={navPhase}
                  navProgress={navProgress}
                  navStatusLabel={navStatusLabel}
                  onRun={onRunPointNav}
                  onCancel={onCancelNav}
                />
              )}
              {active && item.id === 'multi_waypoint' && (
                <MultiNavPad
                  help={t('tankCapMultiNavHelp')}
                  waypointsLabel={t('tankNavWaypoints', { n: waypoints.length })}
                  goLabel={t('tankNavGo')}
                  undoLabel={t('tankNavUndo')}
                  clearLabel={t('tankNavClear')}
                  cancelLabel={t('tankNavCancel')}
                  progressLabel={t('tankNavProgress')}
                  thetaLabel={t('tankNavTheta')}
                  waypoints={waypoints}
                  navPhase={navPhase}
                  navProgress={navProgress}
                  navStatusLabel={navStatusLabel}
                  onRun={onRunMultiNav}
                  onUndo={onUndoWaypoint}
                  onClear={onClearWaypoints}
                  onCancel={onCancelNav}
                />
              )}
              {active && item.id === 'reset' && (
                <ResetPad
                  hint={t('tankResetHint')}
                  label={resetting ? t('tankResetting') : t('tankReset')}
                  disabled={resetting || !sessionReady}
                  onReset={onReset}
                />
              )}
            </CapabilityBlock>
          )
        })}
      </ul>
    </aside>
  )
}

function StatusChip({
  label,
  value,
  tone,
}: {
  label: string
  value: string
  tone: 'cyan' | 'green' | 'red' | 'amber'
}) {
  const color =
    tone === 'green'
      ? 'text-bus-green'
      : tone === 'red'
        ? 'text-bus-red'
        : tone === 'amber'
          ? 'text-bus-amber'
          : 'text-bus-cyan'
  return (
    <div className="flex items-center justify-between gap-1 border border-bus-border rounded-sm px-1.5 py-1">
      <span className="text-bus-muted">{label}</span>
      <span className={color}>{value}</span>
    </div>
  )
}

function CapabilityBlock({
  label,
  active,
  onSelect,
  children,
}: {
  label: string
  active: boolean
  onSelect: () => void
  children?: ReactNode
}) {
  return (
    <li
      className={
        active
          ? 'overflow-hidden rounded-sm border border-bus-cyan/60 bg-[#101417] shadow-[0_0_12px_rgb(0_212_255_/08%),inset_0_1px_0_rgb(255_255_255_/05%)]'
          : undefined
      }
    >
      <button
        type="button"
        onClick={onSelect}
        className={`w-full text-center px-2 py-1.5 transition-colors ${
          active
            ? 'border-0 bg-bus-cyan/18 text-bus-cyan'
            : 'rounded-sm border border-bus-border bg-[#1f2428] hover:border-bus-cyan-dim text-bus-text'
        }`}
      >
        <span className="text-[11px]">{label}</span>
      </button>
      {active && children ? (
        <div className="border-t border-bus-cyan/20 bg-black/35 px-2.5 py-2.5">
          {children}
        </div>
      ) : null}
    </li>
  )
}

function KeyboardPad({
  help,
  pressed,
  velocity,
  onPress,
  onRelease,
}: {
  help: string
  pressed: Set<Direction>
  velocity: { linear: number; angular: number }
  onPress: (direction: Direction) => void
  onRelease: (direction: Direction) => void
}) {
  return (
    <div>
      <p className="text-[10px] leading-5 text-bus-muted mb-2">{help}</p>
      <div className="grid grid-cols-3 gap-2 mx-auto max-w-[200px] mb-3">
        {KEY_BUTTONS.map(({ direction, label, className }) => (
          <button
            type="button"
            key={direction}
            onPointerDown={(event) => {
              event.currentTarget.setPointerCapture(event.pointerId)
              onPress(direction)
            }}
            onPointerUp={() => onRelease(direction)}
            onPointerCancel={() => onRelease(direction)}
            className={`${className} h-10 border rounded-sm text-xs transition-all ${
              pressed.has(direction)
                ? 'border-bus-cyan text-bus-cyan bg-bus-cyan/20 translate-y-px shadow-inner'
                : 'border-bus-border bg-[#1f2428] hover:border-bus-cyan-dim hover:text-bus-cyan'
            }`}
          >
            {label}
          </button>
        ))}
      </div>
      <dl className="grid grid-cols-2 gap-y-1 text-[11px]">
        <dt className="text-bus-muted">linear.x</dt>
        <dd className="text-right">{velocity.linear.toFixed(2)}</dd>
        <dt className="text-bus-muted">angular.z</dt>
        <dd className="text-right">{velocity.angular.toFixed(2)}</dd>
      </dl>
    </div>
  )
}

function PointNavPad({
  help,
  goLabel,
  cancelLabel,
  progressLabel,
  thetaLabel,
  pointGoal,
  navPhase,
  navProgress,
  navStatusLabel,
  onRun,
  onCancel,
}: {
  help: string
  goLabel: string
  cancelLabel: string
  progressLabel: string
  thetaLabel: string
  pointGoal: Pose | null
  navPhase: NavPhase
  navProgress: number
  navStatusLabel: string
  onRun: () => void
  onCancel: () => void
}) {
  return (
    <div className="text-[10px] leading-5">
      <p className="text-bus-muted mb-2">{help}</p>
      {pointGoal && (
        <GoalStats
          x={pointGoal.x}
          y={pointGoal.y}
          theta={pointGoal.theta}
          thetaLabel={thetaLabel}
          xLabel="goal.x"
          yLabel="goal.y"
        />
      )}
      <ActionButton
        tone="go"
        disabled={!pointGoal || navPhase === 'running'}
        onClick={onRun}
        className="w-full mb-2"
      >
        {goLabel}
      </ActionButton>
      <NavProgress
        phase={navPhase}
        progress={navProgress}
        label={navStatusLabel}
        progressLabel={progressLabel}
      />
      {navPhase === 'running' && (
        <ActionButton tone="cancel" onClick={onCancel} className="mt-2 w-full">
          {cancelLabel}
        </ActionButton>
      )}
    </div>
  )
}

function MultiNavPad({
  help,
  waypointsLabel,
  goLabel,
  undoLabel,
  clearLabel,
  cancelLabel,
  progressLabel,
  thetaLabel,
  waypoints,
  navPhase,
  navProgress,
  navStatusLabel,
  onRun,
  onUndo,
  onClear,
  onCancel,
}: {
  help: string
  waypointsLabel: string
  goLabel: string
  undoLabel: string
  clearLabel: string
  cancelLabel: string
  progressLabel: string
  thetaLabel: string
  waypoints: Pose[]
  navPhase: NavPhase
  navProgress: number
  navStatusLabel: string
  onRun: () => void
  onUndo: () => void
  onClear: () => void
  onCancel: () => void
}) {
  return (
    <div className="text-[10px] leading-5">
      <p className="text-bus-muted mb-2">{help}</p>
      <p className="text-[11px] mb-2">{waypointsLabel}</p>
      {waypoints.length > 0 && (
        <GoalStats
          x={waypoints[waypoints.length - 1].x}
          y={waypoints[waypoints.length - 1].y}
          theta={waypoints[waypoints.length - 1].theta}
          thetaLabel={thetaLabel}
          xLabel="last.x"
          yLabel="last.y"
        />
      )}
      <div className="grid grid-cols-3 gap-1.5 mb-2">
        <ActionButton
          tone="go"
          disabled={waypoints.length === 0 || navPhase === 'running'}
          onClick={onRun}
        >
          {goLabel}
        </ActionButton>
        <ActionButton
          tone="undo"
          disabled={waypoints.length === 0 || navPhase === 'running'}
          onClick={onUndo}
        >
          {undoLabel}
        </ActionButton>
        <ActionButton
          tone="clear"
          disabled={waypoints.length === 0 || navPhase === 'running'}
          onClick={onClear}
        >
          {clearLabel}
        </ActionButton>
      </div>
      <NavProgress
        phase={navPhase}
        progress={navProgress}
        label={navStatusLabel}
        progressLabel={progressLabel}
      />
      {navPhase === 'running' && (
        <ActionButton tone="cancel" onClick={onCancel} className="mt-2 w-full">
          {cancelLabel}
        </ActionButton>
      )}
    </div>
  )
}

function ResetPad({
  hint,
  label,
  disabled,
  onReset,
}: {
  hint: string
  label: string
  disabled: boolean
  onReset: () => void
}) {
  return (
    <div className="text-[10px] leading-5">
      <p className="text-bus-muted mb-2">{hint}</p>
      <ActionButton tone="reset" disabled={disabled} title={hint} onClick={onReset} className="w-full">
        {label}
      </ActionButton>
    </div>
  )
}

function GoalStats({
  x,
  y,
  theta,
  xLabel,
  yLabel,
  thetaLabel,
}: {
  x: number
  y: number
  theta: number
  xLabel: string
  yLabel: string
  thetaLabel: string
}) {
  return (
    <dl className="grid grid-cols-2 gap-y-1 text-[11px] mb-2">
      <dt className="text-bus-muted">{xLabel}</dt>
      <dd className="text-right">{x.toFixed(2)}</dd>
      <dt className="text-bus-muted">{yLabel}</dt>
      <dd className="text-right">{y.toFixed(2)}</dd>
      <dt className="text-bus-muted">{thetaLabel}</dt>
      <dd className="text-right">{theta.toFixed(2)}</dd>
    </dl>
  )
}

type ActionTone = 'go' | 'undo' | 'clear' | 'cancel' | 'reset'

const ACTION_TONES: Record<ActionTone, string> = {
  go: 'border-bus-cyan bg-bus-cyan/20 text-bus-cyan hover:bg-bus-cyan/30',
  undo: 'border-bus-border2 bg-[#1f2428] text-bus-text hover:border-bus-cyan-dim hover:text-bus-cyan',
  clear: 'border-bus-amber/55 bg-bus-amber/10 text-bus-amber hover:bg-bus-amber/20',
  cancel: 'border-bus-red/70 bg-bus-red/10 text-bus-red hover:bg-bus-red/20',
  reset: 'border-bus-amber bg-bus-amber/15 text-bus-amber hover:bg-bus-amber/25',
}

function ActionButton({
  tone,
  className = '',
  ...props
}: ButtonHTMLAttributes<HTMLButtonElement> & { tone: ActionTone }) {
  return (
    <button
      type="button"
      className={`h-8 border rounded-sm text-[11px] transition-colors disabled:opacity-40 disabled:pointer-events-none ${ACTION_TONES[tone]} ${className}`}
      {...props}
    />
  )
}

function NavProgress({
  phase,
  progress,
  label,
  progressLabel,
}: {
  phase: NavPhase
  progress: number
  label: string
  progressLabel: string
}) {
  const tone =
    phase === 'failed' ? 'text-bus-red' : phase === 'done' ? 'text-bus-green' : 'text-bus-cyan'
  const bar = phase === 'failed' ? 'bg-bus-red' : phase === 'done' ? 'bg-bus-green' : 'bg-bus-cyan'
  return (
    <div className="space-y-1.5">
      <div className="flex items-center justify-between gap-2">
        <span className={`text-[11px] ${tone}`}>{label}</span>
        {(phase === 'running' || phase === 'done') && (
          <span className="text-[10px] text-bus-muted tabular-nums">
            {progressLabel} {progress}%
          </span>
        )}
      </div>
      <div className="h-1.5 rounded-full bg-[#1a1f24] border border-bus-border overflow-hidden">
        <div
          className={`h-full rounded-full transition-[width] duration-150 ${bar}`}
          style={{ width: `${phase === 'idle' ? 0 : Math.max(0, Math.min(100, progress))}%` }}
        />
      </div>
    </div>
  )
}
