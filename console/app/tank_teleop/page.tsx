import { redirect } from 'next/navigation'

/** Legacy path — teleop lives inside the unified TANK panel. */
export default function TankTeleopPage() {
  redirect('/tank')
}
