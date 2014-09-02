package com.github.wartman4404.gldraw

import java.io.File
import android.graphics.Bitmap
import android.util.Log

import spray.json._

object UniBrush extends AutoProductFormat {

  case class ShaderSource(
    fragmentshader: Option[String],
    vertexshader: Option[String]
  ) {
    def compile[T](compiler: Shader[T]) = {
      compiler(vertexshader.getOrElse(null), fragmentshader.getOrElse(null))
    }
  }

  case class UniBrushSource (
    brushpath: Option[String],
    pointshader: Option[ShaderSource],
    animshader: Option[ShaderSource],
    interpolator: Option[String],
    separatelayer: Option[Boolean]
  )

  case class UniBrush(
    brush: Option[Texture],
    pointshader: Option[PointShader],
    animshader: Option[CopyShader],
    interpolator: Option[LuaScript],
    separatelayer: Boolean)

  def compile(s: String, path: String): Option[UniBrush] = compile(s.parseJson.convertTo[UniBrushSource], path)

  def compile(s: UniBrushSource, path: String): Option[UniBrush] = {
    Log.i("unibrush", "compiling unibrush");
    val brush = {
      s.brushpath.map(bp => {
          DrawFiles.withFileStream(new File(path, bp))
          .map(DrawFiles.decodeBitmap(Bitmap.Config.ALPHA_8) _).opt.flatten
          .map(Texture.apply _)
          .getOrElse(return None)
        })
    }
    val interpolator = s.interpolator.map(LuaScript(_).getOrElse(return None))
    val pointshader = s.pointshader.map(_.compile(PointShader).getOrElse(return None))
    val animshader = s.animshader.map(_.compile(CopyShader).getOrElse(return None))
    val separateLayer = s.separatelayer.getOrElse(false)
    Log.i("unibrush", s"have interpolator: ${interpolator.nonEmpty}");
    Log.i("unibrush", s"have pointshader: ${pointshader.nonEmpty}");
    Log.i("unibrush", s"have animshader: ${animshader.nonEmpty}");
    Log.i("unibrush", s"have separateLayer: ${separateLayer}");
    Log.i("unibrush", s"have brush: ${brush.nonEmpty}");
    Some(UniBrush(brush, pointshader, animshader, interpolator, separateLayer))
  }
}



