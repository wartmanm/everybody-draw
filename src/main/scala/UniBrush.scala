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

case class UniBrushSource (
  brushpath: Option[String],
  pointshader: Option[ShaderSource],
  animshader: Option[ShaderSource],
  interpolator: Option[String],
  separatelayer: Option[Boolean])

case class UniBrush(
  brush: Option[Texture],
  pointshader: Option[PointShader],
  animshader: Option[CopyShader],
  interpolator: Option[LuaScript],
  separatelayer: Boolean)

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
    val interpolator = s.interpolator.flatMap(zr).map(LuaScript(data, _).getOrElse(return None))
    val pointshader = s.pointshader.map(_.compile(data, PointShader, sourceZip).getOrElse(return None))
    val animshader = s.animshader.map(_.compile(data, CopyShader, sourceZip).getOrElse(return None))
    val separateLayer = s.separatelayer.getOrElse(false)
    Log.i("unibrush", s"have interpolator: ${interpolator.nonEmpty}");
    Log.i("unibrush", s"have pointshader: ${pointshader.nonEmpty}");
    Log.i("unibrush", s"have animshader: ${animshader.nonEmpty}");
    Log.i("unibrush", s"have separateLayer: ${separateLayer}");
    Log.i("unibrush", s"have brush: ${brush.nonEmpty}");
    Some(UniBrush(brush, pointshader, animshader, interpolator, separateLayer))
  }
}
