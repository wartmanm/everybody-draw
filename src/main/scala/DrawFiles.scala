package com.github.wartman4404.gldraw

import android.content.Context
import java.io.{InputStream, BufferedInputStream, FileInputStream, File}
import java.util.zip.ZipFile
import android.graphics.{Bitmap, BitmapFactory}

import android.util.Log

import resource._

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
  class Readable[T, U](private var state: ReadState[U]) {
    type BaseUnread = DrawFiles.BaseUnread[T, U]
    type PartiallyRead = DrawFiles.PartiallyRead[T, U]
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
    def toReadable: Readable[T, U] = new Readable(this)
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

  def loadBrushes(c: Context): Array[Readable[Bitmap, Texture]] = {
    val files = allfiles[Texture](c, "brushes")
    files.map(new Unread(_, BitmapReader).toReadable)
  }

  // TODO: make these safe
  def loadPointShaders(c: Context): Array[Readable[String, PointShader]] = {
    val constructor = new ShaderReader(PointShader.apply _)
    val files = allfiles[PointShader](c, "pointshaders")
    files.map(new Unread(_, constructor).toReadable) :+ new DefaultUnread("Default Paint", constructor).toReadable
  }

  def loadAnimShaders(c: Context): Array[Readable[String, CopyShader]] = {
    val constructor = new ShaderReader(CopyShader.apply _)
    val files = allfiles[CopyShader](c, "animshaders")
    files.map(new Unread(_, constructor).toReadable) :+ new DefaultUnread("Default Animation", constructor).toReadable
  }

  def loadScripts(c: Context): Array[Readable[String, LuaScript]] = {
    val files = allfiles[LuaScript](c, "interpolators")
    files.map(new Unread(_, LuaReader).toReadable) :+ new DefaultUnread("Default Interpolator", LuaReader).toReadable
  }

  def loadUniBrushes(c: Context): Array[Readable[UniBrushSource, UniBrush]] = {
    val files = allfiles[UniBrush](c, "unibrushes")
    files.map(new Unread(_, UniBrushReader).toReadable) :+ new DefaultUnread("Nothing", DefaultUniBrush).toReadable
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
