package com.github.wartman4404.gldraw

import android.os.Bundle
import android.widget.{AdapterView, Adapter}

import java.io.{InputStream, OutputStream, OutputStreamWriter, BufferedWriter}

import UniBrush.UniBrush

class PaintControls(inbrushpicker: AdapterView[Adapter], inanimpicker: AdapterView[Adapter], inpaintpicker: AdapterView[Adapter], ininterppicker: AdapterView[Adapter], inunipicker: AdapterView[Adapter]) {
  import PaintControls._

  val animpicker = NamedPicker[CopyShader]("anim", inanimpicker)
  val brushpicker = NamedPicker[Texture]("brush", inbrushpicker)
  val paintpicker = NamedPicker[PointShader]("paint", inpaintpicker)
  val interppicker = NamedPicker[LuaScript]("interp", ininterppicker)
  val unipicker = NamedPicker[UniBrush]("unib", inunipicker)

  val namedPickers = Array(animpicker, brushpicker, paintpicker, interppicker, unipicker)

  def restoreState() = namedPickers.map(_.restoreState())
  def updateState() = namedPickers.map(_.updateState())
  def save(b: Bundle): Unit = namedPickers.foreach(_.save(b))
  def load(b: Bundle): Unit = namedPickers.foreach(_.load(b))
  def save(m: Map[String, String]): Map[String, String] = namedPickers.foldLeft(m)((m, p) => p.save(m))
  def load(m: Map[String, String]): Unit = namedPickers.map(_.load(m))
  def save(os: OutputStream): Unit = {
    val writer = new BufferedWriter(new OutputStreamWriter(os))
    writer.write(save(Map[String,String]()).map { case (k, v) => s"$k=$v" }.mkString("\n"))
    writer.close()
  }
  def load(is: InputStream): Unit = {
    val reader = scala.io.Source.fromInputStream(is)
    val map = reader.getLines.foldLeft(Map[String, String]())((m, line) => {
        val Array(k, v): Array[String] = line.split("=", 2)
        m + (k -> v)
      })
    load(map)
    reader.close()
  }

}
object PaintControls {

  case class NamedPicker[T](name: String, control: AdapterView[Adapter]) {
    private var state: Option[String] = None
    def restoreState() = {
      val index = state.map(s => control.getAdapter().indexWhere(_.asInstanceOf[SpinnerItem[T]].name == s) match {
          case -1 => 0
          case  x => x
        }).getOrElse(0)
      control.setSelection(index)
    }
    def updateState() = state = Option(control.getSelectedItem()).map(_.asInstanceOf[SpinnerItem[T]].name)
    def save(b: Bundle): Unit = for (value <- state) b.putString(name, value)
    def load(b: Bundle): Unit = state = Option(b.getString(name))
    def save(m: Map[String, String]): Map[String, String] = state.map(value => m + (name -> value)).getOrElse(m)
    def load(m: Map[String, String]): Unit = state = m.get(name)
  }

  implicit class AdapterSeq(a: Adapter) extends IndexedSeq[Object] {
    def length = a.getCount()
    def apply(pos: Int) = a.getItem(pos)
  }

  // TODO: ditch this for real adapters
  // also, preview icons? are they achievable?
  case class SpinnerItem[T](name: String, item: T) {
    override def toString() = name
  }
  object SpinnerItem {
    def apply[T](nameitem: (String, T)): SpinnerItem[T] = SpinnerItem(nameitem._1, nameitem._2)
  } 
}
