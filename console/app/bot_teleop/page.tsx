import { redirect } from 'next/navigation'

/** Legacy path — teleop lives inside the unified BOT SIM panel. */
export default function BotTeleopPage() {
  redirect('/bot_sim')
}
