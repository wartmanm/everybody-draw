import sbt._

import Keys._
import sbtandroid.AndroidPlugin._
  
object PaintResources {
  private val resources = Def.task {
    val dirs = Seq("animshaders", "pointshaders", "brushes", "interpolators", "unibrushes")
    val files = dirs map ((mainAssetsPath in Compile).value / _ * "*" get)
    dirs zip files
  }
    
  private def formatResource(path: File, subdir: String, resources: Seq[File]) = {
    val files = resources map { "\"" + _.relativeTo(path).get.getPath() + "\"" }
    val filestr = s"val ${subdir} = Dir(${"\"" + subdir + "\""}, Array(${files.mkString(", ")}))"
    filestr
  }

  private val formatResources = Def.task {
    val paths = resources.value map {
      case (dir, files) => formatResource((mainAssetsPath in Compile).value, dir, files)
    }
    val assetstrs =
s"""|package ${(manifestPackage in Compile).value}
    |
    |object PreinstalledPaintResources {
    |\tcase class Dir(name: String, builtin: Array[String])
      ${paths.map("|\t" + _).mkString("\n")}
    |}
    |""".stripMargin
    assetstrs
  }

  val generatePreinstalledSourcesTask = Def.task {
    val packagedir = (manifestPackage in Compile).value.replace(".", "/")
    val dest = (managedScalaPath in Compile).value / packagedir / "preinstalledpaintresources.scala"
    IO.write(dest, formatResources.value)
    streams.value.log.info(s"Wrote ${dest}")
    Seq(dest)
  }

  val generatePreinstalledSources = TaskKey[Seq[File]]("generate-preinstalled-sources",
    """Generate preinstalledpaintresources.scala""")

  lazy val settings: Seq[Setting[_]] = Seq (
    generatePreinstalledSources <<= generatePreinstalledSourcesTask,
    (sourceGenerators in Compile) <+= generatePreinstalledSources,
    watchSources <++= Def.task { resources.value.flatMap(_._2) }
  )
}
