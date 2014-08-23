package com.github.wartman4404.gldraw
import android.graphics.Bitmap

class CopyShader private (private val nativePtr: Int) extends AnyVal
class PointShader private (private val nativePtr: Int) extends AnyVal
class Texture private (private val nativePtr: Int) extends AnyVal

object CopyShader {
  @native def compile(vec: String, frag: String): Int
  def apply(vec: String, frag: String): Option[CopyShader] = {
    compile(vec, frag) match {
      case -1 => None
      case x => Some(new CopyShader(x))
    }
  }
}
object PointShader {
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

