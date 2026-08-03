package org.indunet.robot.bus;

import com.sun.jna.Pointer;
import com.sun.jna.ptr.LongByReference;
import com.sun.jna.ptr.PointerByReference;
import java.util.ArrayList;
import java.util.Collections;
import java.util.List;
import org.indunet.robot.bus.geometry_msgs.msg.v1.TransformStamped;
import org.indunet.robot.bus.tf2_msgs.msg.v1.TFMessage;

/** In-memory TF tree buffer (static + latest dynamic edges). */
public final class TfBuffer implements AutoCloseable {
    private Pointer ptr;

    public TfBuffer() {
        this.ptr = Errors.checkPtr(RobotBusC.Holder.INSTANCE.robot_bus_tf_buffer_new(), "TfBuffer");
    }

    TfBuffer(Pointer ptr) {
        this.ptr = ptr;
    }

    public void clear() {
        Errors.check(RobotBusC.Holder.INSTANCE.robot_bus_tf_buffer_clear(ptr), "TfBuffer.clear");
    }

    /** Ingest a {@code tf2_msgs/TFMessage}. {@code isStatic} marks {@code /tf_static} traffic. */
    public void setTransformMsg(TFMessage msg, boolean isStatic) {
        if (msg == null) {
            throw new NullPointerException("msg");
        }
        byte[] bytes = ProtoCodec.encode(msg);
        Errors.check(
                RobotBusC.Holder.INSTANCE.robot_bus_tf_buffer_set_transform_msg(
                        ptr, bytes, bytes.length, isStatic ? 1 : 0),
                "TfBuffer.setTransformMsg");
    }

    /** Lookup transform of {@code source} relative to {@code target}. */
    public TransformStamped lookupTransform(String target, String source) {
        PointerByReference outData = new PointerByReference();
        LongByReference outLen = new LongByReference();
        Errors.check(
                RobotBusC.Holder.INSTANCE.robot_bus_tf_buffer_lookup_transform(
                        ptr, target, source, outData, outLen),
                "TfBuffer.lookupTransform");
        byte[] bytes = NativeUtils.readBytes(outData.getValue(), outLen.getValue());
        return ProtoCodec.parse(TransformStamped.class, bytes);
    }

    public boolean canTransform(String target, String source) {
        return RobotBusC.Holder.INSTANCE.robot_bus_tf_buffer_can_transform(ptr, target, source) != 0;
    }

    public List<String> frames() {
        String joined = NativeUtils.takeCString(RobotBusC.Holder.INSTANCE.robot_bus_tf_buffer_frames(ptr));
        if (joined.isEmpty()) {
            return Collections.emptyList();
        }
        List<String> out = new ArrayList<>();
        for (String line : joined.split("\n", -1)) {
            if (!line.isEmpty()) {
                out.add(line);
            }
        }
        return out;
    }

    @Override
    public void close() {
        if (ptr != null) {
            RobotBusC.Holder.INSTANCE.robot_bus_tf_buffer_free(ptr);
            ptr = null;
        }
    }
}
