package com.github.wartman4404.gldraw

object GLSLSyntaxHighlightProcessor {
  object Regex {
    // used as-is
    val comment = """(?m://.*$)|(?s:/\*.*?(?:\*/|$))"""
    val types = joinKeywords(Array("void|bool|int|float", "(?:[ib]?vec|mat)[234]", "sampler[(?:2D|Cube)]", "const"))
    val literals = """\b\d+\.?\b"""

    // used by ShaderKeywords
    val vertOnlyVars = Array("gl_Position", "gl_PointSize")
    val fragOnlyVars = Array("gl_FragCoord", "gl_FrontFacing", "gl_FragColor")

    val keywords = Array("if", "else", "for", "while", "do", "break", "continue", "return")
    val fragOnlyKeywords = Array("discard")

    val builtins = Array("radians", "degrees", "sin", "cos", "tan", "asin", "acos", "atan", "pow", "exp", "log", "exp2", "log2", "sqrt", "inversesqrt", "abs", "sign", "floor", "ceil", "fract", "mod", "min", "max", "clamp", "mix", "step", "smoothstep", "length", "distance", "dot", "cross", "normalize", "faceforward", "reflect", "refract", "matrixCompMult", "lessThan", "lessThanEqual", "greaterThan", "greaterThanEqual", "equal", "notEqual", "any", "all", "not", "texture2D", "texture2DProj", "textureCube")
    val vertOnlyBuiltins = Array("texture2D(?:Proj|Cube|)Lod")

    // not actually used!
    val unsupported = Array("uniform", "varying", "attribute", "struct", "gl_PointCoord", "gl_FragData")
    val constants = "gl_Max(?:VertexAttribs|UniformVectors|VaryingVectors|VertexTextureImageUnits|CombinedTextureImageUnits|TextureImageUnits|FragmentUniformVectors|DrawBuffers)|gl_DepthRange"

    def joinKeywords(kw: Array[String]) = kw.mkString("""\b(?:""", "|", """)\b""")
  }

  val glslColors: Array[Int] = Array(
    0xff808080, // comment
    0xff399ed7, // types
    0xffd79e39, // builtin vars
    0xffd79e39, // builtin fns
    0xff399ed7, // keywords
    0xff7ba212  // literals
  )

  trait ShaderKeywords {
    val builtinFns: Array[String]
    val builtinVars: Array[String]
    val keywords: Array[String]
    def getHighlightRegex() = {
      Array(
        Regex.comment,
        Regex.types,
        Regex.joinKeywords(builtinVars),
        Regex.joinKeywords(builtinFns),
        Regex.joinKeywords(keywords),
        Regex.literals
      ).mkString("(", ")|(", ")")
    }
    def getHighlightProcessor() = {
      val values = new RegexSyntaxHighlightProcessor.RegexValues(this.getHighlightRegex(), null, glslColors);
      new RegexSyntaxHighlightProcessor(values);
    }
  }

  val VertProcessor = {
    val vertKeywords = new ShaderKeywords() {
      val builtinFns = Regex.builtins ++ Regex.vertOnlyBuiltins
      val builtinVars = Regex.vertOnlyVars
      val keywords = Regex.keywords
    }
    vertKeywords.getHighlightProcessor()
  }

  val FragProcessor = {
    val fragKeywords = new ShaderKeywords() {
      val builtinFns = Regex.builtins 
      val builtinVars = Regex.fragOnlyVars
      val keywords = Regex.keywords ++ Regex.fragOnlyKeywords
    }
    fragKeywords.getHighlightProcessor()
  }
}
