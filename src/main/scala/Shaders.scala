package com.github.wartman4404.gldraw
import android.graphics.Bitmap
import android.os.Message

class CopyShader private (private val nativePtr: Int) extends AnyVal
class PointShader private (private val nativePtr: Int) extends AnyVal
class Texture private (private val nativePtr: Int) extends AnyVal
class LuaScript private (private val nativePtr: Int) extends AnyVal
class MotionEventHandler private (private val nativePtr: Int) extends AnyVal
class MotionEventProducer private (private val nativePtr: Int) extends AnyVal
class Replay private (private val nativePtr: Int) extends AnyVal
class LuaException(msg: String) extends RuntimeException(msg)
class GLInit private (private val nativePtr: Int) extends AnyVal {
  def toMessage(m: Message) = {
    m.arg1 = nativePtr
    m
  }
}
trait UndoCallback {
  def undoBufferChanged(newSize: Int): Unit
}
object GLResultTypeDef {
  type GLResult[T] = Either[String, T]
}

import GLResultTypeDef._

trait Shader[T] {
  def apply(data: GLInit, vec: String, frag: String): GLResult[T]
}

object CopyShader extends Shader[CopyShader] {
  @native def compile(data: GLInit, vec: String, frag: String): GLResult[Int]
  def apply(data: GLInit, vec: String, frag: String): GLResult[CopyShader] = {
    compile(data, vec, frag) match {
      case Left(x) => Left(x)
      case Right(x) => Right(new CopyShader(x))
    }
  }
}

object PointShader extends Shader[PointShader] {
  @native def compile(data: GLInit, vec: String, frag: String): GLResult[Int]
  def apply(data: GLInit, vec: String, frag: String): GLResult[PointShader] = {
    compile(data, vec, frag) match {
      case Left(x) => Left(x)
      case Right(x) => Right(new PointShader(x))
    }
  }
}

object Texture {
  @native def init(data: GLInit, image: Bitmap): GLResult[Int];
  def apply(data: GLInit, image: Bitmap): GLResult[Texture] = {
    init(data, image) match {
      case Left(x) => Left(x)
      case Right(x) => Right(new Texture(x))
    }
  }
}

object LuaScript {
  @native def init(data: GLInit, script: String): GLResult[Int]
  def apply(data: GLInit, script: String): GLResult[LuaScript] = {
    init(data, script) match {
      case Left(x) => Left(x)
      case Right(x) => Right(new LuaScript(x))
    }
  }
}

object GLInit {
  @native def initGL(width: Int, height: Int, callback: UndoCallback): Int;
  def apply(width: Int, height: Int, callback: UndoCallback): GLInit = {
    new GLInit(initGL(width, height, callback))
  }
  // helper for texturesurfacethread
  def fromMessage(m: Message) = {
    new GLInit(m.arg1)
  }
  @native def destroy(data: GLInit): Unit
}

object Replay {
  @native def init(data: GLInit): Replay
  @native def destroy(replay: Replay): Unit
  @native def advanceFrame(data: GLInit, replay: Replay, matrix: Array[Float]): Boolean
  val nullReplay = new Replay(0)
}

case class MotionEventHandlerPair(
  val consumer: MotionEventHandler,
  val producer: MotionEventProducer)

object MotionEventHandlerPair {
  @native def init(): MotionEventHandlerPair
  @native def destroy(m: MotionEventHandlerPair): Unit
}
