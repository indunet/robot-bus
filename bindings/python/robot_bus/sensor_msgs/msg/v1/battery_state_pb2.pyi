from robot_bus.std_msgs.msg.v1 import header_pb2 as _header_pb2
from google.protobuf.internal import containers as _containers
from google.protobuf import descriptor as _descriptor
from google.protobuf import message as _message
from collections.abc import Iterable as _Iterable, Mapping as _Mapping
from typing import ClassVar as _ClassVar, Optional as _Optional, Union as _Union

DESCRIPTOR: _descriptor.FileDescriptor

class BatteryState(_message.Message):
    __slots__ = ("header", "voltage", "current", "charge", "capacity", "design_capacity", "percentage", "power_supply_status", "power_supply_health", "power_supply_technology", "present", "cell_voltage", "cell_temperature", "location", "serial_number", "temperature")
    HEADER_FIELD_NUMBER: _ClassVar[int]
    VOLTAGE_FIELD_NUMBER: _ClassVar[int]
    CURRENT_FIELD_NUMBER: _ClassVar[int]
    CHARGE_FIELD_NUMBER: _ClassVar[int]
    CAPACITY_FIELD_NUMBER: _ClassVar[int]
    DESIGN_CAPACITY_FIELD_NUMBER: _ClassVar[int]
    PERCENTAGE_FIELD_NUMBER: _ClassVar[int]
    POWER_SUPPLY_STATUS_FIELD_NUMBER: _ClassVar[int]
    POWER_SUPPLY_HEALTH_FIELD_NUMBER: _ClassVar[int]
    POWER_SUPPLY_TECHNOLOGY_FIELD_NUMBER: _ClassVar[int]
    PRESENT_FIELD_NUMBER: _ClassVar[int]
    CELL_VOLTAGE_FIELD_NUMBER: _ClassVar[int]
    CELL_TEMPERATURE_FIELD_NUMBER: _ClassVar[int]
    LOCATION_FIELD_NUMBER: _ClassVar[int]
    SERIAL_NUMBER_FIELD_NUMBER: _ClassVar[int]
    TEMPERATURE_FIELD_NUMBER: _ClassVar[int]
    header: _header_pb2.Header
    voltage: float
    current: float
    charge: float
    capacity: float
    design_capacity: float
    percentage: float
    power_supply_status: int
    power_supply_health: int
    power_supply_technology: int
    present: bool
    cell_voltage: _containers.RepeatedScalarFieldContainer[float]
    cell_temperature: _containers.RepeatedScalarFieldContainer[float]
    location: str
    serial_number: str
    temperature: float
    def __init__(self, header: _Optional[_Union[_header_pb2.Header, _Mapping]] = ..., voltage: _Optional[float] = ..., current: _Optional[float] = ..., charge: _Optional[float] = ..., capacity: _Optional[float] = ..., design_capacity: _Optional[float] = ..., percentage: _Optional[float] = ..., power_supply_status: _Optional[int] = ..., power_supply_health: _Optional[int] = ..., power_supply_technology: _Optional[int] = ..., present: _Optional[bool] = ..., cell_voltage: _Optional[_Iterable[float]] = ..., cell_temperature: _Optional[_Iterable[float]] = ..., location: _Optional[str] = ..., serial_number: _Optional[str] = ..., temperature: _Optional[float] = ...) -> None: ...
