package com.github.wartman4404.gldraw

import android.content.Context
import java.io.{InputStream, BufferedInputStream, FileInputStream, File}
import java.util.zip.ZipFile
import android.graphics.{Bitmap, BitmapFactory}

import android.util.Log

import resource._

import unibrush.UniBrush

object DrawFiles {
  import GLResultTypeDef._
  type MaybeRead[T] = (InputStream)=>GLResult[T]
  type MaybeReader[T] = (MaybeRead[T])=>GLResult[T]
  def allfiles[T](c: Context, path: String): Array[(String, ()=>ManagedResource[InputStream])] = {
    val builtins = c.getAssets().list(path).map(path ++ "/" ++ _)
    val userdirs = c.getExternalFilesDirs(path).flatMap(Option(_)) // some paths may be null??
    val userfiles = userdirs.flatMap(_.listFiles())
    val builtinOpeners = builtins.map(path => {
        basename(path) -> (()=>withAssetStream[Option[T]](c, path))
      })

    val fileOpeners = userfiles.map(file => {
        file.getName() -> (()=>withFileStream[Option[T]](file))
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

  def useInputStream[T](reader: (InputStream)=>GLResult[T]) = {
    val out: (ManagedResource[InputStream])=>GLResult[T] = (m: ManagedResource[InputStream]) => {
      m.map(reader).opt match {
        case Some(x) => x
        case None => Left("Failed to load file")
      }
    }
    out
  }

  def withFilename[T](reader: (GLInit, InputStream)=>GLResult[T]): ((String, ()=>ManagedResource[InputStream]))=>(String, (GLInit)=>GLResult[T]) = {
    val a = (kv: (String, ()=>ManagedResource[InputStream])) => {
      val (k, v) = kv
      //val withgl = (is: InputStream) => (g: GLInit) => reader(g, is)
      //val withgl = (is: InputStream) => (g: GLInit) => reader(g, is)
      k -> ((g: GLInit) => {
        val is = v()
        val useinput: (ManagedResource[InputStream]) => GLResult[T] = useInputStream(reader(g, _: InputStream))
        useinput(is)
      })
    }
    a
  }

  def decodeBitmap(config: Bitmap.Config)(stream: InputStream): GLResult[Bitmap] = {
    val options = new BitmapFactory.Options
    options.inPreferredConfig = config
    options.inScaled = false
    Option(BitmapFactory.decodeStream(stream, null, options)) match {
      case None => Left("unable to load bitmap!")
      case Some(bitmap) => {
        Log.i("drawfiles", "bitmap: config %s, w: %d, h: %d, alpha: %b".format(
          bitmap.getConfig(), bitmap.getHeight(), bitmap.getWidth(), bitmap.hasAlpha()))
        Right(bitmap)
      }
    }
  }

  def loadShader[T](c: Context, constructor: (GLInit, InputStream)=>GLResult[T], 
      folder: String, defaultName: String, defaultObj: Option[(GLInit)=>T]): Array[(String, (GLInit)=>GLResult[T])] = {
        val default: Option[(String, (GLInit)=>GLResult[T])] = defaultObj.map(x => (defaultName, (data: GLInit) => Right(x(data))))
    val filenamed = withFilename[T](constructor)
    val files = allfiles[T](c, folder)
    val shaders: Seq[(String, (GLInit)=>GLResult[T])] = files.map(filenamed)
    (default.toSeq ++ shaders).toArray
  }

  def loadBrushes(c: Context): Array[(String, (GLInit)=>GLResult[Texture])] = {
    val decoder: ((GLInit, InputStream)=>GLResult[Texture]) = (data: GLInit, is: InputStream) => (decodeBitmap(Bitmap.Config.ALPHA_8)(is).right.flatMap(Texture(data, _)))
    loadShader[Texture](c, decoder, "brushes", null, None)
  }

  // TODO: make these safe
  def loadPointShaders(c: Context): Seq[(String, (GLInit)=>GLResult[PointShader])] = {
    val constructor = readShader(PointShader.apply _) _
    loadShader[PointShader](c, constructor, "pointshaders", "Default Paint", Some((data: GLInit) => PointShader(data, null, null).right.get))
  }

  def loadAnimShaders(c: Context): Seq[(String, (GLInit)=>GLResult[CopyShader])] = {
    val constructor = readShader(CopyShader.apply _) _
    loadShader(c, constructor, "animshaders", "Default Animation", Some((data: GLInit) => CopyShader(data, null, null).right.get))
  }

  def loadScripts(c: Context): Seq[(String, (GLInit)=>GLResult[LuaScript])] = {
    val constructor = (data: GLInit, is: InputStream) => LuaScript(data, readStream(is))
    loadShader(c, constructor, "interpolators", "Default Interpolator", Some((data: GLInit) => LuaScript(data, null).right.get))
  }

  def loadUniBrushes(c: Context): Seq[(String, (GLInit)=>GLResult[UniBrush])] = {
    val constructor = UniBrush.compileFromStream _
    val defaultbrush = UniBrush(None, None, None, None, None, Array.empty)
    loadShader(c, constructor, "unibrushes", "Nothing", Some((data: GLInit) => defaultbrush))
  }

  def halfShaderPair(shader: String) = {
    if (shader.contains("gl_Position")) Some((shader, null))
    else if (shader.contains("gl_FragColor")) Some((null, shader))
    else None
  }

  def readShader[T](constructor: (GLInit, String, String)=>GLResult[T])(data: GLInit, src: InputStream): GLResult[T] = {
    halfShaderPair(readStream(src)) match {
      case Some((vec, frag)) => constructor(data, vec, frag)
      case None => Left("unable to load file")
    }
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
