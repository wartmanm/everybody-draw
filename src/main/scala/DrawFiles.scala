package com.github.wartman4404.gldraw

import android.content.Context
import java.io.{InputStream, BufferedInputStream, FileInputStream, File, Closeable}
import java.util.zip.ZipFile
import android.graphics.{Bitmap, BitmapFactory}
import android.os.Build

import android.util.Log

import unibrush.{UniBrush, UniBrushSource}
import scala.annotation.tailrec
import GLResultTypeDef._

object DrawFiles {
  trait NamedSource {
    def read(filename: String): InputStream
  }

  trait PartialReader[T, U] {
    def readSource(i: InputStream): T
    def compile(g: GLInit, source: T): GLResult[U]
  }

  sealed trait ReadState[U] { }
  class Readable[U](path: String, private var state: ReadState[U]) {
    Log.i("drawfiles", s"created readable ${this}")
    val name = {
      val pathsep = path.lastIndexOf('/')
      val extsep = path.lastIndexOf('.')
      val end = if (extsep == -1 || extsep < pathsep) path.length else extsep
      path.substring(pathsep + 1, end)
    }
    type BaseUnread = DrawFiles.BaseUnread[_, U]
    type PartiallyRead = DrawFiles.PartiallyRead[_, U]
    type FullyRead = DrawFiles.FullyRead[U]
    type FailedRead = DrawFiles.FailedRead[U]
    def read() = {
      Log.i("drawfiles", s"readable ${this} .read()")
      state = state match {
        case s: BaseUnread => try {
          s.read(path)
        } catch {
          case e: Exception => {
            new FailedRead(e)
          }
        }
        case other => other
      }
    }

    @tailrec
    final def compile(gl: GLInit): U = {
      Log.i("drawfiles", s"readable ${this} .compile()")
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
              state = new FailedRead(e)
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
      case s: FullyRead => {
        Log.i("drawfiles", s"readable ${this}: FullyRead: ${s.content}")
        Right(s.content)
      }
      case s: FailedRead => Left(s.error.toString())
      case _ => {
        try {
          Log.i("drawfiles", s"readable ${this}: not fully read yet")
          this.compile(gl)
        } catch {
          case e: Exception => {
            Log.i("drawfiles", s"readable ${this}: failed to read")
          }
        }
        compileSafe(gl)
      }
    }

    def isNotFailed = !state.isInstanceOf[FailedRead]
  }

  abstract class BaseUnread[T, U] extends ReadState[U] {
    def read(path: String): PartiallyRead[T, U]
    def toReadable(name: String): Readable[U] = new Readable(name, this)
  }

  class Unread[T, U](source: NamedSource, reader: PartialReader[T, U]) extends BaseUnread[T,U] {
    def read(path: String) = {
      val stream = source.read(path)
      try {
        new PartiallyRead(reader, reader.readSource(stream))
      } catch {
        case e: Exception => {
          stream.close()
          throw e
        }
      }
    }
  }
  class DefaultUnread[T <: AnyRef, U](reader: PartialReader[T, U]) extends BaseUnread[T, U] {
    def read(path: String) = {
      val content: T = null.asInstanceOf[T]
      new PartiallyRead(reader, content)
    }
  }

  class PartiallyRead[T, U](reader: PartialReader[T, U], val content: T) extends ReadState[U] {
    def compile(g: GLInit) = new FullyRead(reader.compile(g, content))
  }

  class FullyRead[U](val content: U) extends ReadState[U]
  class FailedRead[U](val error: Exception) extends ReadState[U]

  class AssetStreamSource(c: Context) extends NamedSource {
    def read(path: String) = c.getAssets().open(path)
  }
  
  object FileSource extends NamedSource {
    def read(path: String) = new BufferedInputStream(new FileInputStream(path))
  }

  def externalfiles(c: Context, path: String): Array[File] = {
    val userdirs = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.KITKAT) {
      c.getExternalFilesDirs(path)
    } else {
      Array(c.getExternalFilesDir(path))
    }
    val result = userdirs.flatMap (subpath => {
      val files = if (subpath != null) subpath.listFiles() else null // some paths may be null??
      val result: Array[File] = if (files != null) files else Array()
      result
    })
    result
  }

  type MaybeRead[T] = (InputStream)=>GLResult[T]
  type MaybeReader[T] = (MaybeRead[T])=>GLResult[T]

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

  class BitmapReader(config: Bitmap.Config) extends PartialReader[Bitmap, Texture] {
    override def readSource(i: InputStream) = decodeBitmap(config)(i)
    override def compile(g: GLInit, source: Bitmap) = Texture(g, source)
  }
  
  val BitmapReaderAlpha = new BitmapReader(Bitmap.Config.ALPHA_8)
  
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

class LoadedDrawFiles(c: Context, useExternal: Boolean) {
  import DrawFiles._
  private val assetStreamSource = new AssetStreamSource(c)
  private def allfiles[T](builtins: PreinstalledPaintResources.Dir, constructor: PartialReader[_, T], default: Readable[T]): Array[Readable[T]] = {
    // type must be specified or Array() will return an Array[Object], this is probably a compiler bug
    val userfiles: Array[File] =
      if (useExternal) externalfiles(c, builtins.name)
      else Array()
    var i = if (default != null) 1 else 0
    val builtinpaths = builtins.builtin
    val readers = new Array[Readable[T]](builtinpaths.length + userfiles.length + i)
    readers(0) = default
    var bi = 0
    while (bi < builtinpaths.length) {
      readers(i) = new Unread(assetStreamSource, constructor).toReadable(builtinpaths(bi))
      bi += 1
      i += 1
    }
    var fi = 0
    while (fi < userfiles.length) {
      readers(i) = new Unread(FileSource, constructor).toReadable(userfiles(fi).getAbsolutePath())
      fi += 1
      i += 1
    }
    readers
  }

  val brushes: Array[Readable[Texture]] = {
    val constructor = BitmapReaderAlpha
    allfiles[Texture](PreinstalledPaintResources.brushes, constructor, null)
  }

  // TODO: make these safe
  val paints: Array[Readable[PointShader]] = {
    val constructor = new ShaderReader(PointShader.apply _)
    val default = new DefaultUnread(constructor).toReadable("Default Paint")
    val files = allfiles[PointShader](PreinstalledPaintResources.pointshaders, constructor, default)
    files
  }

  val anims: Array[Readable[CopyShader]] = {
    val constructor = new ShaderReader(CopyShader.apply _)
    val default = new DefaultUnread(constructor).toReadable("Default Animation")
    val files = allfiles[CopyShader](PreinstalledPaintResources.animshaders, constructor, default)
    files
  }

  val interpscripts: Array[Readable[LuaScript]] = {
    val default = new DefaultUnread(LuaReader).toReadable("Default Interpolator")
    val files = allfiles[LuaScript](PreinstalledPaintResources.interpolators, LuaReader, default)
    files
  }

  val unibrushes: Array[Readable[UniBrush]] = {
    val default = new DefaultUnread(DefaultUniBrush).toReadable("Nothing")
    val files = allfiles[UniBrush](PreinstalledPaintResources.unibrushes, UniBrushReader, default)
    files
  }
}
