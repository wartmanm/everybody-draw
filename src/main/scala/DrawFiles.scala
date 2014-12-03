package com.github.wartman4404.gldraw

import android.content.Context
import java.io.{InputStream, BufferedInputStream, FileInputStream, File}
import java.util.zip.ZipFile
import android.graphics.{Bitmap, BitmapFactory}

import android.util.Log

import resource._

import unibrush.UniBrush
import GLResultTypeDef._

object DrawFiles {
  trait NamedSource {
    val filename: String
    def read(): InputStream
  }

  trait PartialReader[T, U] {
    def readSource(i: InputStream): T
    def compile(g: GLInit, source: T): GLResult[U]
  }

  abstract class BaseUnread[T, U] {
    val name: String
    def read(): T
  }

  class Unread[T, U](source: NamedSource, reader: PartialReader[T, U]) extends BaseUnread[T,U] {
    val name = source.filename
    def read() = {
      val stream = source.read()
      try {
        new PartiallyRead(name, reader, reader.readSource(stream))
      } catch {
        case e: Exception => {
          stream.close()
          throw e
        }
      }
    }
  }
  class DefaultUnread[T <: AnyRef, U](val name: String, reader: PartialReader[T, U]) extends BaseUnread[T, U] {
    def read() = {
      new PartiallyRead(name, reader, null)
    }
  }

  class PartiallyRead[T, U](val name: String, reader: PartialReader[T, U], content: T) {
    def compile(g: GLInit) = new FullyRead(name, reader.compile(g, content))
  }
  //object PartiallyRead {
    //def default[T <: AnyRef, U](name: String, reader: PartialReader[T, U]) = {
      //new PartiallyRead(name, reader, null)
    //}
  //}

  class FullyRead[U](val name: String, val content: U)

  class AssetStreamSource(c: Context, path: String) extends NamedSource {
    val filename = path
    def read() = c.getAssets().open(path)
  }
  
  class FileSource(file: File) extends NamedSource {
    val filename = file.getName()
    def read() = new BufferedInputStream(new FileInputStream(file))
  }

  type MaybeRead[T] = (InputStream)=>GLResult[T]
  type MaybeReader[T] = (MaybeRead[T])=>GLResult[T]
  def allfiles[T](c: Context, path: String): Array[NamedSource] = {
    val builtins = c.getAssets().list(path).map(path ++ "/" ++ _)
    val userdirs = c.getExternalFilesDirs(path).filter(_ != null) // some paths may be null??
    val userfiles = userdirs.flatMap(_.listFiles())
    val builtinOpeners = builtins.map(new AssetStreamSource(c, _))
    val fileOpeners = userfiles.map(new FileSource(_))
    (builtinOpeners ++ fileOpeners)
  }

  def decodeBitmap(config: Bitmap.Config)(stream: InputStream): GLResult[Bitmap] = {
    val options = new BitmapFactory.Options
    options.inPreferredConfig = config
    options.inScaled = false
    Option(BitmapFactory.decodeStream(stream, null, options)) match {
      case None => throw new GLException("unable to load bitmap!")
      case Some(bitmap) => {
        Log.i("drawfiles", "bitmap: config %s, w: %d, h: %d, alpha: %b".format(
          bitmap.getConfig(), bitmap.getHeight(), bitmap.getWidth(), bitmap.hasAlpha()))
        bitmap
      }
    }
  }

  object BitmapReader extends PartialReader[Bitmap, Texture] {
    override def readSource(i: InputStream) = decodeBitmap(Bitmap.Config.ALPHA_8, i)
    override def compile(g: GLInit, source: Texture) = Texture(g, source)
  }
  
  class ShaderReader[T](constructor: (GLInit, String, String)=>GLResult[T]) extends PartialReader[String, T] {
    override def readSource(i: InputStream) = readStream(i)
    override def compile(g: GLInit, source: String) = {
      halfShaderPair(source) match {
        case Some((vec, frag)) => constructor(data, vec, frag)
        case None => throw new GLException("unable to load file")
      }
    }
  }

  object LuaReader extends PartialReader[String, LuaScript] {
    override def readSource(i: InputStream) = readStream(i)
    override def compile(g: GLInit, source: String) = LuaScript(data, source)
  }

  object UniBrushReader extends PartialReader[UniBrushSource, UniBrush] {
    override def readSource(i: InputStream) = UniBrush.readFromStream(i)
    override def compile(g: GLInit, source: UniBrushSource) = UniBrush.compileFromSource(source)
  }
  object DefaultUniBrush extends PartialReader[UniBrushSource, UniBrush] {
    override def readSource(i: InputStream) = null
    override def compile(g: GLInit, source: UniBrushSource) = UniBrush(None, None, None, None, None, Array.empty)
  }

  def loadShader[T](c: Context, constructor: (GLInit, InputStream)=>GLResult[T], 
      folder: String, defaultName: String, defaultObj: Option[(GLInit)=>T]): Array[(String, (GLInit)=>GLResult[T])] = {
        val default: Option[(String, (GLInit)=>GLResult[T])] = defaultObj.map(x => (defaultName, (data: GLInit) => x(data)))
    val files = allfiles[T](c, folder)
    (default.toArray ++ shaders).toArray
  }

  def loadBrushes(c: Context): Array[(String, (GLInit)=>GLResult[Texture])] = {
    val files = allfiles[Texture](c, "brushes")
    files.map(new Unread(_, BitmapReader))
  }

  // TODO: make these safe
  def loadPointShaders(c: Context): Seq[(String, (GLInit)=>GLResult[PointShader])] = {
    val constructor = new ShaderReader(PointShader.apply _)
    val files = allfiles[PointShader](c, "pointshaders")
    files.map(new Unread(_, constructor)) +: new DefaultUnread("Default Paint", constructor)
  }

  def loadAnimShaders(c: Context): Seq[(String, (GLInit)=>GLResult[CopyShader])] = {
    val constructor = new ShaderReader(CopyShader.apply _)
    val files = allfiles[CopyShader](c, "animshaders")
    files.map(new Unread(_, constructor)) +: new DefaultUnread("Default Animation", constructor)
  }

  def loadScripts(c: Context): Seq[(String, (GLInit)=>GLResult[LuaScript])] = {
    val files = allfiles[LuaScript](c, "interpolators")
    files.map(new Unread(_, LuaReader)) +: new DefaultUnread("Default Interpolator", LuaReader)
  }

  def loadUniBrushes(c: Context): Seq[(String, (GLInit)=>GLResult[UniBrush])] = {
    val files = allfiles[UniBrush](c, "unibrushes")
    files.map(new Unread(_, UniBrushReader) +: new DefaultUnread("Nothing", DefaultUniBrush))
  }

  def halfShaderPair(shader: String) = {
    if (shader == null) Some((null, null))
    else if (shader.contains("gl_Position")) Some((shader, null))
    else if (shader.contains("gl_FragColor")) Some((null, shader))
    else None
  }

  def readShader[T](constructor: (GLInit, String, String)=>GLResult[T])(data: GLInit, src: InputStream): GLResult[T] = {
    halfShaderPair(readStream(src)) match {
      case Some((vec, frag)) => constructor(data, vec, frag)
      case None => throw new GLException("unable to load file")
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
