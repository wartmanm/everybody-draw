package com.github.wartman4404.gldraw

object LuaSyntaxHighlightProcessor {
  object Regex {
    // used as-is
    val comment = """(?m:--[^\[].*$)|(?s:--\[\[.*?(?:\]\]|$))"""
    val keywords = """\b(?:do|end|while|repeat|until|if|then|else|elseif|end|for|in|function|local|and|or|not)\b"""
    val stringbody = """@(?:[^@\\]|\\.)*?@"""
    val literals = joinKeywords(Array(
      """\d+\.?""",
      "true|false|nil")) +
     "|" + stringbody.replace("@", "\"") +
     "|" + stringbody.replace("@", "'")

    val types = """\bShaderPaintPoint\b"""

    val globalmethods = Array(
      "assert", "error", "ipairs", "next", "pairs", "pcall", "print", "select",
      "tonumber", "tostring", "type", "unpack")

    val methods = Array(
      "string\\.byte", "string\\.char", "string\\.dump", "string\\.find", "string\\.format",
      "string\\.gsub", "string\\.len", "string\\.lower", "string\\.rep", "string\\.sub",
      "string\\.upper", "string\\.gmatch", "string\\.match", "string\\.reverse",

      "table\\.maxn", "table\\.concat", "table\\.sort", "table\\.insert", "table\\.remove",

      "math\\.abs", "math\\.acos", "math\\.asin", "math\\.atan", "math\\.atan2", "math\\.ceil",
      "math\\.sin", "math\\.cos", "math\\.tan", "math\\.deg", "math\\.exp", "math\\.floor",
      "math\\.log", "math\\.max", "math\\.min", "math\\.log10", "math\\.huge", "math\\.fmod",
      "math\\.modf", "math\\.cosh", "math\\.sinh", "math\\.tanh", "math\\.pow", "math\\.rad",
      "math\\.sqrt", "math\\.frexp", "math\\.ldexp", "math\\.random", "math\\.randomseed",
      "math\\.pi")
    
    val apimethods = joinKeywords(Array(
      "pushpoint", "pushline", "pushcatmullrom", "pushcubicbezier",
      "loglua", "clearlayer", "savelayers", "saveundo"))

    // not actually used!
    val bannedmethods = joinKeywords(Array(
      "(?:coroutine|io|os|debug|package|jit|ffi)\\.\\w+", "getmetatable", "setmetatable",
      "xpcall", "_G", "loadfile", "rawequal", "require", "getfenv", "setfenv",
      "loadstring", "module"))

    def joinKeywords(kw: Array[String]) = kw.mkString("""\b(?:""", "|", """)\b""")
  }

  val luaColors: Array[Int] = Array(
    0xff808080, // comment
    0xff399ed7, // types
    0xffd79e39, // builtin vars
    0xffd79e39, // builtin fns
    0xff399ed7, // keywords
    0xff7ba212  // literals
  )

  val LuaProcessor = {
    val regex = Array(
      Regex.comment,
      Regex.types,
      Regex.apimethods,
      Regex.joinKeywords(Regex.globalmethods ++ Regex.methods),
      Regex.keywords,
      Regex.literals
    ).mkString("(", ")|(", ")")
    val values = new RegexSyntaxHighlightProcessor.RegexValues(regex, new LuaIndentCounter(), luaColors)
    new RegexSyntaxHighlightProcessor(values)
  }
}
