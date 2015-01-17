import sbt._

import Keys._
import sbtandroid.AndroidPlugin._

import scala.io.Source
  
object CreditResources {
  private val creditFiles = Def.task {
    ((sourceDirectory).value / "credits" * "*" get).sorted
  }

  private val creditContents = Def.task {
    creditFiles.value.map(f => {
      val src = Source.fromFile(f)
      val text = src.mkString
      src.close()
      (f, text)
    })
  }

  private def formatCredits = Def.task {
    val credititems = creditContents.value.map({ case (filename, text) =>
      "CreditItem(\"\"\"%s\"\"\", \"\"\"%s\"\"\"".format(filename, text)
    })
    val creditdata =
s"""|package ${(manifestPackage in Compile).value}
    |trait CreditData {
    |  val creditArray: Array[CreditItem] = Array(
        ${credititems.map("|\t\t" + _).mkString(",\n")}
    |  )
    |}""".stripMargin
    creditdata
  }

  val generateCreditsTask = Def.task {
    val packagedir = (manifestPackage in Compile).value.replace(".", "/")
    val dest = (managedScalaPath in Compile).value / packagedir / "creditdata.scala"
    IO.write(dest, formatCredits.value)
    streams.value.log.info(s"Wrote ${dest}")
    Seq(dest)
  }

  val generateCreditSources = TaskKey[Seq[File]]("generate-credit-sources", "Generate creditsdata.scala")

  lazy val settings: Seq[Setting[_]] = Seq (
    generateCreditSources <<= generateCreditsTask,
    (sourceGenerators in Compile) <+= generateCreditSources,
    (sourceGenerators in Preload) <+= generateCreditSources,
    (sourceGenerators in Release) <+= generateCreditSources,
    watchSources <++= creditFiles
  )
}
