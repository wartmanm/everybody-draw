package com.github.wartman4404.gldraw
import android.graphics.Bitmap
import android.os.Message
import android.view.{MotionEvent, View}

class CopyShader private (private val nativePtr: Int) extends AnyVal
class PointShader private (private val nativePtr: Int) extends AnyVal
class Texture private (val ptr: TexturePtr, val bitmap: Bitmap)
class TexturePtr private (private val nativePtr: Int) extends AnyVal
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
class GLException(msg: String) extends Exception(msg)
object GLResultTypeDef {
  type GLResult[T] = T
  type GLStoredResult[T] = Either[String, T]
  type GLException = com.github.wartman4404.gldraw.GLException
}


import GLResultTypeDef._

trait Shader[T] {
  def apply(data: GLInit, vec: String, frag: String): GLResult[T]
}

object CopyShader extends Shader[CopyShader] {
  @native def compile(data: GLInit, vec: String, frag: String): GLResult[Int]
  def apply(data: GLInit, vec: String, frag: String): GLResult[CopyShader] = {
    new CopyShader(compile(data, vec, frag))
  }
  @native def getSource(gl: GLInit, shader: CopyShader): (String, String)
}

object PointShader extends Shader[PointShader] {
  @native def compile(data: GLInit, vec: String, frag: String): GLResult[Int]
  def apply(data: GLInit, vec: String, frag: String): GLResult[PointShader] = {
    new PointShader(compile(data, vec, frag))
  }
  @native def getSource(gl: GLInit, shader: PointShader): (String, String)
}

object TexturePtr {
  @native def init(data: GLInit, image: Bitmap): GLResult[Int]
  def apply(data: GLInit, image: Bitmap): GLResult[TexturePtr] = {
    new TexturePtr(init(data, image))
  }
}

object Texture {
  def apply(data: GLInit, image: Bitmap): GLResult[Texture] = {
    new Texture(TexturePtr(data, image), image)
  }
}

object LuaScript {
  @native def init(data: GLInit, script: String): GLResult[Int]
  def apply(data: GLInit, script: String): GLResult[LuaScript] = {
    new LuaScript(init(data, script))
  }
  @native def getSource(gl: GLInit, script: LuaScript): String
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

object MotionEventProducer {
  @native def nativeAppendMotionEvent(producer: MotionEventProducer, evt: MotionEvent): Unit
  @native def nativePauseMotionEvent(handler: MotionEventProducer): Unit
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
  @native def init(v: View): MotionEventHandlerPair
  @native def destroy(m: MotionEventHandlerPair): Unit
}

//case class BrushProperties(color: Int, size: Float)
