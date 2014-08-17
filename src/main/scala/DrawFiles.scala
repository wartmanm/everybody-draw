package com.github.wartman4404.gldraw

import android.content.Context
import java.io.{InputStream, BufferedInputStream, FileInputStream, File}
import android.graphics.{Bitmap, BitmapFactory}

import android.util.Log

import resource._

object DrawFiles {
  type MaybeRead[T] = (InputStream)=>Option[T]
  type MaybeReader[T] = (MaybeRead[T])=>Option[T]
  def allfiles[T](c: Context, path: String): Array[(String, ManagedResource[InputStream])] = {
    val builtins = c.getAssets().list(path).map(path ++ "/" ++ _)
    val userdirs = c.getExternalFilesDirs(path)
    val userfiles = userdirs.flatMap(_.listFiles())
    val builtinOpeners = builtins.map(path => {
        basename(path) -> withAssetStream[Option[T]](c, path)
    })

  val fileOpeners = userfiles.map(file => {
      file.getName() -> withFileStream[Option[T]](file)
    })
  (builtinOpeners ++ fileOpeners)
  }

  def withAssetStream[T](c: Context, path: String) = {
    managed(c.getAssets().open(path))
  }

  def withFileStream[T](file: File) = {
    managed(new BufferedInputStream(new FileInputStream(file)))
  }


  def basename(s: String) = {
    s.substring(s.lastIndexOf("/") + 1)
  }

  def withFilename[T](reader: MaybeRead[T]) = {
    val a = (kv: (String, ManagedResource[InputStream])) => {
      val (k, v) = kv
      for (stream <- v) yield reader(stream).map(k -> _)
    }
    a
  }

  def decodeBitmap(config: Bitmap.Config)(stream: InputStream): Option[Bitmap] = {
    val options = new BitmapFactory.Options
    options.inPreferredConfig = config
    options.inScaled = false
    Option(BitmapFactory.decodeStream(stream, null, options)).map(bitmap => {
        Log.i("drawfiles", "bitmap: config %s, w: %d, h: %d, alpha: %b".format(
          bitmap.getConfig(), bitmap.getHeight(), bitmap.getWidth(), bitmap.hasAlpha()))
        bitmap
      })
  }

  def loadBrushes(c: Context) = {
    val decoder = decodeBitmap(Bitmap.Config.ALPHA_8) _
    val filenamed = withFilename[Bitmap](decoder)
    val files = allfiles[Bitmap](c, "brushes")
    files.map(filenamed).flatMap(_.opt).flatten
  }

  // TODO: make these safe
  def loadPointShaders(c: Context) = {
    val defaultShader = PointShader(null, null).map(("Default Paint", _))
    defaultShader.toSeq ++ allfiles[PointShader](c, "pointshaders").map(withFilename(readShader(PointShader.apply _) _)).flatMap(_.opt).flatten.toSeq
  }

  def loadAnimShaders(c: Context): Seq[(String, CopyShader)] = {
    val defaultShader = CopyShader(null, null).map(("Default Animation", _))
    val decoded = readShader(CopyShader.apply _) _
    val filenamed = withFilename[CopyShader](decoded)
    val files = allfiles[CopyShader](c, "animshaders")
    val shaders = files.map(filenamed).flatMap(_.opt).flatten
    defaultShader.toSeq ++ shaders
  }

  def halfShaderPair(shader: String) = {
    if (shader.contains("gl_Position")) Some((shader, null))
      else if (shader.contains("gl_FragColor")) Some((null, shader))
      else None
  }

  def readShader[T](constructor: (String, String)=>Option[T])(src: InputStream): Option[T] = {
    halfShaderPair(readStream(src)).flatMap({case (vec, frag) => {
        constructor(vec, frag)
      }})
  }

  def readStream(src: InputStream) = {
    val source = scala.io.Source.fromInputStream(src)
    val text = source.getLines.mkString("\n")
    source.close()
    text
  }
}
