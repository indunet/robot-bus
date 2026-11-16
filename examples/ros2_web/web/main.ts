import { Node, WsRpcError, type WsGoalHandle } from 'robot-bus';
import { String$ } from 'robot-bus/std_msgs/msg/v1/primitives.js';
import { TriggerRequest, TriggerResponse } from 'robot-bus/std_srvs/srv/v1/trigger.js';
import { FibonacciGoal, FibonacciFeedback, FibonacciResult } from 'robot-bus/example_interfaces/action/v1/fibonacci.js';

const el = <T extends HTMLElement>(id: string) => document.getElementById(id) as T;
const prefix = '/ros2_web';
const brokerUrl = new URL(location.href);
brokerUrl.port = new URLSearchParams(location.search).get('brokerPort') ?? '15560';
brokerUrl.pathname = '/';
brokerUrl.search = '';
brokerUrl.hash = '';
el<HTMLInputElement>('broker').value = brokerUrl.origin;
let node: Node | undefined;
let active: WsGoalHandle<FibonacciResult> | undefined;

function failure(error: unknown): string {
  return error instanceof WsRpcError ? `${error.code}: ${error.message}` : `failed: ${String(error)}`;
}

function controls(enabled: boolean) {
  document.querySelectorAll<HTMLButtonElement>('[data-command]').forEach(button => {
    button.disabled = !enabled;
  });
}

async function connect() {
  el<HTMLButtonElement>('connect').disabled = true;
  controls(false);
  node?.shutdown();
  active = undefined;
  el<HTMLButtonElement>('cancel').disabled = true;
  const current = Node.wsAt('ros2_web_browser', el<HTMLInputElement>('broker').value);
  node = current;
  current.addOnConnectionEvent((_old, next) => {
    if (node !== current) return;
    el('connection').textContent = next;
    if (next !== 'connected') controls(false);
  });
  current.createSubscription(prefix + '/status', message => {
    if (node !== current) return;
    el('telemetry').textContent = message.data;
    const state = JSON.parse(message.data);
    for (const key of ['sequence', 'last_command', 'resets', 'goals_cancelled', 'goals_aborted']) {
      el(key).textContent = String(state[key]);
    }
  }, String$, 1);
  current.start();
  try {
    if (!await current.waitForBroker(10)) throw new Error('broker connection timed out');
    const ready = await Promise.all([
      current.createClient(prefix + '/reset').waitForService(10),
      current.createClient(prefix + '/slow').waitForService(10),
      current.createActionClient(prefix + '/fibonacci').waitForActionServer(10),
      current.createActionClient(prefix + '/deadline').waitForActionServer(10),
    ]);
    if (!ready.every(Boolean)) throw new Error('bridge routes are not ready');
    el('connection').textContent = 'ready';
    controls(true);
  } catch (error) {
    current.shutdown();
    el('connection').textContent = failure(error);
  } finally {
    el<HTMLButtonElement>('connect').disabled = false;
  }
}

el('connect').onclick = () => void connect();
el('send').onclick = async () => {
  if (!node) return;
  try {
    await node.createPublisher(prefix + '/command', String$).publish({ data: el<HTMLInputElement>('command').value });
    el('publish-result').textContent = 'Published; check the ROS acknowledgement below.';
  } catch (error) { el('publish-result').textContent = failure(error); }
};

for (const name of ['reset', 'slow']) {
  el(name).onclick = async () => {
    if (!node) return;
    const button = el<HTMLButtonElement>(name);
    button.disabled = true;
    el('service-result').textContent = 'waiting';
    try {
      const response = await node.createClient(prefix + '/' + name, TriggerRequest, TriggerResponse).call({}, 5);
      el('service-result').textContent = `${response.success ? 'success' : 'failed'}: ${response.message}`;
    } catch (error) { el('service-result').textContent = failure(error); }
    finally { button.disabled = node.connectionState() !== 'connected'; }
  };
}

async function runAction(name: string, order: number) {
  if (!node || active) return;
  const current = node;
  el('feedback').textContent = '';
  el('action-result').textContent = 'running';
  const goal = current.createActionClient(prefix + '/' + name, FibonacciGoal, FibonacciFeedback, FibonacciResult)
    .sendGoal({ order }, { timeoutSeconds: 12, onFeedback: feedback => {
      if (node === current) el('feedback').textContent = feedback.sequence.join(', ');
    } });
  active = goal;
  document.querySelectorAll<HTMLButtonElement>('[data-action]').forEach(b => { b.disabled = true; });
  el<HTMLButtonElement>('cancel').disabled = false;
  try {
    const result = await goal.result();
    if (node === current) el('action-result').textContent = `success: ${result.sequence.join(', ')}`;
  } catch (error) {
    if (node === current) el('action-result').textContent = failure(error);
  } finally {
    if (active === goal) {
      active = undefined;
      el<HTMLButtonElement>('cancel').disabled = true;
      document.querySelectorAll<HTMLButtonElement>('[data-action]').forEach(b => {
        b.disabled = current.connectionState() !== 'connected';
      });
    }
  }
}

el('run').onclick = () => {
  const order = Number(el<HTMLInputElement>('order').value);
  if (!Number.isInteger(order) || order < 1 || order > 30) {
    el('action-result').textContent = 'Enter a whole number from 1 to 30.';
    return;
  }
  void runAction('fibonacci', order);
};
el('reject').onclick = () => void runAction('fibonacci', -1);
el('abort').onclick = () => void runAction('fibonacci', 13);
el('deadline').onclick = () => void runAction('deadline', 30);
el('cancel').onclick = async () => {
  try { await active?.cancel(); }
  catch (error) { el('action-result').textContent = failure(error); }
};
window.addEventListener('pagehide', () => node?.shutdown());
void connect();
