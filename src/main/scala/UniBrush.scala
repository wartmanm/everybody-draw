package com.github.wartman4404.gldraw

import java.io.File
import android.graphics.Bitmap

import org.json4s._
import org.json4s.jackson.JsonMethods._

object UniBrush {
  implicit lazy val formats = DefaultFormats

  case class ShaderSource(
    `fragment-shader`: Option[String],
    `vertex-shader`: Option[String]
  ) {
    def compile[T](compiler: (String, String) => Option[T]) = {
      compiler(`vertex-shader`.getOrElse(null), `fragment-shader`.getOrElse(null))
    }
  }
  
  case class UniBrushSource(
    `brush-path`: Option[String],
    `point-shader`: Option[ShaderSource],
    `anim-shader`: Option[ShaderSource],
    interpolator: Option[String],
    `separate-layer`: Option[Boolean]
  )

  class UniBrush(s: UniBrushSource, path: String) {
    def this(s: String, path: String) = this(UniBrushSource.fromJson(s), path)

    val brush = {
      val bitmap = s.`brush-path`.flatMap(bp => {
          DrawFiles.withFileStream(new File(path, bp))
          .map(DrawFiles.decodeBitmap(Bitmap.Config.ALPHA_8) _).opt.flatten
        })
      bitmap.map(Texture.apply _)
    }
    val interpolator = s.interpolator.flatMap(LuaScript.apply _)
    val pointshader = s.`point-shader`.flatMap(_.compile(PointShader.apply _))
    val animshader = s.`anim-shader`.flatMap(_.compile(CopyShader.apply _))
    val separateLayer = s.`separate-layer`.getOrElse(false)
  }

  object UniBrushSource {
    def fromJson(s: String) = parse(s).extract[UniBrushSource]
  }
}



