"""Shared field conversions for generated topic mappers."""

from __future__ import annotations


def i8_seq_to_bytes(data) -> bytes:
    return bytes((int(v) & 0xFF) for v in data)


def bytes_to_i8_seq(data: bytes):
    return [int.from_bytes(bytes([b]), "little", signed=True) for b in data]


def time_to_timestamp(t):
    from google.protobuf.timestamp_pb2 import Timestamp

    out = Timestamp()
    out.seconds = int(t.sec)
    out.nanos = int(t.nanosec)
    return out


def timestamp_to_time(ts):
    from builtin_interfaces.msg import Time

    out = Time()
    out.sec = int(ts.seconds)
    out.nanosec = int(ts.nanos)
    return out


def duration_to_proto(d):
    from google.protobuf.duration_pb2 import Duration

    out = Duration()
    out.seconds = int(d.sec)
    out.nanos = int(d.nanosec)
    return out


def proto_to_duration(d):
    from builtin_interfaces.msg import Duration

    out = Duration()
    out.sec = int(d.seconds)
    out.nanosec = int(d.nanos)
    return out
