package com.github.wartman4404.gldraw
import android.graphics.Bitmap

class CopyShader private (private val nativePtr: Int) extends AnyVal
class PointShader private (private val nativePtr: Int) extends AnyVal
class Texture private (private val nativePtr: Int) extends AnyVal
class LuaScript private (private val nativePtr: Int) extends AnyVal
class GLInit private (private val nativePtr: Int) extends AnyVal
class MotionEventHandler private (private val nativePtr: Int) extends AnyVal
class MotionEventProducer private (private val nativePtr: Int) extends AnyVal

trait Shader[T] {
  def apply(data: GLInit, vec: String, frag: String): Option[T]
}

object CopyShader extends Shader[CopyShader] {
  @native def compile(data: GLInit, vec: String, frag: String): Int
  def apply(data: GLInit, vec: String, frag: String): Option[CopyShader] = {
    compile(data, vec, frag) match {
      case -1 => None
      case x => Some(new CopyShader(x))
    }
  }
}
object PointShader extends Shader[PointShader] {
  @native def compile(data: GLInit, vec: String, frag: String): Int;
  def apply(data: GLInit, vec: String, frag: String): Option[PointShader] = {
    compile(data, vec, frag) match {
      case -1 => None
      case x => Some(new PointShader(x))
    }
  }
}

object Texture {
  @native def init(data: GLInit, image: Bitmap): Int;
  def apply(data: GLInit, image: Bitmap): Texture = {
    new Texture(init(data, image))
  }
}

object LuaScript {
  @native def init(data: GLInit, script: String): Int;
  def apply(data: GLInit, script: String): Option[LuaScript] = {
    init(data, script) match {
      case -1 => None
      case x => Some(new LuaScript(x))
    }
  }
}

object GLInit {
  @native def initGL(width: Int, height: Int): Int;
  def apply(width: Int, height: Int): GLInit = {
    new GLInit(initGL(width, height))
  }
}

case class MotionEventHandlerPair(
  val consumer: MotionEventHandler,
  val producer: MotionEventProducer)

object MotionEventHandlerPair {
  @native def init(): MotionEventHandlerPair
}
