//resolvers += Resolver.url("scalasbt releases", new URL("http://scalasbt.artifactoryonline.com/scalasbt/sbt-plugin-releases"))(Resolver.ivyStylePatterns)

//resolvers += Resolver.file("local", file("/home/matt/.ivy2/local")) // this is built in

addSbtPlugin("org.scala-sbt" % "sbt-android" % "0.7.2-SNAPSHOT")

