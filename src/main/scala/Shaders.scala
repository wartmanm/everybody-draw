package com.github.wartman4404.gldraw
import android.graphics.Bitmap

class CopyShader private (private val nativePtr: Int) extends AnyVal
class PointShader private (private val nativePtr: Int) extends AnyVal
class Texture private (private val nativePtr: Int) extends AnyVal
class LuaScript private (private val nativePtr: Int) extends AnyVal

trait Shader[T] {
  def apply(vec: String, frag: String): Option[T]
}

object CopyShader extends Shader[CopyShader] {
  @native def compile(vec: String, frag: String): Int
  def apply(vec: String, frag: String): Option[CopyShader] = {
    compile(vec, frag) match {
      case -1 => None
      case x => Some(new CopyShader(x))
    }
  }
}
object PointShader extends Shader[PointShader] {
  @native def compile(vec: String, frag: String): Int;
  def apply(vec: String, frag: String): Option[PointShader] = {
    compile(vec, frag) match {
      case -1 => None
      case x => Some(new PointShader(x))
    }
  }
}

object Texture {
  @native def init(image: Bitmap): Int;
  def apply(image: Bitmap): Texture = {
    new Texture(init(image))
  }
}

object LuaScript {
  @native def init(script: String): Int;
  def apply(script: String): Option[LuaScript] = {
    init(script) match {
      case -1 => None
      case x => Some(new LuaScript(x))
    }
  }
}
