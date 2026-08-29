package org.indunet.robot.bus

import com.google.protobuf.InvalidProtocolBufferException
import com.google.protobuf.MessageLite
import com.sun.jna.Native
import com.sun.jna.Platform
import com.sun.jna.Pointer
import com.sun.jna.ptr.PointerByReference
import java.nio.file.Files
import java.nio.file.Path
import java.util.logging.Level
import java.util.logging.Logger

class RobotBusException(message: String) : RuntimeException(message)
internal object Errors {
    fun lastError() = RobotBusC.Holder.INSTANCE.robot_bus_last_error() ?: ""
    fun check(rc: Int, what: String) { if (rc < 0) throw RobotBusException("$what: ${lastError().ifEmpty { "unknown error" }}") }
    fun checkPtr(p: Pointer?, what: String): Pointer = p ?: throw RobotBusException("$what: ${lastError().ifEmpty { "null" }}")
}
internal object NativeUtils {
    fun takeCString(p: Pointer?): String = if (p == null) "" else try { p.getString(0) } finally { RobotBusC.Holder.INSTANCE.robot_bus_free_string(p) }
    fun readBytes(p: Pointer?, len: Long): ByteArray = if (p == null || len <= 0) ByteArray(0) else try { p.getByteArray(0, len.toInt()) } finally { RobotBusC.Holder.INSTANCE.robot_bus_free_bytes(p, len) }
    fun allocReplyBytes(payload: ByteArray?): Pointer? {
        if (payload == null || payload.isEmpty()) return null
        return Errors.checkPtr(
            RobotBusC.Holder.INSTANCE.robot_bus_alloc_bytes(payload.size.toLong()),
            "robot_bus_alloc_bytes",
        ).also { it.write(0, payload, 0, payload.size) }
    }
    fun endpointCall(block: (PointerByReference) -> Int): String = PointerByReference().let { out -> Errors.check(block(out), "endpoint"); takeCString(out.value) }
}
internal object NativeLoader {
    @Volatile private var preloaded = false

    fun markPreloaded() {
        preloaded = true
    }

    fun loadLibrary(): RobotBusC {
        // Device path: RobotBusAndroid.init already System.loadLibrary'd.
        if (preloaded) {
            return Native.load("robot_bus_c", RobotBusC::class.java)
        }

        // Host unit tests (Android classpath still has android.os.Build) — prefer explicit paths.
        val explicit =
            System.getenv("ROBOT_BUS_NATIVE")?.takeIf { it.isNotBlank() }
                ?: System.getProperty("robot.bus.native")?.takeIf { it.isNotBlank() }
        if (explicit != null) {
            return Native.load(explicit, RobotBusC::class.java)
        }
        val dir =
            System.getenv("ROBOT_BUS_NATIVE_DIR")?.takeIf { it.isNotBlank() }
                ?: System.getProperty("robot.bus.native.dir")?.takeIf { it.isNotBlank() }
        val paths =
            listOfNotNull(
                dir?.let(Path::of),
                Path.of("").toAbsolutePath().resolve("../cpp/native/target/release").normalize(),
                Path.of(System.getProperty("user.dir"), "bindings/cpp/native/target/release"),
                Path.of(System.getProperty("user.dir"), "../cpp/native/target/release").normalize(),
            )
        paths.firstNotNullOfOrNull(::findInDir)?.let {
            return Native.load(it.toAbsolutePath().toString(), RobotBusC::class.java)
        }

        // On-device (or last-resort system name).
        return Native.load("robot_bus_c", RobotBusC::class.java)
    }

