package com.github.wartman4404.gldraw

import android.content.Context
import java.io.{InputStream, BufferedInputStream, FileInputStream, File, Closeable}
import java.util.zip.ZipFile
import android.graphics.{Bitmap, BitmapFactory}

import android.util.Log

import unibrush.{UniBrush, UniBrushSource}
import scala.annotation.tailrec
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

  sealed trait ReadState[U] {
    val name: String
  }
  class Readable[U](private var state: ReadState[U]) {
    type BaseUnread = DrawFiles.BaseUnread[_, U]
    type PartiallyRead = DrawFiles.PartiallyRead[_, U]
    type FullyRead = DrawFiles.FullyRead[U]
    type FailedRead = DrawFiles.FailedRead[U]
    val name = state.name
    def read() = {
      state = state match {
        case s: BaseUnread => try {
          s.read()
        } catch {
          case e: Exception => {
            new FailedRead(s.name, e)
          }
        }
        case other => other
      }
    }

    @tailrec
    final def compile(gl: GLInit): U = {
      state match {
        case s: BaseUnread => {
          this.read()
          this.compile(gl)
        }
        case s: PartiallyRead => {
          val compiled =
          try {
            s.compile(gl)
          } catch {
            case e: Exception => {
              state = new FailedRead(s.name, e)
              throw e
            }
          }
          state = compiled
          compiled.content
        }
        case s: FullyRead => s.content
        case s: FailedRead => throw s.error
      }
    }

    @tailrec
    final def compileSafe(gl: GLInit): GLStoredResult[U] = state match {
      case s: FullyRead => Right(s.content)
      case s: FailedRead => Left(s.error.toString())
      case _ => {
        try {
          this.compile(gl)
        } catch {
          case e: Exception => { }
        }
        compileSafe(gl)
      }
    }

    def isNotFailed = !state.isInstanceOf[FailedRead]
  }

  abstract class BaseUnread[T, U] extends ReadState[U] {
    val name: String
    def read(): PartiallyRead[T, U]
    def toReadable: Readable[U] = new Readable(this)
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
      val content: T = null.asInstanceOf[T]
      new PartiallyRead(name, reader, content)
    }
  }

  class PartiallyRead[T, U](val name: String, reader: PartialReader[T, U], val content: T) extends ReadState[U] {
    def compile(g: GLInit) = new FullyRead(name, reader.compile(g, content))
  }
  //object PartiallyRead {
    //def default[T <: AnyRef, U](name: String, reader: PartialReader[T, U]) = {
      //new PartiallyRead(name, reader, null)
    //}
  //}

  class FullyRead[U](val name: String, val content: U) extends ReadState[U]
  class FailedRead[U](val name: String, val error: Exception) extends ReadState[U]


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
  def allfiles[T](c: Context, builtins: PreinstalledPaintResources.Dir, constructor: PartialReader[_, T], default: DefaultUnread[_, T]): Array[Readable[T]] = {
    val userdirs = c.getExternalFilesDirs(builtins.name).filter(_ != null) // some paths may be null??
    val userfiles = userdirs.flatMap(_.listFiles())
    var i = if (default != null) 1 else 0
    val builtinpaths = builtins.builtin
    val readers = new Array[Readable[T]](builtinpaths.length + userfiles.length + i)
    readers(0) = if (default != null) default.toReadable else null
    var bi = 0
    while (bi < builtinpaths.length) {
      readers(i) = new Unread(new AssetStreamSource(c, builtinpaths(bi)), constructor).toReadable
      bi += 1
      i += 1
    }
    var fi = 0
    while (fi < userfiles.length) {
      readers(i) = new Unread(new FileSource(userfiles(fi)), constructor).toReadable
      fi += 1
      i += 1
    }
    readers
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
    override def readSource(i: InputStream) = decodeBitmap(Bitmap.Config.ALPHA_8)(i)
    override def compile(g: GLInit, source: Bitmap) = Texture(g, source)
  }
  
  class ShaderReader[T](constructor: (GLInit, String, String)=>GLResult[T]) extends PartialReader[String, T] {
    override def readSource(i: InputStream) = readStream(i)
    override def compile(g: GLInit, source: String) = {
      halfShaderPair(source) match {
        case Some((vec, frag)) => constructor(g, vec, frag)
        case None => throw new GLException("unable to load file")
      }
    }
  }

  object LuaReader extends PartialReader[String, LuaScript] {
    override def readSource(i: InputStream) = readStream(i)
    override def compile(g: GLInit, source: String) = LuaScript(g, source)
  }

  object UniBrushReader extends PartialReader[UniBrushSource, UniBrush] {
    override def readSource(i: InputStream) = UniBrush.readFromStream(i)
    override def compile(g: GLInit, source: UniBrushSource) = UniBrush.compileFromSource(g, source)
  }
  object DefaultUniBrush extends PartialReader[UniBrushSource, UniBrush] {
    override def readSource(i: InputStream) = null
    override def compile(g: GLInit, source: UniBrushSource) = UniBrush(None, None, None, None, None, Array.empty)
  }

  def loadBrushes(c: Context): Array[Readable[Texture]] = {
    val files = allfiles[Texture](c, PreinstalledPaintResources.brushes, BitmapReader, null)
    files
  }

  // TODO: make these safe
  def loadPointShaders(c: Context): Array[Readable[PointShader]] = {
    val constructor = new ShaderReader(PointShader.apply _)
    val default = new DefaultUnread("Default Paint", constructor)
    val files = allfiles[PointShader](c, PreinstalledPaintResources.pointshaders, constructor, default)
    files
  }

  def loadAnimShaders(c: Context): Array[Readable[CopyShader]] = {
    val constructor = new ShaderReader(CopyShader.apply _)
    val default = new DefaultUnread("Default Animation", constructor)
    val files = allfiles[CopyShader](c, PreinstalledPaintResources.animshaders, constructor, default)
    files
  }

  def loadScripts(c: Context): Array[Readable[LuaScript]] = {
    val default = new DefaultUnread("Default Interpolator", LuaReader)
    val files = allfiles[LuaScript](c, PreinstalledPaintResources.interpolators, LuaReader, default)
    files
  }

  def loadUniBrushes(c: Context): Array[Readable[UniBrush]] = {
    val default = new DefaultUnread("Nothing", DefaultUniBrush)
    val files = allfiles[UniBrush](c, PreinstalledPaintResources.unibrushes, UniBrushReader, default)
    files
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

  def withCloseable[T](c: Closeable)(cb: =>T) = {
    try {
      cb
    } catch {
      case e: Exception => {
        try {
          c.close()
        } catch { case _: Exception => { } }
        throw e
      }
    }
  }
}
