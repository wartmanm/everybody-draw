package com.github.wartman4404.gldraw

import android.graphics.SurfaceTexture
import android.view.Surface

class EGLHelper {
  private var surface: Surface = null

  @native def nativeInit(surface: Surface): Unit
  @native def nativeFinish(): Unit


  def finish() {
    surface.release()
    nativeFinish()
  }

  def init(mSurface: SurfaceTexture) {
    surface = new Surface(mSurface)
    nativeInit(surface)
  }
}

