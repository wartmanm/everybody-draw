package com.github.wartman4404.gldraw

import android.os.Bundle
import android.widget.{AdapterView, Adapter}

import java.io.{InputStream, OutputStream, OutputStreamWriter, BufferedWriter}

import unibrush.UniBrush

import spray.json._
import PaintControls._

class PaintControls[A <: AV, B <: AV, C <: AV, D <: AV, E <: AV]
    (inanimpicker: A, inbrushpicker: B, inpaintpicker: C, ininterppicker: D, inunipicker: E) extends AutoProductFormat {

  val animpicker = NamedPicker[A, CopyShader]("anim", inanimpicker)
  val brushpicker = NamedPicker[B, Texture]("brush", inbrushpicker)
  val paintpicker = NamedPicker[C, PointShader]("paint", inpaintpicker)
  val interppicker = NamedPicker[D, LuaScript]("interp", ininterppicker)
  val unipicker = NamedPicker[E, UniBrush]("unib", inunipicker)

  val namedPickers = Array(animpicker, brushpicker, paintpicker, interppicker, unipicker)

  def restoreState() = namedPickers.map(_.restoreState())
  def updateState() = namedPickers.map(_.updateState())
  def saveToString(): String = {
    namedPickers.map(_.save()).toJson.toString
  }
  def loadFromString(s: String) = {
    val saved = s.parseJson.convertTo[Array[SavedState]]
    for ((picker, state) <- namedPickers.zip(saved)) {
      picker.load(state)
    }
  }
  def save(b: Bundle): Unit = b.putString("paintcontrols", saveToString())
  def load(b: Bundle): Unit = loadFromString(b.getString("paintcontrols"))
  def save(os: OutputStream): Unit = {
    val writer = new BufferedWriter(new OutputStreamWriter(os))
    writer.write(saveToString())
    writer.close()
  }
  def load(is: InputStream): Unit = {
    val state = DrawFiles.readStream(is)
    loadFromString(state)
  }
}
object PaintControls {
  type AV = AdapterView[Adapter]
  case class NamedPicker[A <: AdapterView[Adapter], T](name: String, control: A) {
    var selected = AdapterView.INVALID_POSITION
    var selectedName = ""
    var enabled = true
    def restoreState(): Unit = {
      selected = control.getAdapter().asInstanceOf[LazyPicker[T]].lazified.indexWhere(_._1 == selectedName) match {
        case -1 => 0
        case  x => x
      }
      if (enabled) control.performItemClick(null, selected, selected)
    }
    def updateState() = selectedName = selected match {
      case AdapterView.INVALID_POSITION => ""
      case x => control.getAdapter().asInstanceOf[LazyPicker[T]].lazified(x)._1
    }
    def save() = SavedState(enabled, selectedName)
    def load(state: SavedState) = {
      enabled = state.enabled
      selectedName = state.selectedName
    }
  }

  case class SavedState(enabled: Boolean, selectedName: String)

  implicit class AdapterSeq(a: Adapter) extends IndexedSeq[Object] {
    def length = a.getCount()
    def apply(pos: Int) = a.getItem(pos)
  }

}
