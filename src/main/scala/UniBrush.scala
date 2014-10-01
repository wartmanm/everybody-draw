package com.github.wartman4404.gldraw.unibrush

import java.io.File
import java.io.IOException
import android.graphics.Bitmap
import android.util.Log

import spray.json._
import org.parboiled.errors.ParsingException
import java.util.zip.ZipFile


import com.github.wartman4404.gldraw._

case class ShaderSource(
  fragmentshader: Option[String],
  vertexshader: Option[String]
) {
  def compile[T](data: GLInit, compiler: Shader[T], sourceZip: ZipFile) = {
    val Seq(frag, vert) = for (path <- Seq(fragmentshader, vertexshader)) yield {
      path.flatMap(DrawFiles.readZip(sourceZip, _)).getOrElse(null)
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
  def compile(data: GLInit, sourceZip: ZipFile): Option[UniBrush] = {
    try {
      DrawFiles.readZip(sourceZip, "brush.json").flatMap(s => {
          compile(data, s.parseJson.convertTo[UniBrushSource], sourceZip)
        })
    } catch {
      case e: ParsingException => None
      case e: IOException => None
    }
  }

  def compile(data: GLInit, s: UniBrushSource, sourceZip: ZipFile): Option[UniBrush] = {
    Log.i("unibrush", "compiling unibrush");
    val brush = {
      s.brushpath.map(bp => {
          Option(sourceZip.getEntry(bp))
          .map(ze => sourceZip.getInputStream(ze))
          .flatMap(DrawFiles.decodeBitmap(Bitmap.Config.ALPHA_8) _)
          .map(Texture.apply(data, _))
          .getOrElse(return None)
        })
    }
    val zr = DrawFiles.readZip(sourceZip, _: String)
    val pointshaders: Array[PointShader] = s.pointshaders.getOrElse(Array.empty).map(_.compile(data, PointShader, sourceZip).getOrElse(return None))
    val copyshaders: Array[CopyShader] = s.animshaders.getOrElse(Array.empty).map(_.compile(data, CopyShader, sourceZip).getOrElse(return None))
    val baseanimshader = s.baseanimshader.map(_.compile(data, CopyShader, sourceZip).getOrElse(return None))
    val basepointshader = s.basepointshader.map(_.compile(data, PointShader, sourceZip).getOrElse(return None))
    val interpolator = s.interpolator.map(zr(_).getOrElse(return None)).map(LuaScript(data, _).getOrElse(return None))
    val layers = s.layers.getOrElse(Array.empty).map(l => {
        val point = l.pointshader.map(pointshaders.lift(_).getOrElse(return None)).getOrElse(PointShader(data, null, null).get)
        val copy = l.copyshader.map(copyshaders.lift(_).getOrElse(return None)).getOrElse(CopyShader(data, null, null).get)
        val idx = l.pointsrc.getOrElse(0)
        Layer(point, copy, idx)
      })

    Log.i("unibrush", s"have interpolator: ${interpolator.nonEmpty}");
    Log.i("unibrush", s"have pointshader: ${basepointshader.nonEmpty}");
    Log.i("unibrush", s"have animshader: ${baseanimshader.nonEmpty}");
    Log.i("unibrush", s"have layers: ${layers.length}");
    Log.i("unibrush", s"have brush: ${brush.nonEmpty}");
    Some(UniBrush(brush, basepointshader, baseanimshader, interpolator, layers))
  }
}
