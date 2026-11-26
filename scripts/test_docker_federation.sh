#!/usr/bin/env bash
# Run with: bash scripts/test_docker_federation.sh [default|explicit] [bin-dir]
# bin-dir may contain the examples directly or under examples/.
set -euo pipefail

mode=${1:-default}
case "$mode" in default|explicit) ;; *) echo "Unknown mode: $mode" >&2; exit 2 ;; esac
root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
bin=$(cd -- "${2:-$root/target/release}" && pwd)
examples=$bin
[[ ! -d $bin/examples ]] || examples=$bin/examples
for exe in "$bin/robot_bus_broker" "$examples"/{topic_imu_listener,topic_imu_talker,service_set_bool_server,service_set_bool_client}; do
    [[ -x $exe ]] || { echo "Missing executable: $exe" >&2; exit 2; }
done

prefix="rb-federation-$$"
lan=$prefix-lan
wan_a=$prefix-wan-a
wan_b=$prefix-wan-b
a=$prefix-a
b=$prefix-b
ip_a=172.28.0.11
ip_b=172.28.0.12
wan_ip_a=172.29.1.11
wan_ip_b=172.29.2.12
logs=$(mktemp -d)
networks=()
containers=()

cleanup() {
    local rc=$? container network
    for container in "${containers[@]}"; do
        docker exec "$container" cat /tmp/broker.log > "$logs/$container.broker.log" 2>&1 || true
        docker rm -f "$container" >/dev/null || true
    done
    for network in "${networks[@]}"; do docker network rm "$network" >/dev/null || true; done
    if (( rc != 0 )); then
        for container in "${containers[@]}"; do
            echo "--- $container broker ---" >&2
            cat "$logs/$container.broker.log" >&2
        done
    fi
    echo "Docker federation ($mode) logs: $logs"
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM

make_network() {
    docker network create --subnet "$2" "$1" >/dev/null
    networks+=("$1")
}
make_container() {
    docker run -d --name "$1" \
        --network "name=$lan,ip=$2" \
        --network "name=$3,ip=$4,gw-priority=1" \
        --mount "type=bind,src=$bin,dst=/broker,readonly" \
        --mount "type=bind,src=$examples,dst=/examples,readonly" \
        "${DOCKER_FEDERATION_IMAGE:-ubuntu:22.04}" sleep infinity >/dev/null
    containers+=("$1")
}
make_network "$lan" 172.28.0.0/24
make_network "$wan_a" 172.29.1.0/24
make_network "$wan_b" 172.29.2.0/24
make_container "$a" "$ip_a" "$wan_a" "$wan_ip_a"
make_container "$b" "$ip_b" "$wan_b" "$wan_ip_b"

start_broker() {
    local container=$1 ip=$2
    shift 2
    local opts=(--tcp-only --no-tank --broker-id "$container" --api-listen 0.0.0.0:15560
        --message-xsub-bind tcp://0.0.0.0:15580 --message-xpub-bind tcp://0.0.0.0:15581
        --service-frontend-bind tcp://0.0.0.0:15662 --service-backend-bind tcp://0.0.0.0:15663
        --action-frontend-bind tcp://0.0.0.0:15664 --action-backend-bind tcp://0.0.0.0:15665)
    if [[ $mode == explicit ]]; then opts+=(--advertise-host "$ip"); fi
    docker exec -d "$container" bash -c \
        'echo "$$" >/tmp/broker.pid; exec "$@" >>/tmp/broker.log 2>&1' \
        _ /broker/robot_bus_broker "${opts[@]}" "$@"
}

# Use Bash's TCP socket so the stock image needs neither curl nor iproute2.
discover() {
    docker exec "$1" timeout 2 bash -c '
        exec 3<>"/dev/tcp/$1/15560"
        printf "GET /api/v1/discover HTTP/1.0\r\nHost: %s\r\n\r\n" "$1" >&3
        IFS= read -r status <&3
        [[ $status == HTTP/*" 200 "* ]] || exit 1
        while IFS= read -r line <&3 && [[ $line != $'\''\r'\'' ]]; do :; done
        cat <&3
    ' _ "$2"
}
wait_discover() {
    local json attempt
    for ((attempt=0; attempt<30; attempt++)); do
        if json=$(discover "$1" "$2" 2>/dev/null); then
            printf '%s\n' "$json"
            return
        fi
        # This delay is inside the smoke test, between readiness probes.
        sleep 0.2
    done
    echo "Discover did not become ready: $1 -> $2" >&2
    return 1
}
check_discover() {
    wait_discover "$1" "$2" | tee "$logs/discover-$2.json" | python3 -c '
import json, sys
d = json.load(sys.stdin)
mode, host, broker = sys.argv[1:]
expected = {"brokerId": broker, "domainId": 0,
            "messageXsubPort": 15580, "messageXpubPort": 15581,
            "serviceFrontendPort": 15662, "serviceBackendPort": 15663,
            "actionFrontendPort": 15664, "actionBackendPort": 15665}
if mode == "explicit":
    expected["advertiseHost"] = host
assert d == expected, d
' "$mode" "$2" "$3"
}
assert_data_path() {
    docker exec "$1" timeout 2 bash -c 'exec 3<>"/dev/tcp/$1/15581"' _ "$2"
    if docker exec "$1" timeout 2 bash -c 'exec 3<>"/dev/tcp/$1/15581"' _ "$3" 2>/dev/null; then
        echo "Isolated WAN unexpectedly reachable: $1 -> $3" >&2
        return 1
    fi
}

stop_broker() {
    docker exec "$1" timeout 5 bash -c '
        pid=$(cat /tmp/broker.pid)
        kill "$pid"
        while kill -0 "$pid" 2>/dev/null; do sleep 0.05; done
    '
}
start_pair() {
    start_broker "$a" "$ip_a"
    check_discover "$b" "$ip_a" "$a"
    start_broker "$b" "$ip_b" --peer "$ip_a:15560"
    check_discover "$a" "$ip_b" "$b"
    stop_broker "$a"
    start_broker "$a" "$ip_a" --peer "$ip_b:15560"
    check_discover "$b" "$ip_a" "$a"
}

start_pair
assert_data_path "$b" "$ip_a" "$wan_ip_a"
assert_data_path "$a" "$ip_b" "$wan_ip_b"

wait_marker() {
    docker exec "$1" timeout "$4" bash -c '
        until grep -Fq -- "$2" "$1"; do sleep 0.1; done
    ' _ "$2" "$3"
}
topic() {
    local label=$1 publisher=$2 subscriber=$3 rc=0
    docker exec -d "$subscriber" bash -c \
        'exec timeout 10 /examples/topic_imu_listener > /tmp/"$1".log 2>&1' _ "$label"
    wait_marker "$subscriber" "/tmp/$label.log" "listening on /examples/imu" 5 || return 2
    docker exec "$publisher" timeout 8 /examples/topic_imu_talker > "$logs/$label.talker.log" 2>&1 || return 2
    grep -Fq 'published Imu #4' "$logs/$label.talker.log" || return 2
    wait_marker "$subscriber" "/tmp/$label.log" 'linear_acceleration.z=' 4 || rc=$?
    docker exec "$subscriber" cat "/tmp/$label.log" > "$logs/$label.listener.log" || return 2
    if (( rc == 124 )); then return 1; fi
    return "$rc"
}
service() {
    local label=$1 server=$2 client=$3 rc=0
    docker exec -d "$server" bash -c \
        'echo "$$" > /tmp/service.pid; exec timeout 20 /examples/service_set_bool_server > /tmp/"$1".log 2>&1' _ "$label"
    wait_marker "$server" "/tmp/$label.log" 'serving /examples/set_bool' 5 || return 2
    docker exec "$client" timeout 15 /examples/service_set_bool_client > "$logs/$label.client.log" 2>&1 || rc=$?
    docker exec "$server" cat "/tmp/$label.log" > "$logs/$label.server.log" || return 2
    # Stop this server before testing the reverse direction.
    docker exec "$server" bash -c 'kill "$(cat /tmp/service.pid)"' || return 2
    if (( rc != 0 )); then
        if grep -Fq 'Error: Timeout("service ' "$logs/$label.client.log"; then return 1; fi
        return 2
    fi
    grep -Fxq 'success=true message=set:true' "$logs/$label.client.log" || return 2
}
check() {
    local description=$1 rc=0
    shift
    "$@" || rc=$?
    if [[ $rc == 0 ]]; then
        echo "$description: passed"
    else
        echo "$description: unexpected result in $mode mode (status $rc)" >&2
        return 1
    fi
}
check 'Topic A -> B' topic topic-ab "$a" "$b"
check 'Topic B -> A' topic topic-ba "$b" "$a"
check 'Service B -> A' service service-ba "$a" "$b"
# Both examples use the same service name. Reset broker registries so a stale
# worker on the first server cannot intercept the reverse-direction request.
stop_broker "$a"
stop_broker "$b"
start_pair
check 'Service A -> B' service service-ab "$b" "$a"
