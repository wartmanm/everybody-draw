package com.github.wartman4404.gldraw.unibrush

import java.io.File
import java.io.IOException
import android.graphics.Bitmap
import android.util.Log

import spray.json._
import org.parboiled.errors.ParsingException


import com.github.wartman4404.gldraw._

case class ShaderSource(
  fragmentshader: Option[String],
  vertexshader: Option[String]
) {
  def compile[T](data: GLInit, compiler: Shader[T]) = {
    compiler(data, vertexshader.getOrElse(null), fragmentshader.getOrElse(null))
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

  def compile(data: GLInit, s: String, path: String): Option[UniBrush] = {
    try {
      compile(data, s.parseJson.convertTo[UniBrushSource], path)
    } catch {
      case e: ParsingException => None
      case e: IOException => None
    }
  }

  def compile(data: GLInit, s: UniBrushSource, path: String): Option[UniBrush] = {
    Log.i("unibrush", "compiling unibrush");
    val brush = {
      s.brushpath.map(bp => {
          DrawFiles.withFileStream(new File(path, bp))
          .map(DrawFiles.decodeBitmap(Bitmap.Config.ALPHA_8) _).opt.flatten
          .map(Texture.apply(data, _))
          .getOrElse(return None)
        })
    }
    val interpolator = s.interpolator.map(LuaScript(data, _).getOrElse(return None))
    val pointshader = s.pointshader.map(_.compile(data, PointShader).getOrElse(return None))
    val animshader = s.animshader.map(_.compile(data, CopyShader).getOrElse(return None))
    val separateLayer = s.separatelayer.getOrElse(false)
    Log.i("unibrush", s"have interpolator: ${interpolator.nonEmpty}");
    Log.i("unibrush", s"have pointshader: ${pointshader.nonEmpty}");
    Log.i("unibrush", s"have animshader: ${animshader.nonEmpty}");
    Log.i("unibrush", s"have separateLayer: ${separateLayer}");
    Log.i("unibrush", s"have brush: ${brush.nonEmpty}");
    Some(UniBrush(brush, pointshader, animshader, interpolator, separateLayer))
  }
}
