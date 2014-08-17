import sbt._

import Keys._
import sbtandroid.AndroidPlugin._
import sbtandroid.AndroidProjects.Standard
  

object General {
  //val optimized = Seq (
    //scalacOptions ++= Seq("-Ybackend:o3", "-Ydelambdafy:method", "-Yinline", "-optimise"),
    //scalaHome := Some(file("/opt/scala"))
  //)


  lazy val compileRust = taskKey[sbt.inc.Analysis]("Compiles native sources.")

  lazy val cleanRust = taskKey[Unit]("Deletes files generated from native sources.")

  lazy val rustDir = settingKey[File]("Rust source directory")

  lazy val processLogger = Def.task {
    new sbt.ProcessLogger() {
      override def buffer[T](f: => T): T = f
      override def error(s: => String): Unit = streams.value.log.warn(s)
      override def info(s: => String): Unit = streams.value.log.info(s)
    }
  }

  lazy val environment = Def.task {
    val outdir = rustDir.value.toString
    Seq(
      "TARGET_TRIPLE" -> "arm-linux-androideabi",
      "OUT_DIR" -> outdir,
      "PLATFORM_NAME" -> platformName.value
    )
  }

  lazy val cleanRustTask = Def.task {
    val result = sbt.Process("make clean",
      rustDir.value,
      environment.value: _*
    ) !< processLogger.value
    if (result != 0)
      sys.error("error cleaning native library")
  }

  lazy val compileRustTask = Def.task {
    val result = sbt.Process("make all",
      rustDir.value,
      environment.value: _*
    ) !< processLogger.value
    if (result != 0)
      sys.error("error compiling native library")
    sbt.inc.Analysis.Empty
  }

  lazy val rustSettings = Seq(
    compileRust <<= compileRustTask,
    cleanRust <<= cleanRustTask,
    rustDir <<= Def.setting { (sourceDirectory in Compile).value / "rust" },
    (ndkBuild in Compile) <<= (ndkBuild in Compile) dependsOn compileRust,
    (ndkBuild in Preload) <<= (ndkBuild in Preload) dependsOn compileRust,
    clean := {
      val _ = cleanRustTask.value
      clean.value
    }
  )

  lazy val debugSettings = Seq (
    scalacOptions ++= Seq("-Ywarn-dead-code", "-Ywarn-unused", "-Ywarn-unused-import", "-Ywarn-adapted-args", "-Ywarn-inaccessible", "-Ywarn-infer-any", "-Ywarn-nullary-override", "-Ywarn-nullary-unit")
  )
  
  lazy val excessiveDebugSettings = Seq (
    scalacOptions ++= Seq("-Ywarn-value-discard")
  )

  val settings = Defaults.defaultSettings ++ Seq (
    resolvers += "Local Maven Repository" at "file://"+Path.userHome.absolutePath+"/.m2/repository",
    name := "everybodydraw",
    version := "0.1",
    versionCode := 0,
    scalaVersion := "2.11.0",
    platformName := "android-19",
    javacOptions ++= Seq("-encoding", "UTF-8", "-source", "1.6", "-target", "1.6"),
    scalacOptions ++= Seq("-feature", "-language:implicitConversions", "-deprecation", "-Xlint")
  ) ++ debugSettings

  lazy val fullAndroidSettings =
    General.settings ++
    androidDefaults ++
    rustSettings ++
    Seq (ndkJniSourcePath <<= Def.setting { baseDirectory.value / "jni" }) ++
    Seq (
      keyalias := "change-me",
      useTypedResources := true,
      libraryDependencies ++= Seq(
        apklib("com.github.iPaulPro" % "aFileChooser" % "0.1" changing() ),
        "com.jsuereth" %% "scala-arm" % "1.5-SNAPSHOT"
      )
    )
}

object AndroidBuild extends Build {
  lazy val main = Project (
    "everybodydraw",
    file("."),
    settings = General.fullAndroidSettings
  )
}
