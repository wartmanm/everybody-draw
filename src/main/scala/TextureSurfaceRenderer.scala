package com.github.wartman4404.gldraw

import java.util.concurrent.atomic.AtomicBoolean
import android.graphics.SurfaceTexture
import android.os.{Handler, Looper, Message, SystemClock}
import android.util.Log
import android.graphics.Bitmap

class TextureSurfaceThread(surface: SurfaceTexture, handlerCallback: (TextureSurfaceThread)=>Unit)
extends Thread with Handler.Callback with AndroidImplicits {
  import TextureSurfaceThread.Constants._

  private var handler: Handler = null
  private val running = new AtomicBoolean(true)
  var targetFramerate = 15
  private val matrix = new Array[Float](16)
  private var eglHelper: EGLHelper = null
  private var outputShader: Option[CopyShader] = None

  @native protected def initGL(w: Int, h: Int): Unit
  @native protected def finishGL(): Unit
  @native protected def nativeUpdateGL(): Unit
  @native protected def nativeDrawQueuedPoints(transformMatrix: Array[Float]): Unit
  @native protected def nativeClearFramebuffer(): Unit
  @native protected def drawImage(bitmap: Bitmap): Unit
  @native protected def nativeSetAnimShader(shader: CopyShader): Boolean
  @native protected def nativeSetCopyShader(shader: CopyShader): Boolean
  @native protected def nativeSetPointShader(shader: PointShader): Boolean
  @native protected def nativeSetBrushTexture(t: Texture): Unit
  @native protected def exportPixels(): Bitmap
  @native protected def nativeSetInterpolator(script: LuaScript): Unit

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
        finishGL()
        eglHelper.finish()
        Looper.myLooper().quit()
      }
      case MSG_BEGIN_GL => {
        Log.i("tst", "got begin_gl message");
        eglHelper = new EGLHelper()
        eglHelper.init(surface)
        Log.i("tst", "egl inited");
        initGL(msg.arg1, msg.arg2)
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
        drawImage(b)
        b.recycle()
      })
  }

  def clearScreen() = runHere {
    nativeClearFramebuffer()
  }

  // callback runs on gl thread
  def getBitmap(cb: (Bitmap)=>Any) = runHere {
    cb(exportPixels())
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

  def drawBitmap(bitmap: Bitmap) = runHere { drawImage(bitmap) }

  // private
  private def initOutputShader() = {
    outputShader = CopyShader(null, null)
    outputShader.map((x) => {
        nativeSetCopyShader(x)
      })
  }

  private def drawQueuedPoints() = {
    nativeDrawQueuedPoints(matrix)
  }

  private def updateGL() {
    nativeUpdateGL()
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
      nativeSetBrushTexture(texture)
    }
  }
  // only set values, could maybe run on main thread
  def setAnimShader(shader: CopyShader) = runHere { nativeSetAnimShader(shader) }
  def setPointShader(shader: PointShader) = runHere { nativeSetPointShader(shader) }
  def setInterpScript(script: LuaScript) = runHere { nativeSetInterpolator(script) }
  //unused
  def setCopyShader(shader: CopyShader) = runHere { nativeSetCopyShader(shader) }
}

object TextureSurfaceThread {
  object Constants {
    val MSG_NEW_FRAME = 1
    val MSG_END_GL = 2
    val MSG_BEGIN_GL = 3
    val MSG_BEGIN_FRAMES = 4
  }
}
