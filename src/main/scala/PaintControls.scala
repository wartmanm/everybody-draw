package com.github.wartman4404.gldraw

import android.os.Bundle
import android.content.Context
import android.widget.{AdapterView, Adapter, GridView}

import java.io.{InputStream, OutputStream, OutputStreamWriter, BufferedWriter}

import unibrush.UniBrush

import spray.json._
import PaintControls._

class PaintControls[A <: UP[CopyShader, AV, _ <: AA], B <: UP[Texture, AV, _ <: AA], C <: UP[PointShader, AV, _ <: AA], D <: UP[LuaScript, AV, _ <: AA], E <: UP[UniBrush, AV, _ <: AA]]
  (val animpicker: A, val brushpicker: B, val paintpicker: C, val interppicker: D, val unipicker: E) 
extends AutoProductFormat {

  val namedPickers = Map(
    "anim" -> animpicker,
    "brush" -> brushpicker,
    "paint" -> paintpicker,
    "interp" -> interppicker,
    "unibrush" -> unipicker
  )

  def restoreState() = namedPickers.values.map(_.restoreState())
  def updateState() = namedPickers.values.map(_.updateState())
  def saveToString(): String = {
    namedPickers.map({case (k,v) => (k, v.save())}).toJson.toString
  }
  def loadFromString(s: String) = {
    val saved = s.parseJson.convertTo[Map[String, SavedState]]
    for ((name, state) <- saved) {
      namedPickers(name).load(state)
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
  type AV = AdapterView[_ <: Adapter]
  type AAV[AA] = AdapterView[AA]
  type UP[T, U <: AAV[V], V <: AA] = UnnamedPicker[T, U, V]
  type AA = Adapter
  //def apply[A <: AV[CopyShader], B <: AV[Texture], C <: AV[PointShader], D <: AV[LuaScript], E <: AV[UniBrush]]
  def apply[A <: AAV[A_A], B <: AAV[B_A], C <: AAV[C_A], D <: AAV[D_A], E <: AAV[E_A], A_A <: AA, B_A <: AA, C_A <: AA, D_A <: AA, E_A <: AA]
  (animpicker: A, brushpicker: B, paintpicker: C, interppicker: D, unipicker: E) = {
    val a: UP[CopyShader, A, A_A] = new UnnamedPicker[CopyShader, A, A_A](animpicker)
    val b: UP[Texture, B, B_A] = new UnnamedPicker[Texture, B, B_A](brushpicker)
    val c: UP[PointShader, C, C_A] = new UnnamedPicker[PointShader, C, C_A](paintpicker)
    val d: UP[LuaScript, D, D_A] = new UnnamedPicker[LuaScript, D, D_A](interppicker)
    val e: UP[UniBrush, E, E_A] = new UnnamedPicker[UniBrush, E, E_A](unipicker)
    
    new PaintControls (a, b, c, d, e)
    //new UnnamedPicker[CopyShader, A](animpicker),
    //new UnnamedPicker[Texture, B](brushpicker),
    //new UnnamedPicker[PointShader, C](paintpicker),
    //new UnnamedPicker[LuaScript, D](interppicker),
    //new UnnamedPicker[UniBrush, E](unipicker)
  //)
  }

  class UnnamedPicker[T, +U <: AdapterView[V], V <: Adapter](val control: U)  {
    var selected = AdapterView.INVALID_POSITION
    var selectedName = ""
    var enabled = true
    private var adapter: LazyPicker[T] = null
    def setAdapter[V](a: LazyPicker[T]) = {
      adapter = a
      control.setAdapter(a)
    }
    def restoreState(): Unit = {
      selected = this.adapter.lazified.indexWhere(_._1 == selectedName) match {
        case -1 => 0
        case  x => x
      }
      if (enabled) this.control.performItemClick(null, selected, selected)
    }
    def updateState() = selectedName = selected match {
      case AdapterView.INVALID_POSITION => ""
      case x => this.adapter.lazified(x)._1
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

//class SavedGridView[T](c: Context, attrs: AttributeSet, defStyleAttr: Int, defStyleRes: Int)
//extends PaintControls.GridView(c, attrs, defStyleAttr, defStyleRes) 
//with UnnamedPicker[T] {
  //def this(c: Context, attrs: AttributeSet, defStyleAttr: Int) = this(c, attrs, defStyleAttr, 0)
  //def this(c: Context, attrs: AttributeSet) = this(c, attrs, defStyleAttr, 0, 0)
  //def this(c: Context) = this(c, attrs, null, 0, 0)
//}
