package com.github.wartman4404.gldraw.unibrush

import java.io.{File, IOException, InputStream, ByteArrayOutputStream, ByteArrayInputStream}
import java.util.zip.{ZipEntry, ZipInputStream}
import android.graphics.Bitmap
import android.util.Log

import scala.collection.mutable
import scala.annotation.tailrec

import spray.json._
import org.parboiled.errors.ParsingException

import com.github.wartman4404.gldraw._

import GLResultTypeDef._

case class ShaderSource(
  fragmentshader: Option[String],
  vertexshader: Option[String]
) {
  def compile[T](data: GLInit, compiler: Shader[T], files: Map[String, Array[Byte]]): GLResult[T] = {
    val Seq(frag, vert) = for (path <- Seq(fragmentshader, vertexshader)) yield {
      path.map(x => new String(files.get(x).getOrElse(return UniBrush.logAbort(s"missing shader file ${x}")))).getOrElse(null)
    }
    compiler(data, vert, frag)
  }
}

case class LayerSource(
  pointshader: Option[Int],
  copyshader: Option[Int],
  pointsrc: Option[Int]
)

case class Layer(
  pointshader: PointShader,
  copyshader: CopyShader,
  pointsrc: Int
)

case class UniBrushSource (
  brushpath: Option[String],
  pointshaders: Option[Array[ShaderSource]],
  animshaders: Option[Array[ShaderSource]],
  basepointshader: Option[ShaderSource],
  baseanimshader: Option[ShaderSource],
  interpolator: Option[String],
  layers: Option[Array[LayerSource]]
)

case class UniBrush(
  brush: Option[Texture],
  basepointshader: Option[PointShader],
  baseanimshader: Option[CopyShader],
  interpolator: Option[LuaScript],
  layers: Array[Layer])

object UniBrush extends AutoProductFormat {
  def logAbort[T](s: String): GLResult[T] = {
    Log.e("unibrush", s"failed to load: ${s}")
    Left(s)
  }

  // iterator to unzip everything into memory
  // this is incredibly wasteful, even more so because the files still have to
  // be converted to strings/bitmaps
  // it would be way better to read the compressed zipfile into memory instead
  // but that involves third-party libraries and looks fussy
  private implicit class ZipInputStream2Iterator(zis: ZipInputStream) extends Iterable[(ZipEntry, Array[Byte])] {
    def iterator = new ZipInputStreamIterator(zis)
  }
  class ZipInputStreamIterator(zis: ZipInputStream) extends Iterator[(ZipEntry, Array[Byte])] {
    private var nextEntry = zis.getNextEntry()
    private var baos = new ByteArrayOutputStream()
    private val ba = new Array[Byte](8192)
    def hasNext =
      if (nextEntry == null) { zis.close(); false }
      else true

    @tailrec final def next(): (ZipEntry, Array[Byte]) = {
      val readBytes = zis.read(ba, 0, ba.length)
      if (readBytes == -1) {
        baos.flush()
        val oldBytes = baos.toByteArray()
        val oldEntry = nextEntry
        zis.closeEntry()
        nextEntry = zis.getNextEntry()
        baos = new ByteArrayOutputStream()
        Log.i("unibrush", s"read ${oldEntry.getName()}: ${oldBytes.length} bytes")
        (oldEntry, oldBytes)
      } else {
        baos.write(ba, 0, readBytes)
        this.next()
      }
    }
  }

  def compile(data: GLInit, sourceZip: InputStream): GLResult[UniBrush] = {
    Log.i("unibrush", "loading unibrush")
    try {
      val files = new ZipInputStream(sourceZip)
        .map { case (entry, bytes) => (entry.getName(), bytes) }
        .toMap
      val brushjson = files.get("brush.json").getOrElse(return logAbort("unable to find brush.json"))
      Log.i("unibrush", "got brush.json")
      compile(data, new String(brushjson).parseJson.convertTo[UniBrushSource], files)
    } catch {
      case e @ (_: ParsingException | _: DeserializationException) => {
        logAbort(s"unable to parse brush.json: ${e}")
      }
      case e: IOException => logAbort(s"IOException ${e}")
      case e: Exception => logAbort(s"Other exception ${e}")
    }
  }

  def compile(data: GLInit, s: UniBrushSource, files: Map[String, Array[Byte]]): GLResult[UniBrush] = {
    Log.i("unibrush", "compiling unibrush");
    val brush = {
      s.brushpath.map(bp => {
          files.get(bp)
          .map(new ByteArrayInputStream(_))
          .flatMap(x => DrawFiles.decodeBitmap(Bitmap.Config.ALPHA_8)(x).right.toOption)
          .map(Texture.apply(data, _))
          .getOrElse(return logAbort(s"missing brush file ${bp}"))
        })
    }
    val pointshaders: Array[PointShader] = s.pointshaders.getOrElse(Array.empty).map(_.compile(data, PointShader, files).right.getOrElse(return logAbort ("pointshader failed to compile")))
    val copyshaders: Array[CopyShader] = s.animshaders.getOrElse(Array.empty).map(_.compile(data, CopyShader, files).right.getOrElse(return logAbort ("copyshader failed to compile")))
    val baseanimshader = s.baseanimshader.map(_.compile(data, CopyShader, files).right.getOrElse(return logAbort("base animation shader failed to compile")))
    val basepointshader = s.basepointshader.map(_.compile(data, PointShader, files).right.getOrElse(return logAbort("base point shader failed to compile")))
    val interpolator = s.interpolator.map(x => new String(files.get(x).getOrElse(return logAbort (s"missing interpolator file ${x}")))).map(LuaScript(data, _).right.getOrElse(return logAbort("interpolator failed to compile")))
    val layers = s.layers.getOrElse(Array.empty).map(l => {
        val point = l.pointshader.map(x => pointshaders.lift(x).getOrElse(return logAbort(s"no point shader numbered ${x}"))).getOrElse(PointShader(data, null, null).right.get)
        val copy = l.copyshader.map(x => copyshaders.lift(x).getOrElse(return logAbort(s"no copy shader numbered ${x}"))).getOrElse(CopyShader(data, null, null).right.get)
        val idx = l.pointsrc.getOrElse(0)
        Layer(point, copy, idx)
      })

    Log.i("unibrush", s"have interpolator: ${interpolator.nonEmpty}");
    Log.i("unibrush", s"have pointshader: ${basepointshader.nonEmpty}");
    Log.i("unibrush", s"have animshader: ${baseanimshader.nonEmpty}");
    Log.i("unibrush", s"have layers: ${layers.length}");
    Log.i("unibrush", s"have brush: ${brush.nonEmpty}");
    Right(UniBrush(brush, basepointshader, baseanimshader, interpolator, layers))
  }
}
