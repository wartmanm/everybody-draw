package com.github.wartman4404.gldraw

import java.util.concurrent.atomic.AtomicBoolean
import android.graphics.SurfaceTexture
import android.os.{Handler, Looper, Message, SystemClock}
import android.util.Log
import android.graphics.Bitmap

class TextureSurfaceThread(surface: SurfaceTexture, private var motionHandler: MotionEventHandler, handlerCallback: (TextureSurfaceThread)=>Unit)
extends Thread with Handler.Callback with AndroidImplicits {
  import TextureSurfaceThread.Constants._

  private var handler: Handler = null
  private val running = new AtomicBoolean(true)
  var targetFramerate = 15
  private val matrix = new Array[Float](16)
  private var eglHelper: EGLHelper = null
  private var outputShader: Option[CopyShader] = None
  var glinit: GLInit = 0.asInstanceOf[GLInit]

  @native protected def finishGL(data: GLInit): Unit
  @native protected def nativeUpdateGL(data: GLInit): Unit
  @native protected def nativeDrawQueuedPoints(data: GLInit, handler: MotionEventHandler, transformMatrix: Array[Float]): Unit
  @native protected def nativeClearFramebuffer(data: GLInit): Unit
  @native protected def drawImage(data: GLInit, bitmap: Bitmap): Unit
  @native protected def nativeSetAnimShader(data: GLInit, shader: CopyShader): Boolean
  @native protected def nativeSetCopyShader(data: GLInit, shader: CopyShader): Boolean
  @native protected def nativeSetPointShader(data: GLInit, shader: PointShader): Boolean
  @native protected def nativeSetBrushTexture(data: GLInit, t: Texture): Unit
  @native protected def exportPixels(data: GLInit): Bitmap
  @native protected def nativeSetInterpolator(data: GLInit, script: LuaScript): Unit
  @native protected def nativeSetSeparateBrushlayer(data: GLInit, separatelayer: Boolean): Unit

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
          drawQueuedPoints()
          updateGL()
          handler.sendEmptyMessageAtTime(MSG_NEW_FRAME, next)
        }
      }
      case MSG_END_GL => {
        finishGL(glinit)
        eglHelper.finish()
        Looper.myLooper().quit()
      }
      case MSG_BEGIN_GL => {
        Log.i("tst", "got begin_gl message");
        eglHelper = new EGLHelper()
        eglHelper.init(surface)
        Log.i("tst", "egl inited");
        glinit = GLInit(msg.arg1, msg.arg2)
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
        updateGL()
        msg.obj.asInstanceOf[()=>Unit]()
      }
    }
    true
  }

  def beginGL(x: Int, y: Int, callback: ()=>Unit): Unit = {
    handler.obtainMessage(MSG_BEGIN_GL, x, y, callback).sendToTarget()
  }
  
  def startFrames() = {
    this.running.set(true)
    handler.obtainMessage(MSG_NEW_FRAME).sendToTarget()
  }

  def stopFrames() = {
    this.running.set(false)
  }

  def runHere(fn: => Unit) = {
    handler.post(() => { fn; () })
  }

  def initScreen(bitmap: Option[Bitmap]) = runHere {
    Log.i("tst", "initing output shader")
    initOutputShader()
    Log.i("tst", s"drawing bitmap: ${bitmap}")
    bitmap.foreach(b => {
        drawImage(glinit, b)
        b.recycle()
      })
  }

  def clearScreen() = runHere {
    nativeClearFramebuffer(glinit)
  }

  // callback runs on gl thread
  def getBitmap(cb: (Bitmap)=>Any) = runHere {
    cb(exportPixels(glinit))
  }

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

  def cleanupGL(copyShaders: Array[CopyShader], pointShaders: Array[PointShader]) = {
    handler.obtainMessage(MSG_END_GL).sendToTarget()
  }

  def drawBitmap(bitmap: Bitmap) = runHere { drawImage(glinit, bitmap) }

  // private
  private def initOutputShader() = {
    outputShader = CopyShader(glinit, null, null)
    outputShader.map((x) => {
        nativeSetCopyShader(glinit, x)
      })
  }

  private def drawQueuedPoints() = {
    nativeDrawQueuedPoints(glinit, motionHandler, matrix)
  }

  private def updateGL() {
    nativeUpdateGL(glinit)
  }

  // no consumers??
  def shaderWrapper[T](constructor: (String, String) => Option[T]) = (vec: String, frag: String) => runHere {
    constructor(vec, frag)
  }

  def shaderWrappers[T](constructor: (String, String) => Option[T]) = (vecfrag: Array[(String, String)]) => runHere {
    vecfrag.foreach(constructor.tupled)
  }

  def createShader[T](constructor: (String, String) => Option[T], vec: String, frag: String) = runHere {
    constructor(vec, frag)
  }

  def setBrushTexture(texture: Texture) = {
    Log.i("tst", s"setting brush texture to ${texture}")
    runHere {
      nativeSetBrushTexture(glinit, texture)
    }
  }

  // only set values, could maybe run on main thread
  def setAnimShader(shader: CopyShader) = runHere { nativeSetAnimShader(glinit, shader) }
  def setPointShader(shader: PointShader) = runHere { nativeSetPointShader(glinit, shader) }
  def setInterpScript(script: LuaScript) = runHere { nativeSetInterpolator(glinit, script) }
  def setSeparateBrushlayer(separatelayer: Boolean) = runHere { nativeSetSeparateBrushlayer(glinit, separatelayer) }
  //unused
  def setCopyShader(shader: CopyShader) = runHere { nativeSetCopyShader(glinit, shader) }
}

object TextureSurfaceThread {
  object Constants {
    val MSG_NEW_FRAME = 1
    val MSG_END_GL = 2
    val MSG_BEGIN_GL = 3
    val MSG_BEGIN_FRAMES = 4
  }
}
