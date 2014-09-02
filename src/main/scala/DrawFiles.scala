package com.github.wartman4404.gldraw

import android.content.Context
import java.io.{InputStream, BufferedInputStream, FileInputStream, File}
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

  def loadBrushes(c: Context, data: GLInit): Array[(String, (Unit)=>Option[Texture])] = {
    val decoder = decodeBitmap(Bitmap.Config.ALPHA_8) _
    val toTexture = (ob: Option[Bitmap]) => ob.map(Texture(data, _))
    val filenamed = withFilename[Texture](toTexture.compose(decoder))
    val files: Array[(String, (Unit)=>ManagedResource[InputStream])] = allfiles[Texture](c, "brushes")
    files.map(filenamed)
  }

  // TODO: make these safe
  def loadPointShaders(c: Context, data: GLInit): Seq[(String, (Unit)=>Option[PointShader])] = {
    val defaultShader = PointShader(data, null, null).map(x => ("Default Paint", (_: Unit) => Some(x)))
    defaultShader.toSeq ++ allfiles[PointShader](c, "pointshaders").map(withFilename(readShader(PointShader(data, _, _)) _))
  }

  def loadAnimShaders(c: Context, data: GLInit): Seq[(String, (Unit)=>Option[CopyShader])] = {
    val defaultShader = CopyShader(data, null, null).map(x => ("Default Animation", (_: Unit) => Some(x)))
    val decoded = readShader(CopyShader(data, _, _)) _
    val filenamed = withFilename[CopyShader](decoded)
    val files = allfiles[CopyShader](c, "animshaders")
    val shaders: Seq[(String, (Unit)=>Option[CopyShader])] = files.map(filenamed)
    defaultShader.toSeq ++ shaders
  }

  def loadScripts(c: Context, data: GLInit): Seq[(String, (Unit)=>Option[LuaScript])] = {
    val defaultScript = LuaScript(data, null).map(x => ("Default Interpolator", (_: Unit) => Some(x)))
    val filenamed = withFilename((LuaScript(data, _: String)).compose(readStream _))
    defaultScript.toSeq ++ allfiles[String](c, "interpolators").map(filenamed)
  }

  def loadUniBrushes(c: Context, data: GLInit): Seq[(String, (Unit)=>Option[UniBrush])] = {
    val userdirs = c.getExternalFilesDirs("unibrushes").flatMap(Option(_))
    userdirs.flatMap(_.listFiles())
    .filter(dir => dir.isDirectory() && new File(dir, "brush.json").isFile())
    .map(dir => (dir.getName(), (_: Unit) => {
        withFileStream(new File(dir, "brush.json")).map(readStream _).opt
        .flatMap(src => UniBrush.compile(data, src, dir.getAbsolutePath()))
      }))
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
