package com.github.wartman4404.gldraw

import android.content.Context
import java.io.{InputStream, BufferedInputStream, FileInputStream, File}
import java.util.zip.ZipFile
import android.graphics.{Bitmap, BitmapFactory}

import android.util.Log

import resource._

import unibrush.UniBrush

object DrawFiles {
  type MaybeRead[T] = (InputStream)=>Option[T]
  type MaybeReader[T] = (MaybeRead[T])=>Option[T]
  def allfiles[T](c: Context, path: String): Array[(String, (Unit)=>ManagedResource[InputStream])] = {
    val builtins = c.getAssets().list(path).map(path ++ "/" ++ _)
    val userdirs = c.getExternalFilesDirs(path).flatMap(Option(_)) // some paths may be null??
    val userfiles = userdirs.flatMap(_.listFiles())
    val builtinOpeners = builtins.map(path => {
        basename(path) -> ((_: Unit)=>withAssetStream[Option[T]](c, path))
      })

    val fileOpeners = userfiles.map(file => {
        file.getName() -> ((_: Unit)=>withFileStream[Option[T]](file))
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

  def useInputStream[T](reader: (InputStream)=>Option[T]) = {
    val out: (ManagedResource[InputStream])=>Option[T] = (m: ManagedResource[InputStream]) => {
      m.map(reader).opt.flatten
    }
    out
  }

def withFilename[T](reader: MaybeRead[T]): ((String, (Unit)=>ManagedResource[InputStream]))=>(String, (Unit)=>Option[T]) = {
    val a = (kv: (String, (Unit)=>ManagedResource[InputStream])) => {
      val (k, v) = kv
      k -> useInputStream(reader).compose(v)
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

  def loadShader[T](c: Context, constructor: MaybeRead[T], folder: String, defaultName: String, defaultObj: Option[T]) = {
    val default = defaultObj.map(x => (defaultName, (_: Unit) => Some(x)))
    val filenamed = withFilename[T](constructor)
    val files = allfiles[T](c, folder)
    val shaders: Seq[(String, (Unit)=>Option[T])] = files.map(filenamed)
    (default.toSeq ++ shaders).toArray
  }

  def loadBrushes(c: Context, data: GLInit): Array[(String, (Unit)=>Option[Texture])] = {
    val decoder = (is: InputStream) => (decodeBitmap(Bitmap.Config.ALPHA_8)(is).map(Texture(data, _)))
    loadShader(c, decoder, "brushes", null, None)
  }

  // TODO: make these safe
  def loadPointShaders(c: Context, data: GLInit): Seq[(String, (Unit)=>Option[PointShader])] = {
    val constructor = readShader(PointShader(data, _, _)) _
    loadShader(c, constructor, "pointshaders", "Default Paint", PointShader(data, null, null))
  }

  def loadAnimShaders(c: Context, data: GLInit): Seq[(String, (Unit)=>Option[CopyShader])] = {
    val constructor = readShader(CopyShader(data, _, _)) _
    loadShader(c, constructor, "animshaders", "Default Animation", CopyShader(data, null, null))
  }

  def loadScripts(c: Context, data: GLInit): Seq[(String, (Unit)=>Option[LuaScript])] = {
    val constructor = (LuaScript(data, _: String)).compose(readStream _)
    loadShader(c, constructor, "interpolators", "Default Interpolator", LuaScript(data, null))
  }

  def loadUniBrushes(c: Context, data: GLInit): Seq[(String, (Unit)=>Option[UniBrush])] = {
    val userdirs = c.getExternalFilesDirs("unibrushes").flatMap(Option(_))
    userdirs.flatMap(_.listFiles())
    .map(file => (file.getName(), (_: Unit) => UniBrush.compile(data, new ZipFile(file))))
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

  def readZip(zip: ZipFile, path: String) = {
    Option(zip.getEntry(path)).map(ze => readStream(zip.getInputStream(ze)))
  }
}
