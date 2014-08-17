package com.github.wartman4404.gldraw

class CopyShader private (private val nativePtr: Int) extends AnyVal {
  @native def destroy()
}
class PointShader private (private val nativePtr: Int) extends AnyVal {
  @native def destroy()
}

object CopyShader {
  @native def compile(vec: String, frag: String): Int
  def apply(vec: String, frag: String): Option[CopyShader] = {
    compile(vec, frag) match {
      case 0 => None
      case x => Some(new CopyShader(x))
    }
  }
}
object PointShader {
  @native def compile(vec: String, frag: String): Int;
  def apply(vec: String, frag: String): Option[PointShader] = {
    compile(vec, frag) match {
      case 0 => None
      case x => Some(new PointShader(x))
    }
  }
}

