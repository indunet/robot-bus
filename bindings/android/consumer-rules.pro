# Keep JNA / native entry points when the app enables minify.
# The shared Java SDK (compiled into this AAR) maps C structs by reflection:
# RobotBusC's Structure subclasses and Callback types must keep their fields.
-keep class com.sun.jna.** { *; }
-keep class org.indunet.robot.bus.** { *; }
-keepclassmembers class * extends com.sun.jna.** { public *; }
