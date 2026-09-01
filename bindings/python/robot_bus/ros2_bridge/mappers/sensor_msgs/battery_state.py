"""Generated mapper for `sensor_msgs/msg/BatteryState`."""

from __future__ import annotations

from robot_bus.ros2_bridge.mappers import _convert
from robot_bus.ros2_bridge.mappers.std_msgs.header import header_to_bus, header_to_ros

def battery_state_to_bus(msg):
    from robot_bus.sensor_msgs.msg.v1 import BatteryState as BusMsg

    bus = BusMsg()
    bus.header.CopyFrom(header_to_bus(msg.header))
    bus.voltage = msg.voltage
    bus.current = msg.current
    bus.charge = msg.charge
    bus.capacity = msg.capacity
    bus.design_capacity = msg.design_capacity
    bus.percentage = msg.percentage
    bus.power_supply_status = msg.power_supply_status
    bus.power_supply_health = msg.power_supply_health
    bus.power_supply_technology = msg.power_supply_technology
    bus.present = msg.present
    bus.cell_voltage.extend(list(msg.cell_voltage))
    bus.cell_temperature.extend(list(msg.cell_temperature))
    bus.location = str(msg.location)
    bus.serial_number = str(msg.serial_number)
    bus.temperature = msg.temperature
    return bus


def battery_state_to_ros(bus):
    from sensor_msgs.msg import BatteryState as RosMsg

    out = RosMsg()
    out.header = header_to_ros(bus.header)
    out.voltage = bus.voltage
    out.current = bus.current
    out.charge = bus.charge
    out.capacity = bus.capacity
    out.design_capacity = bus.design_capacity
    out.percentage = bus.percentage
    out.power_supply_status = bus.power_supply_status
    out.power_supply_health = bus.power_supply_health
    out.power_supply_technology = bus.power_supply_technology
    out.present = bus.present
    out.cell_voltage = list(bus.cell_voltage)
    out.cell_temperature = list(bus.cell_temperature)
    out.location = str(bus.location)
    out.serial_number = str(bus.serial_number)
    out.temperature = bus.temperature
    return out


class SensorMsgsBatteryStateMapper:
    def type_name(self) -> str:
        return "sensor_msgs/msg/BatteryState"

    def ros_msg_type(self):
        from sensor_msgs.msg import BatteryState as RosMsg

        return RosMsg

    def ros_to_bus(self, msg) -> bytes:
        return battery_state_to_bus(msg).SerializeToString()

    def bus_to_ros(self, payload: bytes):
        from robot_bus.sensor_msgs.msg.v1 import BatteryState as BusMsg

        bus = BusMsg()
        bus.ParseFromString(payload)
        return battery_state_to_ros(bus)