    private fun findInDir(dir: Path): Path? {
        if (!Files.isDirectory(dir)) return null
        val names =
            when {
                Platform.isMac() -> listOf("librobot_bus_c.dylib", "librobot_bus.dylib")
                Platform.isWindows() -> listOf("robot_bus_c.dll", "robot_bus.dll")
                else -> listOf("librobot_bus_c.so", "librobot_bus.so")
            }
        return names.map(dir::resolve).firstOrNull(Files::isRegularFile)
    }
}
object NativePreload { @JvmStatic fun markRobotBusNativePreloaded() = NativeLoader.markPreloaded() }
object Endpoints {
    @JvmStatic fun messageXsubEndpoint()=messageXsubEndpoint("localhost","tcp"); @JvmStatic fun messageXsubEndpoint(host:String)=messageXsubEndpoint(host,"tcp")
    @JvmStatic fun messageXsubEndpoint(host:String,transport:String)=NativeUtils.endpointCall { RobotBusC.Holder.INSTANCE.robot_bus_message_xsub_endpoint(host,transport,it) }
    @JvmStatic fun messageXpubEndpoint()=messageXpubEndpoint("localhost","tcp"); @JvmStatic fun messageXpubEndpoint(host:String)=messageXpubEndpoint(host,"tcp")
    @JvmStatic fun messageXpubEndpoint(host:String,transport:String)=NativeUtils.endpointCall { RobotBusC.Holder.INSTANCE.robot_bus_message_xpub_endpoint(host,transport,it) }
}
class Parameter(@JvmField val name:String,@JvmField val value:Any) { fun getName()=name; fun getValue()=value }
class TopicMessage(@JvmField val topic:String="",@JvmField val payload:ByteArray=ByteArray(0)) { fun getTopic()=topic; fun getPayload()=payload; override fun equals(o:Any?)=o is TopicMessage&&topic==o.topic&&payload.contentEquals(o.payload); override fun hashCode()=31*topic.hashCode()+payload.contentHashCode() }
class ActionMessage(@JvmField val kind:String="",@JvmField val body:ByteArray=ByteArray(0),@JvmField val goalId:String="",@JvmField val actionName:String="") { fun getKind()=kind; fun getBody()=body; fun getGoalId()=goalId; fun getActionName()=actionName; override fun equals(o:Any?)=o is ActionMessage&&kind==o.kind&&body.contentEquals(o.body)&&goalId==o.goalId&&actionName==o.actionName; override fun hashCode()=listOf(kind,goalId,actionName).hashCode()+body.contentHashCode() }
class ActionPhase(@JvmField val phase:String="",@JvmField val body:ByteArray=ByteArray(0)) { fun getPhase()=phase; fun getBody()=body; override fun equals(o:Any?)=o is ActionPhase&&phase==o.phase&&body.contentEquals(o.body); override fun hashCode()=31*phase.hashCode()+body.contentHashCode() }
enum class CallbackGroupType(val code: Int) {
    MutuallyExclusive(0),
    Reentrant(1),
}
fun interface MsgCallback { fun onMessage(payload:ByteArray) }
fun interface TimerCallback { fun onTimer() }
fun interface ServiceHandler { fun handle(body:ByteArray):ByteArray }
fun interface ActionHandler { fun handle(body:ByteArray):List<ActionPhase> }
fun interface TypedMsgCallback<T:MessageLite> { fun onMessage(message:T) }
fun interface TypedServiceHandler<Req:MessageLite,Resp:MessageLite> { fun handle(request:Req):Resp }
fun interface TypedActionHandler<Goal:MessageLite> { fun handle(goal:Goal):List<TypedActionPhase> }
class TypedActionPhase(@JvmField val phase:String="",@JvmField val body:MessageLite) { fun getPhase()=phase; fun getBody()=body }
class TypedActionMessage internal constructor(@JvmField val kind:String="",@JvmField val body:MessageLite?,@JvmField val rawBody:ByteArray=ByteArray(0),@JvmField val goalId:String="",@JvmField val actionName:String="") { fun getKind()=kind; fun getBody()=body; fun getRawBody()=rawBody; fun getGoalId()=goalId; fun getActionName()=actionName }
internal object ProtoCodec {
    private val log=Logger.getLogger("org.indunet.robot.bus")
    fun encode(msg:MessageLite?):ByteArray=(msg ?: throw NullPointerException("expected a protobuf MessageLite")).toByteArray()
    fun <T:MessageLite> parse(type:Class<T>, payload:ByteArray?):T=try { type.getMethod("parseFrom",ByteArray::class.java).invoke(null,payload?:ByteArray(0)) as T } catch(e:java.lang.reflect.InvocationTargetException) { when(val c=e.cause) { is RuntimeException->throw c; is InvalidProtocolBufferException->throw IllegalArgumentException("invalid protobuf payload for ${type.simpleName}",c); else->throw IllegalStateException("parseFrom failed for ${type.name}",c) } } catch(e:ReflectiveOperationException) { throw IllegalArgumentException("${type.name} is not a generated protobuf message (missing parseFrom)",e) }
    fun <T:MessageLite> tryParse(type:Class<T>, payload:ByteArray?):T?=try { parse(type,payload) } catch(e:Exception) { log.log(Level.WARNING,"typed decode failed for ${type.simpleName}: $e",e); null }
    fun requireMessageType(type:Class<*>?,what:String) { require(type != null && MessageLite::class.java.isAssignableFrom(type)) { "$what must be a com.google.protobuf.MessageLite subclass, got $type" } }
}
