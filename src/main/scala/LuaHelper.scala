package com.github.wartman4404.gldraw
object LuaHelper {
  @native protected def nativeLoadScript(script: String): Unit
  def loadScript(script: String) = {
    nativeLoadScript(script)
  }
}
