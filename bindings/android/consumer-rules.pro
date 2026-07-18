# Keep JNA / native entry points when consumer enables minify.
-keep class com.sun.jna.** { *; }
-keep class org.indunet.robot.bus.** { *; }
-keepclassmembers class * extends com.sun.jna.** { public *; }
