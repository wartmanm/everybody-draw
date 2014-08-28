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
    def compile[T](compiler: (String, String) => Option[T]) = {
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

  object UniBrushSource {
    def fromJson(s: String) = s.parseJson.convertTo[UniBrushSource]
  }

  class UniBrush(s: UniBrushSource, path: String) {
    Log.i("unibrush", s"initing unibrush at ${path}")
    def this(s: String, path: String) = this(UniBrushSource.fromJson(s), path)

    val brush = {
      val bitmap = s.brushpath.flatMap(bp => {
          DrawFiles.withFileStream(new File(path, bp))
          .map(DrawFiles.decodeBitmap(Bitmap.Config.ALPHA_8) _).opt.flatten
        })
      bitmap.map(Texture.apply _)
    }
    Log.i("unibrush", "compiling unibrush");
    val interpolator = s.interpolator.flatMap(LuaScript.apply _)
    val pointshader = s.pointshader.flatMap(_.compile(PointShader.apply _))
    val animshader = s.animshader.flatMap(_.compile(CopyShader.apply _))
    val separateLayer = s.separatelayer.getOrElse(false)
    Log.i("unibrush", s"have interpolator: ${interpolator.nonEmpty}");
    Log.i("unibrush", s"have pointshader: ${pointshader.nonEmpty}");
    Log.i("unibrush", s"have animshader: ${animshader.nonEmpty}");
    Log.i("unibrush", s"have separateLayer: ${separateLayer}");
    Log.i("unibrush", s"have brush: ${brush.nonEmpty}");
  }
}



