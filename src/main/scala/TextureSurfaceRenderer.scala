package com.github.wartman4404.gldraw

import java.util.concurrent.atomic.AtomicBoolean
import android.graphics.SurfaceTexture
import android.os.{Handler, Looper, Message, SystemClock}
import android.util.Log
import android.graphics.Bitmap
import unibrush.Layer

class TextureSurfaceThread(surface: SurfaceTexture, private var motionHandler: MotionEventHandler, handlerCallback: (TextureSurfaceThread)=>Unit, errorCallback: (Exception)=>Unit)
extends Thread with Handler.Callback with AndroidImplicits {
  import TextureSurfaceThread.Constants._
  import TextureSurfaceThread._

  private var handler: Handler = null
  private val running = new AtomicBoolean(true)
  var targetFramerate = 15
  private val matrix = new Array[Float](16)
  private var eglHelper: EGLHelper = null
  private var pOutputShader: Option[CopyShader] = None
  def outputShader = pOutputShader
  private var glinit: Option[GLInit] = None
  private var replay = Replay.nullReplay

  @native protected def nativeUpdateGL(data: GLInit): Unit
  @native protected def nativeDrawQueuedPoints(data: GLInit, handler: MotionEventHandler, transformMatrix: Array[Float]): Unit
  @native protected def nativeFinishLuaScript(data: GLInit, handler: MotionEventHandler): Unit
  @native protected def nativeClearFramebuffer(data: GLInit): Unit
  @native protected def drawImage(data: GLInit, bitmap: Bitmap): Unit
  @native protected def nativeSetAnimShader(data: GLInit, shader: CopyShader): Boolean
  @native protected def nativeSetCopyShader(data: GLInit, shader: CopyShader): Boolean
  @native protected def nativeSetPointShader(data: GLInit, shader: PointShader): Boolean
  @native protected def nativeSetBrushTexture(data: GLInit, t: TexturePtr): Unit
  @native protected def exportPixels(data: GLInit): Bitmap
  @native protected def nativeSetInterpolator(data: GLInit, script: LuaScript): Unit
  @native protected def nativeAddLayer(data: GLInit, copyshader: CopyShader, pointshader: PointShader, pointidx: Int): Unit
  @native protected def nativeClearLayers(data: GLInit): Unit
  @native protected def nativeLoadUndo(data: GLInit, pos: Int): Unit
  //@native protected def nativeSetBrushProperties(props: BrushProperties): Unit
  @native protected def nativeSetBrushColor(data: GLInit, color: Int): Unit
  @native protected def nativeSetBrushSize(data: GLInit, size: Float): Unit

  override def run() = {
    Looper.prepare()
    handler = new Handler(this)
    handlerCallback(this)
    Log.i("tst", "entering message loop")
    Looper.loop()
  }

  override def handleMessage(msg: Message): Boolean = {
    msg.what match {
      case MSG_NEW_FRAME => {
        if (running.get()){
          val next = SystemClock.uptimeMillis() + 1000 / targetFramerate
          val gl: GLInit = GLInit.fromMessage(msg)
          try {
            if (replay == Replay.nullReplay) {
              drawQueuedPoints(gl)
            } else {
              drawReplayFrame(gl, replay)
            }
          } catch {
            case e: LuaException => {
              nativeSetInterpolator(gl, LuaScript(gl, null).right.get)
              errorCallback(e)
            }
          }
          updateGL(gl)
          val newmessage = gl.toMessage(handler.obtainMessage(MSG_NEW_FRAME))
          handler.sendMessageAtTime(newmessage, next)
        }
      }
      case MSG_END_GL => {
        glinit.foreach(GLInit.destroy _)
        glinit = None
        eglHelper.finish()
        Looper.myLooper().quit()
      }
      case MSG_BEGIN_GL => {
        Log.i("tst", "got begin_gl message");
        eglHelper = new EGLHelper()
        eglHelper.init(surface)
        Log.i("tst", "egl inited");
        val BeginGLArgs(undoCallback, beginGLCallback) = msg.obj.asInstanceOf[BeginGLArgs]
        val gl = GLInit(msg.arg1, msg.arg2, undoCallback)
        glinit = Some(gl)
        initOutputShader(gl)
        android.opengl.Matrix.orthoM(matrix, 0,
          0, msg.arg1,
          msg.arg2, 0,
          -1, 1)
        Log.i("tst", "set up matrix for %d, %d: \n[[%5.03f,%5.03f,%5.03f,%5.03f]\n [%5.03f,%5.03f,%5.03f,%5.03f]\n [%5.03f,%5.03f,%5.03f,%5.03f]\n [%5.03f,%5.03f,%5.03f,%5.03f]]".format(
          msg.arg1, msg.arg2,
          matrix(0), matrix(1), matrix(2), matrix(3),
          matrix(4), matrix(5), matrix(6), matrix(7),
          matrix(8), matrix(9), matrix(10), matrix(11),
          matrix(12), matrix(13), matrix(14), matrix(15)))
        Log.i("tst", "gl inited");
        updateGL(gl)
        beginGLCallback(gl)
      }
    }
    true
  }

  def beginGL(x: Int, y: Int, initCallback: (GLInit)=>Unit, undoCallback: UndoCallback): Unit = {
    handler.obtainMessage(MSG_BEGIN_GL, x, y, BeginGLArgs(undoCallback, initCallback)).sendToTarget()
  }

  def startFrames(): Unit = {
    glinit match {
      case Some(gl) => startFrames(gl)
      case None => Log.e("tst", "unable to start frames, no gl inited!")
    }
  }
  
  def startFrames(gl: GLInit): Unit = {
    this.running.set(true)
    gl.toMessage(handler.obtainMessage(MSG_NEW_FRAME)).sendToTarget()
  }

  def stopFrames() = {
    this.running.set(false)
  }

  // TODO: check if we're already on the gl thread
  private def runHere(fn: => Unit) = {
    handler.post(() => { fn; () })
  }

  def initScreen(gl: GLInit, bitmap: Option[Bitmap]) = {
    Log.i("tst", "initing output shader")
    Log.i("tst", s"drawing bitmap: ${bitmap}")
    for (b <- bitmap) {
      drawImage(gl, b)
      b.recycle()
    }
  }

  def clearScreen() = withGL(nativeClearFramebuffer _)

  // callback runs on gl thread
  def getBitmap(cb: (Bitmap)=>Any) = withGL(gl => cb(exportPixels(gl)))

  def getBitmapSynchronized() = {
    var bitmap: Bitmap = null
    val notify = new Object()
    notify.synchronized {
      getBitmap(x => {
          bitmap = x
          notify.synchronized { notify.notify() }
        })
      notify.wait()
    }
    bitmap
  }

  def cleanupGL() = {
    handler.obtainMessage(MSG_END_GL).sendToTarget()
  }

  def drawBitmap(bitmap: Bitmap) = withGL(drawImage(_, bitmap))

  // private
  private def initOutputShader(g: GLInit) = {
    pOutputShader = CopyShader(g, null, null).right.toOption
    pOutputShader.map((x) => {
        nativeSetCopyShader(g, x)
      })
  }

  private def drawQueuedPoints(g: GLInit) = {
    nativeDrawQueuedPoints(g, motionHandler, matrix)
  }

  def finishLuaScript(gl: GLInit) = {
    Log.i("tst", "finishing lua script - final draw")
    nativeDrawQueuedPoints(gl, motionHandler, matrix)
    Log.i("tst", "finishing lua script - unloading")
    nativeFinishLuaScript(gl, motionHandler)
  }

  private def drawReplayFrame(gl: GLInit, r: Replay) = {
    val finished = Replay.advanceFrame(gl, r, matrix)
    if (finished) {
      Replay.destroy(r)
      this.replay = Replay.nullReplay
    }
  }

  private def updateGL(g: GLInit) {
    nativeUpdateGL(g)
  }

  def setBrushTexture(gl: GLInit, texture: Texture) {
    Log.i("tst", s"setting brush texture to ${texture}")
    nativeSetBrushTexture(gl, texture.ptr)
  }

  def beginReplay() {
    withGL(gl => {
      replay = Replay.init(gl)
    })
  }

  def clearLayers(gl: GLInit) = nativeClearLayers(gl)

  def addLayer(gl: GLInit, copyshader: CopyShader, pointshader: PointShader, pointidx: Int) = {
    nativeAddLayer(gl, copyshader, pointshader, pointidx)
  }

  def loadUndo(gl: GLInit, pos: Int) = nativeLoadUndo(gl, pos)

  // only set values, could maybe run on main thread
  def setAnimShader(gl: GLInit, shader: CopyShader) = nativeSetAnimShader(gl, shader)
  def setPointShader(gl: GLInit, shader: PointShader) = nativeSetPointShader(gl, shader)
  def setInterpScript(gl: GLInit, script: LuaScript) = nativeSetInterpolator(gl, script)
  def setCopyShader(gl: GLInit, shader: CopyShader) = nativeSetCopyShader(gl, shader)
  def setBrushColor(gl: GLInit, color: Int) = nativeSetBrushColor(gl, color)
  def setBrushSize(gl: GLInit, size: Float) = nativeSetBrushSize(gl, size)

  def withGL(cb: (GLInit) => Unit) = {
    for (gl <- glinit) { runHere {
      cb(gl)
    }}
  }
}

object TextureSurfaceThread {
  object Constants {
    val MSG_NEW_FRAME = 1
    val MSG_END_GL = 2
    val MSG_BEGIN_GL = 3
    val MSG_BEGIN_FRAMES = 4
  }

  case class BeginGLArgs(undoCallback: UndoCallback, initCallback: (GLInit) => Unit)
}
